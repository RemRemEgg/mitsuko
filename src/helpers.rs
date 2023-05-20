// i need more than just programming help

use std::borrow::Borrow;
use std::ffi::OsStr;
use std::fs;
use std::fs::{DirEntry, File, read_dir, ReadDir};
use std::io::Write;
use crate::{error, join, *};

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

pub struct MFile {
    path: String,
    file: File,
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

pub fn get_msk_files_split(fn_f: ReadDir, offset: usize) -> Vec<(String, Vec<String>)> {
    let mut out = vec![];
    for dir_r in fn_f {
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

pub fn get_cli_args() -> (String, Option<String>, bool) {
    let mut args = env::args().collect::<Vec<String>>().into_iter();
    args.next();

    let mut pck;
    let (mut mov, mut clr) = (None, false);
    match &*args.next().unwrap_or_else(|| {
        status_color("No pack specified".into(), str::RED);
        "-h".into()
    }) {
        "-h" | "--help" | "?" => {
            print_help();
            std::process::exit(0);
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
            _ => {
                status_color(join!("Unknown arg '", &*arg, "'"), str::RED);
                std::process::exit(errors::BAD_CLI_ARGS);
            }
        }
    }

    if clr && mov.is_none() {
        status_color("Clear enabled without specifying a location".into(), str::RED);
        exit(errors::BAD_CLI_ARGS);
    }

    (pck, mov, clr)
}

fn print_help() {
    println!("Usage: mitsuko <pack_location> [options]\n\t{}\n", &*[
        "(-h | --help | ?)", "\tDisplay this message",
        "(-m | --move) <locations>", "\tMove the compiled pack to <location>/datapacks",
        "(-c | --clear)", "\tRemove the old datapack at <location>/datapacks"
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
