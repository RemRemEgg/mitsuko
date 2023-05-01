// i want smaller main files ok???

use std::env;
use crate::join;
use crate::server::*;

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
        std::process::exit(errors::BAD_CLI_ARGS);
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