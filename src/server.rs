use crate::Datapack;

pub fn print_lines(input: &Datapack) {
    for (i, e) in input.lines.iter().enumerate() {
        println!("[{}] {}", i, e);
    }
}

pub fn trim_white_space(input: Vec<String>) -> (Vec<String>, usize) {
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

pub fn warn(warning: &str) {
    println!("[Warning] {}", warning);
}

pub fn error(error: &str) -> ! {
    println!("[Error] {}", error);
    panic!();
}