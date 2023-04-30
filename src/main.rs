// code go brrrrrrrrrr

use server::*;
use std::cmp::min;
use std::fmt::{Display, Formatter};
use std::fs::{read_dir, remove_dir_all, File, ReadDir};
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use std::{env, fs, io};

mod server;
mod tests;
mod build;

static VERBOSE: i32 = 0;
static CURRENT_PACK_VERSION: u8 = 13;

fn main() {
    status_color(env::args().collect::<Vec<String>>()[1..].join(" "), str::GRY);
    let mut imports = vec![];
    let ims = read_dir("imports");
    if ims.is_ok() {
        for im in ims.unwrap().map(|x| x.unwrap()) {
            if !im.path().is_dir() && im.path().to_str().unwrap_or("").ends_with(".msk") {
                let i = fs::read_to_string(im.path()).unwrap_or("".to_string());
                let i = i.split(['\n', '\r']).collect::<Vec<&str>>();
                let ins = im.file_name().to_string_lossy().to_string();
                let ins = ins.split_once(".").unwrap_or(("minecraft", "msk")).0;
                for s in i.into_iter() {
                    if s != "" {
                        imports.push(join![ins, ":", s]);
                    }
                }
            }
        }
    } else {
        warn("No import folder found".to_string(), &mut vec![]);
    }
    status(join!["Found ", &*imports.len().to_string(), " external functions"]);

    let mut args = env::args().collect::<Vec<String>>().into_iter();
    let (mut pck, mut mov, mut clr) = ("".to_string(), None, false);
    while let Some(arg) = args.next() {
        match &*arg {
            "-pack" => pck = args.next().unwrap_or("".to_string()),
            "-move" => mov = args.next(),
            "-clear" => clr = true,
            _ => {}
        }
    }
    status(["Compiling '", &*pck.form_foreground(str::PNK), "'"].join(""));

    let pack = fs::read_to_string([&*pck, "src/pack.msk"].join("/"));
    if pack.is_err() {
        error(join!("Could not find '",&*"pack.msk".form_foreground(str::ORN),"'"));
    }

    let mut data = Datapack::new(
        pack.unwrap(),
        &pck,
    );
    data.callable_functions.append(&mut imports);

    let nss = read_dir([&*pck, "src"].join("/"));
    if nss.is_ok() {
        for ns in nss.unwrap().map(|x| x.unwrap()) {
            if ns.path().is_dir() {
                data = compile_namespace(
                    data,
                    &ns.file_name().to_string_lossy().to_string(),
                    ns.path().to_str().unwrap_or(""),
                )
            }
        }
    } else {
        warn("No namespaces found".to_string(), &mut data.warnings);
    }

    data.warn_unknown_functions();

    let name = &*data.meta.name.clone();

    data = save_pack(data);

    if mov.is_some() {
        let mut result_path = env::current_dir().unwrap();
        result_path.push(&*pck);
        result_path.push("generated");
        result_path.push(&name);
        let world = mov.unwrap();
        let world = [&*world, "datapacks", &name].join("\\");
        if clr {
            let t = remove_dir_all(&world);
            if t.is_err() {
                warn(
                    join!["Could not clear pre-existing datapack (", &*t.unwrap_err().to_string(), ")"].form_foreground(str::RED),
                    &mut Vec::new(),
                );
            }
        }
        println!(
            "Copying {} to {}",
            result_path.to_str().unwrap_or("error").form_underline(),
            &*world.form_underline()
        );
        copy_dir_all(result_path, world).expect(&*"Failed to copy datapack".form_foreground(str::RED));
    }

    print_warnings(&data);

    status_color("Done".form_bold(), str::GRN);
}

fn compile_namespace(mut pack: Datapack, namespace: &String, src: &str) -> Datapack {
    let mut ns = Namespace::new(namespace.clone(), pack.meta.clone());

    let fn_fr = read_dir([&src, "functions"].join("/"));
    if fn_fr.is_ok() {
        let fn_f = fn_fr.unwrap();
        let functions: Vec<(String, Vec<String>)> = get_msk_files_split(fn_f);
        for (file, lines) in functions.into_iter() {
            ns = process_function_file(ns, file, lines)
        }
        finalize_functions(&mut ns, &pack);
    } else {
        warn(["No '", &*"functions".form_foreground(str::BLU), "' folder found for '", &*namespace.form_foreground(str::PNK), "'"].join(""), &mut pack.warnings);
    }

    let el_fr = read_dir([&src, "event_links"].join("/"));
    if el_fr.is_ok() {
        let el_f = el_fr.unwrap();
        let links: Vec<(String, Vec<String>)> = get_msk_files_split(el_f);
        for (file, lines) in links.into_iter() {
            ns = process_link_file(ns, file, lines)
        }
    }

    let it_fr = read_dir([&src, "items"].join("/"));
    if it_fr.is_ok() {
        let it_f = it_fr.unwrap();
        let items: Vec<(String, Vec<String>)> = get_msk_files_split(it_f);
        for (file, lines) in items.into_iter() {
            ns = process_item_file(ns, file, lines)
        }
    }

    pack.warnings.append(&mut ns.warnings);
    let mut k: Vec<String> = vec![];
    for f in ns.functions.iter() {
        k.push(join!(&*ns.id, ":", &*f.name));
    }
    pack.callable_functions.append(&mut k);
    pack.namespaces.push(ns);
    pack
}

fn get_msk_files_split(fn_f: ReadDir) -> Vec<(String, Vec<String>)> {
    let mut out = vec![];
    for dir_r in fn_f {
        let dir = dir_r.expect("Error occurred while trying to read .msk file");
        if dir.path().is_dir() {
            let add = get_msk_files_split(read_dir(dir.path()).unwrap());
            let mut vect = add
                .iter()
                .map(|x| {
                    (
                        [
                            dir.path().file_name().unwrap().to_str().unwrap(),
                            "$",
                            &*x.0,
                        ]
                            .join(""),
                        x.1.clone(),
                    )
                })
                .collect::<Vec<(String, Vec<String>)>>();
            out.append(&mut vect);
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
                out.push((name.split(".").next().unwrap_or("null").to_string(), lines));
            }
        }
    }
    out.iter_mut().for_each(|mut fl| {
        fl.0 = fl.0.replace('$', "/");
    });
    out.sort_by(|a, b| {
        if a.0.eq("functions") {
            return std::cmp::Ordering::Less;
        }
        if b.0.eq("functions") {
            return std::cmp::Ordering::Greater;
        }
        let u = a.0.matches('/').count();
        let v = b.0.matches('/').count();
        return if u > v {
            std::cmp::Ordering::Less
        } else if u < v {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        };
    });
    out
}

fn process_function_file(ns: Namespace, name: String, lines: Vec<String>) -> Namespace {
    println!();
    status(["Processing function file '", &*name.form_foreground(str::BLU), "' from '", &*ns.id.form_foreground(str::PNK), "'"].join(""));
    let t_scan = Instant::now();
    let (mut ns, mut fns, mut warnings) = scan_for_functions(ns, &*name, lines);
    status(format!(
        "Found {} functions in {} µs, {} total",
        fns.len(),
        t_scan.elapsed().as_micros(),
        ns.functions.len() + fns.len()
    ));
    ns.warnings.append(&mut warnings);
    ns.functions.append(&mut fns);
    ns
}

fn scan_for_functions(
    ns: Namespace,
    prefix: &str,
    lines: Vec<String>,
) -> (Namespace, Vec<MCFunction>, Vec<String>) {
    let mut file: Carrier<MCFunction> = Carrier::new(prefix.to_string(), ns.meta.clone(), ns, "functions".to_string());
    file.lines = lines;
    'lines: loop {
        if file.lines.len() <= 0 {
            if file.meta.vb >= 1 {
                status("Found EOF".to_string());
            }
            break 'lines;
        }
        let remove = scan_pack_line(file.lines[0].to_string(), &mut file);
        file.ln += remove;
        for _ in 0..remove {
            file.lines.remove(0);
        }
        file.meta = file.ns.meta.clone();
    }
    (file.ns, file.items, file.warnings)
}

fn scan_pack_line(line: String, file: &mut Carrier<MCFunction>) -> usize {
    let rem: usize;
    let keys: Vec<&str> = line.trim().split(" ").collect::<Vec<_>>();
    let key_1: &str = keys.get(0).unwrap_or(&"◙");
    match key_1 {
        "fn" => {
            let key_2 = *keys.get(1).unwrap_or(&"");
            if MCFunction::is_valid_fn(key_2) && !key_2.contains(":") {
                let result = MCFunction::extract_from(keys, file);
                rem = result.0;
                file.add_item(result.1);
            } else {
                error(format_out(
                    &*["Invalid function name \'", &*key_2.form_foreground(str::BLU), "\'"].join(""),
                    &*file.get_path(&file.ns),
                    file.ln,
                ));
            }
        }
        _ => rem = scan_pack_char(line, file),
    }
    rem
}

