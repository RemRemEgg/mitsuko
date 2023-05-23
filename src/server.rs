// i need more than just programming help

use std::fs::{DirEntry, File, read_dir, ReadDir};
use std::ffi::OsStr;
use std::io;
use std::io::Write;
use std::path::Path;
use crate::*;

pub static COMMANDS: [&str; 65] = ["return", "advancement", "attribute", "bossbar", "clear", "clone", "data", "datapack", "debug", "defaultgamemode", "difficulty",
    "effect", "enchant", "execute", "experience", "fill", "forceload", "function", "gamemode", "gamerule", "give", "help", "kick", "kill",
    "list", "locate", "loot", "me", "msg", "particle", "playsound", "publish", "recipe", "reload", "item", "say", "schedule", "scoreboard",
    "seed", "setblock", "setworldspawn", "spawnpoint", "spectate", "spreadplayers", "stopsound", "summon", "tag", "team", "teammsg", "teleport",
    "tell", "tellraw", "time", "title", "tm", "tp", "trigger", "weather", "worldborder", "xp", "jfr", "place", "fillbiome", "ride", "damage"];

static mut WARNINGS: Vec<String> = vec![];
pub static mut SUPPRESS_WARNINGS: bool = false;
pub static mut KNOWN_FUNCTIONS: Vec<String> = vec![];
pub static mut EXPORT_FUNCTIONS: Vec<String> = vec![];
pub static mut HIT_ERROR: i32 = 0;

pub mod errors {
    pub const UNKNOWN_ERROR: i32 = -86;
    pub const BAD_CLI_ARGS: i32 = 10;
    pub const TOO_MANY_ERRORS: i32 = 20;
    pub const NO_PACK_MSK: i32 = 30;
    pub const IMPORT_NOT_FOUND: i32 = 40;
}

pub fn print_warnings(pack: &Datapack) {
    unsafe {
        if WARNINGS.len() > 0 && !SUPPRESS_WARNINGS {
            println!();
            status(format!(
                "'{}' Generated {} Warnings: ",
                pack.get_view_name(),
                WARNINGS.len()
            ).form_foreground(str::ORN));
            for (i, e) in WARNINGS.iter().enumerate() {
                print_warning(
                    format!(
                        "{}{}",
                        e,
                        if i == WARNINGS.len() - 1 {
                            "\n"
                        } else {
                            ""
                        }
                    ), i,
                );
            }
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
            println!("{}", join!("\x1b[93m‼»\x1b[m   [", &*"Warning".form_foreground(String::ORN).form_bold(), "] ", &*message));
        }
        WARNINGS.push(message);
    }
}

unsafe fn print_warning(message: String, i: usize) {
    println!("\x1b[93m‼»\x1b[m   [{}] {}", (i + 1).to_string().form_foreground(str::ORN), message);
}

pub fn format_out(message: &str, path: &str, ln: usize) -> String {
    message.to_string() + " " + &join!["./src/", path, ".msk:", &*ln.to_string(), ""].replace("\\", "/").replace("/.msk", "/functions.msk")
        .form_underline().form_foreground(str::GRY)
}

pub fn death_error(message: String) -> ! {
    death_error_type(message, errors::UNKNOWN_ERROR);
}

pub fn death_error_type(message: String, etype: i32) -> ! {
    error(message);
    status_color("Aborting".into(), str::RED);
    exit(etype);
}

pub fn error(message: String) {
    unsafe { HIT_ERROR += 1 }
    eprintln!("{}", join!("⮾   [", &*"Error".form_foreground(String::RED).form_italic().form_bold(), "] ", &*message));
}

pub fn status(message: String) {
    println!(" »   {}", message);
}

pub fn status_color(message: String, color: usize) {
    println!(" »   {}", message.form_foreground(color));
}

pub fn debug(message: String) {
    println!("\x1b[96m§»\x1b[m   {}", message);
}

#[macro_export]
macro_rules! join {
    ( $( $x:expr ),* ) => {
            [$($x,)*""].join("")
    };
}

