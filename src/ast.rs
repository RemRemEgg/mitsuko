// i prefer painting abstract trees

use std::cmp::min;
use std::slice::Iter;
use crate::{Namespace, MCFunction, error, format_out, join, qc, death_error, SaveFiles};
use crate::helpers::{Blocker, path_without_functions};
use crate::NodeType::{Block, Command, Comment};
use crate::server::FancyText;

#[derive(Debug, Clone)]
pub struct Node {
    pub node: NodeType,
    children: Vec<Node>,
    pub lines: Vec<String>,
    ln: usize,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    None,
    Root,
    EOF,

    Command,
    Block(char),

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

    pub fn get_save_files(&mut self, files: &mut SaveFiles, lines: &mut Vec<String>, mcf: &MCFunction) {
        use crate::NodeType::*;
        match &self.node {
            Root => {
                self.children.iter_mut().for_each(|mut c| {
                    c.get_save_files(files, lines, mcf);
                });
                files.push((mcf.get_path(), lines.clone()));
            }
            Block(id) => {
                let mut blines = vec![];
                self.children.iter_mut().for_each(|mut c| {
                    c.get_save_files(files, &mut blines, mcf);
                });
                if blines.len() <= 1 && mcf.meta.opt_level >= 1 {
                    lines.push(blines.join(" "));
                } else {
                    let path = join![&*mcf.get_path(), ".", &*id.to_string(), &*self.ln.to_string()];
                    let path = path.strip_prefix("/").unwrap_or(&*path);
                    lines.push(join!("function ", &*mcf.ns_id, ":", &*path));
                    files.push((path.into(), blines.clone()));
                }
            }

            Command => {
                self.children.iter_mut().for_each(|mut c| {
                    c.get_save_files(files, &mut self.lines, mcf);
                });
                lines.push(self.lines.join(" "));
            }
            Comment if mcf.meta.comments => {
                lines.append(&mut self.lines.clone());
            }

            _ | None | EOF => {}
        }
    }