fn scan_pack_char(line: String, file: &mut Carrier<MCFunction>) -> usize {
    let rem: usize = 1;
    let char_1: char = *line
        .trim()
        .chars()
        .collect::<Vec<_>>()
        .get(0)
        .unwrap_or(&'◙');
    match char_1 {
        '#' => test_tag(line, file),
        '/' | '◙' | ' ' => {}
        _ => error(format_out(
            &*["Unexpected token ->", {
                let mut v = line.to_string();
                v.truncate(5);
                &*v.to_string()
            }, ""].join(""),
            &*file.get_path(&file.ns),
            file.ln,
        )),
    }
    rem
}

fn test_tag(line: String, mut file: &mut Carrier<MCFunction>) {
    let mut line = line.trim();
    let (mut pre, mut post) = ("error", "null");
    if {
        if line.starts_with("#[") && line.ends_with("]") {
            line = &line[2..line.len() - 1];
            let p = line.split_once("=").unwrap_or((pre, post));
            (pre, post) = (p.0.trim(), p.1.trim());
            true
        } else { false }
    } {
        set_tag(pre, post, &mut file);
    } else {
        error(format_out(
            &*["Malformed argument tag \'", &*line.form_foreground(str::BLU), "\'"].join(""),
            &*file.get_path(&file.ns),
            file.ln,
        ))
    }
}

fn set_tag(tag: &str, val: &str, file: &mut Carrier<MCFunction>) {
    let mut suc = true;
    match tag {
        "optimizations" => file.meta.opt_level = min(val.parse::<u8>().unwrap_or(0u8), 4u8),
        "debug" => file.meta.vb = min(val.parse::<i32>().unwrap_or(0), 3),
        "recursive_replace" => file.meta.recursive_replace = val.parse::<u8>().unwrap_or(3),
        "comments" => file.meta.comments = val.to_uppercase().eq("TRUE"),
        _ => {
            if file.meta.vb >= 1 {
                warn(format_out(
                    &*["Unknown tag: \'", &*tag.form_foreground(str::BLU), "\' (value = \'", &*val.form_foreground(str::GRY), "\')"].join(""),
                    &*file.get_path(&file.ns),
                    file.ln,
                ), &mut file.warnings);
            }
            suc = false
        }
    }
    if suc && file.meta.vb >= 1 {
        debug(format!("Set arg \'{}\' to \'{}\'", tag.form_foreground(str::BLU), val.form_foreground(str::AQU)));
    }
}

fn set_pack_meta(meta: &str, val: &str, mut pack: &mut Datapack) {
    let mut suc = true;
    match meta {
        "remgine" => pack.meta.remgine = val.to_uppercase().eq("TRUE"),
        "name" => pack.meta.name = val.to_string(),
        "comments" => pack.meta.comments = val.to_uppercase().eq("TRUE"),
        "version" => pack.meta.version = val.parse::<u8>().unwrap_or(CURRENT_PACK_VERSION),
        "description" => pack.meta.description = val.to_string(),
        "optimizations" => pack.meta.opt_level = min(val.parse::<u8>().unwrap_or(0u8), 4u8),
        "debug" => pack.meta.vb = min(val.parse::<i32>().unwrap_or(0), 3),
        "recursive_replace" => pack.meta.recursive_replace = val.parse::<u8>().unwrap_or(3),
        _ => {
            warn(
                format_out(
                    &*["Unknown pack tag: \'", &*meta.form_foreground(str::BLU), "\' (value = \'", &*val.form_foreground(str::GRY), "\')"].join(""),
                    "pack",
                    pack.ln,
                ),
                &mut pack.warnings,
            );
            suc = false
        }
    }
    if suc && pack.meta.vb >= 1 {
        debug(format!("Set arg \'{}\' to \'{}\'", meta.form_foreground(str::BLU), val.form_foreground(str::AQU)));
    }
}

fn finalize_functions(mut ns: &mut Namespace, mut _pack: &Datapack) {
    println!();
    let t_total = Instant::now();

    let t_compile = Instant::now();
    let (mut ln_total, mut cm_total) = (0, 0);
    ns.functions.iter().for_each(|f| {
        ln_total += f.lines.len();
        ns.export_functions.push(f.name.clone());
    });
    compile_functions(&mut ns);
    ns.functions.iter().for_each(|f| {
        cm_total += f.commands.len();
    });
    status(format!(
        "Compiled {} lines into {} commands within {} µs\n",
        ln_total,
        cm_total,
        t_compile.elapsed().as_micros()
    ));

    /* CLEAN */
    let t_clean = Instant::now();
    clean_functions(&mut ns);
    status(join!(
        &*"Cleaned up".form_foreground(str::GRN), " in ", &*t_clean.elapsed().as_micros().to_string(), " µs\n"
    ));

    status(format!(
        "Finished function generation for {} in {} µs",
        ns.id,
        t_total.elapsed().as_micros()
    ));
}

fn compile_functions(mut ns: &mut Namespace) -> &Namespace {
    ns.ln = 0;
    'functions: loop {
        if ns.ln >= ns.functions.len() {
            break 'functions ns;
        }
        if ns.meta.vb >= 1 {
            debug(format!("Compiling Function '{}'", ns.functions[ns.ln].name.form_foreground(str::AQU)));
        }
        let mut f = ns.functions.remove(ns.ln);
        f.compile(&mut ns);
        ns.functions.insert(ns.ln, f);
        ns.ln += 1;
    }
}

fn clean_functions(ns: &mut Namespace) {
    for fi in 0..ns.functions.len() {
        for _i in 0..ns.functions[fi].commands.len() {
            // let mut c = ns.functions[fi].commands[i].clone();
            // optimize code!
            // ns.functions[fi].commands[i] = c;
        }
    }
}

fn process_link_file(mut ns: Namespace, path: String, lines: Vec<String>) -> Namespace {
    println!();
    status(["Processing link file '", &*join!(&*ns.id, "/event_links/", &*path).form_foreground(str::BLU), "'"].join(""));
    let t_scan = Instant::now();

    let mut lks = Vec::new();
    for (ln, line) in lines.into_iter().enumerate() {
        if line.eq("") { continue; }
        let line = line.trim().split(" : ").collect::<Vec<_>>();
        if line.len() < 2 {
            warn(format_out("Not enough arguments to link", &*[&*ns.id, "event_links", &*path].join("/"), ln + 1), &mut ns.warnings);
        } else {
            let links = line[1].trim().replace(" ", "");
            let links = links.split(",").filter(|l| !l.eq(&"none"));
            let links = links.map(|f| MCFunction::parse_name(&*[f, "()"].join(""), &*ns.id).unwrap_or("◙".to_string()));
            let links = links.filter(|f| !f.eq("◙"));
            let links = links.collect::<Vec<_>>();
            lks.push(Link::new(path.clone(), line[0].clone().to_string(), links));
        }
    }

    status(format!(
        "Made {} links in {} µs, {} total",
        lks.len(),
        t_scan.elapsed().as_micros(),
        ns.links.len() + lks.len()
    ));
    ns.links.append(&mut lks);
    ns
}

fn process_item_file(mut ns: Namespace, path: String, lines: Vec<String>) -> Namespace {
    println!();
    status(["Processing item file '", &*join!(&*ns.id, "/items/", &*path).form_foreground(str::BLU), "'"].join(""));
    let t_scan = Instant::now();

    let mut item = Item::new(path, lines, &ns);
    ns.functions.push(item.call.clone());
    ns.functions.append(&mut item.adds.0.drain(..).collect::<Vec<MCFunction>>());
    ns.warnings.append(&mut item.adds.1.drain(..).collect::<Vec<String>>());

    status(format!(
        "Made item '{}' in {} µs",
        &*item.file_name,
        t_scan.elapsed().as_micros(),
    ));
    ns.items.push(item);
    ns
}