#[macro_export]
macro_rules! qc {
    ($s:expr, $t:expr, $f:expr) => {
        if $s {$t} else {$f}
    };
}

pub trait FancyText: ToString {
    //ignore
    const GRY: usize = 0;
    //errors
    const RED: usize = 1;
    //good stuff
    const GRN: usize = 2;
    //warns
    const ORN: usize = 3;
    //names
    const BLU: usize = 4;
    //ns
    const PNK: usize = 5;
    //debug
    const AQU: usize = 6;
    //unused
    const WHT: usize = 7; 

    fn form_bold(&self) -> String {
        join!("\x1b[1m", &*self.to_string(), "\x1b[m")
    }
    fn form_italic(&self) -> String {
        join!("\x1b[3m", &*self.to_string(), "\x1b[m")
    }
    fn form_underline(&self) -> String {
        join!("\x1b[4m", &*self.to_string(), "\x1b[m")
    }
    fn form_custom(&self, id: usize) -> String {
        join!("\x1b[", &*id.to_string(), "m", &*self.to_string(), "\x1b[m")
    }
    fn form_foreground(&self, id: usize) -> String {
        join!("\x1b[", &*(90+id).to_string(), "m", &*self.to_string(), "\x1b[m")
    }
    fn form_background(&self, id: usize) -> String {
        join!("\x1b[", &*(100+id).to_string(), "m", &*self.to_string(), "\x1b[m")
    }
    fn form_background_custom(&self, r: u8, g: u8, b: u8) -> String {
        join!("\x1b[48;2;",&*r.to_string(),";",&*g.to_string(),";",&*b.to_string(), "m", &*self.to_string(), "\x1b[m")
    }
    fn form_foreground_custom(&self, r: u8, g: u8, b: u8) -> String {
        join!("\x1b[38;2;",&*r.to_string(),";",&*g.to_string(),";",&*b.to_string(), "m", &*self.to_string(), "\x1b[m")
    }
}

impl FancyText for String {}

impl FancyText for str {}

impl FancyText for char {}

pub static META_TEMPLATE: &str = include_str!("pack.mcmeta");
pub static TAG_TEMPLATE: &str = include_str!("tag.json");
pub static RECIPE_TEMPLATE: &str = include_str!("recipe.json");
pub static ADV_CRAFT_TEMPLATE_119: &str = include_str!("advancement_craft_1.19.json");
pub static ADV_CRAFT_TEMPLATE_120: &str = include_str!("advancement_craft_1.20.json");
pub static MAT_TEMPLATE: &str = r#""$ID$": {"item": "minecraft:$TYPE$"}"#;
pub static MAT_TAG_TEMPLATE: &str = r#""$ID$": {"tag": "minecraft:$TYPE$"}"#;

pub static mut DATAROOT: String = String::new();

pub fn read_src<T: ToString>(loc: T) -> std::io::Result<ReadDir> {
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

pub(crate) fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
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

#[derive(Debug)]
pub struct MFile {
    pub path: String,
    pub file: File,
}

impl MFile {
    pub fn new(path: String) -> MFile {
        make_folder(&*path.rsplit_once("/").unwrap_or(("", "")).0);
        MFile {
            file: File::create(&*path).expect(&*join!["Could not make '\x1b[93m", &*path, "\x1b[m'"]),
            path,
        }
    }

    pub fn save<T: ToString>(mut self, mut write: T) {
        self.file.write_all(write.to_string().as_bytes())
            .expect(&*join!["Could not make '\x1b[93m", &*self.path, "\x1b[m'"]);
    }
}

pub fn get_msk_files_split(msk_f: ReadDir, offset: usize) -> Vec<(String, Vec<String>)> {
    let mut out = vec![];
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
                out.push((direntry_to_name_loc(&dir, offset), lines));
            }
        }
    }
    out.iter_mut().for_each(|mut fl| fl.0 = fl.0.replace('$', "/"));
    out
}

