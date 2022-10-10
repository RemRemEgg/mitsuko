use std::cmp::min;
use std::fmt::{Display, Formatter};
use regex::Regex;
use server::*;
use std::fs;
use std::time::Instant;

mod server;
mod tests;

static VERBOSE: i32 = 0;

fn main() {
    let contents;
    if true {
        contents = fs::read_to_string("input.txt").expect("Should have been able to read the file");
    } else {
        contents = String::from(
            "///This is a comment
// And Another
#[remgine=false]
#[namespace=ms]
#[dead=true]

fn main() {
fn init() {
fn over_load() {
fn baller()
}
}
    balls();

    //Should this be called?
    overthrow();
}

fn balls() {

}

// This is a bad function!!!
fn overthrow () {}
",
        );
    }
    make_pack(contents);
}

fn make_pack(input: String) {
    status("Compiling".to_string());
    let t_total = Instant::now();
    let lines = input
        .split("\n")
        .collect::<Vec<&str>>()
        .iter()
        .map(|s| String::from(String::from(*s).trim()))
        .collect::<Vec<String>>();

    let t = lines.len();

    let mut pack = Datapack::new(lines, String::from("ex"));

    let t_scan = Instant::now();
    pack = scan_pack(pack);
    status(format!(
        "Scanned {} functions in {} µs\n",
        pack.functions.len(),
        t_scan.elapsed().as_micros()
    ));

    let t_compile = Instant::now();
    pack = compile_pack(pack);
    status(format!(
        "Compiled {} lines in {} µs\n",
        t,
        t_compile.elapsed().as_micros()
    ));


    let t_clean = Instant::now();
    pack = clean_pack(pack);
    status(format!(
        "Cleaned up in {} µs\n",
        t_clean.elapsed().as_micros()
    ));

    status(format!(
        "Finished {} [opt {} + {}remgine] in {} µs\n",
        pack,
        pack.opt_level,
        if pack.remgine { "" } else { "no " },
        t_total.elapsed().as_micros()
    ));

    print_warnings(&pack);
}

fn scan_pack(mut pack: Datapack) -> Datapack {
    'lines: loop {
        if pack.lines.len() <= 0 {
            status("Found EOF".to_string());
            break 'lines;
        }
        let remove = scan_pack_line(pack.lines[0].to_string(), &mut pack);
        pack.ln += remove;
        for _ in 0..remove {
            pack.lines.remove(0);
        }
    }
    if !pack.functions.iter().any(|fun| -> bool { fun.path.eq("main") }) {
        warn("No 'main' function found, is this intentional?".to_string(), &mut pack);
    }
    if !pack.functions.iter().any(|fun| -> bool { fun.path.eq("init") }) {
        warn("No 'init' function found, is this intentional?".to_string(), &mut pack);
    }
    status("Generated Function Data\n".to_string());
    pack
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
                    "Invalid function name: \'{}\' @{}",
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
            if pack.vb >= 3 {
                debug(format!("Found non-code line @{}", pack.ln))
            }
        }
        _ => error(format!("Unexpected token \'{}\' @{}", char_1, pack.ln)),
    }
    rem
}

fn test_arg(line: String, pack: &mut Datapack) {
    if Regex::new("#\\[\\S+\\s*=\\s*[\\S _]+]")
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

fn set_arg(arg: &str, val: &str, mut pack: &mut Datapack) {
    let mut suc = true;
    match arg {
        "remgine" => pack.remgine = val.to_uppercase().eq("TRUE"),
        "optimizations" => pack.opt_level = min(val.parse::<u8>().unwrap_or(0u8), 4u8),
        "namespace" => pack.namespace = val.to_string(),
        "name" => pack.name = val.to_string(),
        "debug" => pack.vb = min(val.parse::<i32>().unwrap_or(0), 3),
        _ => {
            if pack.vb >= 1 { warn(format!("Unknown arg: \'{}\' (value = \'{}\') @{}", arg, val, pack.ln), &mut pack); }
            suc = false
        }
    }
    if suc && pack.vb >= 1 {
        debug(format!("Set arg \'{}\' to \'{}\'", arg, val));
    }
}


fn compile_pack(mut pack: Datapack) -> Datapack {
    for fi in 0..pack.functions.len() {
        compile_function(fi, &mut pack);
    }
    pack
}

fn compile_function(fi: usize, mut pack: &mut Datapack) {
    let p = &*pack;
    let mut function = &mut pack.functions[fi];
    while function.lines.len() > 0 {
        let line = &function.lines[0];
        let mut rem = compile_function_line(&function.lines, p);
        for _ in 0..rem {
            function.lines.remove(0);
        }
    }
}

fn compile_function_line(lines: &Vec<String>, pack : &Datapack) -> usize {
    1
}

fn clean_pack(mut pack: Datapack) -> Datapack {
    for fi in 0..pack.functions.len() {
        for ci in 0..pack.functions[fi].calls.len() {
            let c = &pack.functions[fi].calls[ci];
            if !pack.functions.iter().any(|f| -> bool { f.path.eq(&c.0) }) {
                warn(format!("No such function '{}' found @{}", c.0, c.1), &mut pack);
            }
        }
    }
    pack
}

pub struct Datapack {
    ln: usize,
    vb: i32,
    remgine: bool,
    opt_level: u8,
    _comments: bool,
    _call: bool,
    namespace: String,
    name: String,
    lines: Vec<String>,
    functions: Vec<MCFunction>,
    warnings: Vec<String>,
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
            namespace,
            name: "Untitled".to_string(),
            lines,
            functions: vec![],
            warnings: vec![],
        }
    }
}