fn save_pack(mut pack: Datapack) -> Datapack {
    let save_time = Instant::now();
    if pack.meta.vb >= 1 {
        status(format!(
            "Saving '{}' @ '{}'",
            &pack.meta.name.form_foreground(str::PNK),
            match env::current_dir() {
                Ok(mut path) => {
                    path.push(&*pack.meta.name);
                    path.to_str().unwrap_or("").form_underline()
                }
                Err(err) => {
                    err.to_string()
                }
            }
        ));
    } else {
        status(format!("Saving '{}'", &pack.meta.name.form_foreground(str::PNK)));
    }

    remove_dir_all([&*pack.src, "generated"].join("/")).ok();
    remove_dir_all([&*pack.src, "exports"].join("/")).ok();
    let root_path = [&*pack.src, "generated", &*pack.meta.name].join("/");

    remove_dir_all(&*root_path).ok();
    make_folder(&*root_path);
    make_folder(&*join![&*pack.src, "/exports"]);

    let mut meta =
        File::create([&*root_path, "/pack.mcmeta"].join("")).expect("Could not make '\x1b[93mpack.mcmeta\x1b[m'");
    let meta_template = include_str!("pack.mcmeta")
        .replace("{VERS}", &*pack.meta.version.to_string())
        .replace("{DESC}", &pack.meta.description);
    meta.write_all(meta_template.as_bytes())
        .expect("Could not make '\x1b[93mpack.mcmeta\x1b[m'");
    let tag_template = include_str!("tag.json");
    let recipe_template = include_str!("recipe.json");
    let adv_craft_template_119 = include_str!("advancement_craft_1.19.json");
    let adv_craft_template_120 = include_str!("advancement_craft_1.20.json");
    let mat_template = r#""$ID$": {"item": "minecraft:$TYPE$"}"#.to_string();
    let mat_tag_template = r#""$ID$": {"tag": "minecraft:$TYPE$"}"#.to_string();
    for namespace in pack.namespaces.iter_mut() {
        let ns_path = &*[&*root_path, "/data/", &*namespace.id].join("");

        make_folder(ns_path);

        let rc_path = &*join![ns_path, "/recipes"];
        let av_path = &*join![ns_path, "/advancements"];

        make_folder(rc_path);
        make_folder(av_path);

        for item in &namespace.items {
            let path = &*[rc_path, "/", &*item.path, ".json"].join("");
            if item.path.contains("/") {
                let mut path = item.path.split("/").collect::<Vec<_>>();
                path.pop();
                path.insert(0, "/");
                path.insert(0, &*rc_path);
                make_folder(&*path.join("/"));
            }
            let mut write_recipe = recipe_template.to_string();
            let mut file =
                File::create(path).expect(&*["Could not make item recipe '", &*path.form_foreground(str::BLU), "'"].join(""));
            write_recipe = write_recipe.replace("$PATTERN$", &*item.recipe.join(",\n    "));

            let mut mats = vec![];
            for mat in &item.materials {
                if mat.1.starts_with("#") {
                    mats.push(mat_tag_template.clone().replace("$ID$", &*mat.0).replace("$TYPE$", &mat.1[1..]));
                } else {
                    mats.push(mat_template.clone().replace("$ID$", &*mat.0).replace("$TYPE$", &*mat.1));
                }
            }
            write_recipe = write_recipe.replace("$MATERIALS$", &*mats.join(",\n    "));

            file.write_all(write_recipe.as_bytes())
                .expect(&*["Could not write item recipe file '", &*path.form_foreground(str::BLU), "'"].join(""));

            let path = &*[av_path, "/", &*item.path, ".json"].join("");
            if item.path.contains("/") {
                let mut path = item.path.split("/").collect::<Vec<_>>();
                path.pop();
                path.insert(0, "/");
                path.insert(0, &*av_path);
                make_folder(&*path.join("/"));
            }
            let mut write_adv = if pack.meta.version >= 14 {adv_craft_template_120} else {adv_craft_template_119}.to_string();
            let mut file =
                File::create(path).expect(&*["Could not make item advancement '", &*path.form_foreground(str::BLU), "'"].join(""));
            write_adv = write_adv.replace("$PATH$", &*join![&*namespace.id, ":", &*item.path]);
            write_adv = write_adv.replace("$CALL$", &*join![&*namespace.id, ":", &*item.path]);
            file.write_all(write_adv.as_bytes())
                .expect(&*["Could not write item advancement file '", &*path.form_foreground(str::BLU), "'"].join(""));
        }

        let fn_path = &*[ns_path, "/functions"].join("");

        make_folder(fn_path);

        for function in &namespace.functions {
            if function.name.ends_with("_IMPORT") {
                continue;
            }
            let path = &*[fn_path, "/", &*function.name, ".mcfunction"].join("");
            if function.name.contains("/") {
                let mut path = function.name.split("/").collect::<Vec<_>>();
                path.pop();
                path.insert(0, "/");
                path.insert(0, &*fn_path);
                make_folder(&*path.join("/"));
            }
            let mut file =
                File::create(path).expect(&*["Could not make function '", &*path.form_foreground(str::BLU), "'"].join(""));
            file.write_all(function.commands.join("\n").as_bytes())
                .expect(&*["Could not write function '", &*path.form_foreground(str::BLU), "'"].join(""));
        }

        if read_dir(&*[&*pack.src, "/src/", &*namespace.id, "/extras"].join("")).is_ok() {
            copy_dir_all(&*[&*pack.src, "/src/", &*namespace.id, "/extras"].join(""), &*ns_path).expect("Could not copy 'extras' folder");
        }

        let mut file =
            File::create(&*join![&*pack.src,"/exports/",&*namespace.id,".msk"]).expect(&*["Could not make export file '", &*namespace.id.form_foreground(str::PNK), "'"].join(""));
        file.write_all(namespace.export_functions.join("\n").as_bytes())
            .expect(&*["Could not write export file '", &*namespace.id.form_foreground(str::PNK), "'"].join(""));
    }

    let mut links: Vec<Link> = Vec::new();
    pack.namespaces.iter_mut().for_each(|ns| {
        'lks: loop {
            if ns.links.len() <= 0 {
                break 'lks;
            }
            let mut l = ns.links.remove(0);
            let mut p2 = false;
            l.links = l.links.iter().filter(|&ld| {
                p2 = false;
                let split = ld.split_once(":").unwrap_or((&*ns.id, &**ld));
                let p3 = pack.callable_functions.contains(&join!(split.0, ":", split.1));
                p2 = p2 || p3;
                if !p2 {
                    warn(["No such function '", &*ld.form_foreground(str::ORN), "' found for link '", &*l.path.form_foreground(str::BLU), "' ./src/", &*ns.id, "/event_links/", &*l.path].join(""), &mut pack.warnings);
                }
                p2
            }).map(|s| s.clone()).collect();
            let mut p = true;
            for l2 in links.iter_mut() {
                if l.name == l2.name && l.path == l2.path {
                    l2.links.append(&mut l.links);
                    p = false;
                    break;
                }
            }
            if p {
                links.push(l);
            }
        }
    });

    for link in links.into_iter() {
        let lk_path = &*[&*root_path, "/data/", &*link.path, "/tags/functions"].join("");
        make_folder(&*lk_path);
        let path = &*[lk_path, "/", &*link.name, ".json"].join("");
        if link.name.contains("/") {
            let mut path = link.name.split("/").collect::<Vec<_>>();
            path.pop();
            path.insert(0, "/");
            path.insert(0, &*lk_path);
            make_folder(&*path.join("/"));
        }
        let mut file =
            File::create(path).expect(&*["Could not make link file '", &*path.form_foreground(str::BLU), "'"].join(""));
        let len = link.links.len();
        let w = link.links.iter().enumerate().map(|(pos, lk)| ["\"", &**lk, "\"", if pos + 1 >= len { "" } else { "," }].join("")).collect::<Vec<_>>();
        let write = tag_template.clone().replace("$VALUES$", &*w.join("\n    "));
        file.write_all(write.as_bytes())
            .expect(&*["Could not write link file '", &*path.form_foreground(str::BLU), "'"].join(""));
    }

    status(format!(
        "Saved Datapack in {} µs",
        save_time.elapsed().as_micros()
    ));

    pack
}

fn make_folder(path: &str) {
    fs::create_dir_all(path).unwrap_or_else(|e| {
        error(format!("Could not generate '{path}' folder: {e}"));
    });
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

struct Carrier<T> {
    items: Vec<T>,
    lines: Vec<String>,
    ln: usize,
    name: String,
    meta: Meta,
    ns: Namespace,
    warnings: Vec<String>,
    classification: String,
}

impl<T> Carrier<T> {
    fn new(name: String, meta: Meta, ns: Namespace, classification: String) -> Carrier<T> {
        Carrier {
            items: vec![],
            lines: vec![],
            ln: 1,
            name,
            meta,
            ns,
            warnings: vec![],
            classification,
        }
    }

    pub fn add_item(&mut self, item: T) -> &mut Carrier<T> {
        self.items.push(item);
        self
    }

    pub fn get_path(&self, ns: &Namespace) -> String {
        [&*ns.id, &*self.classification, &*self.name].join("/")
    }
}

#[derive(Clone, Debug)]
struct Meta {
    vb: i32,
    version: u8,
    remgine: bool,
    opt_level: u8,
    comments: bool,
    name: String,
    description: String,
    recursive_replace: u8,
}

impl Meta {
    fn new() -> Meta {
        Meta {
            vb: VERBOSE,
            version: CURRENT_PACK_VERSION,
            remgine: true,
            opt_level: 0,
            comments: false,
            name: "Untitled".to_string(),
            description: "A Datapack".to_string(),
            recursive_replace: 3,
        }
    }
}

#[derive(Clone)]
struct Namespace {
    id: String,
    functions: Vec<MCFunction>,
    links: Vec<Link>,
    items: Vec<Item>,
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
            functions: Vec::new(),
            links: Vec::new(),
            items: Vec::new(),
            meta,
            ln: 0,
            warnings: vec![],
            export_functions: vec![],
        }
    }
}

