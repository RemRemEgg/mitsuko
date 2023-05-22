// craftmine ong :krill:

use std::cmp::min;
use std::fs::{DirEntry, File, read_dir, ReadDir, remove_dir_all};
use std::io::Write;
use std::time::Instant;
use crate::{*, server::*};
use crate::NodeType::Block;

pub struct Datapack {
    meta: Meta,
    ln: usize,
    pub namespaces: Vec<Namespace>,
    pub src_loc: String,
    callable_functions: Vec<String>,
}

impl Datapack {
    pub fn new(path: String) -> Datapack {
        unsafe {
            SRC = path.clone();
            SRC.push_str("/src");
        }
        Datapack {
            meta: Meta::new(),
            ln: 1,
            src_loc: path,
            namespaces: vec![],
            callable_functions: vec![],
        }
    }

    pub fn get_view_name(&self) -> String {
        self.meta.view_name.clone()
    }

    pub fn gen_meta(&mut self, pack: String) {
        self.meta.view_name = self.src_loc.clone();
        let tags = pack.split("\n").collect::<Vec<&str>>();

        for tag in tags {
            let s = tag.split("=").collect::<Vec<&str>>();
            self.meta.set_property(s[0].trim(), s[1].trim(), true, ("pack".into(), self.ln));
            self.ln += 1;
        }
    }

    pub fn read_namespaces(&mut self) {
        let nss = read_dir([&*self.src_loc, "src"].join("/"));
        if nss.is_ok() {
            for ns in nss.unwrap().map(|x| x.unwrap()) {
                if ns.path().is_dir() {
                    let nsnew = Namespace::new(
                        ns,
                        self.meta.clone(),
                    );
                    if nsnew.is_some() {
                        self.namespaces.push(nsnew.unwrap());
                    }
                }
            }
        } else {
            warn("No namespaces found".to_string());
        }
        status(join!["Loaded ", &*self.namespaces.len().to_string(), " namespaces ", 
            &*join!["['", &*self.namespaces.iter().map(|n|n.id.clone()).collect::<Vec<String>>()
                .join("\', \'"), "']"].form_foreground(str::GRY)])
    }

    pub fn compile_namespaces(&mut self) {
        for i in 0..self.namespaces.len() {
            self.namespaces[i].compile();
        }
    }

    pub fn get_dir(&self, loc: &str) -> String {
        join![&*self.src_loc, loc]
    }

    pub fn root(&self, loc: &str) -> String {
        join![&*self.src_loc, "/generated/", &*self.meta.view_name, loc]
    }

    pub fn data(&self, loc: &str) -> String {
        join![&*self.src_loc, "/generated/", &*self.meta.view_name, "/data/", loc]
    }

    pub fn save(&mut self) {
        let save_time = Instant::now();
        status(format!("Saving '{}'", &self.meta.view_name.form_foreground(str::PNK)));

        unsafe {
            DATAROOT = self.data("");
        }

        remove_dir_all(self.get_dir("/generated")).ok();
        make_folder(&*self.data(""));

        let meta = MFile::new(self.root("/pack.mcmeta"));
        meta.save(META_TEMPLATE.clone()
            .replace("{VERS}", &*self.meta.version.to_string())
            .replace("{DESC}", &self.meta.description));

        for nsi in 0..self.namespaces.len() {
            self.namespaces[nsi].save();
        }
    }
}

#[derive(Clone, Debug)]
pub struct Meta {
    pub vb: i32,
    pub version: u8,
    pub remgine: bool,
    pub opt_level: u8,
    pub comments: bool,
    pub view_name: String,
    pub description: String,
    pub recursive_replace: u8,
}

impl Meta {
    const VB: i32 = 0;
    const REMGINE: bool = false;
    const OPT_LEVEL: u8 = 0;
    const COMMENTS: bool = false;
    const RE_REPLACE: u8 = 3;

    fn new() -> Meta {
        Meta {
            vb: Meta::VB,
            version: CURRENT_PACK_VERSION,
            remgine: Meta::REMGINE,
            opt_level: Meta::OPT_LEVEL,
            comments: Meta::COMMENTS,
            view_name: "Untitled".to_string(),
            description: "A Datapack".to_string(),
            recursive_replace: Meta::RE_REPLACE,
        }
    }

