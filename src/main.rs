// new code go brrrrrrrrrr

extern crate core;

mod server;
mod ast;
mod minecraft;
mod compile;

use std::{env, fs};
use std::process::exit;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use server::*;
use ast::*;
use minecraft::*;

static CURRENT_PACK_VERSION: u8 = 15;
/**projects/NDL/src*/
static mut SRC: String = String::new();
/**projects/NDL*/
static mut PROJECT_ROOT: String = String::new();
static MITSUKO: &str = include_str!("mitsuko.txt");

//todo
//  remove only parts of cache that need to be removed
//  cache warnings, warnings stop caches
//  option for re-using compiled output for cache
//  cache extras folder
//  match statement
//  add multi datapack bundling

fn main() {
    let mut times = (Instant::now(), Instant::now(), Instant::now(), Instant::now());
    println!();
    let msgs = MITSUKO.split("\n").collect::<Vec<_>>();
    let msg = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::from_millis(0)).as_micros() as usize % msgs.len();
    status(join![&*join!["[Mitsuko: ", msgs[msg].trim(), "]"].form_background(str::GRY), " ", &*env::args().collect::<Vec<String>>()[1..].join(" ").form_foreground(str::GRY)]);

    let args = get_cli_args();

    unsafe {
        PROJECT_ROOT = args.input.clone();
        SRC = args.input.clone();
        SRC.push_str("/src");
    }
    
    if args.cache {
        read_cached_data(&*args.input);
    }

    let mut data = Datapack::new(args.input.clone());

    //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
    times.1 = Instant::now();

    status(["Building '", &*data.src_loc.form_foreground(str::PNK), "'"].join(""));

    let pack = fs::read_to_string(join![&*data.src_loc, "/src/pack.msk"]).unwrap_or_else(|e| {
        death_error(join!("Could not read '",&*"pack.msk".form_foreground(str::ORN),"' (", &*e.to_string(), ")"), errors::NO_PACK_MSK);
    });

    data.gen_meta(pack, args.cache);
    stop_if_errors();

    data.read_namespaces();
    stop_if_errors();

    //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
    times.2 = Instant::now();

    data.compile_namespaces();
    stop_if_errors();

    //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
    times.3 = Instant::now();

    data.save(args.output, args.cache);
    if args.export {
        data.export();
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
    unsafe {
        if HIT_ERROR != 0 {
            status_color("Aborting due to previous errors [".to_string() + &*HIT_ERROR.to_string() + "]", str::RED);
            exit(errors::UNKNOWN_ERROR);
        }
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