pub struct Datapack {
    meta: Meta,
    ln: usize,
    namespaces: Vec<Namespace>,
    warnings: Vec<String>,
    src: String,
    callable_functions: Vec<String>,
}

impl Datapack {
    fn new(meta: String, src: &String) -> Datapack {
        let mut pack = Datapack::blank();
        pack.src = src.to_string();
        pack.meta.name = src.to_string();
        let input = meta.split("\n").collect::<Vec<&str>>();

        for tag in input {
            let s = tag.split("=").collect::<Vec<&str>>();
            set_pack_meta(s[0].trim(), s[1].trim(), &mut pack);
            pack.ln += 1;
        }

        pack
    }

    fn blank() -> Datapack {
        Datapack {
            meta: Meta::new(),
            ln: 1,
            warnings: vec![],
            src: "".to_string(),
            namespaces: vec![],
            callable_functions: vec![],
        }
    }

    fn warn_unknown_functions(&mut self) {
        let mut calls = Vec::new();
        for ns in &self.namespaces {
            for fi in 0..ns.functions.len() {
                for call in &ns.functions[fi].calls {
                    let path = ns.functions[fi].get_path(&ns);
                    let mut name = call.0.to_string();
                    name.push_str("()");
                    calls.push((MCFunction::parse_name(&*name, &*ns.id).unwrap_or("error".to_string()), path, call.1));
                }
            }
        }
        calls.retain(|c| {
            let split = c.0.split_once(":").unwrap_or(("internal_error", "ignore_me"));
            !self.is_known_function(split.1, split.0)
        });
        for c in calls {
            warn(format_out(
                &*["Unknown or undefined function '", &*c.0.form_foreground(str::ORN), "'"].join(""),
                &*c.1.to_string(), c.2,
            ), &mut self.warnings);
        }
    }

    fn is_known_function(&self, function: &str, ns: &str) -> bool {
        self.callable_functions.contains(&join![ns, ":", function])
    }
}

impl Display for Datapack {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut c = 0usize;
        self.namespaces
            .iter()
            .for_each(|f| f.functions.iter().for_each(|a| c += a.commands.len()));
        f.write_fmt(format_args!("Datapack['{}']", self.meta.name))
    }
}

#[derive(Debug, Clone)]
pub struct MCFunction {
    lines: Vec<String>,
    commands: Vec<String>,
    name: String,
    calls: Vec<(String, usize)>,
    ln: usize,
    vars: Vec<(String, String)>,
    meta: Meta,
    file: String,
    compiled: bool,
    premc: usize,
}

impl MCFunction {
    fn new(name: &str, ln: usize, meta: &Meta, file: &String) -> MCFunction {
        MCFunction {
            lines: vec![],
            commands: vec![],
            name: name[..name.len() - 2].to_string(),
            calls: vec![],
            ln,
            vars: vec![],
            meta: meta.clone(),
            file: file.to_string(),
            compiled: false,
            premc: 0,
        }
    }

    pub fn is_valid_fn(function: &str) -> bool {
        let mut function = function;
        if function.contains(":") {
            function = function.split_once(":").unwrap_or(("", "")).1;
        }
        if function.ends_with("()") {
            let mut nid = function[..function.len() - 2].replace(|ch| ch >= 'a' && ch <= 'z', "");
            nid = nid.replace(|ch| ch >= '0' && ch <= '9', "");
            nid = nid.replace("_", "");
            nid = nid.replace("/", "");
            nid = nid.replace(".", "");
            nid.len() == 0
        } else {
            false
        }
    }

    fn extract_from(keys: Vec<&str>, file: &mut Carrier<MCFunction>) -> (usize, MCFunction) {
        let mut mcf = MCFunction::new(
            &*[&*file.name, "/", keys[1]]
                .join("")
                .replace("functions/", ""),
            file.ln,
            &file.meta,
            &file.name,
        );
        if keys.get(2).unwrap_or(&"").starts_with("{") {
            if file
                .items
                .iter()
                .any(|fun| -> bool { fun.name.eq(&mcf.name) })
            {
                error(format_out(
                    &*["Duplicate function name \'", &*mcf.name.form_foreground(str::BLU), "\'"].join(""),
                    &*mcf.get_path(&file.ns),
                    file.ln,
                ));
            }

            let rem = mcf.extract_block(file);
            if file.meta.vb >= 1 {
                debug(format!(
                    "Found function \'{}\' ./src/{}",
                    mcf.name.form_foreground(str::BLU),
                    file.get_path(&file.ns)
                ).replace("/", "\\"));
                if file.meta.vb >= 2 {
                    debug(format!(" -> {} Lines REM", rem));
                }
            }
            (rem, mcf)
        } else {
            error(format_out(
                &*["Expected '{' after \'fn ", keys[1], "\'"].join(""),
                &*mcf.get_path(&file.ns),
                file.ln,
            ))
        }
    }

    fn extract_block(&mut self, file: &mut Carrier<MCFunction>) -> usize {
        if file.lines[0].ends_with('}') {
            return 1;
        }
        let mut b = Blocker::new();
        let rem = match b.find_size_vec(&file.lines, file.lines[0].find("{").unwrap_or(0)) {
            Ok(o) => {
                if o.0 != Blocker::NOT_FOUND {
                    for i in 1..o.0 {
                        self.lines.push(file.lines[i].to_string());
                    }
                    o.0 + 1
                } else {
                    error(format_out("Unterminated function", &*self.get_path(&file.ns), file.ln))
                }
            }
            Err(e) => error(format_out(&*e.0, &self.get_path(&file.ns), e.1 + self.ln)),
        };
        rem
    }

    fn compile(&mut self, ns: &mut Namespace) {
        if !self.compiled {
            let (funs, warns) = &mut self.compile_to(ns);
            ns.functions.append(funs);
            ns.warnings.append(warns);
            if ns.meta.vb >= 2 {
                debug(format!(" -> Resulted in {} commands", self.commands.len()));
            }
            self.compiled = true;
        }
    }

