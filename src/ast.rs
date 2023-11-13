// i prefer painting abstract trees

use std::cmp::min;
use std::fmt::Debug;
use remtools::{join, qc};

use crate::{compile, error, format_out, MCFunction, SaveFiles};
use crate::ast::NodeType::Macro;
use crate::compile::require;
use crate::NodeType::{Command, Comment, FnCall, Scoreboard};
use crate::server::{Blocker, death_error};
use crate::server::errors::AST_ERROR;

#[derive(Debug, Clone)]
pub struct Node {
    pub node: NodeType,
    pub children: Vec<Node>,
    pub lines: Vec<String>,
    pub ln: usize,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    None,
    Root,

    Command,
    Scoreboard,
    Block(char),
    FnCall(String, Option<String>),
    Macro(Box<Node>),

    Comment,

    Alias(String, String),
}

impl NodeType {
    fn is_none(&self) -> bool {
        return match self {
            NodeType::None => true,
            _ => false,
        };
    }
}

impl Node {
    fn print_tree(&self, idlvl: usize) {
        println!("{}[{:?}@{}]: {:?}", &*["| ".to_string()].into_iter().cycle().take(idlvl).collect::<Vec<_>>().join(""),
                 self.node, self.ln, self.lines);
        for child in self.children.iter() {
            child.print_tree(idlvl + 1);
        }
    }

    pub fn _tree_size(&self, mut current: u64) -> u64 {
        for n in self.children.iter() {
            current += 1;
            n._tree_size(current);
        }
        current
    }

    pub fn new(ty: NodeType, ln: usize) -> Node {
        Node {
            node: ty,
            children: vec![],
            lines: vec![],
            ln,
        }
    }

    pub fn get_save_files(&mut self, files: &mut SaveFiles, lines: &mut Vec<String>, mcf: &mut MCFunction) {
        use crate::NodeType::*;
        match &mut self.node {
            Root => {
                // self.print_tree(0);
                self.children.iter_mut().for_each(|c| {
                    c.get_save_files(files, lines, mcf);
                });
                self.add_to_files(files, mcf.get_path(), lines, mcf);
            }
            Macro(node) => {
                let fln = lines.len();
                node.get_save_files(files, lines, mcf);
                lines[fln].insert(0, '$');
                self.add_to_files(files, mcf.get_path(), lines, mcf);
            }
            Block(id) => {
                let mut blines = vec![];
                self.children.iter_mut().for_each(|c| {
                    c.get_save_files(files, &mut blines, mcf);
                });
                if (*id).eq(&'r') {
                    let amo = (blines.len() - 1) * blines.remove(0).parse::<usize>().unwrap_or(1);
                    compile::finish_lines(&mut blines, mcf);
                    blines = blines.into_iter().cycle().take(amo).collect::<Vec<_>>();
                    let path = join![&*mcf.get_path(), ".", &*id.to_string(), &*self.ln.to_string()];
                    let path = path.strip_prefix("/").unwrap_or(&*path);
                    lines.push(join!("function ", &*mcf.ns_id, ":", &*path));
                    files.push((path.into(), blines));
                    return;
                } else {
                    if blines.len() <= 1 && mcf.meta.opt_level >= 1 {
                        lines.push(blines.join(" "));
                    } else {
                        let path = join![&*mcf.get_path(), ".", &*id.to_string(), &*self.ln.to_string()];
                        let path = path.strip_prefix("/").unwrap_or(&*path);
                        lines.push(join!("function ", &*mcf.ns_id, ":", &*path));
                        self.add_to_files(files, path.into(), &mut blines, mcf);
                    }
                }
            }

            Alias(exns, exna) => {
                files.push((join!["@ALIAS", &*exns, "/functions/", &*exna], self.lines.drain(..).collect()));
            }

            Command => {
                let mut last = vec![];
                self.children.iter_mut().for_each(|c| {
                    c.get_save_files(files, &mut last, mcf);
                });
                if !last.is_empty() {
                    if self.lines.len() > 1 {
                        if last.len() > 1 {
                            error(format_out("Cannot stack multiline statements", &*mcf.get_file_loc(), self.ln));
                        } else {
                            let melast = self.lines.last_mut().unwrap();
                            melast.push(' ');
                            melast.push_str(&*last[0]);
                        }
                    } else {
                        let me = self.lines[0].clone();
                        self.lines = last;
                        let last = self.lines.last_mut().unwrap();
                        *last = join![&*me, " ", &**last];
                    }
                }
                lines.append(&mut self.lines);
            }
            Scoreboard => {
                lines.append(&mut self.lines);
            }
            FnCall(name, extras) => {
                if extras.is_some() {
                    lines.push(join![" ": "function", &**name, &extras.clone().unwrap()]);
                } else {
                    lines.push(join![" ": "function", &**name]);
                }
            }
            Comment if mcf.meta.comments => {
                lines.append(&mut self.lines.clone());
            }

            Comment | None => {}
        }
    }

