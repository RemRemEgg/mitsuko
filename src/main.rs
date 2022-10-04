#![allow(warnings)]

mod server;
mod tests;

use regex::Regex;
use server::*;
use std::cmp::max;
use std::fmt::format;
use std::fs;
use std::time::Instant;

static VERBOSE: i32 = 1;

fn main() {
    let mut contents = "".to_string();
    if false {
        contents = fs::read_to_string("input.txt").expect("Should have been able to read the file");
    } else {
        contents = String::from(
            "/This is a comment
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
fn overthrow () {}",
        );
    }
    compile(contents);
}

fn compile(input: String) {
    status("Compiling".to_string());
    let mut s = Instant::now();
    let mut lines = input
        .split("\n")
        .collect::<Vec<&str>>()
        .iter()
        .map(|s| String::from(String::from(*s).trim()))
        .collect::<Vec<String>>();

    let t = lines.len();

    let mut pack = Datapack::new(lines, String::from("ex"));

    let (pack, _stat) = scan_pack(pack);

    print_lines(&pack);

    status(format!(
        "Finished Compiling {} lines in {} µs, {} functions",
        t,
        s.elapsed().as_micros(),
        pack.functions.len()
    ));
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
    let mut rem: usize = 1;
    let mut keys: Vec<&str> = line.trim().split(" ").collect::<Vec<_>>();
    let mut key_1: &str = keys.get(0).unwrap_or(&"§");
    match key_1 {
        "fn" => {
            let key_2 = *keys.get(1).unwrap_or(&"");
            if MCFunction::is_valid_fn(key_2) {
                rem = MCFunction::compile_from(keys, key_1, key_2, pack);
            } else {
                error(format!(
                    "Invalid function name: \'{}\' at #{}",
                    key_2, pack.ln
                ));
            }
        }
        _ => rem = scan_pack_char(line, pack),
    }
    rem
}

fn scan_pack_char(line: String, pack: &mut Datapack) -> usize {
    let mut rem: usize = 1;
    let mut char_1: char = *line
        .trim()
        .chars()
        .collect::<Vec<_>>()
        .get(0)
        .unwrap_or(&'§');
    match char_1 {
        '#' => test_arg(line, pack),
        '/' | '§' | ' ' => {
            if pack.vb >= 2 {
                warn(format!("Found non-code line at #{}", pack.ln))
            }
        }
        _ => error(format!("Unexpected token \'{}\' at #{}", char_1, pack.ln)),
    }
    rem
}

fn test_arg(line: String, pack: &mut Datapack) {
    if Regex::new("#\\[\\S+\\s*=\\s*\\S+]")
        .unwrap()
        .is_match(&line)
    {
        let s = &line[2..(line.len() - 1)].split("=").collect::<Vec<_>>();
        set_arg(s[0].trim(), s[1].trim(), pack);
    } else {
        error(format!(
            "Invalid argument tag \'{}\' at line {}",
            line, pack.ln
        ))
    }
}

fn set_arg(arg: &str, val: &str, pack: &mut Datapack) {
    let mut suc = true;
    match arg {
        "remgine" => pack.remgine = val.to_uppercase().eq("TRUE"),
        "optimizations" => pack.opt_level = max(val.parse::<u8>().unwrap_or(0u8), 4u8),
        "namespace" => pack.namespace = val.to_string(),
        _ => {
            warn(format!("Unknown arg: \'{}\' (value = \'{}\')", arg, val));
            suc = false
        }
    }
    if suc && pack.vb >= 1 {
        println!("Set arg \'{}\' to \'{}\'", arg, val);
    }
}

pub struct Datapack {
    ln: usize,
    vb: i32,
    remgine: bool,
    opt_level: u8,
    comments: bool,
    call: bool,
    namespace: String,
    lines: Vec<String>,
    functions: Vec<MCFunction>,
}

impl Datapack {
    fn new(lines: Vec<String>, namespace: String) -> Datapack {
        Datapack {
            ln: 1,
            vb: VERBOSE,
            remgine: true,
            opt_level: 0,
            comments: false,
            call: false,
            namespace,
            lines,
            functions: vec![],
        }
    }
}

pub struct MCFunction {
    lines: Vec<String>,
    commands: Vec<String>,
    path: String,
}

impl MCFunction {
    fn new(name: &str) -> MCFunction {
        MCFunction {
            lines: vec![],
            commands: vec![],
            path: name[..name.len() - 2].to_string(),
        }
    }

    fn find_block(&self, lines: &Vec<String>, ln: usize) -> usize {
        let b = Blocker::new();
        let rem = match b.find_size_vec(lines) {
            Ok(o) => o,
            Err(e) => error(e)
        };
        rem
    }

    pub fn is_valid_fn(function: &str) -> bool {
        let find = Regex::new("[A-Za-z0-9]\\S*[A-Za-z0-9]\\(\\);*")
            .unwrap()
            .find(function);
        return if find.is_none() {
            false
        } else {
            find.unwrap().start() == 0
        };
    }

    fn compile_from(keys: Vec<&str>, key_1: &str, key_2: &str, pack: &mut Datapack) -> usize {
        if *keys.get(2).unwrap_or(&"") == "{" {
            let mcf = MCFunction::new(key_2);
            if pack.vb >= 1 {
                println!("Found function \'{}\' at #{}", mcf.path, pack.ln);
            }
            let rem = mcf.find_block(&pack.lines, pack.ln);
            pack.functions.push(mcf);
            rem
        } else {
            error(format!(
                "Expected open bracket \'{} {}\'<-- [HERE] at #{}",
                key_1, key_2, pack.ln
            ));
        }
    }
}

pub struct Statement {
    line: String,
    action: String,
}

struct Blocker {
    stack: Vec<char>,
    string: bool,
}

impl Blocker {
    fn new() -> Blocker {
        Blocker {
            stack: Vec::new(),
            string: false,
        }
    }

    pub fn find_size_vec(&self, lines: &Vec<String>) -> Result<usize, String> {
        Ok(1)
    }

    pub fn find_size(&mut self, line: &String) -> Result<usize, String> {
        let mut cs = line.chars();
        let mut pos: usize = 0;
        while let Some(c) = cs.next() {
            pos += 1;
            match c {
                '\\' => { cs.nth(1); }
                '{' if !self.string => self.stack.push(c),
                '}' if !self.string => { if self.stack.last().eq(&Some(&'{')) { self.stack.pop(); } else { return Err(format!("Unexpected \'{}\' at {}", c, pos)); } },
                '(' if !self.string => self.stack.push(c),
                ')' if !self.string => { if self.stack.last().eq(&Some(&'(')) { self.stack.pop(); } else { return Err(format!("Unexpected \'{}\' at {}", c, pos)); } },
                '[' if !self.string => self.stack.push(c),
                ']' if !self.string => { if self.stack.last().eq(&Some(&'[')) { self.stack.pop(); } else { return Err(format!("Unexpected \'{}\' at {}", c, pos)); } },
                _ => {}
            }
            if self.stack.len() == 0 {
                return Ok(pos);
            }
        }
        Ok(usize::MAX)
    }
}