    fn compile_to(&mut self, ns: &Namespace) -> (Vec<MCFunction>, Vec<String>) {
        if !self.compiled {
            let mut funs = vec![];
            let mut warns = vec![];
            let mut ln = 0;
            'lines: loop {
                if ln >= self.lines.len() {
                    break 'lines;
                }
                let (rem, mut frets, mut w) = self.compile_line(ns, ln);
                ln += rem;
                funs.append(&mut frets);
                warns.append(&mut w);
            }
            self.vars.clear();
            self.compiled = true;
            return (funs, warns);
        }
        (vec![], vec![])
    }

    fn compile_line(&mut self, mut ns: &Namespace, ln: usize) -> (usize, Vec<MCFunction>, Vec<String>) {
        let (rem, mut add, f, w) = self.compile_text(&mut ns, ln);
        self.premc = add.len();
        self.commands.append(&mut add);
        (rem, f, w)
    }

    fn compile_text(&mut self, ns: &Namespace, ln: usize) -> (usize, Vec<String>, Vec<MCFunction>, Vec<String>) {
        if self.lines[ln].starts_with("//") {
            return (1, vec![], vec![], vec![]);
        }
        if self.lines[ln].starts_with("}") {
            return (self.lines.len() - ln, vec![], vec![], vec![]);
        }
        let path = &*self.get_path(ns);
        let text: &mut String = &mut self.lines[ln];

        if text.starts_with("cmd") || ns.meta.opt_level == 255 {
            return (1, vec![text[4..].into()], vec![], vec![]);
        }
        if text.starts_with("@NOLEX cmd") {
            return (1, vec![text[11..].into()], vec![], vec![]);
        }

        for _ in 0..self.meta.recursive_replace {
            for i in self.vars.iter() {
                *text = text.replace(&*["*{", &*i.0, "}"].join(""), &*i.1);
            }
        }
        *text = text.replace("*{NS}", &*ns.id)
            .replace("*{NAME}", &*ns.meta.name)
            .replace("*{INT_MAX}", "2147483647")
            .replace("*{INT_MIN}", "-2147483648")
            .replace("*{PATH}", &*join![&*self.file, "/"].replace("functions/", ""))
            .replace("*NEAR1", "limit=1,sort=nearest")
            .replace("positioned as @s ", "positioned as @s[] ")
            .replace("as @s ", "")
            .replace("@s[]", "@s")
            .replace(" run execute", "")
            .replace("execute run ", "")
            .replace(" run run", " run")
            .replace("execute execute ", "execute ");
        
        MCFunction::parse_json_all(text);
        let mut cmds = vec![];
        let mut funs = vec![];
        let mut warns = vec![];
        let keys = Blocker::new().split_in_same_level(" ", text).unwrap_or_else(|e| {
            error(format_out(&*join!("Failed to parse command: ", &*e), path, self.ln + ln + 1));
        });
        let keys: Vec<String> = keys.iter().map(|s| MCFunction::replace_local_tags(s, ns)).collect::<Vec<_>>();
        let mut keys: Vec<&str> = keys.iter().map(|v| &**v).collect();
        *text = keys.join(" ");
        if keys.len() == 0 {
            return (1, cmds, funs, warns);
        }
        let rem: usize = match keys[0] {
            "@DEBUG" => {
                println!("\x1b[96m@DEBUG [{}]: {} for {}\x1b[0m", keys[1..].join(" "), self.ln + ln + 1, &self.name);
                1
            }
            "@OUTPUT" => {
                let output = self.commands.get((self.commands.len() - self.premc)..(self.commands.len())).map(|x| x.join("\n    ")).unwrap_or("".into());
                println!("\x1b[94m@OUTPUT {}:{} [{}]:\x1b[0m \n    {}\n", self.premc, self.ln + ln + 1, keys[1..].join(" "), output);
                1
            }
            "@ERROR" => {
                error(format_out(
                    "Found @ERROR",
                    &*self.get_path(ns),
                    self.ln + ln + 1,
                ));
            }
            "@NOLEX" => {
                *text = text[7..].into();
                return self.compile_text(ns, ln);
            }
            "{" => {
                let (res, mut fun, (mut funs2, mut warn)) = self.code_to_function(ns, ln, "bl");
                self.calls.append(&mut fun.calls);
                funs.append(&mut funs2);
                warns.append(&mut warn);
                cmds.push(fun.get_call_method(ns));
                funs.push(fun);
                res
            }
            "}" => self.lines.len() - ln,
            "i_cmd" => {
                cmds.push(text[6..].to_string());
                1
            }
            "//" => {
                if ns.meta.comments {
                    cmds.push(["#", &text[2..]].join(""));
                }
                1
            }
            "" => {
                if ns.meta.comments {
                    cmds.push(join![""]);
                }
                1
            }
            "set" => {
                self.vars.retain(|x| !x.0.eq(keys[1]));
                self.vars
                    .insert(0, (keys[1].to_string(), keys[2..].join(" ").to_string()));
                1
            }
            "loop" => {
                //TODO ast + loop removes more than necessary (also double ifs and other stuff)
                let count = keys[1].parse::<usize>().unwrap_or(0);
                if count != 0 {
                    *text = keys[2..].join(" ");
                    let (res, mut fun, (mut funs2, mut warn)) = self.code_to_function(ns, ln, "loop");
                    self.calls.append(&mut fun.calls);
                    funs.append(&mut funs2);
                    warns.append(&mut warn);
                    let lps = fun.commands.clone();
                    fun.commands = lps.into_iter().cycle().take(fun.commands.len() * count).collect::<Vec<_>>();
                    if fun.commands.len() > 0 {
                        cmds.push(fun.get_callable(ns));
                        funs.push(fun);
                    } else {
                        cmds.push(join!["# <loop code produced no result>"]);
                    }
                    res
                } else {
                    error(format_out(
                        &*["Invalid Loop Count (", keys[1], ")"].join(""),
                        &*self.get_path(ns),
                        self.ln + ln + 1,
                    ));
                }
            }
            "if" => {
                let mut pre = String::from("execute ");
                let mode = if keys[1].starts_with('!') {
                    keys[1] = &keys[1][1..];
                    "unless "
                } else {
                    "if "
                };
                keys[1] = &keys[1][1..keys[1].len() - 1];
                *text = keys[2..].join(" ");
                let cds = self.compile_condition(keys[1].to_string(), &ns, ln);
                for cd in cds {
                    pre.push_str(mode);
                    pre.push_str(&*cd);
                    pre.push_str(" ");
                }
                if keys.len() >= 3 {
                    pre.push_str("run ");

                    let (res, mut fun, (mut funs2, mut warn)) = self.code_to_function(ns, ln, "if");
                    self.calls.append(&mut fun.calls);
                    funs.append(&mut funs2);
                    warns.append(&mut warn);
                    if fun.commands.len() > 1 {
                        cmds.push(join![&*pre, &*fun.get_callable(ns)]);
                        funs.push(fun);
                    } else {
                        if fun.commands.len() == 0 {
                            cmds.push(join!["# ", &*pre.clone(), "<code produced no result>"]);
                        } else {
                            cmds.push(pre.clone() + &*fun.get_callable(ns));
                        }
                    }
                    res
                } else {
                    cmds.push(pre);
                    1
                }
            }
            "for" => {
                if keys.len() < 3 || !keys[2].eq("{") {
                    error(format_out(
                        "Invalid 'for' function",
                        &*self.get_path(ns),
                        self.ln + ln + 1,
                    ));
                }
                keys[1] = &keys[1][1..keys[1].len() - 1];
                let largs = keys[1].split(",").map(|x| x.trim().to_string()).collect::<Vec<String>>();
                if largs.len() < 2 {
                    error(format_out(
                        "Invalid 'for' function specifiers",
                        &*self.get_path(ns),
                        self.ln + ln + 1,
                    ));
                }
                *text = keys[2..].join(" ");
                let target = self.compile_score_path(&largs[0], ns, ln);
                let start = if largs.len() > 2 { largs[1].parse::<i32>().unwrap_or(0) } else { 0 };
                let end = if largs.len() > 2 { largs[2].parse::<i32>().unwrap_or(0) } else { largs[1].parse::<i32>().unwrap_or(0) };
                let (res, mut fun, (mut funs2, mut warn)) = self.code_to_function(ns, ln, "for");
                self.calls.append(&mut fun.calls);
                funs.append(&mut funs2);
                warns.append(&mut warn);
                fun.commands.append(&mut vec![join!["scoreboard players add", &*target, " 1"],
                                              join!["execute unless score", &*target, " matches ", &*end.to_string(), ".. run ", &*fun.get_call_method(ns)]]);
                cmds.append(&mut vec![join!["scoreboard players set", &*target, " ", &*start.to_string()], join!["execute unless score", &*target, " matches ", &*end.to_string(), ".. run ", &*fun.get_call_method(ns)]]);
                funs.push(fun);
                res
            }
            "while" => {
                if keys.len() < 3 || !keys[2].eq("{") {
                    error(format_out(
                        "Invalid 'while' function",
                        &*self.get_path(ns),
                        self.ln + ln + 1,
                    ));
                }
                let unless = keys[1].starts_with("!");
                keys[1] = &keys[1][1..keys[1].len() - 1];
                if unless {
                    keys[1] = &keys[1][1..];
                }
                *text = keys[2..].join(" ");
                let cds = self.compile_condition(keys[1].to_string(), ns, ln);
                let mode = if unless { "unless" } else { "if" };
                let (res, mut fun, (mut funs2, mut warn)) = self.code_to_function(ns, ln, "while");
                self.calls.append(&mut fun.calls);
                funs.append(&mut funs2);
                warns.append(&mut warn);
                let command = join!["execute ", mode, " ", &*cds.join(&*join![" ", mode, " "]), " run ", &*fun.get_call_method(ns)];
                fun.commands.push(command.clone());
                cmds.push(command);
                funs.push(fun);
                res
            }
            "execute" => {
                let pos = text.match_indices(" ast @").map(|s| s.0).collect::<Vec<_>>();
                for p in pos {
                    if text.chars().collect::<Vec<char>>()[p + 7].eq(&'[') {
                        if let Ok(out) = Blocker::new().find_size(text, p + 7) {
                            if out != Blocker::NOT_FOUND {
                                text.replace_range(p..out, &*[" as ", &text[p + 5..out], " at @s"].join(""));
                            }
                        }
                    } else {
                        text.replace_range(p..p + 7, &*[" as ", &text[p + 5..p + 7], " at @s"].join(""));
                    }
                }
                // TODO make exe & ast parse inline ifs
                // let pos = text.match_indices(" if ").map(|s| s.0).collect::<Vec<_>>();
                // for p in pos {
                //     if text.chars().collect::<Vec<char>>()[p + 7].eq(&'[') {
                //         if let Ok(out) = Blocker::new().find_size(text, p + 7) {
                //             if out != Blocker::NOT_FOUND {
                //                 text.replace_range(p..out, &*[" as ", &text[p + 5..out], " at @s"].join(""));
                //             }
                //         }
                //     } else {
                //         text.replace_range(p..p + 7, &*[" as ", &text[p + 5..p + 7], " at @s"].join(""));
                //     }
                // }
                let run = Blocker::new().find_in_same_level("run", text).unwrap_or(Blocker::NOT_FOUND);
                if run != Blocker::NOT_FOUND {
                    let pre = text[..run + 4].to_string();
                    *text = text[run + 4..].to_string();
                    let (res, mut fun, (mut funs2, mut warn)) = self.code_to_function(ns, ln, "exe");
                    self.calls.append(&mut fun.calls);
                    funs.append(&mut funs2);
                    warns.append(&mut warn);
                    if fun.commands.len() > 1 {
                        cmds.push(join![&*pre, &*fun.get_callable(ns)]);
                        funs.push(fun);
                    } else {
                        if fun.commands.len() == 0 {
                            cmds.push(join!["# ", &*pre.clone(), "<code produced no result>"]);
                        } else {
                            cmds.push(pre.clone() + &*fun.get_callable(ns));
                        }
                    }
                    res
                } else {
                    cmds.push(text.to_string());
                    1
                }
            }
            "ast" => {
                *text = ["execute ", &*text].join("");
                return self.compile_text(ns, ln);
            }
            "exe" => {
                *text = ["execute ", &text[4..]].join("");
                return self.compile_text(ns, ln);
            }
            "rmm" => {
                let c = "function remgine:utils/rmm".to_string();
                if keys.len() == 1 {
                    return self.remgine("rmm", ns, ln, (1, vec![c], funs, warns));
                }
                return if keys[1].eq("set") {
                    let power = keys[2].parse::<u8>().unwrap_or(8);
                    self.remgine("rmm", ns, ln, (1, vec![["scoreboard players set @s remgine.rmm", &*power.to_string()].join(" ")], funs, warns))
                } else {
                    let power = keys[1].parse::<u8>().unwrap_or(8);
                    self.remgine("rmm", ns, ln, (1, vec![["scoreboard players set @s remgine.rmm", &*power.to_string()].join(" "), c], funs, warns))
                };
            }
            _ if keys[0].starts_with("//") => {
                if ns.meta.comments {
                    cmds.push(["#", &text[2..]].join(""));
                }
                1
            }
            "create" => {
                if keys.len() == 2 {
                    keys.push("dummy");
                }
                let mut r = keys[1].to_string();
                if keys[1].starts_with("&") {
                    r.replace_range(0..1, ".");
                    r.replace_range(0..0, &*ns.id);
                }
                keys[1] = r.as_str();
                cmds.push(["scoreboard objectives add", &*keys[1..].join(" ")].join(" "));
                1
            }
            "remove" => {
                let mut r = keys[1].to_string();
                if keys[1].starts_with("&") {
                    r.replace_range(0..1, ".");
                    r.replace_range(0..0, &*ns.id);
                }
                keys[1] = r.as_str();
                cmds.push(["scoreboard objectives remove", keys[1]].join(" "));
                1
            }
            "tag" => {
                if keys.len() >= 4 {
                    if keys[3].starts_with("&") {
                        let v = join![&*ns.id, ".", &keys[3][1..]];
                        keys[3] = &*v;
                        *text = keys[..].join(" ");
                    }
                }
                cmds.push(text.to_string());
                1
            }
            _ if MCFunction::is_score_path(&keys[0].to_string()) => {
                let res = if keys.len() > 1 {
                    if keys.len() >= 3 {
                        if keys[1].eq("result") || keys[1].eq("success") {
                            *text = keys[2..].join(" ");
                            let target = &*self.compile_score_path(&keys[0].to_string(), ns, ln);
                            let command = join!["execute store ", keys[1], " score", target, " run "];
                            let (res, mut fun, (mut funs2, mut warn)) = self.code_to_function(ns, ln, &keys[2][0..2]);
                            self.calls.append(&mut fun.calls);
                            funs.append(&mut funs2);
                            warns.append(&mut warn);
                            if fun.commands.len() > 1 {
                                cmds.push(join![&*command, &*fun.get_callable(ns)]);
                                funs.push(fun);
                            } else {
                                if fun.commands.len() == 0 {
                                    cmds.push(join!["# ", &*command, "<code produced no result>"]);
                                } else {
                                    cmds.push(join![&*command, &*fun.get_callable(ns)]);
                                }
                            }
                            res
                        } else {
                            cmds.append(&mut self.compile_score_command(&keys, ns, ln));
                            1
                        }
                    } else {
                        cmds.append(&mut self.compile_score_command(&keys, ns, ln));
                        1
                    }
                } else {
                    let target = &*self.compile_score_path(&keys[0].to_string(), ns, ln);
                    cmds.push(join!("scoreboard players get", target));
                    1
                };
                res
            }
            mut f @ _ => {
                let v = f.replacen("&", &*join![&*self.file, "/"].replace("functions/", ""), 1);
                f = &*v;
                if let Ok(name) = MCFunction::parse_name(f, &*ns.id) {
                    if !name.starts_with("#") {
                        self.calls.push((f[..f.len() - 2].to_string(), self.ln + ln + 1));
                    }
                    cmds.push(["function ", &*name].join(""));
                    1
                } else {
                    *text = ["i_cmd ", &*text].join("");
                    return self.compile_text(ns, ln);
                }
            }
        };
        (rem, cmds, funs, warns)
    }

    fn replace_local_tags(selector: &String, ns: &Namespace) -> String {
        let mut v = selector.to_string();
        if selector.starts_with("@") {
            if selector.len() >= 4 && selector.chars().nth(2).unwrap_or(' ').eq(&'[') {
                let b = selector[3..selector.len() - 1].to_string();
                let options = Blocker::new().split_in_same_level(",", &b);
                if options.is_ok() {
                    let ops = options.unwrap().into_iter().map(|o| -> String {
                        let mut t = o.clone();
                        t.retain(|c| !c.is_whitespace());
                        if t.starts_with("tag=") && t.contains("&") {
                            o.replace("&", &*join![&*ns.id, "."])
                        } else {
                            o
                        }
                    }).collect::<Vec<String>>();
                    v = join![&selector[0..3], &*ops.join(","), "]"];
                }
            }
        }
        v
    }

    fn remgine<T>(&self, ability: &str, ns: &Namespace, ln: usize, t: T) -> T {
        if !ns.meta.remgine {
            error(format_out(
                &*["Remgine is required to use '", &*ability.form_foreground(str::ORN), "', enable it in pack.msk (remgine = true)"].join(""),
                &*self.get_path(ns),
                self.ln + ln + 1,
            ));
        }
        t
    }

    fn code_to_function(&self, ns: &Namespace, ln: usize, id: &str) -> (usize, MCFunction, (Vec<MCFunction>, Vec<String>)) {
        if self.lines[ln].eq("{") {
            let res = Blocker::auto_vec(
                &self.lines[ln..].to_vec(),
                self.lines[ln].len() - 1,
                self.get_path(ns).clone(),
                self.ln + ln + 2,
            ).0;
            let mut f = MCFunction::new(&*join![&*self.name, ".", id, &*ln.to_string(), "()"], self.ln + ln + 1, &self.meta, &self.file);
            for i in 1..res {
                f.lines.push(self.lines[i + ln].clone());
            }
            f.vars = self.vars.clone();
            let v = f.compile_to(ns);
            (res + 1, f, v)
        } else if self.lines[ln].ends_with("{") {
            let res = Blocker::auto_vec(
                &self.lines[ln..].to_vec(),
                self.lines[ln].len() - 1,
                self.get_path(ns).clone(),
                self.ln + ln + 2,
            ).0 + 1;
            let mut f = MCFunction::new(&*join![&*self.name, ".", id, &*ln.to_string(), "()"], self.ln + ln + 1, &self.meta, &self.file);
            for i in 0..res {
                f.lines.push(self.lines[i + ln].clone());
            }
            f.vars = self.vars.clone();
            let v = f.compile_to(ns);
            (res + 1, f, v)
        } else {
            let mut f = MCFunction::new(&*join![&*self.name, ".", id, &*ln.to_string(), "()"], self.ln + ln, &self.meta, &self.file);
            f.lines.push(self.lines[ln].clone());
            f.vars = self.vars.clone();
            let v = f.compile_to(ns);
            (1, f, v)
        }
    }

    fn get_callable(&self, ns: &Namespace) -> String {
        if self.commands.len() == 1 {
            return self.commands[0].clone();
        }
        self.get_call_method(ns)
    }

    fn get_call_method(&self, ns: &Namespace) -> String {
        join!("function ", &*ns.id, ":", &*self.name)
    }

    fn compile_score_command(&mut self, keys: &Vec<&str>, ns: &Namespace, ln: usize) -> Vec<String> {
        let mut cds = vec![];
        let mut command = "scoreboard players ".to_string();
        let target = &*self.compile_score_path(&keys[0].to_string(), ns, ln);
        let mut operation = false;
        let target2 = &*if keys.len() >= 3 {
            let a = self.get_numerical_type(&keys[2].to_string(), ns, ln);
            operation = a.1;
            a.0
        } else {
            " 0".to_string()
        };
        if operation {
            command.push_str("operation");
            match keys[1] {
                "=" | "+=" | "-=" | "%=" | "*=" | "/=" | "<" | ">" | "><" => {
                    command.push_str(&*join!(target, " ", keys[1], target2));
                }
                _ => {
                    error(format_out(
                        &*join!("Failed to parse score function, unknown operation '", &*keys[1].form_foreground(str::BLU), "'"),
                        &*self.get_path(ns),
                        self.ln + ln + 1,
                    ));
                }
            }
        } else {
            match keys[1] {
                "=" => {
                    command.push_str(&*join!("set", target, target2));
                }
                "+=" => {
                    command.push_str(&*join!("add", target, target2));
                }
                "-=" => {
                    command.push_str(&*join!("remove", target, target2));
                }
                "reset" | "enable" | "get" => {
                    command.push_str(&*join!(keys[1], target));
                }
                "--" => {
                    command.push_str(&*join!("remove", target, " 1"));
                }
                "++" => {
                    command.push_str(&*join!("add", target, " 1"));
                }
                "*=" | "%=" | "/=" | ">" | "<" | "><" => {
                    self.remgine(keys[1], ns, ln, ());
                    cds.push(join!("scoreboard players set $temp remgine.temp", target2));
                    command.push_str(&*join!("operation", target, " ", keys[1], " $temp remgine.temp"));
                }
                _ => {
                    error(format_out(
                        &*join!("Failed to parse score function, unknown operation '", &*keys[1].form_foreground(str::BLU), "'"),
                        &*self.get_path(ns),
                        self.ln + ln + 1,
                    ));
                }
            }
        }
        cds.push(command);
        return cds;
    }

    fn compile_condition(&mut self, condition: String, ns: &Namespace, ln: usize) -> Vec<String> {
        if condition.contains(" && ") {
            let cds = condition.split_once(" && ").unwrap_or(("", ""));
            let mut a = self.compile_condition(cds.0.to_string(), ns, ln);
            let mut b = self.compile_condition(cds.1.to_string(), ns, ln);
            a.append(&mut b);
            return a;
        }
        let args = Blocker::new().split_in_same_level(" ", &condition);
        return if args.is_err() {
            vec![condition]
        } else {
            let mut args = args.unwrap();
            args = args.into_iter().map(|a| MCFunction::replace_local_tags(&a, ns)).collect();
            match &*args[0] {
                "random" => self.remgine("random", ns, ln, vec![["predicate remgine:random/", &*args[1]].join("")]),
                _ if MCFunction::is_score_path(&args[0]) => {
                    let comp = self.compile_score_path(&args[0], ns, ln);
                    if args.len() < 3 {
                        error(format_out(
                            &*join!("Failed to parse score-based condition '", &*condition.form_foreground(str::BLU), "', not enough arguments"),
                            &*self.get_path(ns),
                            self.ln + ln + 1,
                        ));
                    }
                    vec![(|| {
                        join!["score", &*comp, &*match &*args[1] {
                            "==" => {
                                join![" matches ", &*args[2]]
                            }
                            "<" | "<=" | "=" | ">=" | ">" => {
                                join![" ", &*args[1], &*self.compile_score_path(&args[2], ns, ln)]
                            }
                            _ => error(format_out(&*join!("Failed to parse score-based condition '", &*condition.form_foreground(str::BLU), "', unknown operator"), &*self.get_path(ns), self.ln + ln + 1,))}]
                    })()]
                }
                _ => {
                    vec![args.join(" ")]
                }
            }
        };
    }

    fn get_numerical_type(&mut self, num: &String, ns: &Namespace, ln: usize) -> (String, bool) {
        let int = num.parse::<i32>();
        if int.is_ok() {
            return (join!(" ", &*int.unwrap().to_string()), false);
        }
        if MCFunction::is_score_path(num) {
            return (self.compile_score_path(num, ns, ln), true);
        }
        error(format_out(
            &*join!("Failed to parse numerical '", &*num.form_foreground(str::BLU), "' ", &*int.err().unwrap().to_string()),
            &*self.get_path(ns),
            self.ln + ln + 1,
        ));
    }

    fn is_score_path(path: &String) -> bool {
        (path.starts_with("$") && path.split(":").collect::<Vec<_>>().len() == 1) ||
            (path.contains(":") && !path.ends_with("()"))
    }

    fn compile_score_path(&mut self, path: &String, ns: &Namespace, ln: usize) -> String {
        let mut path = path.clone();
        if path.starts_with("$") && path.split(":").collect::<Vec<_>>().len() == 1 {
            path.push_str(":remgine.temp");
        }
        let mut pp = Blocker::new().split_in_same_level(":", &path).unwrap_or_else(|_| {
            error(format_out(
                &*join!("Failed to parse score path '", &*path.form_foreground(str::BLU), "', unclosed brackets"),
                &*self.get_path(ns),
                self.ln + ln + 1,
            ));
        });
        if pp.len() < 2 {
            error(format_out(
                &*join!("Failed to parse score path '", &*path.form_foreground(str::BLU), "', expected board identifier"),
                &*self.get_path(ns),
                self.ln + ln + 1,
            ));
        }
        if pp[1].starts_with("&") {
            pp[1].replace_range(0..1, ".");
            pp[1].replace_range(0..0, &*ns.id);
        }
        if pp[1].starts_with("r&") {
            pp[1].replace_range(0..2, "remgine.");
        }
        join![" ", &*MCFunction::replace_local_tags(&pp[0], ns), " ", &*pp[1]]
    }

    fn get_path(&self, ns: &Namespace) -> String {
        [&*ns.id, "functions", &*self.file].join("/")
    }

    fn parse_name(f: &str, ns: &str) -> Result<String, ()> {
        let fnn = f.to_string();
        if fnn.starts_with("&") {}
        if MCFunction::is_valid_fn(&*fnn) {
            Ok(
                [
                    if fnn.contains(":") { "" } else { &ns },
                    if fnn.contains(":") { "" } else { ":" },
                    &fnn[0..fnn.len() - 2],
                ].join("")
            )
        } else {
            if MCFunction::is_valid_fn(&fnn[1..]) && fnn.starts_with("#") {
                Ok(
                    [
                        "#",
                        if fnn.contains(":") { "" } else { &ns },
                        if fnn.contains(":") { "" } else { ":" },
                        &fnn[1..fnn.len() - 2],
                    ].join("")
                )
            } else {
                Err(())
            }
        }
    }

    fn parse_json_all(text: &mut String) {
        let pos = text.match_indices("*JSON{").map(|s| s.0).collect::<Vec<_>>();
        let mut rep = Vec::new();
        for p in pos {
            if let Ok(out) = Blocker::new().find_size(text, p + 5) {
                let (mut italic, mut bold, mut strike, mut underline, mut obfuscated, mut color) =
                    (None, None, None, None, None, None);
                let options = text[(p + 6)..out - 1].to_string();
                let options = options.split(" : ").collect::<Vec<_>>();
                let phrase = options[1..].join(" : ");
                let options = options[0].split(" ").collect::<Vec<_>>();
                for opt in options.iter() {
                    let mut set = true;
                    let mut t = opt.to_string();
                    if opt.starts_with("!") {
                        set = false;
                        t = opt[1..].to_string();
                    }
                    let t = &*t;
                    match t {
                        "italic" => italic = Some(set),
                        "bold" => bold = Some(set),
                        "strike" => strike = Some(set),
                        "underline" => underline = Some(set),
                        "obfuscated" => obfuscated = Some(set),
                        _ if color == None && !t.eq("") => color = Some(*opt),
                        _ => {}
                    }
                }
                let mut json = "{\"text\":\"".to_string();
                json.push_str(&*phrase);
                json.push_str("\"");
                if italic.is_some() { json.push_str(&*[",\"italic\":\"", &*italic.unwrap().to_string(), "\""].join("")); }
                if bold.is_some() { json.push_str(&*[",\"bold\":\"", &*bold.unwrap().to_string(), "\""].join("")); }
                if strike.is_some() { json.push_str(&*[",\"strikethrough\":\"", &*strike.unwrap().to_string(), "\""].join("")); }
                if underline.is_some() { json.push_str(&*[",\"underlined\":\"", &*underline.unwrap().to_string(), "\""].join("")); }
                if obfuscated.is_some() { json.push_str(&*[",\"obfuscated\":\"", &*obfuscated.unwrap().to_string(), "\""].join("")); }
                if color.is_some() { json.push_str(&*[",\"color\":\"", color.unwrap(), "\""].join("")); }
                json.push_str("}");
                rep.push((p..out, json.to_string()));
            }
        }
        for (r, t) in rep.into_iter().rev() {
            text.replace_range(r, &*t);
        }
    }
}

