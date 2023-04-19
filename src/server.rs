// for printing stuff i dont want in the main file
use crate::*;

pub fn print_warnings(pack: &Datapack) {
    if pack.warnings.len() > 0 {
        println!();
        status(format!(
            "'{}' Generated {} Warnings: ",
            pack.meta.name,
            pack.warnings.len()
        ));
        let mut t = Datapack::blank();
        for (i, e) in pack.warnings.iter().enumerate() {
            print_warning(
                format!(
                    "{}{}",
                    e,
                    if i == pack.warnings.len() - 1 {
                        "\n"
                    } else {
                        ""
                    }
                ),
                &mut t,
            );
        }
    }
}

pub fn warn(message: String, warnings: &mut Vec<String>) {
    println!("{}", join!("\x1b[93m‼»\x1b[m   [", &*"Warning".form_foreground(String::ORN).form_bold(), "] ", &*message));
    warnings.push(message);
}

pub fn print_warning(message: String, pack: &mut Datapack) {
    println!("\x1b[93m‼»\x1b[m   [{}] {}", pack.warnings.len(), message);
    pack.warnings.push(message);
}

pub fn format_out(message: &str, path: &str, ln: usize) -> String {
    message.to_string() + &[" ./src/", path, ":", &*ln.to_string()].join("").replace("/", "\\")
}

pub fn error(message: String) -> ! {
    eprintln!("{}", join!("💀   [", &*"Error".form_foreground(String::RED).form_italic().form_bold(), "] ", &*message));
    panic!("{}", message);
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