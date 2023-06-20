// craftmine ong :krill:

use std::cmp::min;
use std::fs::{read_dir, ReadDir, remove_dir_all};
use crate::{*, server::*};
use crate::compile::require;

pub struct Datapack {
    meta: Meta,
    ln: usize,
    pub namespaces: Vec<Namespace>,
    pub src_loc: String,
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
        }
    }

    pub fn get_view_name(&self) -> String {
        self.meta.view_name.clone()
    }

    pub fn gen_meta(&mut self, pack: String) {
        self.meta.view_name = self.src_loc.clone();
        let tags = pack.split("\n").collect::<Vec<&str>>();

        for tag in tags {
            if tag.trim().is_empty() { continue; }
            if tag.starts_with("use ") {
                self.import(&tag[4..]);
                continue;
            }
            let s = tag.trim().split("=").collect::<Vec<&str>>();
            self.meta.set_property(s[0].trim(), s[1].trim(), true, ("pack".into(), self.ln));
            self.ln += 1;
        }
    }

    fn import(&self, name: &str) {
        unsafe {
            let import = fs::read_to_string(join!["./imports/", name, ".export.msk"]).unwrap_or_else(|e| {
                death_error_type(join!("Could not read '",&*join!["./imports/", name, ".export.msk"].form_foreground(str::ORN),"' (", &*e.to_string(), ")"), errors::IMPORT_NOT_FOUND);
            });
            KNOWN_FUNCTIONS.append(&mut import.split(",").map(|s| s.to_string()).collect());
        }
    }

    pub fn read_namespaces(&mut self) {
        let nss = read_dir([&*self.src_loc, "src"].join("/"));
        if nss.is_ok() {
            for ns in nss.unwrap().map(|x| x.unwrap()) {
                if ns.path().is_dir() {
                    let nsnew = Namespace::new(
                        ns.file_name().to_string_lossy().to_string(),
                        self.meta.clone(),
                    );
                    if let Some(mut ns) = nsnew {
                        ns.load_files();
                        self.namespaces.push(ns);
                    }
                }
            }
        } else {
            warn("No namespaces found".to_string());
        }
        status(join!["Loaded ", &*self.namespaces.len().to_string().form_foreground(str::GRN), " namespaces ", 
            &*join!["['", &*self.namespaces.iter().map(|n|n.id.clone()).collect::<Vec<String>>()
                .join("\', \'"), "']"].form_foreground(str::GRY)])
    }

    pub fn compile_namespaces(&mut self) {
        for i in 0..self.namespaces.len() {
            self.namespaces[i].build();
        }
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
        status(format!("Saving '{}'", &self.meta.view_name.form_foreground(str::PNK)));

        unsafe {
            DATAROOT = self.data("");
            ALIAS_FUNCTIONS.iter().for_each(|(ns, na, fnc)| {
                let added = self.namespaces.iter_mut().any(|tns| -> bool {
                    if tns.id.eq(ns) {
                        let mut mcf = MCFunction::new("".into(), na.clone(), 0, &tns);
                        mcf.node.as_mut().unwrap().lines = vec![fnc.into()];
                        mcf.compile();
                        tns.functions.push(mcf);
                        true
                    } else {
                        false
                    }
                });
                if !added {
                    if let Some(mut nsnew) = Namespace::new(ns.into(), self.meta.clone()) {
                        let mut mcf = MCFunction::new("".into(), na.clone(), 0, &nsnew);
                        mcf.node.as_mut().unwrap().lines = vec![fnc.into()];
                        mcf.compile();
                        nsnew.functions.push(mcf);
                        self.namespaces.push(nsnew);
                    }
                }
            });
        }

        remove_dir_all(self.get_dir("/generated")).ok();
        make_folder(&*self.data(""));

        let meta = MFile::new(self.root("/pack.mcmeta"));
        meta.save(META_TEMPLATE.clone()
            .replace("{VERS}", &*self.meta.version.to_string())
            .replace("{DESC}", &self.meta.description));

        for nsi in 0..self.namespaces.len() {
            self.namespaces[nsi].save();
            if read_src(&*join!["/", &*self.namespaces[nsi].id, "/extras"]).is_ok() {
                copy_dir_all(get_src_dir(&*join!["/", &*self.namespaces[nsi].id, "/extras"]),
                             self.namespaces[nsi].extend_path(""))
                    .expect("Could not copy 'extras' folder");
            }
        }

        let mut links: Vec<Link> = Vec::new();
        self.namespaces.iter_mut().for_each(|ns| {
            ns.links.iter_mut().for_each(|link| {
                link.functions = link.functions.clone().into_iter().filter(|flink| {
                    if !(unsafe {
                        KNOWN_FUNCTIONS.contains(&qc!(flink.contains(":"), flink.to_string(), join![&*ns.id, ":", &**flink]))
                    }) {
                        warn(format_out(&*join!["No such function '", &*flink.form_foreground(str::ORN), "' found for link '", &*link.path.form_foreground(str::BLU), "'"],
                                        &*join![&*ns.id, "/event_links/", &*link.path], link.ln));
                        false
                    } else {
                        true
                    }
                }).collect();
            });
            links.append(&mut ns.links);
        });

        for link in links.into_iter() {
            let file = MFile::new(self.data(&*join![&*link.path, "/tags/functions/", &*link.name, ".json"]));
            let write = link.functions.clone().into_iter().map(|s| join!["\"", &*s, "\""]).collect::<Vec<String>>();
            file.save(TAG_TEMPLATE.replace("$VALUES$", &*write.join(",\n    ")));
        }
    }

    pub fn move_clear(&self, mov: Option<String>, clear: bool) {
        if let Err(message) = read_dir(self.root("")) {
            error(join!["Failed to copy datapack: ", &*message.to_string()]);
        } else {
            let world = mov.unwrap();
            let world = join![&*world, "/datapacks/", &self.meta.view_name];
            if clear {
                let t = remove_dir_all(&world);
                if t.is_err() {
                    warn(join!["Could not clear pre-existing datapack (", &*t.unwrap_err().to_string(), ")"].form_foreground(str::RED));
                }
            }
            status(format!(
                "Copying {} {} {}",
                join!["./", &*self.root("").replace("\\", "/")].form_underline(),
                "to".form_foreground(str::GRY),
                &*world.replace("\\", "/").form_underline().form_foreground(str::GRY)
            ));
            copy_dir_all(self.root(""), world).expect(&*"Failed to copy datapack".form_foreground(str::RED));
        }
    }

    pub fn export(&self) {
        unsafe {
            let file = MFile::new(self.get_dir(&*join!["/", &*self.meta.view_name, ".export.msk"]));
            file.save(EXPORT_FUNCTIONS.join(","));
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
            "suppress_warnings" if extended => unsafe { SUPPRESS_WARNINGS = val.to_uppercase().eq("TRUE") },
            "remgine" | "name" | "version" | "description" | "suppress_warnings" if !extended => {
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
    links: Vec<Link>,
    items: Vec<Item>,
    meta: Meta,
    loaded_files: Option<[MSKFiles; 3]>,
}

impl Namespace {
    fn new(name: String, meta: Meta) -> Option<Namespace> {
        if name.eq(&"".to_string()) || {
            let mut nid = name.replace(|ch| ch >= 'a' && ch <= 'z', "");
            nid = nid.replace(|ch| ch >= '0' && ch <= '9', "");
            nid = nid.replace("_", "");
            nid.len() != 0
        } {
            error(join!["Invalid Namespace: ", &*name]);
            return None;
        }
        Some(Namespace {
            id: name,
            functions: vec![],
            links: Vec::new(),
            items: Vec::new(),
            meta,
            loaded_files: None,
        })
    }

    pub fn load_files(&mut self) {
        let mut files = [MSKFiles::new(), MSKFiles::new(), MSKFiles::new()];
        if let Ok(fn_f) = self.read_src_ns("/functions") {
            files[0] = get_msk_files_split(fn_f, 0);
        } else if self.id.ne(&"minecraft".to_string()) {
            warn(join!["No '", &*"functions".form_foreground(str::BLU), "' folder found for '", &*self.id.form_foreground(str::PNK), "'"]);
        }

        if let Ok(el_f) = self.read_src_ns("/event_links") {
            files[1] = get_msk_files_split(el_f, 0);
        }

        if let Ok(it_f) = self.read_src_ns("/items") {
            files[2] = get_msk_files_split(it_f, 0);
        }
        self.loaded_files = Some(files);
    }

    fn read_src_ns<T: ToString>(&self, loc: T) -> std::io::Result<ReadDir> {
        read_src(join!["/", &*self.id, &*loc.to_string()])
    }

    fn build(&mut self) {
        let mut files = self.loaded_files.take().unwrap();
        for (file, lines) in files[1].iter_mut() {
            self.process_link_file(file, lines)
        }
        for (file, lines) in files[0].iter_mut() {
            MCFunction::process_function_file(self, file, lines);
        }
        for (file, lines) in files[2].iter_mut() {
            self.process_item_file(file, lines)
        }
        for function in self.functions.iter_mut() {
            unsafe {
                let value = join![&*self.id, ":", &*function.get_path().trim_start_matches("/").to_string()];
                if KNOWN_FUNCTIONS.contains(&value) {
                    error(format_out(&*join!["A function with the name '", &*function.get_path().trim_start_matches("/").form_foreground(str::ORN), "' already exists"], 
                                     &*function.get_file_loc(), function.ln));
                } else {
                    KNOWN_FUNCTIONS.push(value);
                    if function.allow_export {
                        EXPORT_FUNCTIONS.push(join![&*self.id, ":", &*function.get_path().trim_start_matches("/").to_string()]);
                    }
                }
            }
        }
    }

    fn compile(&mut self) {
        for function in self.functions.iter_mut() {
            function.compile();
        }
    }

    fn process_link_file(&mut self, file: &mut String, lines: &mut Vec<String>) {
        qc!(self.meta.vb > 0, status(join!["Processing link file '", &*file.form_foreground(str::BLU), "'"]), ());
        let mut lks = Vec::new();
        for (ln, line) in lines.into_iter().enumerate() {
            if (*line).eq("") { continue; }
            let line = line.trim().split(" : ").collect::<Vec<_>>();
            if line.len() < 2 {
                warn(format_out("Not enough arguments to link", &*[&*self.id, "event_links", &*file].join("/"), ln + 1));
            } else {
                let line_link_functions = line[1].trim().replace(" ", "");
                let line_link_functions = line_link_functions.split(",")
                    .map(|f| qc!(MCFunction::is_valid_fn(&*join![f, "()"]), f, {
                        warn(format_out(&*join!["Invalid function name: '", f, "'"], &*[&*self.id, "event_links", &*file].join("/"), ln + 1));
                        "§"}).to_string())
                    .filter(|l| !l.eq(&"none"))
                    .filter(|f| !f.eq("§"))
                    .map(|l| { qc!(l.contains(":"), l, join![&*self.id, ":", &*l]) })
                    .collect::<Vec<_>>();
                unsafe {
                    KNOWN_FUNCTIONS.push(join!["#", &*file, ":", line[0]]);
                    EXPORT_FUNCTIONS.push(join!["#", &*file, ":", line[0]]);
                }
                lks.push(Link::new(file.clone(), line[0].to_string(), line_link_functions, ln));
            }
        }
        self.links.append(&mut lks);
    }

    fn process_item_file(&mut self, file: &mut String, lines: &mut Vec<String>) {
        qc!(self.meta.vb > 0, status(join!["Processing item file '", &*file.form_foreground(str::BLU), "'"]), ());
        let item = Item::new(file, lines, self);
        // item.function.compile();
        self.functions.push(item.function.clone());
        self.items.push(item);
    }

    fn extend_path(&self, loc: &str) -> String {
        join![unsafe {&*DATAROOT}, &*self.id, "/", loc]
    }

    fn file(&self, loc: &str) -> MFile {
        MFile::new(self.extend_path(loc))
    }

    fn save(&mut self) {
        let mut files: SaveFiles = vec![];
        for function in self.functions.iter_mut() {
            files.append(&mut function.get_save_files());
        }
        for save in files {
            let file = self.file(&*join!["functions/", &*save.0, ".mcfunction"]);
            file.save(save.1.join("\n"));
        }

        for item in self.items.iter() {
            let mut write_recipe = RECIPE_TEMPLATE.to_string().replace("$PATTERN$", &*item.recipe.join(",\n    "));
            let mut mats = vec![];
            for mat in &item.materials {
                if mat.1.starts_with("#") {
                    mats.push(MAT_TAG_TEMPLATE.to_string().replace("$ID$", &*mat.0).replace("$TYPE$", &mat.1[1..]));
                } else {
                    mats.push(MAT_TEMPLATE.to_string().replace("$ID$", &*mat.0).replace("$TYPE$", &*mat.1));
                }
            }
            write_recipe = write_recipe.replace("$MATERIALS$", &*mats.join(",\n    "));
            let file = self.file(&*join!["recipes/", &*item.fn_call_path, ".json"]);
            file.save(write_recipe);

            let mut write_adv = qc!(self.meta.version >= 14, ADV_CRAFT_TEMPLATE_120, ADV_CRAFT_TEMPLATE_119).to_string();
            write_adv = write_adv.replace("$PATH$", &*join![&*self.id, ":", &*item.fn_call_path])
                .replace("$CALL$", &*join![&*self.id, ":", &*item.fn_call_path]);
            let file = self.file(&*join!["advancements/", &*item.fn_call_path, ".json"]);
            file.save(write_adv);
        }
    }
}

#[derive(Debug, Clone)]
pub struct MCFunction {
    node: Option<Node>,
    pub file_path: String,
    call_path: String,
    call_name: String,
    pub calls: Vec<(String, usize)>,
    pub vars: Vec<(String, String)>,
    pub meta: Meta,
    pub ln: usize,
    pub ns_id: String,
    allow_export: bool,
}

type FileData = (Vec<(String, String)>, (bool, Option<String>), bool);

impl MCFunction {
    fn process_function_file(ns: &mut Namespace, file: &mut String, lines: &mut Vec<String>) {
        let meta = ns.meta.clone();
        qc!(ns.meta.vb > 0, status(join!["Processing function file '", &*file.form_foreground(str::BLU), "'"]), ());
        let mut fns = vec![];
        let mut ln = 1usize;
        let mut data: FileData = (vec![], (false, None), true);
        'lines: loop {
            if lines.len() <= 0 {
                break 'lines;
            }
            let (remove, optfn) = MCFunction::scan_function_line(file, lines, ns, ln, &mut data);
            ln += remove;
            *lines = lines[remove..].to_vec();
            if let Some(mut gn) = optfn {
                if data.1.0 {
                    let path = data.1.1.unwrap_or(gn.call_name.clone());
                    let (exns, exna) = path.split_once(":").unwrap_or(("minecraft", &*path));
                    unsafe {
                        ALIAS_FUNCTIONS.push((exns.into(), exna.into(), join!("function ", &*ns.id, ":", &*gn.get_path().trim_start_matches("/"))));
                    }
                }
                gn.allow_export = data.2;
                data.1 = (false, None);
                data.2 = true;
                fns.push(gn);
                ns.meta = meta.clone();
            }
        }
        ns.functions.append(&mut fns);
    }

    fn scan_function_line(file: &String, lines: &Vec<String>, ns: &mut Namespace, ln: usize, data: &mut FileData) -> (usize, Option<MCFunction>) {
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
                let mut res = MCFunction::extract_from(lines, file, &keys, ns, ln);
                res.1.vars.append(&mut data.0.clone());
                optfn = Some(res.1);
                rem = res.0;
            }
            "@set" if require::min_args_path(3, &keys, join![&*ns.id, "/functions/", &*file], ln) => {
                data.0.push((keys[1].clone(), keys[2..].join(" ")));
                rem = 1;
            }
            "@meta" if require::min_args_path(3, &keys, join![&*ns.id, "/functions/", &*file], ln) => {
                let prop = &*keys[1];
                let value = &*keys[2];
                ns.meta.set_property(prop, value, false, (ns.extend_path(&*file), ln));
                rem = 1;
            }
            "@alias" => {
                data.1.0 = true;
                data.1.1 = keys.get(1).cloned();
                rem = 1;
            }
            "@no_export" => {
                data.2 = false;
                rem = 1;
            }
            _ => rem = MCFunction::scan_pack_char(file, &lines[0], ns, ln),
        }
        (rem, optfn)
    }

    fn scan_pack_char(file: &String, line: &String, ns: &mut Namespace, ln: usize) -> usize {
        let rem: usize = 1;
        let char_1: char = *line
            .trim()
            .chars()
            .collect::<Vec<_>>()
            .get(0)
            .unwrap_or(&'◙');
        match char_1 {
            '/' | '◙' | ' ' | '@' => {}
            c @ _ => error(format_out(
                &*join!["Unexpected token '", &*c.form_foreground(str::ORN), "'"],
                &*ns.extend_path(&*file), ln)),
        }
        rem
    }

    pub fn is_valid_fn(function: &str) -> bool {
        let function = function.split_once(":").unwrap_or(("", function)).1;
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
        let mut mcf = MCFunction::new(file.to_string(),
                                      keys[1].to_string().replace("()", ""), ln, ns);
        if keys.get(2).is_some() && keys[2].starts_with("{") {
            let rem = mcf.extract_block(lines, ns, ln);
            if mcf.meta.vb >= 1 {
                debug(format!(
                    "Found function \'{}\' {}",
                    mcf.call_name.form_foreground(str::BLU),
                    ns.extend_path(&*mcf.file_path)
                ).replace("/", "\\"));
            }
            (rem, mcf)
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
            let v = call_name.rsplit_once("/").unwrap_or(("error", "error"));
            path.push('/');
            path.push_str(v.0);
            call_name = v.1.to_string();
        }
        MCFunction {
            node: Some(Node::new(NodeType::Root, ln)),
            call_path: path_without_functions(path),
            file_path,
            call_name,
            calls: vec![],
            vars: vec![],
            meta: ns.meta.clone(),
            ln,
            ns_id: ns.id.clone(),
            allow_export: true,
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
                            (ch == '-') ||
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
                (ch == '_') ||
                (ch == '-') ||
                (ch == '.'));
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
                        compile::replace_local_tags(&mut split, mcf);
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
        return join![&*self.call_path, "/", &*self.call_name];
    }

    fn get_save_files(&mut self) -> SaveFiles {
        let mut saves = vec![];
        self.node.clone().unwrap().get_save_files(&mut saves, &mut vec![], self);
        saves
    }
}

