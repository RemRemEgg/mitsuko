// new code go brrrrrrrrrr

extern crate core;

mod server;
mod ast;
mod minecraft;
mod compile;

use std::{env, fs};
use std::process::exit;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use remtools::{*, colors::*};
use server::*;
use ast::*;
use minecraft::*;

static CURRENT_PACK_VERSION: u8 = 18;
/**projects/NDL/src*/
static mut SRC: String = String::new();
/**projects/NDL*/
static mut PROJECT_ROOT: String = String::new();
static MITSUKO: &str = include_str!("mitsuko.txt");

//todo
//  macro support
//  match statement
//  add multi datapack bundling
//  add macro support for @alias
//  scalar for result
//  warnings for un-known functions dont appear

//todo caching
//  There is a very high chance im going to completely rework the caching system to make it faster and smaller
//  redo all caching probs idk
//  cache extras folder
//  remove only parts of cache that need to be removed
//  cache warnings, warnings stop caches
//  option for re-using compiled output for cache

fn main() {
    let mut times = (Instant::now(), Instant::now(), Instant::now(), Instant::now());
    println!();
    let msgs = MITSUKO.split("\n").collect::<Vec<_>>();
    let msg = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::from_millis(0)).as_micros() as usize % msgs.len();
    status(join![&*join!["[Mitsuko: ", msgs[msg].trim(), "]"].background(GRY).end(), " ", &*env::args().collect::<Vec<String>>()[1..].join(" ").foreground(GRY).end()]);

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

    status(["Building '", &*data.src_loc.clone().foreground(PNK).end(), "'"].join(""));

    let pack = fs::read_to_string(join![&*data.src_loc, "/src/pack.msk"]).unwrap_or_else(|e| {
        death_error(join!("Could not read '",&*"pack.msk".foreground(ORN).end(),"' (", &*e.to_string(), ")"), errors::NO_PACK_MSK);
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

    status(format!("{} \x1b[96m\x1b[3m[{}/{}/{}/{} ms (s/r/c/w)]\x1b[m", "Done".foreground(GRN).end(),
                   time(&times.0), time(&times.1), time(&times.2), time(&times.3)));
}

fn stop_if_errors() {
    unsafe {
        if HIT_ERROR != 0 {
            status_color("Aborting due to previous errors [".to_string() + &*HIT_ERROR.to_string() + "]", RED);
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
