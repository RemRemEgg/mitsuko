// i need more than just programming help

use std::collections::hash_map::DefaultHasher;
use std::fs::{DirEntry, read_dir, ReadDir};
use std::ffi::OsStr;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::{io, vec};
use std::path::Path;
use std::vec::IntoIter;
use remtools::{colors::*, *};
use crate::*;
use crate::CachedType::*;

pub static COMMANDS: [&str; 66] = ["return", "advancement", "attribute", "bossbar", "clear", "clone", "data", "datapack", "debug", "defaultgamemode", "difficulty",
    "effect", "enchant", "execute", "experience", "fill", "forceload", "function", "gamemode", "gamerule", "give", "help", "kick", "kill",
    "list", "locate", "loot", "me", "msg", "particle", "playsound", "publish", "recipe", "reload", "item", "say", "schedule", "scoreboard",
    "seed", "setblock", "setworldspawn", "spawnpoint", "spectate", "spreadplayers", "stopsound", "summon", "tag", "team", "teammsg", "teleport",
    "tell", "tellraw", "time", "title", "tm", "tp", "trigger", "weather", "worldborder", "xp", "jfr", "place", "fillbiome", "ride", "damage", "random"];

pub static DEFAULT_REPLACEMENTS: [&str; 6] = ["NAME", "INT_MAX", "INT_MIN", "PATH", "NEAR1", "LN"];

static mut WARNINGS: Vec<String> = vec![];
pub static mut O_GEN_FRAGMENTS: Vec<CachedFrag> = vec![];
pub static mut I_CACHED_MSK: CacheFiles = vec![];
pub static mut SUPPRESS_WARNINGS: bool = false;
pub static mut KNOWN_FUNCTIONS: Vec<String> = vec![];
pub static mut EXPORT_FUNCTIONS: Vec<String> = vec![];
pub static mut HIT_ERROR: i32 = 0;

pub mod errors {
    pub const UNKNOWN_ERROR: i32 = -86;
    pub const BAD_CLI_OPTIONS: i32 = 10;
    pub const NO_PACK_MSK: i32 = 30;
    pub const IMPORT_NOT_FOUND: i32 = 40;
    pub const AST_ERROR: i32 = 50;
}


pub fn print_warnings(pack: &Datapack) {
    unsafe {
        if WARNINGS.len() > 0 && !SUPPRESS_WARNINGS {
            println!();
            status(format!(
                "'{}' Generated {} Warning(s)",
                pack.get_view_name(),
                WARNINGS.len()
            ).foreground(ORN).end());
        }
        if SUPPRESS_WARNINGS {
            SUPPRESS_WARNINGS = false;
            warn("SUPPRESS_WARNINGS is turned on!".into());
            SUPPRESS_WARNINGS = true;
        }
    }
}

pub fn warn(message: String) {
    unsafe {
        if !SUPPRESS_WARNINGS {
            println!("{}", join!("\x1b[93m‼»\x1b[m   [", &*"Warning".foreground(ORN).modifier(BOLD).end(), "] ", &*message));
        }
        WARNINGS.push(message);
    }
}

unsafe fn _print_warning(message: String, i: usize) {
    println!("\x1b[93m‼»\x1b[m   [{}] {}", (i + 1).to_string().foreground(ORN).end(), message);
}

pub fn format_out(message: &str, path: &str, ln: usize) -> String {
    message.to_string() + " " + &join!["./src/", path, ".msk:", &*ln.to_string(), ""].replace("\\", "/").replace("/.msk", "/functions.msk")
        .modifier(UNDERLINE).foreground(GRY).end()
}

pub fn death_error(message: String, etype: i32) -> ! {
    error(message);
    status_color("Aborting".into(), RED);
    exit(etype);
}

pub fn soft_error(message: String) {
    eprintln!("{}", join!("!   [", &*"Soft Error".foreground(RED).modifier(ITALIC).modifier(BOLD).end(), "] ", &*message));
}

