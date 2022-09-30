#![allow(warnings)]

#[allow(warnings)]
mod server;

use std::fs;
use server::*;
use std::time::Instant;

static DEBUG: bool = true;

fn main() {
    let contents = fs::read_to_string("input.txt").expect("Should have been able to read the file");
    /*    let contents = String::from("
    //This is a comment
    // And Another
    #[remgine=false]
    #[dead=true]

    fn main() {
        balls();

        //Should this be called?
        overthrow();
    }

    fn balls() {

    }

    // This is a bad function!!!
    fn overthrow () {}");*/
    compile(contents);
}

fn compile(input: String) {
    println!("> Compiling");
    let mut s = Instant::now();
    let mut lines = input.split("\n").collect::<Vec<&str>>().iter().map(|s| String::from(String::from(*s).trim())).collect::<Vec<String>>();

    let (lines, rem) = trim_white_space(lines);
    let t = lines.len() + rem;
    println!("> Trimmed {} lines ({}, {}%) in {} ms", rem, t, rem * 100 / t, s.elapsed().as_millis());

    let mut pack = Datapack::new(lines);

    let (pack, status) = scan_pack(pack);

    print_lines(&pack);

    println!("> Compilition  {} lines ({}, {}%) in {} ms", rem, t, rem * 100 / t, s.elapsed().as_millis());
}

fn scan_pack(mut pack: Datapack) -> (Datapack, u8) {
    'lines: loop {
        if pack.lines.len() > 0 {
            println!("Found EOF");
            break 'lines;
        }
        let remove = scan_pack_line(pack.lines[0].to_string());
        for r in 0..remove {
            pack.lines.remove(r);
        }
    }
    (pack, 0)
}

fn scan_pack_line(line: String) -> usize {
    0
}

pub struct Datapack {
    ln: i32,
    remgine: bool,
    opt_level: i32,
    call: bool,
    lines: Vec<String>,
    functions: Vec<MCFunction>,
}

impl Datapack {
    fn new(lines: Vec<String>) -> Datapack {
        Datapack { ln: 0, remgine: true, opt_level: 0, call: false, lines, functions: vec![] }
    }
}

pub struct MCFunction {
    lines: Vec<String>,
    commands: Vec<String>,
}

impl MCFunction {
    fn new() -> MCFunction {
        MCFunction { lines: vec![], commands: vec![] }
    }
}