fn direntry_to_name_loc(dir: &DirEntry, offset: usize) -> String {
    let text_path = dir.path();
    let text_path = text_path.iter().collect::<Vec<_>>();
    let text_path = text_path.get((text_path.len() - offset - 1)..).unwrap().join(OsStr::new("/"));
    let mut text_path = text_path.to_str().unwrap().to_string().replace(".msk", "");
    text_path
}

pub fn path_without_functions(path: String) -> String {
    if path.ends_with("functions") {
        path.rsplit_once("functions").unwrap_or(("", "")).0.into()
    } else {
        path
    }
}

pub fn get_cli_args() -> (String, Option<String>, bool, bool) {
    let mut args = env::args().collect::<Vec<String>>().into_iter();
    args.next();

    let mut pck;
    let (mut mov, mut clr, mut exp) = (None, false, false);
    match &*args.next().unwrap_or_else(|| {
        status_color("No pack specified".into(), str::RED);
        "-h".into()
    }) {
        "-h" | "--help" | "?" => {
            print_help();
            exit(0);
        }
        p @ _ => {
            pck = p.into();
        }
    }

    while let Some(arg) = args.next() {
        match &*arg {
            "--help" | "-h" | "?" => {
                print_help();
            }
            "--pack" | "-p" => pck = args.next().unwrap_or("".to_string()),
            "--move" | "-m" => mov = args.next(),
            "--clear" | "-c" => clr = true,
            "--export" | "-e" => exp = true,
            _ => {
                status_color(join!("Unknown arg '", &*arg, "'"), str::RED);
                exit(errors::BAD_CLI_ARGS);
            }
        }
    }

    if clr && mov.is_none() {
        status_color("Clear enabled without specifying a location".into(), str::RED);
        exit(errors::BAD_CLI_ARGS);
    }

    (pck, mov, clr, exp)
}

fn print_help() {
    println!("Usage: mitsuko <pack_location> [options]\n\t{}\n", &*[
        "(-h | --help | ?)", "\tDisplay this message",
        "(-m | --move) <locations>", "\tMove the compiled pack to <location>/datapacks",
        "(-c | --clear)", "\tRemove the old datapack at <location>/datapacks",
        "(-e | --export)", "\tEnable creation of export file"
    ].join("\n\t"));
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

    pub fn reset(&mut self) -> &mut Blocker {
        self.stack.clear();
        self.string = false;
        self
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
                    death_error(format_out("Unterminated block", &*path, ln))
                }
            }
            Err(e) => death_error(format_out(&*[&*e.0, " /", &path, ":", &*(e.1 + ln).to_string()].join(""), &*path, ln)),
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

    pub fn find_size(&mut self, line: &String, offset: usize) -> Result<usize, String> {
        if line.starts_with("//") || line.starts_with("cmd") || line.starts_with("@NOLEX cmd") {
            return Ok(Blocker::NOT_FOUND);
        }
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
                '}' if !self.string => {
                    if self.stack.last().eq(&Some(&'{')) {
                        self.stack.pop();
                    } else {
                        return Err(format!("Unexpected \'{}\' ({})", c.form_foreground(str::ORN), pos));
                    }
                }
                '(' if !self.string => self.stack.push(c),
                ')' if !self.string => {
                    if self.stack.last().eq(&Some(&'(')) {
                        self.stack.pop();
                    } else {
                        return Err(format!("Unexpected \'{}\' ({})", c.form_foreground(str::ORN), pos));
                    }
                }
                '[' if !self.string => self.stack.push(c),
                ']' if !self.string => {
                    if self.stack.last().eq(&Some(&'[')) {
                        self.stack.pop();
                    } else {
                        return Err(format!("Unexpected \'{}\' ({})", c.form_foreground(str::ORN), pos));
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
            }
            let res = self.find_size(&haystack, pos)?;
            if res != Blocker::NOT_FOUND {
                pos = res;
            } else {
                pos += 1;
            }
        }
    }
}