    fn add_to_files(&mut self, files: &mut SaveFiles, path: String, lines: &mut Vec<String>, mcf: &mut MCFunction) {
        compile::finish_lines(lines, mcf);
        files.push((path, lines.clone()));
    }

    pub fn generate(&mut self, mcf: &mut MCFunction) {
        use crate::NodeType::*;
        match &mut self.node {
            Block(_) | Root => {
                self.generate_text(mcf);
            }

            Scoreboard | Command | FnCall(_, _) => {
                compile::node_text(self, mcf);
            }

            Macro(node) => {
                node.generate(mcf);
            }

            None | Comment | Alias(_, _) => {}
        }
        self.generate_children(mcf);
    }

    fn generate_children(&mut self, mcf: &mut MCFunction) {
        self.children.iter_mut().for_each(|c| {
            c.generate(mcf);
        });
    }

    fn generate_text(&mut self, mcf: &mut MCFunction) {
        self.lines = self.lines.iter()
            .map(|l| if l.starts_with("@NOLEX") {
                l.replacen("@NOLEX", "", 1).trim().to_string()
            } else { l.clone() }).collect();
        let mut ln = 1;
        while self.lines.len() > 0 {
            compile::replacements(self, mcf, ln);

            let (rem, nn) = Node::build_from_lines(&mut self.lines, mcf, self.ln + ln);
            for _ in 0..min(rem, self.lines.len()) {
                self.lines.remove(0);
                ln += 1;
            }
            if let Some(nnu) = nn {
                self.children.push(nnu);
            }
        }
    }

