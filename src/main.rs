use std::cmp::min;
use std::fmt::{Display, Formatter};
use regex::Regex;
use server::*;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::thread::sleep;
use std::time::{Duration, Instant};

mod server;
mod tests;

static VERBOSE: i32 = 0;
static CURRENT_PACK_VERSION: u8 = 10;

fn main() {
    let contents;
    if 1 == 1 {
        contents = fs::read_to_string("input.txt").expect("Should have been able to read the file");
    } else {
        contents = String::from(
            "// This is a comment
// And Another
#[remgine=false]
#[namespace=ms]
#[dead=true]

fn main() {
}
fn init() {}
fn over_load() {}
fn baller() {
    balls()

    // Should this be called?
    overthrow()
}

fn balls() {

}

// This is a bad function!!!
fn overthrow() {}
",
        );
    }
    let pack = make_pack(contents);

    save_pack(pack);

    status("Done".to_string());
    sleep(Duration::from_secs(0));
}

fn make_pack(input: String) -> Datapack {
    status("Processing...\n".to_string());
    let t_total = Instant::now();
    let lines = input
        .split("\n")
        .collect::<Vec<&str>>()
        .iter()
        .map(|s| String::from((*s).trim()))
        .collect::<Vec<String>>();

    let mut pack = Datapack::new(lines, String::from("ex:"));

    /* SCAN */
    let t_scan = Instant::now();
    pack = scan_pack(pack);
    status(format!(
        "Scanned {} functions in {} µs\n",
        pack.functions.len(),
        t_scan.elapsed().as_micros()
    ));

    /* COMPILE */
    let t_compile = Instant::now();
    let mut ln_total = 0;
    pack.functions.iter().for_each(|f| {
        ln_total += f.lines.len();
    });
    pack = compile_pack(pack);
    let mut cm_total = 0;
    pack.functions.iter().for_each(|f| {
        cm_total += f.commands.len();
    });
    status(format!(
        "Compiled {} lines into {} commands within {} µs\n",
        ln_total,
        cm_total,
        t_compile.elapsed().as_micros()
    ));


    /* CLEAN */
    let t_clean = Instant::now();
    pack = clean_pack(pack);
    status(format!(
        "Cleaned up in {} µs\n",
        t_clean.elapsed().as_micros()
    ));

    /* FINISH */
    print_warnings(&pack);
    status(format!(
        "Finished {} [opt {} + {}remgine] in {} µs",
        pack,
        pack.settings.opt_level,
        if pack.settings.remgine { "" } else { "no " },
        t_total.elapsed().as_micros()
    ));
    pack
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
    status("Generated Function Data".to_string());
    pack
}

