mod server;

use std::fs;
use server::*;

static DEBUG: bool = true;

fn main() {
    let contents = fs::read_to_string("input.txt").expect("Should have been able to read the file");
    compile(contents);
}


fn compile(input: String) {
    if DEBUG { println!("Received {}", input) };
    let mut lines = input.split("\n").collect::<Vec<&str>>().iter().map(|s| String::from(*s)).collect::<Vec<String>>();

    lines = trim_white_space(lines);

    let mut pack = Datapack::new(lines);

    print_lines(pack);
}

pub struct Datapack {
    ln : i32,
    remgine : bool,
    opt_level : i32,
    call : bool,
    lines : Vec<String>,
    functions : Vec<MCFunction>
}

impl Datapack {
    fn new(lines : Vec<String>) -> Datapack {
        Datapack { ln: 0, remgine: true, opt_level: 0, call: false, lines, functions: vec![] }
    }
}

pub struct MCFunction {
    lines : Vec<String>,
    commands : Vec<String>
}

impl MCFunction {
    fn new() -> MCFunction {
        MCFunction{ lines: vec![], commands: vec![] }
    }
}

/*
// Comment
#[var=val]
cmd say Direct Command

if (statement) {
    commands
}

eif (statement) command

 */