    #[allow(arithmetic_overflow)]
    fn build_from_lines(lines: &mut Vec<String>, mcf: &mut MCFunction, ln: usize) -> (usize, Option<Node>) {
        if lines[0].starts_with("//") {
            let mut node = Node::new(Comment, ln);
            node.lines = vec![join!["#", &lines[0][2..]]];
            return (1, Some(node));
        }
        let keys = Blocker::new().split_in_same_level(" ", &lines[0]);
        if let Err(e) = keys {
            error(format_out(&*e, &*mcf.get_file_loc(), ln));
            return (0 - 1 as usize, None);
        }
        let mut keys = keys.unwrap();
        let mut node = Node::new(NodeType::None, ln);
        let mut rem = 1;
        match &*keys[0] {
            "@DEBUG" => {
                println!("\x1b[96m@DEBUG [{}]: {} for {}\x1b[0m", keys[1..].join(" "), ln, lines[0]);
            }
            "@TREE" => {
                lines.remove(0);
                let (remx, nna) = Node::build_from_lines(lines, mcf, ln);
                println!("\x1b[94m@TREE {}:{} [{}]:\x1b[0m", &*mcf.get_file_loc(), ln, keys[1..].join(" "));
                if let Some(node) = nna.clone() {
                    node.print_tree(0);
                }
                return (remx, nna);
            }
            "@DBG_ERROR" => {
                error(format_out(
                    &*format!("\x1b[94m@DBG_ERROR [{}]\x1b[0m", keys[1..].join(" ")),
                    &*mcf.get_file_loc(),
                    ln,
                ));
            }
            "ast" => {
                lines[0] = ["execute ", &*lines[0]].join("");
                return Node::build_from_lines(lines, mcf, ln);
            }
            "exe" => {
                lines[0] = ["execute ", &lines[0][4..]].join("");
                return Node::build_from_lines(lines, mcf, ln);
            }
            "execute" => {
                node.node = Command;
                node.lines = vec![lines[0].clone()];
                if let Ok(Some(run)) = Blocker::new().find_in_same_level(" run ", &node.lines[0]) {
                    if lines[0][run + 5..run + 6].eq("{") {
                        let (remx, nna) = Node::build_extract_block(lines, &mut node, mcf, 'e');
                        node.children.push(nna);
                        rem = remx;
                        node.lines[0] = node.lines[0][0..node.lines[0].len() - 2].to_string();
                    } else {
                        node.lines[0] = node.lines[0][..run + 4].into();
                        lines[0] = lines[0][run + 5..].into();
                        let (remx, nn) = Node::build_from_lines(lines, mcf, node.ln);
                        if let Some(nnu) = nn {
                            node.children.push(nnu);
                        }
                        rem = remx;
                    }
                }
            }
            "set" if require::min_args(3, &keys, mcf, ln) => {
                require::not_default_replacement(&keys[1], mcf.get_file_loc(), ln);
                node.node = NodeType::None;
                mcf.vars.retain(|x| !x.0.eq(&*keys[1]));
                mcf.vars
                    .insert(0, (keys[1].to_string(), keys[2..].join(" ").to_string()));
            }
            "{" => {
                let (remx, nna) = Node::build_extract_block(lines, &mut node, mcf, 'b');
                return (remx, Some(nna));
            }
            "repeat" if require::exact_args(3, &keys, mcf, ln) => {
                let (remx, mut nna) = Node::build_extract_block(lines, &mut node, mcf, 'r');
                nna.lines.insert(0, join!["cmd ", &*keys[1].parse::<u32>().unwrap_or_else(|_| {
                    error(format_out(&*join!["Failed to parse '", &*keys[1], "' as a number"],
                                     &*mcf.get_file_loc(), ln));
                    1
                }).to_string()]);
                return (remx, Some(nna));
            }
            "if" if require::min_args(2, &keys, mcf, ln) => {
                qc!(keys.len() > 2, keys.insert(2, "run".into()), ());
                lines[0] = join!["execute ", &*keys.join(" ")];
                return Node::build_from_lines(lines, mcf, ln);
            }
            "while" if require::min_args(2, &keys, mcf, ln) => {
                node.node = Command;
                node.lines = vec![lines[0].clone()];
                let (remx, nna) = Node::build_extract_block(lines, &mut node, mcf, 'w');
                node.children.push(nna);
                rem = remx;
            }
            "for" if require::min_args(2, &keys, mcf, ln) => {
                node.node = Command;
                node.lines = vec![lines[0].clone()];
                let (remx, nna) = Node::build_extract_block(lines, &mut node, mcf, 'f');
                node.children.push(nna);
                rem = remx;
            }
            "tag" if keys.len() > 3 => {
                keys[3] = keys[3].replace("r&", "remgine.").replace("&", &*join![&*mcf.ns_id, "."]);
                node.node = Command;
                node.lines = vec![keys.join(" ")];
            }
            "macro" => {
                lines[0] = lines[0][6..].into();
                let (remx, nn) = Node::build_from_lines(lines, mcf, node.ln);
                if let Some(nnu) = nn {
                    node.node = Macro(Box::new(nnu));
                }
                rem = remx;
            }
            _ if MCFunction::is_score_path(&keys[0], mcf, ln) => {
                node.node = Scoreboard;
                node.lines = vec![lines[0].clone()];
                if keys.len() >= 3 && ((&*keys[1]).eq("result") || (&*keys[1]).eq("success")) {
                    lines[0] = keys[2..].join(" ");
                    let target = MCFunction::compile_score_path(&keys[0], mcf, ln);
                    node.node = Command;
                    node.lines[0] = join!["execute store ", &*keys[1], " score ", &*target.join(" "), " run"];
                    if let (remx, Some(post)) = Node::build_from_lines(lines, mcf, ln) {
                        node.children.push(post);
                        rem = remx;
                    } else {
                        error(format_out(&*join!["No result produced for '", &*lines[0], "'"], &*mcf.get_file_loc(), ln));
                        node.node = Comment;
                        node.lines = vec![join!["#", &*node.lines[0], " <no result produced>"]];
                        node.children.clear();
                    }
                }
            }
            _ if keys[0].starts_with("//") => {
                node.node = Comment;
                node.lines = vec![join!["#", &lines[0][2..]]];
            }
            _ if keys[0].is_empty() => {
                node.node = Comment;
                node.lines = vec!["".into()];
            }
            f @ _ => {
                if let Some((path, extras)) = compile::is_fn_call(f, mcf, &keys) {
                    node.node = FnCall(path, extras);
                } else {
                    node.node = Command;
                    node.lines = vec![lines[0].clone()];
                }
            }
        }
        (rem, if !node.node.is_none() { Some(node) } else { None })
    }

    fn build_extract_block(lines: &mut Vec<String>, node: &Node, mcf: &mut MCFunction, ident: char) -> (usize, Node) {
        let mut b = Blocker::new();
        match b.find_size_vec(lines, (0, lines[0].rfind("{").unwrap_or(0))) {
            Ok(o) => {
                if o.0 != Blocker::NOT_FOUND {
                    let mut nna = Node::new(NodeType::Block(ident), node.ln + 1);
                    lines[1..o.0].clone_into(&mut nna.lines);
                    return (o.0 + 1, nna);
                } else {
                    death_error(format_out("Unterminated block", &*mcf.get_file_loc(), node.ln), AST_ERROR);
                }
            }
            Err(e) => death_error(format_out(&*e.0, &*mcf.get_file_loc(), e.1 + node.ln), AST_ERROR),
        }
    }
}