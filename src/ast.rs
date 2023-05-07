// i prefer painting abstract trees

use std::cmp::min;
use std::slice::Iter;
use crate::{Namespace, MCFunction, error, format_out, join};
use crate::helpers::Blocker;
use crate::NodeType::{Command, Comment};

#[derive(Debug, Clone)]
pub struct Node {
    pub node: NodeType,
    children: Vec<Node>,
    pub lines: Vec<String>,
    ln: usize
}

#[derive(Debug, Clone)]
pub enum NodeType {
    None,
    Root,
    EOF,
    
    Command,
    
    Comment
}

impl Node {
    pub fn new(ty: NodeType, ln: usize) -> Node {
        Node {
            node: ty,
            children: vec![],
            lines: vec![],
            ln
        }
    }

    pub fn append_children(&mut self, childs: &mut Vec<Node>) {
        self.children.append(childs);
    }

    pub fn get_first_child(&self) -> Option<&Node> {
        self.children.get(0)
    }

    pub fn get_last_child(&self) -> Option<&Node> {
        self.children.get(self.children.len()-1)
    }

    pub fn get_child(&self, idx: usize) -> Option<&Node> {
        self.children.get(idx)
    }

    pub fn get_children(&self) -> Iter<Node> {
        self.children.iter()
    }
    
    pub fn get_write(&self, lines: &mut Vec<String>, mcf: &MCFunction) {
        use crate::NodeType::*;
        match &self.node {
            Root => {
                self.children.iter().for_each(|mut c| {
                    c.get_write(lines, mcf);
                });
            }
            
            Command => {
                lines.append(&mut self.lines.clone());
            }
            Comment if mcf.meta.comments => {
                lines.append(&mut self.lines.clone());
            }

            _ | None | EOF => {}
        }
    }
    
    pub fn compile(&mut self, mcf: &mut MCFunction) {
        use crate::NodeType::*;
        match &self.node {
            Root => {
                self.compile_lines(mcf);
            }

            Command | Comment => {
                
            }

            None | EOF => {}
        }
    }
    
    fn compile_children(&mut self, mcf: &mut MCFunction) {
        self.children.iter_mut().for_each(|mut c| {
            c.compile(mcf);
        });
    }
    
    fn compile_lines(&mut self, mcf: &mut MCFunction) {
        self.lines = self.lines.iter()
            .map(|l| if l.starts_with("@NOLEX") {
                l.replacen("@NOLEX", "", 1).trim().to_string()
            } else {l.clone()}).collect();
        let mut ln = 1;
        while self.lines.len() > 0 {
            for _ in 0..mcf.meta.recursive_replace {
                for i in mcf.vars.iter() {
                    self.lines[0] = self.lines[0].replace(&*["*{", &*i.0, "}"].join(""), &*i.1);
                }
            }
            let (rem, nn) = Node::lines_to_node(&mut self.lines, mcf, self.ln + ln);
            for _ in 0..min(rem, self.lines.len()) {
                self.lines.remove(0);
                ln += 1;
            }
            self.children.push(nn);
        }
    }
    
    #[allow(arithmetic_overflow)]
    fn lines_to_node(lines: &mut Vec<String>, mcf: &mut MCFunction, ln: usize) -> (usize, Node) {
        let keys = Blocker::new().split_in_same_level(" ", &lines[0]);
        if let Err(e) = keys {
            error(format_out(&*e, &*mcf.get_file_loc(), ln));
            return (0-1 as usize, Node::new(NodeType::None, ln));
        }
        let mut keys = keys.unwrap();
        let mut node = Node::new(NodeType::None, ln);
        match &*keys[0] {
            "cmd" => {
                node.node = Command;
                node.lines = vec![keys.get(1..).unwrap_or(&[String::new()]).join(" ")];
            }
            "set" => {
                node.node = NodeType::None;
                mcf.vars.retain(|x| !x.0.eq(&*keys[1]));
                mcf.vars
                    .insert(0, (keys[1].to_string(), keys[2..].join(" ").to_string()));
            }
            "ast" => {
                lines[0] = ["execute ", &*lines[0]].join("");
                return Node::lines_to_node(lines, mcf, ln);
            }
            "exe" => {
                lines[0] = ["execute ", &lines[0][4..]].join("");
                return Node::lines_to_node(lines, mcf, ln);
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
        (1, node)
    }
}