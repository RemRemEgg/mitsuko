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

pub fn _trim_white_space(input: Vec<String>) -> (Vec<String>, usize) {
    let mut out = Vec::new();
    let mut c: usize = 0usize;
    for i in input {
        if !(i.is_empty() || i.starts_with("//")) {
            out.push(i);
        } else {
            c = c + 1usize;
        }
    }
    (out, c)
}

pub fn warn(message: String, warnings: &mut Vec<String>) {
    println!("‼»   [Warning] {}", message);
    warnings.push(message);
}

pub fn print_warning(message: String, pack: &mut Datapack) {
    println!("‼»   [{}] {}", pack.warnings.len(), message);
    pack.warnings.push(message);
}

pub fn format_out(message: &str, path: &str, ln: usize) -> String {
    message.to_string() + &[" ./src/", path, ":", &*ln.to_string()].join("").replace("/", "\\")
}

pub fn error(message: String) -> ! {
    eprintln!("💀   [Error] {}", message);
    panic!("{}", message);
}

pub fn status(message: String) {
    println!(" »   {}", message);
}

pub fn debug(message: String) {
    println!("§»   {}", message);
}

#[macro_export]
macro_rules! join {
    ( $( $x:expr ),* ) => {
            [$($x,)*""].join("")
    };
}