pub fn error(message: String) {
    unsafe { HIT_ERROR += 1 }
    eprintln!("{}", join!("⮾   [", &*"Error".foreground(RED).modifier(ITALIC).modifier(BOLD).end(), "] ", &*message));
}

pub fn status(message: String) {
    println!(" »   {}", message);
}

pub fn status_color(message: String, color: u8) {
    println!(" »   {}", message.foreground(color).end());
}

pub fn debug(message: String) {
    println!("\x1b[96m§»\x1b[m   {}", message);
}

pub static META_TEMPLATE: &str = include_str!("pack.mcmeta");
pub static TAG_TEMPLATE: &str = include_str!("tag.json");
pub static RECIPE_TEMPLATE: &str = include_str!("recipe.json");
pub static ADV_CRAFT_TEMPLATE_119: &str = include_str!("advancement_craft_1.19.json");
pub static ADV_CRAFT_TEMPLATE_120: &str = include_str!("advancement_craft_1.20.json");
pub static MAT_TEMPLATE: &str = r#""$ID$": {"item": "minecraft:$TYPE$"}"#;
pub static MAT_TAG_TEMPLATE: &str = r#""$ID$": {"tag": "minecraft:$TYPE$"}"#;

pub static mut DATAROOT: String = String::new();
pub static mut GEN_LOC: String = String::new();

pub fn read_src<T: ToString>(loc: T) -> io::Result<ReadDir> {
    read_dir(get_src_dir(loc))
}

pub fn get_src_dir<T: ToString>(loc: T) -> String {
    join![unsafe {&*SRC}, &*loc.to_string()]
}

