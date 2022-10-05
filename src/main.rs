use std::cmp::max;
use regex::Regex;
use server::*;
use std::fs;
use std::time::Instant;

mod server;
mod tests;

static VERBOSE: i32 = 1;

fn main() {
    let contents;
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
    let s = Instant::now();
    let lines = input
        .split("\n")
        .collect::<Vec<&str>>()
        .iter()
        .map(|s| String::from(String::from(*s).trim()))
        .collect::<Vec<String>>();

    let t = lines.len();

    let pack = Datapack::new(lines, String::from("ex"));

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
    let rem: usize;
    let keys: Vec<&str> = line.trim().split(" ").collect::<Vec<_>>();
    let key_1: &str = keys.get(0).unwrap_or(&"§");
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
    let rem: usize = 1;
    let char_1: char = *line
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
        "namespace" => pack._namespace = val.to_string(),
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
    _comments: bool,
    _call: bool,
    _namespace: String,
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
            _comments: false,
            _call: false,
            _namespace: namespace,
            lines,
            functions: vec![],
        }
    }
}

pub struct MCFunction {
    _lines: Vec<String>,
    _commands: Vec<String>,
    path: String,
}

impl MCFunction {
    fn new(name: &str) -> MCFunction {
        MCFunction {
            _lines: vec![],
            _commands: vec![],
            path: name[..name.len() - 2].to_string(),
        }
    }

    fn extract_block(&self, lines: &Vec<String>, ln: usize) -> usize {
        let mut b = Blocker::new();
        let rem = match b.find_size_vec(lines, 0) {
            Ok(o) => { if o.0 != Blocker::NOT_FOUND {
                for i in 0..o.0 {}
                o.0
            } else { error(format!("Unterminated function: \'{}\' at #{}", self.path, ln)) } }
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
            let rem = mcf.extract_block(&pack.lines, pack.ln);
            pack.functions.push(mcf);
            rem
        } else {
            error(format!(
                "Expected open bracket \'{} {}\'<-- [HERE] at #{}",
                key_1, key_2, pack.ln
            ))
        }
    }
}

pub struct Blocker {
    stack: Vec<char>,
    string: bool,
}

impl Blocker {
    pub const NOT_FOUND: usize = 404_0000000;

    fn new() -> Blocker {
        Blocker {
            stack: Vec::new(),
            string: false,
        }
    }

    pub fn find_size_vec(&mut self, lines: &Vec<String>, offset: usize) -> Result<(usize, usize), String> {
        let mut c: usize = 0;
        loop {
            if c >= lines.len() {
                return Ok((Blocker::NOT_FOUND, 0));
            }
            let r = self.find_size(&lines[c], if c == 0 { offset } else { 0 })?;
            if r != Blocker::NOT_FOUND {
                return Ok((c, r));
            }
            c += 1;
        }
    }

    pub fn find_size(&mut self, line: &String, offset: usize) -> Result<usize, String> {
        let mut cs = line.chars();
        if offset > 0 {
            cs.nth(offset - 1);
        }
        let mut pos: usize = offset;
        while let Some(c) = cs.next() {
            pos += 1;
            match c {
                '\\' => {
                    cs.next();
                    pos += 1;
                }
                '{' if !self.string => self.stack.push(c),
                '}' if !self.string => { if self.stack.last().eq(&Some(&'{')) { self.stack.pop(); } else { return Err(format!("Unexpected \'{}\' at {}", c, pos)); } }
                '(' if !self.string => self.stack.push(c),
                ')' if !self.string => { if self.stack.last().eq(&Some(&'(')) { self.stack.pop(); } else { return Err(format!("Unexpected \'{}\' at {}", c, pos)); } }
                '[' if !self.string => self.stack.push(c),
                ']' if !self.string => { if self.stack.last().eq(&Some(&'[')) { self.stack.pop(); } else { return Err(format!("Unexpected \'{}\' at {}", c, pos)); } }
                '\'' => {
                    if self.string {
                        self.string = !self.stack.last().eq(&Some(&'\''));
                        if !self.string { self.stack.pop(); }
                    } else {
                        self.stack.push(c);
                        self.string = true;
                    }
                }
                '\"' => {
                    if self.string {
                        self.string = !self.stack.last().eq(&Some(&'\"'));
                        if !self.string { self.stack.pop(); }
                    } else {
                        self.stack.push(c);
                        self.string = true;
                    }
                }
                _ => {}
            }
            if self.stack.len() == 0 {
                return Ok(pos);
            }
        }
        Ok(Blocker::NOT_FOUND)
    }
}