    fn set_property(&mut self, property: &str, val: &str, extended: bool, warns: (String, usize)) {
        let mut suc = true;
        match property {
            "remgine" if extended => self.remgine = val.to_uppercase().eq("TRUE"),
            "name" if extended => self.view_name = val.to_string(),
            "version" if extended => self.version = val.parse::<u8>().unwrap_or(CURRENT_PACK_VERSION),
            "description" if extended => self.description = val.to_string(),
            "remgine" | "name" | "version" | "description" if !extended => {
                warn(
                    format_out(
                        &*["Cannot override property \'", &*property.form_foreground(str::BLU), "\' in this context (value = \'", &*val.form_foreground(str::GRY), "\')"].join(""),
                        &*warns.0,
                        warns.1,
                    ),
                );
                suc = false
            }
            "comments" => self.comments = val.to_uppercase().eq("TRUE"),
            "optimizations" => self.opt_level = min(val.parse::<u8>().unwrap_or(Meta::OPT_LEVEL), 4u8),
            "debug" => self.vb = min(val.parse::<i32>().unwrap_or(Meta::VB), 3),
            "recursive_replace" => self.recursive_replace = val.parse::<u8>().unwrap_or(Meta::RE_REPLACE),
            _ => {
                warn(
                    format_out(
                        &*["Unknown property: \'", &*property.form_foreground(str::BLU), "\' (value = \'", &*val.form_foreground(str::GRY), "\')"].join(""),
                        &*warns.0,
                        warns.1,
                    ),
                );
                suc = false
            }
        }
        if suc && self.vb >= 1 {
            debug(format!("Set property \'{}\' to \'{}\'", property.form_foreground(str::BLU), val.form_foreground(str::AQU)));
        }
    }
}

pub struct Namespace {
    pub id: String,
    functions: Vec<MCFunction>,
    // links: Vec<Link>,
    // items: Vec<Item>,
    meta: Meta,
    ln: usize,
    export_functions: Vec<String>,
}

impl Namespace {
    fn new(loc: DirEntry, meta: Meta) -> Option<Namespace> {
        let name = loc.file_name().to_string_lossy().to_string();
        if name.eq(&"".to_string()) || {
            let mut nid = name.replace(|ch| ch >= 'a' && ch <= 'z', "");
            nid = nid.replace(|ch| ch >= '0' && ch <= '9', "");
            nid = nid.replace("_", "");
            nid.len() != 0
        } {
            error(join!["Invalid Namespace: ", &*name]);
        }
        Some(Namespace {
            id: name,
            functions: vec![],
            // links: Vec::new(),
            // items: Vec::new(),
            meta,
            ln: 0,
            export_functions: vec![],
        })
    }

    fn read_src_ns<T: ToString>(&self, loc: T) -> std::io::Result<ReadDir> {
        read_src(join!["/", &*self.id, &*loc.to_string()])
    }

    fn compile(&mut self) {
        let fn_fr = self.read_src_ns("/functions");
        if fn_fr.is_ok() {
            let fn_f = fn_fr.unwrap();
            let functions: Vec<(String, Vec<String>)> = get_msk_files_split(fn_f, 0);
            for (file, lines) in functions.into_iter() {
                MCFunction::process_function_file(self, file, lines)
            }
            for function in self.functions.iter_mut() {
                function.compile();
            }
        } else if self.id.ne(&"minecraft".to_string()) {
            warn(join!["No '", &*"functions".form_foreground(str::BLU), "' folder found for '", &*self.id.form_foreground(str::PNK), "'"]);
        }
    }

    fn extend_path(&self, loc: &str) -> String {
        join![unsafe {&*DATAROOT}, &*self.id, "/", loc]
    }

    fn file(&self, loc: &str) -> MFile {
        MFile::new(self.extend_path(loc))
    }