impl Display for Datapack {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Datapack['{}' @ '{}' ({}l {}f)]", self.name, self.namespace, self.lines.len(), self.functions.len()))
    }
}

pub struct MCFunction {
    lines: Vec<String>,
    _commands: Vec<String>,
    path: String,
    calls: Vec<(String, usize)>,
}

impl MCFunction {
    fn new(name: &str) -> MCFunction {
        MCFunction {
            lines: vec![],
            _commands: vec![],
            path: name[..name.len() - 2].to_string(),
            calls: vec![],
        }
    }

    fn extract_block(&mut self, lines: &Vec<String>, _ln: usize) -> usize {
        let mut b = Blocker::new();
        let rem = match b.find_size_vec(lines, lines[0].find('{').unwrap_or(0)) {
            Ok(o) => {
                if o.0 != Blocker::NOT_FOUND {
                    for i in 1..o.0 {
                        self.lines.push(lines[i].to_string());
                    }
                    o.0 + 1
                } else { error(format!("Unterminated function: \'{}\' within {}", self.path, self.path)) }
            }
            Err(e) => error(e)
        };
        rem
    }

    pub fn is_valid_fn(function: &str) -> bool {
        let find = Regex::new("([a-z0-9_][a-z0-9_/]*[a-z0-9_]|[a-z0-9_])\\(\\);*")
            .unwrap()
            .find(function);
        return if find.is_none() {
            false
        } else {
            find.unwrap().start() == 0
        };
    }

    fn compile_from(keys: Vec<&str>, key_1: &str, key_2: &str, pack: &mut Datapack) -> usize {
        if keys.get(2).unwrap_or(&"").starts_with("{") {
            let mut mcf = MCFunction::new(key_2);
            if pack.functions.iter().any(|fun| -> bool { fun.path.eq(&mcf.path) }) {
                error(format!("Duplicate function name \'{}\' @{}", mcf.path, pack.ln));
            }
            let rem = mcf.extract_block(&pack.lines, pack.ln);
            if pack.vb >= 1 {
                debug(format!("Found function \'{}\' @{}", mcf.path, pack.ln));
                if pack.vb >= 2 {
                    debug(format!(" -> {} Lines REM", rem));
                }
            }
            pack.functions.push(mcf);
            rem
        } else {
            error(format!(
                "Expected '{{' after \'{} {}\' @{}",
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
            let r = self.find_size(&lines[c], if c == 0 { offset } else { 0 }).map_err(|mut e| {
                e.push_str(&*(c + offset - 1).to_string());
                e
            })?;
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
                '}' if !self.string => { if self.stack.last().eq(&Some(&'{')) { self.stack.pop(); } else { return Err(format!("Unexpected \'{}\' @({})", c, pos)); } }
                '(' if !self.string => self.stack.push(c),
                ')' if !self.string => { if self.stack.last().eq(&Some(&'(')) { self.stack.pop(); } else { return Err(format!("Unexpected \'{}\' @({})", c, pos)); } }
                '[' if !self.string => self.stack.push(c),
                ']' if !self.string => { if self.stack.last().eq(&Some(&'[')) { self.stack.pop(); } else { return Err(format!("Unexpected \'{}\' @({})", c, pos)); } }
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
            if self.stack.len() == 0 && !self.string {
                return Ok(pos);
            }
        }
        Ok(Blocker::NOT_FOUND)
    }
}