fn scan_pack_line(line: String, pack: &mut Datapack) -> usize {
    let rem: usize;
    let keys: Vec<&str> = line.trim().split(" ").collect::<Vec<_>>();
    let key_1: &str = keys.get(0).unwrap_or(&"§");
    match key_1 {
        "fn" => {
            let key_2 = *keys.get(1).unwrap_or(&"");
            if MCFunction::is_valid_fn(key_2) && !key_2.contains(":") {
                rem = MCFunction::extract_from(keys, key_1, key_2, pack);
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
            // if pack.settings.vb >= 2 {
            //     debug(format!("Found non-code line @{}", pack.ln))
            // }
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
        "remgine" => pack.settings.remgine = val.to_uppercase().eq("TRUE"),
        "optimizations" => pack.settings.opt_level = min(val.parse::<u8>().unwrap_or(0u8), 4u8),
        "namespace" => pack.settings.namespace = val.to_string() + ":",
        "name" => pack.settings.name = val.to_string(),
        "debug" => pack.settings.vb = min(val.parse::<i32>().unwrap_or(0), 3),
        "comments" => pack.settings.comments = val.to_uppercase().eq("TRUE"),
        "version" => pack.settings.version = val.parse::<u8>().unwrap_or(CURRENT_PACK_VERSION),
        "description" => pack.settings.description = val.to_string(),
        _ => {
            if pack.settings.vb >= 1 { warn(format!("Unknown arg: \'{}\' (value = \'{}\') @{}", arg, val, pack.ln), &mut pack); }
            suc = false
        }
    }
    if suc && pack.settings.vb >= 1 {
        debug(format!("Set arg \'{}\' to \'{}\'", arg, val));
    }
}


fn compile_pack(mut pack: Datapack) -> Datapack {
    pack.ln = 0;
    'functions: loop {
        if pack.ln >= pack.functions.len() {
            break 'functions pack;
        }
        if pack.settings.vb >= 1 {
            debug(format!("Compiling Function '{}'", pack.functions[pack.ln].path));
        }
        pack.functions[pack.ln].compile(&pack.settings);
        pack.ln += 1;
    }
}

fn clean_pack(mut pack: Datapack) -> Datapack {
    for fi in 0..pack.functions.len() {
        for ci in 0..pack.functions[fi].calls.len() {
            let c = &pack.functions[fi].calls[ci];
            if !pack.functions.iter().any(|f| -> bool { f.path.eq(&c.0) }) {
                warn(format!("Unknown or undefined function '{}' found @{}", c.0, c.1), &mut pack);
            }
        }
        for i in 0..pack.functions[fi].commands.len() {
            pack.functions[fi].commands[i] = pack.functions[fi].commands[i].replace(" run execute", "");
        }
    }
    pack
}

fn save_pack(pack: Datapack) {
    let save_time = Instant::now();
    if pack.settings.vb >= 1 {
        status(format!(
            "Saving '{}' @ '{}/generated/'",
            &pack.settings.name,
            match std::env::current_exe() {
                Ok(mut path) => {
                    path.pop();
                    path.to_str().unwrap_or("Unknown").to_string().replace('\\',"/")
                }
                Err(err) => { err.to_string() }
            }
        ));
    } else {
        status(format!(
            "Saving '{}'",
            &pack.settings.name
        ));
    }

    let root_path = "./generated/".to_string() + &*pack.settings.name;
    let pack_path = &*[&*root_path, "/data/", &pack.settings.namespace[0..(pack.settings.namespace.len() - 1)]].join("");

    make_folder(&*root_path);

    let mut meta = File::create([&*root_path, "/pack.mcmeta"].join("")).expect("Could not make 'pack.mcmeta'");
    let meta_template = include_str!("pack.mcmeta").replace("{VERS}", &*pack.settings.version.to_string()).replace("{DESC}", &pack.settings.description);
    meta.write_all(meta_template.as_bytes()).expect("Could not make 'pack.mcmeta'");

    make_folder(&*pack_path);

    let fn_path = &*[&*pack_path, "/functions"].join("");

    make_folder(&*fn_path);

    for function in pack.functions {
        let path = &*[fn_path, "/", &*function.path, ".mcfunction"].join("");
        if function.path.contains("/") {
            let mut path = function.path.split("/").collect::<Vec<_>>();
            path.pop();
            path.insert(0, "/");
            path.insert(0, &*fn_path);
            make_folder(&*path.join("/"));
        }
        let mut file = File::create(path).expect(&*["Could not make function '", path, "'"].join(""));
        file.write_all(function.commands.join("\n").as_bytes()).expect(&*["Could not write function '", path, "'"].join(""));
    }

    status(format!(
        "Saved Datapack in {} µs",
        save_time.elapsed().as_micros()
    ));
}

fn make_folder(path: &str) {
    fs::create_dir_all(path).unwrap_or_else(|e| {
        error(format!("Could not generate '{path}' folder: {e}"));
    });
}

struct Settings {
    vb: i32,
    version: u8,
    remgine: bool,
    opt_level: u8,
    comments: bool,
    namespace: String,
    name: String,
    description: String,
}

pub struct Datapack {
    settings: Settings,
    ln: usize,
    lines: Vec<String>,
    functions: Vec<MCFunction>,
    warnings: Vec<String>,
}

impl Datapack {
    fn new(lines: Vec<String>, namespace: String) -> Datapack {
        Datapack {
            settings: Settings {
                vb: VERBOSE,
                version: CURRENT_PACK_VERSION,
                remgine: true,
                opt_level: 0,
                comments: false,
                namespace,
                name: "Untitled".to_string(),
                description: "A Datapack".to_string(),
            },
            ln: 1,
            lines,
            functions: vec![],
            warnings: vec![],
        }
    }
}

impl Display for Datapack {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Datapack['{}' @ '{}' ({}l {}f)]", self.settings.name, self.settings.namespace, self.lines.len(), self.functions.len()))
    }
}

pub struct MCFunction {
    lines: Vec<String>,
    commands: Vec<String>,
    path: String,
    calls: Vec<(String, usize)>,
    ln: usize,
}

impl MCFunction {
    fn new(name: &str, ln: usize) -> MCFunction {
        MCFunction {
            lines: vec![],
            commands: vec![],
            path: name[..name.len() - 2].to_string(),
            calls: vec![],
            ln,
        }
    }

