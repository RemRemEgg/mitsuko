// new code go brrrrrrrrrr

mod server;
mod helpers;
mod ast;
mod io_extra;

use std::{env, fs};
use server::*;
use io_extra::*;
use ast::*;

static CURRENT_PACK_VERSION: u8 = 13;

fn main() {
    println!("{}", "[Mitsuko v2: New and improved!]".form_background(str::GRY));
    status_color(env::args().collect::<Vec<String>>()[1..].join(" "), str::GRY);

    let (path, _mov, _clr) = get_cli_args();
    
    let mut data = Datapack::new(path);

    status(["Compiling '", &*data.src_loc.form_foreground(str::PNK), "'"].join(""));

    let pack = fs::read_to_string(join![&*data.src_loc, "/src/pack.msk"]).unwrap_or_else(|e| {
        error(join!("Could not read '",&*"pack.msk".form_foreground(str::ORN),"' (", &*e.to_string(), ")"));
    });
}

pub struct Datapack {
    meta: Meta,
    ln: usize,
    namespaces: Vec<Namespace>,
    warnings: Vec<String>,
    src_loc: String,
    callable_functions: Vec<String>,
}

impl Datapack {
    fn new(path: String) -> Datapack {
        Datapack {
            meta: Meta::new(),
            ln: 1,
            warnings: vec![],
            src_loc: path,
            namespaces: vec![],
            callable_functions: vec![],
        }
    }
}

#[derive(Clone, Debug)]
struct Meta {
    vb: i32,
    version: u8,
    remgine: bool,
    opt_level: u8,
    comments: bool,
    view_name: String,
    description: String,
    recursive_replace: u8,
}

impl Meta {
    fn new() -> Meta {
        Meta {
            vb: 0,
            version: CURRENT_PACK_VERSION,
            remgine: false,
            opt_level: 0,
            comments: false,
            view_name: "Untitled".to_string(),
            description: "A Datapack".to_string(),
            recursive_replace: 3,
        }
    }
}

struct Namespace {
    id: String,
    functions: Vec<MCFunction>,
    // links: Vec<Link>,
    // items: Vec<Item>,
    meta: Meta,
    ln: usize,
    warnings: Vec<String>,
    export_functions: Vec<String>,
}

impl Namespace {
    fn new(id: String, meta: Meta) -> Namespace {
        if id.eq(&"".to_string()) || {
            let mut nid = id.replace(|ch| ch >= 'a' && ch <= 'z', "");
            nid = nid.replace(|ch| ch >= '0' && ch <= '9', "");
            nid = nid.replace("_", "");
            nid.len() != 0
        } {
            error(join!["Invalid Namespace: ", &*id]);
        }
        Namespace {
            id,
            functions: vec![],
            // links: Vec::new(),
            // items: Vec::new(),
            meta,
            ln: 0,
            warnings: vec![],
            export_functions: vec![],
        }
    }
}

#[derive(Debug, Clone)]
struct MCFunction {
    node: Node,
    lines: Vec<String>,
    call_name: String,
    calls: Vec<(String, usize)>,
    ln: usize,
    vars: Vec<(String, String)>,
    meta: Meta,
    file_name: String,
    compiled: bool,
    premc: usize,
}
