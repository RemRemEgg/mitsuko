// new code go brrrrrrrrrr

extern crate core;

mod server;
mod helpers;
mod ast;
mod minecraft;

use std::{env, fs};
use std::alloc::System;
use std::process::exit;
use server::*;
use ast::*;
use minecraft::*;
use crate::helpers::get_cli_args;

static CURRENT_PACK_VERSION: u8 = 13;
static mut SRC: String = String::new();

fn main() {
    println!("{}", "[Mitsuko v2: New and improved!]".form_background(str::GRY));
    status_color(env::args().collect::<Vec<String>>()[1..].join(" "), str::GRY);

    let (path, _mov, _clr) = get_cli_args();

    let mut data = Datapack::new(path);

    status(["Compiling '", &*data.src_loc.form_foreground(str::PNK), "'"].join(""));

    let pack = fs::read_to_string(join![&*data.src_loc, "/src/pack.msk"]).unwrap_or_else(|e| {
        death_error(join!("Could not read '",&*"pack.msk".form_foreground(str::ORN),"' (", &*e.to_string(), ")"));
    });

    data.gen_meta(pack);
    stop_if_errors();
    
    data.read_namespaces();
    stop_if_errors();
        
    data.compile_namespaces();
    stop_if_errors();
    
    data.save();
    stop_if_errors();
    
    print_warnings(&data);
}

fn stop_if_errors() {
    if unsafe {HIT_ERROR} {
        status_color("Aborting due to previous errors".into(), str::RED);
        exit(errors::TOO_MANY_ERRORS);
    }
}
