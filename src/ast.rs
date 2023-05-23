// i prefer painting abstract trees

use std::cmp::min;
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::process::Termination;
use std::slice::Iter;
use crate::{death_error, error, format_out, join, MCFunction, MCValue, Namespace, qc, SaveFiles, compile, COMMANDS, warn};
use crate::compile::require;
use crate::NodeType::{Block, Command, Comment, FnCall, Scoreboard};
use crate::server::{Blocker, FancyText, path_without_functions};

#[derive(Debug, Clone)]
pub struct Node {
    pub node: NodeType,
    children: Vec<Node>,
    pub lines: Vec<String>,
    pub(crate) ln: usize,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    None,
    Root,
    EOF,

    Command,
    Scoreboard,
    Block(char),
    FnCall(String),

    Comment,
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

    pub fn new(ty: NodeType, ln: usize) -> Node {
        Node {
            node: ty,
            children: vec![],
            lines: vec![],
            ln,
        }
    }

    pub fn append_children(&mut self, childs: &mut Vec<Node>) {
        self.children.append(childs);
    }

    pub fn get_first_child(&self) -> Option<&Node> {
        self.children.get(0)
    }

    pub fn get_last_child(&self) -> Option<&Node> {
        self.children.get(self.children.len() - 1)
    }

    pub fn get_child(&self, idx: usize) -> Option<&Node> {
        self.children.get(idx)
    }

    pub fn get_children(&self) -> Iter<Node> {
        self.children.iter()
    }

    pub fn get_save_files(&mut self, files: &mut SaveFiles, lines: &mut Vec<String>, mcf: &mut MCFunction) {
        use crate::NodeType::*;
        match &self.node {
            Root => {
                // self.print_tree(0);
                self.children.iter_mut().for_each(|mut c| {
                    c.get_save_files(files, lines, mcf);
                });
                self.add_to_files(files, mcf.get_path(), lines, mcf);
            }
            Block(id) => {
                let mut blines = vec![];
                self.children.iter_mut().for_each(|mut c| {
                    c.get_save_files(files, &mut blines, mcf);
                });
                if id.eq(&'r') {
                    let amo = (blines.len() - 1) * blines.remove(0).parse::<usize>().unwrap_or(1);
                    blines = blines.into_iter().cycle().take(amo).collect::<Vec<_>>();
                }
                if blines.len() <= 1 && mcf.meta.opt_level >= 1 {
                    lines.push(blines.join(" "));
                } else {
                    let path = join![&*mcf.get_path(), ".", &*id.to_string(), &*self.ln.to_string()];
                    let path = path.strip_prefix("/").unwrap_or(&*path);
                    lines.push(join!("function ", &*mcf.ns_id, ":", &*path));
                    self.add_to_files(files, path.into(), &mut blines, mcf);
                }
            }

            Command => {
                let mut last = vec![];
                self.children.iter_mut().for_each(|mut c| {
                    c.get_save_files(files, &mut last, mcf);
                });
                if !last.is_empty() {
                    if self.lines.len() > 1 {
                        error(format_out("Cannot stack multiline statements", &*mcf.get_file_loc(), self.ln));
                    } else {
                        let me = self.lines[0].clone();
                        self.lines = last;
                        let last = self.lines.last_mut().unwrap();
                        *last = join![&*me, " ", &**last];
                    }
                }
                lines.append(&mut self.lines);
            }
            Scoreboard | FnCall(_) => {
                lines.append(&mut self.lines);
            }
            Comment if mcf.meta.comments => {
                lines.append(&mut self.lines.clone());
            }

            Comment | None | EOF => {}
        }
    }

    fn add_to_files(&mut self, files: &mut SaveFiles, path: String, lines: &mut Vec<String>, mcf: &mut MCFunction) {
        compile::finish_lines(lines, mcf);
        files.push((path, lines.clone()));
    }

    pub fn generate(&mut self, mcf: &mut MCFunction) {
        use crate::NodeType::*;
        match &self.node {
            Block(_) | Root => {
                self.generate_text(mcf);
            }

            Scoreboard | Command | FnCall(_) => {
                compile::node_text(self, mcf);
            }

            None | Comment | EOF => {}
        }
        self.generate_children(mcf);
    }

    fn generate_children(&mut self, mcf: &mut MCFunction) {
        self.children.iter_mut().for_each(|mut c| {
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
            for _ in 0..mcf.meta.recursive_replace {
                for i in mcf.vars.iter() {
                    self.lines[0] = self.lines[0].replace(&*["*{", &*i.0, "}"].join(""), &*i.1);
                }
            }
            self.lines[0] = compile::replacements(&self.lines[0], self, mcf, ln);

            let (rem, mut nn) = Node::build_from_lines(&mut self.lines, mcf, self.ln + ln);
            for _ in 0..min(rem, self.lines.len()) {
                self.lines.remove(0);
                ln += 1;
            }
            if let Some(mut nnu) = nn {
                self.children.push(nnu);
            }
        }
    }

    #[allow(arithmetic_overflow)]
    fn build_from_lines(lines: &mut Vec<String>, mcf: &mut MCFunction, ln: usize) -> (usize, Option<Node>) {
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
                        let (remx, mut nn) = Node::build_from_lines(lines, mcf, node.ln);
                        if let Some(mut nnu) = nn {
                            node.children.push(nnu);
                        }
                        rem = remx;
                    }
                }
            }
            "set" if require::min_args(3, &keys, mcf, ln) => {
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
                nna.lines.insert(0, join!["cmd ", &*keys[1].parse::<u32>().unwrap_or_else(|e| {
                    error(format_out(&*join!["Failed to parse '", &*keys[1], "' to a number"],
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
                if let Some(path) = compile::is_fn_call(f, mcf) {
                    node.lines = vec![join!["function ", &*path]];
                    node.node = FnCall(path);
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
                    death_error(format_out("Unterminated block", &*mcf.get_file_loc(), node.ln))
                }
            }
            Err(e) => death_error(format_out(&*e.0, &*mcf.get_file_loc(), e.1 + node.ln)),
        }
    }
}