pub fn make_folder(path: &str) {
    fs::create_dir_all(path).unwrap_or_else(|e| {
        error(format!("Could not generate '{path}' folder: {e}"));
    });
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn get_msk_files_split(msk_f: ReadDir, offset: usize) -> MSKFiles {
    let mut out: MSKFiles = vec![];
    for dir_r in msk_f {
        if dir_r.is_err() {
            error(join!["Failed to read file (", &*dir_r.expect_err("spaghetti").to_string(), ")"]);
            continue;
        }
        let dir = dir_r.unwrap();
        if dir.path().is_dir() {
            out.append(&mut get_msk_files_split(read_dir(dir.path()).unwrap(), offset + 1));
            continue;
        }
        let name = dir.file_name();
        let name = name.to_str().unwrap_or("null.null");
        if let Some(ext) = name.split(".").nth(1) {
            if ext.eq("msk") {
                let lines = fs::read_to_string(dir.path())
                    .unwrap_or("".to_string())
                    .split("\n")
                    .collect::<Vec<&str>>()
                    .iter()
                    .map(|x| String::from((*x).trim()))
                    .collect::<Vec<String>>();
                out.push((direntry_to_name_loc(&dir, offset), lines, MskCache::from_msk(&dir)));
            }
        }
    }
    out.iter_mut().for_each(|fl| {
        fl.0 = fl.0.replace('$', "/");
        fl.1.iter_mut().for_each(|f| {
            *f = f.trim().trim_start_matches("@NOLEX").trim().to_string();
        })
    });
    out
}

pub fn get_cache_files(msk_f: ReadDir) -> CacheFiles {
    let mut cache: CacheFiles = vec![];
    for dir_r in msk_f {
        if dir_r.is_err() {
            error(join!["Failed to read file (", &*dir_r.expect_err("spaghetti").to_string(), ")"]);
            continue;
        }
        let dir = dir_r.unwrap();
        if dir.path().is_dir() {
            continue;
        }
        let name = dir.file_name();
        let name = name.to_str().unwrap_or("unknown");
        if let Some(ext) = dir.path().extension() {
            if ext.eq("cache") {
                let lines = fs::read(dir.path()).unwrap_or_else(|e| {
                    error(join!["Failed to read file '", name, "' (", &*e.to_string(), ")"]);
                    vec![]
                });
                cache.push((lines, MskCache::read_from_file(&dir)));
            }
        }
    }
    cache
}

fn direntry_to_name_loc(dir: &DirEntry, offset: usize) -> String {
    let text_path = dir.path();
    let text_path = text_path.iter().collect::<Vec<_>>();
    let text_path = text_path.get((text_path.len() - offset - 1)..).unwrap().join(OsStr::new("/"));
    let text_path = text_path.to_str().unwrap().to_string().replace(".msk", "");
    text_path
}

pub fn path_without_functions(path: String) -> String {
    if path.ends_with("functions") {
        let t = path.rsplit_once("functions").unwrap_or(("", "")).0;
        t.trim_end_matches(|c| c == '/' || c == '\\').into()
    } else {
        path
    }
}

#[derive(Default)]
pub struct CliArgs {
    pub input: String,
    pub output: String,
    pub export: bool,
    pub cache: bool,
    pub reuse_output: bool,
}

pub fn get_cli_args() -> CliArgs {
    let mut args = env::args().collect::<Vec<String>>().into_iter();
    args.next();

    if let Some(mode) = args.next() {
        match &*mode {
            "help" => {
                print_help();
                exit(0);
            }
            "build" => {
                let mut cliargs = CliArgs::default();
                cliargs.input = args.next().unwrap_or_else(|| {
                    death_error(join!("No pack specified"), errors::BAD_CLI_OPTIONS)
                }).to_string().replace("\\", "/");
                while cliargs.input.ends_with("/") {
                    cliargs.input.pop();
                }
                cliargs.output = join![&*cliargs.input, "/generated"];

                let mut matching = |arg: String, args: &mut IntoIter<String>| {
                    match &*arg {
                        "--gen-output" | "-g" => cliargs.output = args.next().unwrap_or_else(|| {
                            death_error(join!("No output location specified"), errors::BAD_CLI_OPTIONS)
                        }),
                        "--export" | "-e" => cliargs.export = true,
                        "--cache" | "-C" => cliargs.cache = true,
                        "--reuse-output" | "-R" => cliargs.reuse_output = true,
                        _ => {
                            death_error(join!("Unknown option '", &*arg, "'"), errors::BAD_CLI_OPTIONS);
                        }
                    }
                };

                while let Some(arg) = args.next() {
                    match &*arg {
                        _ if arg.starts_with("-") && !arg.starts_with("--") => {
                            let options = arg[1..].split("").collect::<Vec<_>>();
                            for opt in options {
                                if opt != "" {
                                    matching(join!["-", &*opt], &mut args);
                                }
                            }
                        }
                        _ => {
                            matching(arg, &mut args);
                        }
                    }
                }

                cliargs
            }
            _ => { death_error(join!("Unknown mode '", &*mode, "', use 'help' to see all available commands"), errors::BAD_CLI_OPTIONS) }
        }
    } else {
        soft_error(join!("No mode specified (use 'mitsuko help' for help)"));
        exit(errors::BAD_CLI_OPTIONS);
    }
}

fn print_help() {
    println!("Usage: mitsuko <mode> [options]");
    println!("Modes:");
    println!("\thelp\n\t\tPrints this message");
    println!("\tbuild <pack_location> [options]\n\t\t{}\n", &*[
        "Builds the specified datapack",
        "(-g | --gen-output) <location>", "\tChange the generation output to <location>/datapacks",
        "(-e | --export)", "\tEnable creation of export file",
        "(-C | --cache)", "\tEnable caching",
    ].join("\n\t\t"));
}

pub fn read_cached_data(path: &str) {
    match read_dir(join![path, "/.cache"]) {
        Ok(cpath) => {
            unsafe {
                I_CACHED_MSK = get_cache_files(cpath);
            }
        }
        Err(_err) => {
            match Path::try_exists(Path::new(&join![path, "/.cache"])) {
                Ok(_) => {} //dont care, means the project hasn't been ran with cache before
                Err(e) => {
                    soft_error(join!["Failed to read /.cache/, Assuming default (", &*e.to_string(), ")"]);
                }
            }
        }
    }
}

pub struct Blocker {
    stack: Vec<char>,
    string: bool,
}

impl Blocker {
    pub const NOT_FOUND: usize = 404_0000000usize;

    pub fn new() -> Blocker {
        Blocker {
            stack: Vec::new(),
            string: false,
        }
    }

    // pub fn find_rapid_close(&mut self, lines: &Vec<String>, closer: char) -> Result<usize, String> {
    //     let mut c: usize = 0;
    //     loop {
    //         if c >= lines.len() {
    //             return Ok(Blocker::NOT_FOUND);
    //         }
    //         if lines[c].trim().starts_with(closer) {
    //             return Ok(c);
    //         }
    //         c += 1;
    //     }
    // }

    /**
    (line_number, offset)
     */
    pub fn auto_vec(lines: &Vec<String>, offset: (usize, usize), path: String, ln: usize) -> (usize, usize) {
        let mut b = Blocker::new();
        match b.find_size_vec(lines, offset) {
            Ok(o) => {
                if o.0 != Blocker::NOT_FOUND {
                    return o;
                } else {
                    death_error(format_out("Unterminated block", &*path, ln), errors::AST_ERROR)
                }
            }
            Err(e) => death_error(format_out(&*[&*e.0, " /", &path, ":", &*(e.1 + ln).to_string()].join(""), &*path, ln), errors::AST_ERROR),
        }
    }

    /**
     * Returns OK(line_number, offset)
     *
     * or Err(message, offset)
     */
    pub fn find_size_vec(&mut self, lines: &Vec<String>, offset: (usize, usize))
                         -> Result<(usize, usize), (String, usize)> {
        let mut c: usize = offset.0;
        loop {
            if c >= lines.len() {
                return Ok((Blocker::NOT_FOUND, c));
            }
            let r = self
                .find_size(&lines[c], if c == offset.0 { offset.1 } else { 0 })
                .map_err(|e| (e, c))?;
            if r != Blocker::NOT_FOUND {
                return Ok((c, r));
            }
            if self.string {
                self.stack.pop();
                self.string = false;
            }
            c += 1;
        }
    }

    pub fn quick_block_end(&mut self, lines: &Vec<String>) -> Result<usize, (String, usize)> {
        let mut c: usize = 0;
        loop {
            if c >= lines.len() {
                return Ok(Blocker::NOT_FOUND);
            }
            let line = lines[c].trim();
            if line.starts_with("set ") || line.starts_with("//") { 
                c += 1;
                continue;
            }
            if line.ends_with("{") {
                self.stack.push('}');
            } else if line.starts_with("}") {
                if self.stack.len() > 1 {
                    self.stack.pop();
                } else {
                    return Ok(c);
                }
            }
            c += 1;
        }
    }

    pub fn fix_index(line: &String, index: usize) -> usize {
        let bytes = line.as_bytes();
        if !bytes.is_ascii() {
            let mut fix = 0;
            let chars = line.chars().collect::<Vec<char>>();
            for i in 0..index {
                fix += !chars[i].is_ascii() as usize;
            }
            return fix;
        }
        0
    }

    pub fn find_size(&mut self, line: &String, offset: usize) -> Result<usize, String> {
        fn char_error(c: char, l: usize) -> String {
            format!("Unexpected \'{}{}{}\' ({})", foreground(ORN), c, END, l)
        }
        if line.starts_with("//") || line.starts_with("@NOLEX") {
            return Ok(Blocker::NOT_FOUND);
        }
        let mut cs = line.chars();
        if offset > 0 {
            cs.nth(offset - 1);
        }
        let mut pos: usize = 0;
        while let Some(c) = cs.next() {
            pos += c.to_string().len(); // I LOVE ENCODING
            match c {
                '\\' => {
                    cs.next();
                    pos += 1;
                }
                '{' if !self.string => self.stack.push(c),
                '}' if !self.string => {
                    if self.stack.last().eq(&Some(&'{')) {
                        self.stack.pop();
                    } else {
                        return Err(char_error(c, pos));
                    }
                }
                '(' if !self.string => self.stack.push(c),
                ')' if !self.string => {
                    if self.stack.last().eq(&Some(&'(')) {
                        self.stack.pop();
                    } else {
                        return Err(char_error(c, pos));
                    }
                }
                '[' if !self.string => self.stack.push(c),
                ']' if !self.string => {
                    if self.stack.last().eq(&Some(&'[')) {
                        self.stack.pop();
                    } else {
                        return Err(char_error(c, pos));
                    }
                }
                '\'' => {
                    if self.string {
                        self.string = !self.stack.last().eq(&Some(&'\''));
                        if !self.string {
                            self.stack.pop();
                        }
                    } else {
                        self.stack.push(c);
                        self.string = true;
                    }
                }
                '\"' => {
                    if self.string {
                        self.string = !self.stack.last().eq(&Some(&'\"'));
                        if !self.string {
                            self.stack.pop();
                        }
                    } else {
                        self.stack.push(c);
                        self.string = true;
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

    pub fn find_in_same_level(&mut self, needle: &str, haystack: &String) -> Result<Option<usize>, String> {
        let mut pos = 0;
        loop {
            if pos >= haystack.len() {
                return Ok(None);
            }
            if haystack[pos..].starts_with(needle) {
                return Ok(Some(pos));
            }
            let res = self.find_size(&haystack, pos)?;
            if res != Blocker::NOT_FOUND {
                pos = res;
            } else {
                pos += 1;
            }
        }
    }

    pub fn split_in_same_level(&mut self, blade: &str, haystack: &String) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        let mut pos = 0;
        let mut pos_old = 0;
        loop {
            if pos >= haystack.len() {
                out.push(haystack[pos_old..].to_string());
                return Ok(out);
            }
            if haystack[pos..].starts_with(blade) {
                out.push(haystack[pos_old..pos].to_string());
                pos += blade.len();
                pos_old = pos;
            } else {
                let res = self.find_size(&haystack, pos)?;
                if res != Blocker::NOT_FOUND {
                    pos = res;
                } else {
                    pos += 1;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct MskCache {
    pub timestamp: SystemTime,
    pub size: u64,
    pub file_path: String,
    pub extern_frag: Magnet<CachedFrag>,
    pub cached_frags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CachedFrag {
    pub name: String,
    pub files: Magnet<SaveFiles>,
    pub hash: u64,
    pub size: u64,
}

impl MskCache {
    pub fn blank() -> MskCache {
        MskCache {
            timestamp: SystemTime::UNIX_EPOCH,
            size: 0,
            file_path: "".to_string(),
            extern_frag: Magnet::new(None),
            cached_frags: vec![],
        }
    }

    pub fn pull_data(&mut self) -> Self {
        MskCache {
            timestamp: self.timestamp.clone(),
            size: self.size,
            file_path: self.file_path.clone(),
            extern_frag: self.extern_frag.pull_data(),
            cached_frags: self.cached_frags.drain(..).collect(),
        }
    }

    pub fn from_msk(file: &DirEntry) -> Self {
        if let Ok(data) = file.metadata() {
            let file_path = if let Some(path) = file.path().to_str() { path.to_string() } else {
                error(join!["Failed to make cache for file '", &*file.path().to_str().unwrap_or(""), "', ", &*file.metadata().err().unwrap().to_string()]);
                join![unsafe{&*PROJECT_ROOT}, "/src/pack/unknown/unknown.msk"]
            }
                [unsafe { &PROJECT_ROOT }.len()..].replace("\\", "/")
                .rsplit_once(".").unwrap_or(("/src/pack/unknown/unknown", "")).0.to_string();
            let dur = data.modified().unwrap_or(SystemTime::UNIX_EPOCH).duration_since(UNIX_EPOCH).unwrap_or(Duration::from_millis(0));
            let m = MskCache {
                timestamp: UNIX_EPOCH + Duration::from_millis(dur.as_millis() as u64),
                size: data.len(),
                file_path: file_path[5..].replace("/", "$"), // pack$functions$functions
                extern_frag: Magnet::new(None),
                cached_frags: vec![],
            };
            m
        } else {
            error(join!["Failed to make cache for file '", &*file.path().to_str().unwrap_or(""), "', ", &*file.metadata().err().unwrap().to_string()]);
            MskCache::blank()
        }
    }

    pub fn read_from_file(file: &DirEntry) -> MskCache {
        let file_path = if let Some(path) = file.path().to_str() { path.to_string() } else {
            error(join!["Failed to get cache for file '", &*file.path().to_str().unwrap_or(""), "', ", &*file.metadata().err().unwrap().to_string()]);
            join![unsafe{&*PROJECT_ROOT}, "/.cache/pack$functions$functions.cache"]
        }.replace("\\", "/");
        let file_path = file_path.rsplit_once("/").unwrap_or(("", "pack$functions$functions.cache")).1
            .rsplit_once(".").unwrap_or(("pack$functions$functions", "")).0.to_string();
        let mut m = MskCache {
            timestamp: SystemTime::UNIX_EPOCH,
            size: 0,
            file_path: file_path.clone(),
            extern_frag: Magnet::new(None),
            cached_frags: vec![],
        };

        // for dir_r in msk_f {
        //     if dir_r.is_err() {
        //         error(join!["Failed to read file (", &*dir_r.expect_err("spaghetti").to_string(), ")"]);
        //         continue;
        //     }
        //     let dir = dir_r.unwrap();
        //     if dir.path().is_dir() {
        //         continue;
        //     }
        // }

        match fs::read(file.path()) {
            Ok(mut d_in) => {
                m.timestamp = SystemTime::UNIX_EPOCH + Duration::from_millis(u128::from_be_bytes(d_in.drain(..16).as_slice().try_into().unwrap_or([0; 16])) as u64);
                m.size = u64::from_be_bytes(d_in.drain(..8).as_slice().try_into().unwrap_or([0; 8]));
                m.extern_frag.attach(CachedFrag::from_path("_EXTERN", &m));
                m
            }
            Err(e) => {
                soft_error(join!["Failed to read cache '", unsafe{&*PROJECT_ROOT}, "/.cache/", &*file_path, ".cache', Assuming default (", &*e.to_string(), ")"]);
                m
            }
        }
    }

    pub fn save_to_file(&mut self) {
        let file = MFile::new(unsafe { join![&*PROJECT_ROOT, "/.cache/", &*self.file_path, ".cache"] });
        let mut write: Vec<u8> = vec![];
        // pub timestamp: SystemTime,
        match self.timestamp.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(dur) => write.extend_from_slice(&dur.as_millis().to_be_bytes()),
            Err(e) => error(join!["Failed to save cache file '", &*self.file_path, "', ", &*e.to_string()]),
        }
        // pub size: u64,
        write.extend_from_slice(&self.size.to_be_bytes());
        file.save_bytes(&write[..]).map_err(|e| {
            soft_error(e.to_string());
        }).ok();

        if self.extern_frag.is_attached() {
            let mut fragment = self.extern_frag.unattach();
            fragment.save_to_file();
        }
    }

    pub fn compare_to(&self, other: &Self) -> CachedType {
        let mut bez = Recompile;
        if self.size == other.size {
            if self.timestamp != UNIX_EPOCH && self.timestamp == other.timestamp {
                bez = Unchanged;
            }
        }
        if bez == Recompile && self.extern_frag.is_attached() {
            if self.extern_frag != other.extern_frag {
                bez = Changed;
            }
        }
        bez
    }
}

impl CachedFrag {
    pub fn new(name: String) -> CachedFrag {
        CachedFrag {
            name,
            files: Magnet::new(None),
            hash: 0,
            size: 0,
        }
    }

    pub fn make_frag(name: String, cache: &MskCache) -> Self {
        let frag = Self::new(qc!(cache.file_path == "pack", cache.file_path.to_string(), 
                join![&*cache.file_path, "/", &*name.replace("/", "$")]));
        frag
    }

    pub fn update_hash(&mut self, node: &Node) {
        let mut hasher = DefaultHasher::new();
        node.lines.hash(&mut hasher);
        self.hash = hasher.finish();
        self.size = node.lines.len() as u64;
    }

    pub fn from_path(name: &str, cache: &MskCache) -> Self {
        let mut frag = Self::make_frag(name.to_string(), cache);
        frag.read_from_file();
        frag
    }

    pub fn from_mcfunction(mcf: &MCFunction) -> Self {
        // pack $functions$functions/fragment
        // ns_id$file_path$call_path/call_name
        let mut frag = Self::new(join![&*mcf.ns_id, "$", &*mcf.file_path, "$", &*mcf.call_path, "/", &*mcf.call_name]);
        frag.update_hash(&mcf.node);
        dbg!(&mcf);
        todo!()
    }

    pub fn read_from_file(&mut self) -> bool {
        match fs::read(join![unsafe{&*PROJECT_ROOT}, "/.cache/", &*self.name, ".cache.fragment"]) {
            Ok(mut d_in) => {
                self.hash = u64::from_be_bytes(d_in.drain(..8).as_slice().try_into().unwrap_or([0; 8]));
                self.size = u64::from_be_bytes(d_in.drain(..8).as_slice().try_into().unwrap_or([0; 8]));
                let mut files = vec![];
                while !d_in.is_empty() {
                    let name_size = u64::from_be_bytes(d_in.drain(..8).as_slice().try_into().unwrap_or([0; 8]));
                    let name = String::from_utf8_lossy(d_in.drain(..(name_size as usize)).as_slice()).to_string();
                    let content_size = u64::from_be_bytes(d_in.drain(..8).as_slice().try_into().unwrap_or([0; 8]));
                    let content = String::from_utf8_lossy(d_in.drain(..(content_size as usize)).as_slice()).split("\n").map(str::to_string).collect::<Vec<_>>();
                    files.push((name, content));
                }
                self.files.attach(files);
                true
            }
            Err(e) => {
                if self.name.contains("_EXTERN") {
                    soft_error(join!["Failed to read cache fragment '", unsafe{&*PROJECT_ROOT}, "/.cache/", &*self.name, ".cache.fragment', Assuming default (", &*e.to_string(), ")"]);
                }
                false
            }
        }
    }

    pub fn save_to_file(&mut self) {
        let file = MFile::new(unsafe { join![&*PROJECT_ROOT, "/.cache/", &*self.name, ".cache.fragment"] });
        let mut write: Vec<u8> = vec![];

        // pub hash: u64,
        write.extend_from_slice(&self.hash.to_be_bytes());
        // pub size: u64,
        write.extend_from_slice(&self.size.to_be_bytes());
        // pub files: Magnet<SaveFiles>,
        // pub type SaveFiles = Vec<(String, Vec<String>)>;
        if self.files.is_attached() {
            for (name, lines) in self.files.unattach() {
                write.extend_from_slice(&(name.len() as u64).to_be_bytes());
                write.extend_from_slice(&name.into_bytes());
                let content = lines.join("\n");
                write.extend_from_slice(&(content.len() as u64).to_be_bytes());
                write.extend_from_slice(&content.into_bytes());
            }
        }

        file.save_bytes(&write[..]).map_err(|e| {
            soft_error(e.to_string());
        }).ok();
    }
}

impl PartialEq<Self> for CachedFrag {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size && self.hash == other.hash && qc!(self.name.ends_with("/_EXTERN") || self.name.eq("pack"), self.files == other.files, true)
    }
}