pub struct Link {
    path: String,
    name: String,
    functions: Vec<String>,
    ln: usize,
}

impl Link {
    fn new(path: String, name: String, links: Vec<String>, ln: usize) -> Link {
        Link {
            path,
            name,
            functions: links,
            ln,
        }
    }
}

pub type MSKFiles = Vec<(String, Vec<String>)>;
pub type SaveFiles = Vec<(String, Vec<String>)>;

pub enum MCValue {
    Score { name: String, board: String },
    Number { value: i32 },
}

impl MCValue {
    pub fn new(key: &String, mcf: &mut MCFunction, ln: usize) -> MCValue {
        if MCFunction::is_score_path(key, mcf, ln) {
            let vv = MCFunction::compile_score_path(key, mcf, ln);
            MCValue::Score { name: vv[0].clone(), board: vv[1].clone() }
        } else {
            MCValue::Number {
                value: if let Ok(val) = key.parse::<i32>() { val } else {
                    error(format_out(&*join!["Failed to parse '", &**key, "' as a number"], &*mcf.get_file_loc(), ln));
                    0
                }
            }
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

    pub fn set_equal_to(&self, other: &MCValue, mcf: &MCFunction, ln: usize) -> String {
        if self.is_number() {
            error(format_out("Cannot assign a number a value", &*mcf.get_file_loc(), ln));
        }
        return match other {
            MCValue::Score { name, board } => {
                join!["scoreboard players operation ", &*self.get(), " = ", &*name, " ", &*board]
            }
            MCValue::Number { value } => {
                join!["scoreboard players set ", &*self.get(), " ", &*value.to_string()]
            }
        };
    }

    pub fn get_less_than(&self, other: &MCValue, mcf: &MCFunction, ln: usize) -> String {
        if self.is_number() {
            error(format_out("Cannot have a number on the left-hand side of a test", &*mcf.get_file_loc(), ln));
        }
        return match other {
            MCValue::Score { name, board } => {
                join!["if score ", &*self.get(), " < ", &*name, " ", &*board]
            }
            MCValue::Number { value } => {
                join!["if score ", &*self.get(), " matches ..", &*(value - 1).to_string()]
            }
        };
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    recipe: Vec<String>,
    materials: Vec<(String, String)>,
    fn_call_path: String,
    file_name: String,
    function: MCFunction,
}

impl Item {
    fn new(name: &String, lines: &mut Vec<String>, ns: &mut Namespace) -> Item {
        let mut item = Item {
            recipe: vec![],
            materials: vec![],
            fn_call_path: name.to_string(),
            function: MCFunction::new(name.to_string(), join!["item_", &*name], 0, &ns),
            file_name: name.to_string(),
        };

        let mut ln = 0;
        while ln < lines.len() {
            let rem = item.parse_line(ln, lines, ns);
            ln += rem;
        }


        item
    }

    fn parse_line(&mut self, ln: usize, lines: &Vec<String>, ns: &mut Namespace) -> usize {
        let mut keys = Blocker::new().split_in_same_level(" ", &lines[ln]).unwrap_or_else(|e| {
            error(format_out(&*join!("Failed to parse item: ", &*e), &*self.get_path(ns), ln + 1));
            return vec!["ERROR".into()];
        });
        match &*keys[0] {
            "ERROR" => {
                return lines.len();
            }
            "recipe" => {
                if !keys[1].eq("{") {
                    error(format_out("Invalid 'recipe' block", &*self.get_path(ns), ln + 1));
                }
                let rem = Blocker::auto_vec(&lines, (ln, lines[ln].len() - 1), self.get_path(ns), ln).0 + 1;
                let pattern = lines[(ln + 1)..(ln + rem - 1)].to_vec();
                if pattern.len() < 1 || pattern.len() > 3 {
                    error(format_out("Invalid recipe pattern", &*self.get_path(ns), ln + 1));
                }
                self.recipe = pattern;
                rem
            }
            "materials" => {
                if !keys[1].eq("{") {
                    error(format_out("Invalid 'materials' block", &*self.get_path(ns), ln + 1));
                }
                let rem = Blocker::auto_vec(&lines[ln..].to_vec(), (0, lines[ln].len() - 1), self.get_path(ns), ln).0 + 1;
                let mats = lines[(ln + 1)..(ln + rem - 1)].to_vec();
                let mats = mats.into_iter().map(|s| -> (String, String) {
                    let v = s.split(" : ").collect::<Vec<&str>>();
                    let v = v.into_iter().map(|s| s.to_string()).collect::<Vec<String>>();
                    (v[0].to_string(), v[1].to_string())
                }).collect::<Vec<_>>();
                self.materials = mats;
                rem
            }
            "path" if require::exact_args(3, &keys, &mut self.function, ln) => {
                let (path, name) = keys[2].rsplit_once("/").unwrap_or(("", &*keys[2]));
                self.fn_call_path = keys[2].to_string();
                self.function.file_path = path.to_string();
                self.function.call_path = path.to_string();
                self.function.call_name = name.to_string();
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
                keys.insert(0, "fn".into());

                let (remx, mut nna) = MCFunction::extract_from(&lines[ln..].to_vec(), &self.file_name, &keys, ns, ln);

                if let Some(ref mut node) = nna.node {
                    node.lines.insert(0, "{".into());
                    node.lines.append(&mut vec![
                        join!["}"],
                        join!["clear @s knowledge_book"],
                        join!["advancement revoke @s only ", &*ns.id, ":", &*self.fn_call_path],
                        // join!["recipe take @s ", &*ns.id, ":", &*self.fn_call_path],
                    ])
                }

                nna.file_path = self.function.file_path.to_string();
                nna.call_path = self.function.call_path.to_string();
                nna.call_name = self.function.call_name.to_string();
                self.function = nna;
                remx + 1
            }
            _ => 1
        }
    }

    fn get_path(&self, ns: &Namespace) -> String {
        [&*ns.id, "items", &*self.file_name].join("/")
    }
}