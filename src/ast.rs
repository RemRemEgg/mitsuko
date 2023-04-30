// i prefer painting abstract trees

use std::slice::Iter;

struct Node {
    node: NodeType,
    children: Vec<Node>,
    text: String,
}

enum NodeType {
    None,
    EOF,
    Keyword,
    Ident,
    Line
}

impl Node {
    fn new(ty: NodeType, text: String) -> Node {
        Node {
            node: ty,
            children: vec![],
            text
        }
    }
    
    fn append_children(&mut self, childs: &mut Vec<Node>) {
        self.children.append(childs);
    }

    fn get_first_child(&self) -> Option<&Node> {
        self.children.get(0)
    }

    fn get_last_child(&self) -> Option<&Node> {
        self.children.get(self.children.len()-1)
    }

    fn get_child(&self, idx: usize) -> Option<&Node> {
        self.children.get(idx)
    }

    fn get_children(&self) -> Iter<Node> {
        self.children.iter()
    }
}