    pub fn generate(&mut self, mcf: &mut MCFunction) {
        use crate::NodeType::*;
        match &self.node {
            Block(_) | Root => {
                self.build_text(mcf);
            }

            Command => {
                self.compile_text(mcf);
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

    fn build_text(&mut self, mcf: &mut MCFunction) {
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
            "ast" if require::min_args(2, &keys, mcf, ln) => {
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
                    if lines[0][run+5..run+5].eq("{") {
                        rem = Node::build_extract_block(lines, &mut node, mcf, 'e');
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
            "if" if require::min_args(2, &keys, mcf, ln) => {
                
            }
            "set" if require::min_args(3, &keys, mcf, ln) => {
                node.node = NodeType::None;
                mcf.vars.retain(|x| !x.0.eq(&*keys[1]));
                mcf.vars
                    .insert(0, (keys[1].to_string(), keys[2..].join(" ").to_string()));
            }
            "{" => {
                node.node = Command;
                rem = Node::build_extract_block(lines, &mut node, mcf, 'b');
                node.lines = vec![];
            }
            _ if MCFunction::is_score_path(&keys[0], mcf, ln) => {
                node.node = Command;
                if keys.len() > 1 {
                    // if keys.len() >= 3 {
                    //     if keys[1].eq("result") || keys[1].eq("success") {
                    //         *text = keys[2..].join(" ");
                    //         let target = &*self.compile_score_path(&keys[0].to_string(), ns, ln);
                    //         let command = join!["execute store ", keys[1], " score", target, " run "];
                    //         let (res, mut fun, (mut funs2, mut warn)) = self.code_to_function(ns, ln, &keys[2][0..2]);
                    //         self.calls.append(&mut fun.calls);
                    //         funs.append(&mut funs2);
                    //         warns.append(&mut warn);
                    //         if fun.commands.len() > 1 {
                    //             cmds.push(join![&*command, &*fun.get_callable(ns)]);
                    //             funs.push(fun);
                    //         } else {
                    //             if fun.commands.len() == 0 {
                    //                 cmds.push(join!["# ", &*command, "<code produced no result>"]);
                    //             } else {
                    //                 cmds.push(join![&*command, &*fun.get_callable(ns)]);
                    //             }
                    //         }
                    //         res
                    //     } else {
                    //         cmds.append(&mut self.compile_score_command(&keys, ns, ln));
                    //         1
                    //     }
                    // } else {
                    //     cmds.append(&mut self.compile_score_command(&keys, ns, ln));
                    //     1
                    // }
                } else {
                    let target = &*MCFunction::compile_score_path(&keys[0].to_string(), mcf, ln);
                    node.lines = vec![join!("scoreboard players get ", target)];
                }
            }
            _ if keys[0].starts_with("//") => {
                node.node = Comment;
                node.lines = vec![join!["#", &lines[0][2..]]];
            }
            _ => {
                node.node = Command;
                node.lines = vec![lines[0].clone()];
            }
        }
        (rem, if !node.node.is_none() { Some(node) } else { None })
    }

    fn build_extract_block(lines: &mut Vec<String>, node: &mut Node, mcf: &mut MCFunction, ident: char) -> usize {
        let mut b = Blocker::new();
        let rem = match b.find_size_vec(lines, (0, lines[0].rfind("{").unwrap_or(0))) {
            Ok(o) => {
                if o.0 != Blocker::NOT_FOUND {
                    let mut nna = Node::new(NodeType::Block(ident), node.ln + 1);
                    lines[1..o.0].clone_into(&mut nna.lines);
                    node.children.push(nna);
                    o.0 + 1
                } else {
                    death_error(format_out("Unterminated block", &*mcf.get_file_loc(), node.ln))
                }
            }
            Err(e) => death_error(format_out(&*e.0, &*mcf.get_file_loc(), e.1 + node.ln)),
        };
        rem
    }

    fn compile_text(&mut self, mcf: &mut MCFunction) {
        if self.lines.len() == 0 {
            return;
        }
        match &self.node {
            Command => {
                let keys = Blocker::new().split_in_same_level(" ", &self.lines[0]);
                if let Err(e) = keys {
                    error(format_out(&*e, &*mcf.get_file_loc(), self.ln));
                    return;
                }
                let mut keys = keys.unwrap();
                match &*keys[0] {
                    "cmd" => {
                        self.lines = vec![keys.get(1..).unwrap_or(&[String::new()]).join(" ")];
                    }
                    "ast" => {
                        self.lines[0] = ["execute ", &*self.lines[0]].join("");
                        self.compile_text(mcf);
                    }
                    "exe" => {
                        self.lines[0] = ["execute ", &self.lines[0][4..]].join("");
                        self.compile_text(mcf);
                    }
                    "execute" => {
                        let mut blk = Blocker::new();
                        while let Ok(Some(pos1)) = blk.reset().find_in_same_level(" ast ", &self.lines[0]) {
                            if let Ok(Some(pos2)) = blk.reset().find_in_same_level(" ", &self.lines[0][pos1 + 5..].into()) {
                                self.lines[0].insert_str(pos1 + 5 + pos2, " at @s");
                                self.lines[0].replace_range(pos1..pos1 + 5, " as ");
                            } else {
                                break;
                            }
                        }
                    }
                    "create" => {
                        if keys.len() == 2 {
                            keys.push("dummy".into());
                        }
                        if keys[1].starts_with("&") {
                            keys[1].replace_range(0..1, ".");
                            keys[1].replace_range(0..0, &*mcf.ns_id);
                        }
                        self.lines[0] = join!["scoreboard objectives add ", &*keys[1..].join(" ")];
                    }
                    "remove" => {
                        if keys[1].starts_with("&") {
                            keys[1].replace_range(0..1, ".");
                            keys[1].replace_range(0..0, &*mcf.ns_id);
                        }
                        self.lines[0] = join!["scoreboard objectives remove ", &*keys[1]];
                    }
                    mut f @ _ => {//todo
                        if let Some(path) = Node::is_fn_call(f, mcf) {
                            self.lines[0] = join!["function ", & * path];
                        }
                    }
                }
            }
            NodeType::None | NodeType::EOF => {
                self.lines = vec![];
            }
            _ => {}
        }
    }

    fn is_fn_call(call: &str, mcf: &mut MCFunction) -> Option<String> {
        let mut call = call.clone();
        if call.len() < 2 { return None; };
        let local = call.starts_with("&");
        let tag = call.starts_with("#");
        call = call[(local as usize + tag as usize)..].into();
        return if MCFunction::is_valid_fn(&*call) {
            call = call.trim_end_matches("()");
            let (ns, name) = call.split_once(":").unwrap_or((&*mcf.ns_id, call));
            let path = qc!(local, path_without_functions(mcf.file_path.clone()), "".into());
            Some(join![qc!(tag, "#", ""), ns, ":", &*path, qc!(local && path.len() > 0, "/", ""), name])
        } else {
            None
        };
    }
}

pub mod require {
    use std::fmt::{Debug, Display};
    use crate::{error, format_out, join, MCFunction};
    use crate::server::FancyText;

    pub fn min_args<T: Display>(count: usize, keys: &Vec<T>, mcf: &mut MCFunction, ln: usize) -> bool {
        if keys.len() < count {
            error(format_out(&*format!("Not enough arguments for '{}' ({} expected, found {})", keys[0], count, keys.len()), &*mcf.get_file_loc(), ln))
        }
        keys.len() >= count
    }

    pub fn exact_args<T: Display>(count: usize, keys: &Vec<T>, mcf: &mut MCFunction, ln: usize) -> bool {
        if keys.len() != count {
            error(format_out(&*format!("Wrong number of arguments for '{}' ({} expected, found {})", keys[0], count, keys.len()), &*mcf.get_file_loc(), ln))
        }
        keys.len() == count
    }
    
    pub fn remgine(item: &str, mcf: &mut MCFunction, ln: usize) -> bool {
        if !mcf.meta.remgine {
            error(format_out(&*join!("Remgine is required to use [", &*item.form_foreground(str::AQU), "]"), &*mcf.get_file_loc(), ln))
        }
        mcf.meta.remgine
    }
}