    fn save(&mut self) {
        for function in self.functions.iter() {
            let files = function.get_save_files();
            for save in files {
                let file = self.file(&*join!["functions/", &*save.0, ".mcfunction"]);
                file.save(save.1.join("\n"));
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MCFunction {
    node: Option<Node>,
    path: String,
    pub file_path: String,
    call_name: String,
    pub calls: Vec<(String, usize)>,
    pub vars: Vec<(String, String)>,
    pub meta: Meta,
    pub ln: usize,
    pub ns_id: String,
}

impl MCFunction {
    fn process_function_file(ns: &mut Namespace, file: String, mut lines: Vec<String>) {
        status(join!["Processing function file '", &*file.form_foreground(str::BLU), "'"]);
        let mut fns = vec![];
        let mut ln = 1usize;
        'lines: loop {
            if lines.len() <= 0 {
                break 'lines;
            }
            let (remove, optfn) = MCFunction::scan_function_line(&file, &lines, ns, ln);
            ln += remove;
            for _ in 0..remove {
                lines.remove(0);
            }
            if let Some(gn) = optfn {
                fns.push(gn);
            }
        }
        ns.functions.append(&mut fns);
    }

    fn scan_function_line(file: &String, lines: &Vec<String>, ns: &mut Namespace, ln: usize) -> (usize, Option<MCFunction>) {
        let rem: usize;
        let mut optfn: Option<MCFunction> = None;
        let keys: Vec<String> = lines[0].trim().split(" ").map(|x| x.to_string()).collect::<Vec<_>>();
        let fail = "◙".to_string();
        let key_1 = keys.get(0).unwrap_or(&fail);
        match &**key_1 {
            "fn" => {
                let key_2 = keys.get(1).unwrap_or(&fail);
                if !(MCFunction::is_valid_fn(key_2) && !key_2.contains(":")) {
                    error(format_out(
                        &*join!["Invalid function name \'", &*key_2.form_foreground(str::BLU), "\'"],
                        &*ns.extend_path(&*file),
                        ln,
                    ));
                }
                let res = MCFunction::extract_from(lines, file, &keys, ns, ln);
                rem = res.0;
                optfn = Some(res.1);
            }
            _ => rem = 1,//scan_pack_char(line, file),
        }
        (rem, optfn)
    }

    pub fn is_valid_fn(function: &str) -> bool {
        let mut function = function.split_once(":").unwrap_or(("", function)).1;
        if function.len() < 3 {
            return false;
        }
        let mut nid = function[..function.len() - 2].replace(|ch| ch >= 'a' && ch <= 'z', "");
        nid = nid.replace(|ch| ch >= '0' && ch <= '9', "");
        nid = nid.replace("_", "");
        nid = nid.replace("/", "");
        nid = nid.replace(".", "");
        nid.len() == 0 && function.ends_with("()")
    }

    fn extract_from(lines: &Vec<String>, file: &String, keys: &Vec<String>, ns: &mut Namespace, ln: usize)
                    -> (usize, MCFunction) {
        let mut mcf = MCFunction::new(path_without_functions(file.to_string()),
                                      keys[1].to_string().replace("()", ""), ln, ns);
        if keys.get(2).is_some() && keys[2].starts_with("{") {
            let rem = mcf.extract_block(lines, ns, ln);
            if mcf.meta.vb >= 1 {
                debug(format!(
                    "Found function \'{}\' {}",
                    mcf.call_name.form_foreground(str::BLU),
                    ns.extend_path(&*mcf.file_path)
                ).replace("/", "\\"));
                // if file.meta.vb >= 2 {
                //     debug(format!(" -> {} Lines REM", rem));
                // }
            }
            (1, mcf)
        } else {
            death_error(format_out(
                &*["Expected '{' after \'fn ", &*keys[1], "\'"].join(""),
                &*ns.extend_path(&*mcf.file_path),
                ln,
            ));
        }
    }

    fn extract_block(&mut self, lines: &Vec<String>, ns: &mut Namespace, ln: usize) -> usize {
        if lines[0].ends_with('}') {
            return 1;
        }
        let mut b = Blocker::new();
        let rem = match b.find_size_vec(lines, (0, lines[0].find("{").unwrap_or(0))) {
            Ok(o) => {
                if o.0 != Blocker::NOT_FOUND {
                    lines[1..o.0].clone_into(&mut self.node.as_mut().unwrap().lines);
                    o.0 + 1
                } else {
                    death_error(format_out("Unterminated function", &*ns.extend_path(&*self.file_path), ln))
                }
            }
            Err(e) => death_error(format_out(&*e.0, &*ns.extend_path(&*self.file_path), e.1 + ln)),
        };
        rem
    }

    fn new(mut path: String, mut call_name: String, ln: usize, ns: &Namespace) -> MCFunction {
        let file_path = path.clone();
        if call_name.contains("/") {
            let mut v = call_name.rsplit_once("/").unwrap_or(("error", "error"));
            path.push('/');
            path.push_str(v.0);
            call_name = v.1.to_string();
        }
        MCFunction {
            node: Some(Node::new(NodeType::Root, ln)),
            path,
            file_path,
            call_name,
            calls: vec![],
            vars: vec![],
            meta: ns.meta.clone(),
            ln,
            ns_id: ns.id.clone(),
        }
    }

    pub fn is_score_path(path: &String, mcf: &mut MCFunction, ln: usize) -> bool {
        if let Ok(keys) = Blocker::new().split_in_same_level(":", &path) {
            if keys.len() == 2 && MCFunction::is_board_id(&keys[1], mcf, ln) {
                return if keys[0].starts_with("@") {
                    MCFunction::is_at_ident(keys[0].clone(), true, mcf, ln)
                } else {
                    let nb = keys[0].trim_start_matches(|ch|
                        (ch >= 'a' && ch <= 'z') ||
                            (ch >= 'A' && ch <= 'Z') ||
                            (ch >= '0' && ch <= '9') ||
                            (ch >= '#' && ch <= '%') ||
                            (ch == '_'));
                    nb.len() == 0
                };
            }
        }
        path.starts_with("$") && MCFunction::is_board_id(&path[1..].to_string(), mcf, ln) && require::remgine("temporary scores", mcf, ln)
    }

    pub fn is_board_id(board: &String, mcf: &mut MCFunction, ln: usize) -> bool {
        let mut board = board.clone();
        if let Some(nb) = board.strip_prefix("r&") {
            require::remgine("remgine scoreboards", mcf, ln);
            board = nb.into();
        } else if let Some(nb) = board.strip_prefix("&") {
            board = nb.into();
        }
        let nb = board.trim_start_matches(|ch|
            (ch >= 'a' && ch <= 'z') ||
                (ch >= 'A' && ch <= 'Z') ||
                (ch >= '0' && ch <= '9') ||
                (ch == '_'));
        return nb.len() == 0;
    }

    pub fn is_at_ident(mut selector: String, error_if_not: bool, mcf: &mut MCFunction, ln: usize) -> bool {
        let esave = selector.clone();
        return if selector.starts_with("@") && {
            selector.remove(0);
            selector.starts_with(['s', 'e', 'a', 'r', 'p'])
        } && {
            if selector.len() > 1 {
                selector[1..2] == *"[" && selector.strip_suffix(']').is_some()
            } else { true }
        } { true } else if error_if_not {
            error(format_out(&*format!("Selector expected, got '{}'", esave), &*mcf.get_file_loc(), ln));
            false
        } else {
            false
        };
    }

    pub fn compile_score_path(path: &String, mcf: &mut MCFunction, ln: usize) -> [String; 2] {
        if let Ok(mut split) = Blocker::new().split_in_same_level(":", path) {
            match split.len() {
                2 => {
                    if !split[1].contains(" ") {
                        if split[1].starts_with("r&") && require::remgine("remgine scoreboards", mcf, ln) {
                            split[1] = split[1].replace("r&", "remgine.")
                        }
                        split[1] = split[1].replace("&", &*join![&*mcf.ns_id, "."]);
                        return [split[0].clone(), split[1].clone()];
                    }
                }
                1 => {
                    if split[0].starts_with("$") && require::remgine("remgine temp scoreboard", mcf, ln) {
                        return [split[0].clone(), "remgine.temp".to_string()];
                    }
                }
                _ => {}
            }
        }
        error(format_out(&*format!("Failed to compile '{}' to a scoreboard", path), &*mcf.get_file_loc(), ln));
        ["".into(), "".into()]
    }

    pub fn compile_score_command(keys: &Vec<String>, mcf: &mut MCFunction, ln: usize) -> Vec<String> {
        let mut cds = vec![];
        let mut command = "scoreboard players ".to_string();
        let target = MCValue::new(&keys[0], mcf, ln);
        if keys.len() < 3 {
            match &**keys.get(1).unwrap_or(&"get".into()) {
                "--" => {
                    command.push_str(&*join!("remove ", &*target.get(), " 1"));
                }
                "++" => {
                    command.push_str(&*join!("add ", &*target.get(), " 1"));
                }
                v @ ("reset" | "enable" | "get") => {
                    command.push_str(&*join!(v, " ", &*target.get()));
                }
                _ => {
                    error(format_out(
                        &*join!("Failed to parse score function, unknown operation '", &*keys[1].form_foreground(str::BLU), "'"),
                        &*mcf.get_file_loc(), ln));
                }
            }
            cds.push(command);
            return cds;
        }
        let target2 = MCValue::new(&keys[2].to_string(), mcf, ln);
        if !target2.is_number() {
            command.push_str("operation ");
            match &*keys[1] {
                "=" | "+=" | "-=" | "%=" | "*=" | "/=" | "<" | ">" | "><" => {
                    command.push_str(&*join!(&*target.get(), " ", &*keys[1], " ", &*target2.get()));
                }
                _ => {
                    error(format_out(
                        &*join!("Failed to parse score function, unknown operation '", &*keys[1].form_foreground(str::BLU), "'"),
                        &*mcf.get_file_loc(), ln));
                }
            }
        } else {
            match &*keys[1] {
                "=" => {
                    command.push_str(&*["set", &*target.get(), &*target2.get()].join(" "));
                }
                "+=" => {
                    command.push_str(&*["add", &*target.get(), &*target2.get()].join(" "));
                }
                "-=" => {
                    command.push_str(&*["remove", &*target.get(), &*target2.get()].join(" "));
                }
                "*=" | "%=" | "/=" | ">" | "<" | "><" if require::remgine("advanced operations", mcf, ln) => {
                    warn(format_out("Advanced operations on numbers are sub-optimal", &*mcf.get_file_loc(), ln));
                    cds.push(join!("scoreboard players set $adv_op remgine.temp ", &*target2.get()));
                    command.push_str(&*join!("operation ", &*target.get(), " ", &*keys[1], " $adv_op remgine.temp"));
                }
                _ => {
                    error(format_out(
                        &*join!("Failed to parse score function, unknown operation '", &*keys[1].form_foreground(str::BLU), "'"),
                        &*mcf.get_file_loc(), ln));
                }
            }
        }
        cds.push(command);
        return cds;
    }

    fn compile(&mut self) {
        let mut node = self.node.take().unwrap();
        node.generate(self);
        self.node = Some(node);
    }

    pub fn get_file_loc(&self) -> String {
        join![&*self.ns_id, "/functions/", &*self.file_path]
    }

    pub fn get_path(&self) -> String {
        return join![&*self.path, "/", &*self.call_name];
    }

    // fn get_write(&self) -> String {
    //     let mut v = vec![];
    //     self.node.as_ref().unwrap().get_write(, &mut v, self);
    //     v.join("\n")
    // }

    fn get_save_files(&self) -> SaveFiles {
        let mut saves = vec![];
        self.node.clone().unwrap().get_save_files(&mut saves, &mut vec![], self);
        saves
    }
}

pub type SaveFiles = Vec<(String, Vec<String>)>;

pub enum MCValue {
    Score { name: String, board: String },
    Number { value: i32 },
}

impl MCValue {
    pub fn new(key: &String, mcf: &mut MCFunction, ln: usize) -> MCValue {
        if MCFunction::is_score_path(key, mcf, ln) {
            let vv = MCFunction::compile_score_path(key, mcf, ln);
            MCValue::Score {name: vv[0].clone(), board: vv[1].clone()}
        } else {
            MCValue::Number {value: if let Ok(val) = key.parse::<i32>() {val} else {
                error(format_out(&*join!["Failed to parse '", &**key, "' as a number"], &*mcf.get_file_loc(), ln));
                0
            }}
        }
    }
    
    pub fn is_number(&self) -> bool {
        match self {
            MCValue::Score { .. } => { false }
            MCValue::Number { .. } => { true }
        }
    }
    
    pub fn get(&self) -> String {
        match self {
            MCValue::Score { name, board } => { join![&**name, " ", &**board] }
            MCValue::Number { value } => { value.to_string() }
        }
    }
}