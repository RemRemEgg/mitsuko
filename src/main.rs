// new code go brrrrrrrrrr

extern crate core;

mod server;
mod ast;
mod minecraft;
mod compile;

use std::{env, fs};
use std::alloc::System;
use std::ops::SubAssign;
use std::process::exit;
use std::time::Instant;
use server::*;
use ast::*;
use minecraft::*;
use server::get_cli_args;

static CURRENT_PACK_VERSION: u8 = 13;
static mut SRC: String = String::new();
//todo inline tagging
//todo exports
//todo marcos?
fn main() {
    let mut times = (Instant::now(), Instant::now(), Instant::now(), Instant::now());
    println!();
    status_color(env::args().collect::<Vec<String>>()[1..].join(" "), str::GRY);

    let (path, mov, clear) = get_cli_args();

    let mut data = Datapack::new(path);

    //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
    times.1 = Instant::now();

    status(["Compiling '", &*data.src_loc.form_foreground(str::PNK), "'"].join(""));

    let pack = fs::read_to_string(join![&*data.src_loc, "/src/pack.msk"]).unwrap_or_else(|e| {
        death_error_type(join!("Could not read '",&*"pack.msk".form_foreground(str::ORN),"' (", &*e.to_string(), ")"), errors::NO_PACK_MSK);
    });

    data.gen_meta(pack);
    stop_if_errors();

    data.read_namespaces();
    stop_if_errors();

    //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
    times.2 = Instant::now();

    data.compile_namespaces();
    stop_if_errors();

    //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
    times.3 = Instant::now();

    data.save();
    if mov.is_some() {
        data.move_clear(mov, clear);
    }
    stop_if_errors();

    times.0 += times.1.elapsed();
    times.1 += times.2.elapsed();
    times.2 += times.3.elapsed();
    times.3 += Instant::now().elapsed();
    
    print_warnings(&data);

    status(format!("{} \x1b[96m\x1b[3m[{}/{}/{}/{} ms (s/r/c/w)]\x1b[m", "Done".form_foreground(str::GRN),
                   time(&times.0), time(&times.1), time(&times.2), time(&times.3)));
}

fn stop_if_errors() {
    if unsafe { HIT_ERROR } {
        status_color("Aborting due to previous errors".into(), str::RED);
        exit(errors::TOO_MANY_ERRORS);
    }
}

fn time(inst: &Instant) -> f32 {
    (inst.elapsed().as_micros() as f32 / 100f32).round() / 10f32
}


//         |         
//      |  |  |      
//      |  |  |  |   
// ----------------- 
//   |  |  |  |      
//      |  |  |__    
//         |         