pub struct Blocker {
    stack: Vec<char>,
    string: bool,
}

impl Blocker {
    pub const NOT_FOUND: usize = 404_0000000;

    fn new() -> Blocker {
        Blocker {
            stack: Vec::new(),
            string: false,
        }
    }

    pub fn find_rapid_close(&mut self, lines: &Vec<String>, closer: char) -> Result<usize, String> {
        let mut c: usize = 0;
        loop {
            if c >= lines.len() {
                return Ok(Blocker::NOT_FOUND);
            }
            if lines[c].trim().starts_with(closer) {
                return Ok(c);
            }
            c += 1;
        }
    }

    /**
    (line_number, offset)
     */
    pub fn auto_vec(lines: &Vec<String>, offset: usize, path: String, ln: usize) -> (usize, usize) {
        let mut b = Blocker::new();
        match b.find_size_vec(lines, offset) {
            Ok(o) => {
                if o.0 != Blocker::NOT_FOUND {
                    return o;
                } else {
                    error(format_out("Unterminated block", &*path, ln))
                }
            }
            Err(e) => error(format_out(&*[&*e.0, " /", &path, ":", &*(e.1 + ln).to_string()].join(""), &*path, ln)),
        };
    }

    /**
     * Returns OK(line_number, offset)
     *
     * or Err(message, offset)
     */
    pub fn find_size_vec(
        &mut self,
        lines: &Vec<String>,
        offset: usize,
    ) -> Result<(usize, usize), (String, usize)> {
        let mut c: usize = 0;
        loop {
            if c >= lines.len() {
                return Ok((Blocker::NOT_FOUND, c));
            }
            let r = self
                .find_size(&lines[c], if c == 0 { offset } else { 0 })
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
        let mut comment = false;
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
                        return Err(format!("Unexpected \'{}\' @{}", c.form_foreground(str::ORN), pos));
                    }
                }
                '(' if !self.string => self.stack.push(c),
                ')' if !self.string => {
                    if self.stack.last().eq(&Some(&'(')) {
                        self.stack.pop();
                    } else {
                        return Err(format!("Unexpected \'{}\' @{}", c.form_foreground(str::ORN), pos));
                    }
                }
                '[' if !self.string => self.stack.push(c),
                ']' if !self.string => {
                    if self.stack.last().eq(&Some(&'[')) {
                        self.stack.pop();
                    } else {
                        return Err(format!("Unexpected \'{}\' @{}", c.form_foreground(str::ORN), pos));
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
                '/' => {
                    if comment {
                        return Ok(Blocker::NOT_FOUND);
                    } else {
                        comment = true;
                    }
                }
                '§' => {
                    pos += 1;
                }
                _ => {}
            }
            if !c.eq(&'/') {
                comment = false;
            }
            if self.stack.len() == 0 && !self.string {
                return Ok(pos + offset);
            }
        }
        Ok(Blocker::NOT_FOUND)
    }

    pub fn find_in_same_level(&mut self, needle: &str, haystack: &String) -> Result<usize, String> {
        let mut pos = 0;
        loop {
            if pos >= haystack.len() {
                return Ok(Blocker::NOT_FOUND);
            }
            if haystack[pos..].starts_with(needle) {
                return Ok(pos);
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

#[derive(Clone)]
pub struct Link {
    path: String,
    name: String,
    links: Vec<String>,
}

impl Link {
    fn new(path: String, name: String, links: Vec<String>) -> Link {
        Link {
            path,
            name,
            links,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    recipe: Vec<String>,
    materials: Vec<(String, String)>,
    path: String,
    file_name: String,
    call: MCFunction,
    adds: (Vec<MCFunction>, Vec<String>),
}

impl Item {
    fn new(name: String, lines: Vec<String>, ns: &Namespace) -> Item {
        let mut item = Item {
            recipe: vec![],
            materials: vec![],
            path: name.to_string(),
            call: MCFunction::new("null()", 0, &ns.meta, &name),
            file_name: name,
            adds: (vec![], vec![]),
        };

        let mut ln = 0;
        while ln < lines.len() {
            let rem = item.parse_line(ln, &lines, ns);
            ln += rem;
        }
        item
    }

    fn parse_line(&mut self, ln: usize, lines: &Vec<String>, ns: &Namespace) -> usize {
        let keys = Blocker::new().split_in_same_level(" ", &lines[ln]).unwrap_or_else(|e| {
            error(format_out(&*join!("Failed to parse item: ", &*e), &*self.get_path(ns), ln + 1));
        });
        match &*keys[0] {
            "recipe" => {
                if !keys[1].eq("{") {
                    error(format_out(
                        "Invalid 'recipe' block",
                        &*self.get_path(ns),
                        ln + 1,
                    ));
                }
                let rem = Blocker::auto_vec(&lines[ln..].to_vec(), lines[ln].len() - 1, self.get_path(ns), ln).0 + 1;
                let pattern = lines[(ln + 1)..(ln + rem - 1)].to_vec();
                if pattern.len() < 1 || pattern.len() > 3 {
                    error(format_out(
                        "Invalid recipe pattern",
                        &*self.get_path(ns),
                        ln + 1,
                    ));
                }
                self.recipe = pattern;
                rem
            }
            "materials" => {
                if !keys[1].eq("{") {
                    error(format_out(
                        "Invalid 'materials' block",
                        &*self.get_path(ns),
                        ln + 1,
                    ));
                }
                let rem = Blocker::auto_vec(&lines[ln..].to_vec(), lines[ln].len() - 1, self.get_path(ns), ln).0 + 1;
                let mats = lines[(ln + 1)..(ln + rem - 1)].to_vec();
                let mats = mats.into_iter().map(|s| -> (String, String) {
                    let v = s.split(" : ").collect::<Vec<&str>>();
                    let v = v.into_iter().map(|s| s.to_string()).collect::<Vec<String>>();
                    (v[0].to_string(), v[1].to_string())
                }).collect::<Vec<_>>();
                self.materials = mats;
                rem
            }
            "path" => {
                self.path = lines[ln].trim().split(" : ").nth(1).unwrap_or(&*self.path).to_string();
                1
            }
            "item" => {
                if !keys[1].eq("{") {
                    error(format_out(
                        "Invalid 'item' block",
                        &*self.get_path(ns),
                        ln + 1,
                    ));
                }
                let rem = Blocker::auto_vec(&lines[ln..].to_vec(), lines[ln].len() - 1, self.get_path(ns), ln).0 + 1;
                let mut lines = lines[ln..(ln + rem)].to_vec();
                lines[0] = join!["{"];
                let mut f = MCFunction::new(&*join![&*self.path, "()"], ln, &ns.meta, &self.file_name);
                f.lines = lines;
                let (rem, mut f, adds) = f.code_to_function(ns, 0, "item");
                self.adds = adds;
                f.name = self.path.clone();

                // recipe take @s radium:fireball
                // advancement revoke @s only radium:fireball
                // clear @s knowledge_book

                f.commands.insert(0, join!["clear @s knowledge_book"]);
                f.commands.insert(0, join!["advancement revoke @s only ", &*ns.id, ":", &*self.path]);
                f.commands.insert(0, join!["recipe take @s ", &*ns.id, ":", &*self.path]);

                self.call = f;
                rem
            }
            _ => 1
        }
    }

    fn get_path(&self, ns: &Namespace) -> String {
        [&*ns.id, "items", &*self.file_name].join("/")
    }
}