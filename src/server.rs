use crate::Datapack;

pub fn print_lines(input : Datapack) {
    for (i, e) in input.lines.iter().enumerate() {
        println!("[{}] {}", i, e);
    }
}

pub fn trim_white_space(input : Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for i in input {
        if i.chars().next() != Some('\r') {
            out.push(i);
        }
    }
    out
}