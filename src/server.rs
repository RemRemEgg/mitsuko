use crate::*;

pub fn print_lines(input: &Datapack) {
    for (i, e) in input.lines.iter().enumerate() {
        println!("[{}] {}", i, e);
    }
}

pub fn _trim_white_space(input: Vec<String>) -> (Vec<String>, usize) {
    let mut out = Vec::new();
    let mut c: usize = 0usize;
    for i in input {
        if !(i.is_empty() || i.starts_with("//")) {
            out.push(i);
        } else {
            c = c + 1usize;
        }
    }
    (out, c)
}

pub fn warn(message: String) {
    println!("‼»   [Warning] {}", message);
}

pub fn error(message: String) -> ! {
    eprintln!("💀   [Error] {}", message);
    panic!("{}", message);
}

pub fn status(message: String) {
    println!(" »   {}", message);
}

pub fn debug(message: String) {
    println!("§»   {}", message);
}