    fn extract_block(&mut self, lines: &Vec<String>, _ln: usize) -> usize {
        if lines[0].ends_with('}') {
            return 1;
        }
        let mut b = Blocker::new();
        let rem = match b.find_rapid_close(lines, '}') {
            Ok(o) => {
                if o != Blocker::NOT_FOUND {
                    for i in 1..o {
                        self.lines.push(lines[i].to_string());
                    }
                    o + 1
                } else { error(format!("Unterminated function: \'{}\'", self.path)) }
            }
            Err(e) => error(e)
        };
        rem
    }

    pub fn is_valid_fn(function: &str) -> bool {
        let find = Regex::new("([a-z_]+:)?([a-z0-9_][a-z0-9_/]*[a-z0-9_]|[a-z0-9_])\\(\\)")
            .unwrap()
            .find(function);
        return if find.is_none() {
            false
        } else {
            find.unwrap().start() == 0
        };
    }

    fn extract_from(keys: Vec<&str>, key_1: &str, key_2: &str, pack: &mut Datapack) -> usize {
        if keys.get(2).unwrap_or(&"").starts_with("{") {
            let mut mcf = MCFunction::new(key_2, pack.ln);
            if pack.functions.iter().any(|fun| -> bool { fun.path.eq(&mcf.path) }) {
                error(format!("Duplicate function name \'{}\' @{}", mcf.path, pack.ln));
            }
            let rem = mcf.extract_block(&pack.lines, pack.ln);
            if pack.settings.vb >= 1 {
                debug(format!("Found function \'{}\' @{}", mcf.path, pack.ln));
                if pack.settings.vb >= 2 {
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

    fn compile(&mut self, settings: &Settings) {
        let mut ln = 1;
        'lines: loop {
            if self.lines.len() == 0 {
                break 'lines;
            }
            let rem = self.compile_line(settings, ln);
            for _ in 0..rem {
                self.lines.remove(0);
            }
            ln += rem;
        }
        if settings.vb >= 2 {
            debug(format!(" -> Resulted in {} commands", self.commands.len()));
        }
    }

    fn compile_line(&mut self, settings: &Settings, ln: usize) -> usize {
        let line = self.lines[0].to_owned();
        let keys = line.split(" ").collect::<Vec<_>>();
        if keys.len() == 0 {
            return 1;
        }
        match keys[0] {
            "cmd" => {
                self.commands.push(line[4..].to_string());
                1
            }
            "//" if settings.comments => {
                self.commands.push(["#", &line[2..]].join(""));
                1
            }
            "" if settings.comments => {
                self.commands.push("".to_string());
                1
            }
            f @ _ => {
                return if MCFunction::is_valid_fn(f) {
                    let fnn = f[..f.len() - 2].to_string();
                    self.calls.push((fnn, self.ln + ln));
                    self.commands.push(["function ", if f.contains(":") { "" } else { &settings.namespace }, f].join(""));
                    1
                } else {
                    self.lines[0] = ["cmd ", &*self.lines[0]].join("");
                    0
                };
            }
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

    pub fn find_rapid_close(&mut self, lines: &Vec<String>, closer: char) -> Result<usize, String> {
        let mut c: usize = 0;
        loop {
            if c >= lines.len() {
                return Ok(Blocker::NOT_FOUND);
            }
            if lines[c].trim().starts_with(closer) {
                return Ok(c);
            }
            c += 1;
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
        let mut pos: usize = 0;
        while let Some(c) = cs.next() {
            pos += 1;
            match c {
                '\\' => {
                    cs.next();
                    pos += 1;
                }
                '{' if !self.string => self.stack.push(c),
                '}' if !self.string => { if self.stack.last().eq(&Some(&'{')) { self.stack.pop(); } else { return Err(format!("Unexpected \'{}\' @({})", c, pos + offset)); } }
                '(' if !self.string => self.stack.push(c),
                ')' if !self.string => { if self.stack.last().eq(&Some(&'(')) { self.stack.pop(); } else { return Err(format!("Unexpected \'{}\' @({})", c, pos + offset)); } }
                '[' if !self.string => self.stack.push(c),
                ']' if !self.string => { if self.stack.last().eq(&Some(&'[')) { self.stack.pop(); } else { return Err(format!("Unexpected \'{}\' @({})", c, pos + offset)); } }
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
                '/' => {
                    if cs.next().unwrap_or(' ').eq(&'/') {
                        return Ok(Blocker::NOT_FOUND);
                    } else {
                        cs = line.chars();
                        cs.nth(pos - 1);
                    }
                }
                _ => {}
            }
            if self.stack.len() == 0 && !self.string {
                return Ok(pos + offset);
            }
        }
        Ok(Blocker::NOT_FOUND)
    }
}