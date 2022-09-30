#![allow(warnings)]

#[allow(warnings)]
mod server;

use std::cmp::max;
use std::fmt::format;
use std::fs;
use server::*;
use std::time::Instant;
use regex::Regex;

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
    println!("> Trimmed {} lines ({}, {}%) in {} µs", rem, t, rem * 100 / t, s.elapsed().as_micros());

    let mut pack = Datapack::new(lines);

    let (pack, status) = scan_pack(pack);

    print_lines(&pack);

    println!("> Finished Compiling {} lines in {} µs, {} functions", t, s.elapsed().as_micros(), pack.functions.len());
}

fn scan_pack(mut pack: Datapack) -> (Datapack, u8) {
    'lines: loop {
        if pack.lines.len() <= 0 {
            println!("Found EOF");
            break 'lines;
        }
        let remove = scan_pack_line(pack.lines[0].to_string(), &mut pack);
        pack.ln += remove;
        for _ in 0..remove {
            pack.lines.remove(0);
        }
    }
    (pack, 0)
}

fn scan_pack_line(line: String, pack: &mut Datapack) -> usize {
    match line.chars().next().unwrap_or('§') {
        '#' => {
            if Regex::new("#\\[.+=.+]").unwrap().is_match(&line) {
                let s = &line[2..(line.len() - 1)].split("=").collect::<Vec<_>>();
                set_arg(s[0], s[1], pack);
            } else {
                error(format!("Invalid argument tag \'{}\' at line {}", line, pack.ln))
            }
        }
        '§' => error(format!("Hit blank string on line #{}", pack.ln)),
        _ => error(format!("Invalid token \'{}\' at line {}", line.chars().collect::<Vec<char>>()[0], pack.ln))
    }
    1
}

fn set_arg(arg: &str, val: &str, pack: &mut Datapack) {
    let mut suc = true;
    match arg {
        "remgine" => pack.remgine = val.to_uppercase().eq("TRUE"),
        "optimizations" => pack.opt_level = max(val.parse::<u8>().unwrap_or(0u8), 4u8),
        _ => {
            warn(format!("Unknown arg: {}", arg));
            suc = false
        }
    }
    if suc {
        println!("Set arg \'{}\' to \'{}\'", arg, val);
    }
}

pub struct Datapack {
    ln: usize,
    vb: bool,
    remgine: bool,
    opt_level: u8,
    call: bool,
    lines: Vec<String>,
    functions: Vec<MCFunction>,
}

impl Datapack {
    fn new(lines: Vec<String>) -> Datapack {
        Datapack { ln: 0, vb: true, remgine: true, opt_level: 0, call: false, lines, functions: vec![] }
    }
}

pub struct MCFunction {
    lines: Vec<String>,
    commands: Vec<String>,
    path: String,
}

impl MCFunction {
    fn new() -> MCFunction {
        MCFunction { lines: vec![], commands: vec![], path: "".to_string() }
    }
}