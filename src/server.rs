// for printing stuff i dont want in the main file

use std::fs::{read_dir, ReadDir};
use crate::*;

static mut WARNINGS: Vec<String> = vec![];
pub static mut HIT_ERROR: bool = false;

pub mod errors {
    pub const BAD_CLI_ARGS: i32 = 1;
    pub const TOO_MANY_ERRORS: i32 = 2;
}

pub fn print_warnings(pack: &Datapack) {
    unsafe {
        if WARNINGS.len() > 0 {
            println!();
            status(format!(
                "'{}' Generated {} Warnings: ",
                pack.get_view_name(),
                WARNINGS.len()
            ));
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
                    ),
                );
            }
        }
    }
}

pub fn warn(message: String) {
    println!("{}", join!("\x1b[93m‼»\x1b[m   [", &*"Warning".form_foreground(String::ORN).form_bold(), "] ", &*message));
    unsafe {
        WARNINGS.push(message);
    }
}

unsafe fn print_warning(message: String) {
    println!("\x1b[93m‼»\x1b[m   [{}] {}", WARNINGS.len(), message);
}

pub fn format_out(message: &str, path: &str, ln: usize) -> String {
    message.to_string() + &[" ./src/", path, ":", &*ln.to_string()].join("").replace("/", "\\")
}

pub fn death_error(message: String) -> ! {
    error(message);
    stop_if_errors();
    panic!();
}

pub fn error(message: String) {
    unsafe {HIT_ERROR = true}
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
    const GRY: usize = 0; //ignore
    const RED: usize = 1; //errors
    const GRN: usize = 2; //good stuff
    const ORN: usize = 3; //warns
    const BLU: usize = 4; //names
    const PNK: usize = 5; //ns
    const AQU: usize = 6; //debug
    const WHT: usize = 7; //unused

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