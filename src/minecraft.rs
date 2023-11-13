// craftmine ong :krill:

use std::cmp::min;
use std::ffi::OsStr;
use std::fs::{read_dir, ReadDir, remove_dir_all};
use remtools::{*, colors::*};
use crate::{*, server::*};
use crate::compile::require;
use crate::minecraft::CachedType::{Recompile, Unchanged};

pub struct Datapack {
    meta: Meta,
    ln: usize,
    pub namespaces: Vec<Namespace>,
    pub src_loc: String,
    pub pack_frag: Magnet<MskCache>,
}

impl Datapack {
    pub fn new(path: String) -> Datapack {
        Datapack {
            meta: Meta::new(),
            ln: 1,
            src_loc: path,
            namespaces: vec![],
            pack_frag: Magnet::Unattached,
        }
    }

    pub fn get_view_name(&self) -> String {
        self.meta.view_name.clone()
    }

    pub fn gen_meta(&mut self, pack: String, cache: bool) {
        self.meta.view_name = self.src_loc.clone();
        let bind = pack.clone();
        let tags = bind.split("\n").collect::<Vec<&str>>();

        for tag in tags {
            if tag.trim().is_empty() { continue; }
            if tag.starts_with("use ") {
                self.import(tag[4..].trim().to_string());
                continue;
            }
            let s = tag.trim().split("=").collect::<Vec<&str>>();
            self.meta.set_property(s[0].trim(), s[1].trim(), true, ("pack".into(), self.ln));
            self.ln += 1;
        }
        if cache {
            for file in read_dir(join![&*self.src_loc, "/src/"]).unwrap() {
                if let Ok(dir) = file {
                    if dir.file_name() == OsStr::new("pack.msk") {
                        let node = Node {
                            node: NodeType::None,
                            children: vec![],
                            lines: bind.split("\n").map(String::from).collect(),
                            ln: 0,
                        };
                        self.pack_frag.attach(MskCache::from_msk(&dir));
                        self.pack_frag.extern_frag.attach(CachedFrag::new("pack".into()));
                        self.pack_frag.extern_frag.update_hash(&node);
                        self.pack_frag.extern_frag.files.attach(vec![("pack".to_string(), node.lines)]);
                        break;
                    }
                }
            }
            unsafe {
                let mut irem = None;
                'caches: for (i, (_, i_cache)) in I_CACHED_MSK.iter().enumerate() {
                    if i_cache.file_path.eq("pack".into()) {
                        irem = Some(i);
                        let cached_type = i_cache.compare_to(&self.pack_frag);
                        if cached_type != Unchanged {
                            status_color("[!] pack.msk was changed, clearing cache".into(), GRY);
                            I_CACHED_MSK.clear();
                            irem = None;
                        }
                        break 'caches;
                    }
                }
                if let Some(i) = irem {
                    I_CACHED_MSK.remove(i);
                }
            }
        }
    }

    fn import(&self, name: String) {
        unsafe {
            let import = fs::read_to_string(join!["./imports/", &*name, ".export.msk"]).unwrap_or_else(|e| {
                death_error(join!("Could not read '",&*join!["./imports/", &*name, ".export.msk"].foreground(ORN).end(),"' (", &*e.to_string(), ")"), errors::IMPORT_NOT_FOUND);
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
        status(join!["Loaded ", &*self.namespaces.len().to_string().foreground(GRN).end(), " namespaces ", 
            &*join!["['", &*self.namespaces.iter().map(|n|n.id.clone()).collect::<Vec<String>>()
                .join("\', \'"), "']"].foreground(GRY).end()])
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
        join![unsafe{&*GEN_LOC}, "/", &*self.meta.view_name, loc]
    }

    pub fn data(&self, loc: &str) -> String {
        join![unsafe{&*GEN_LOC}, "/", &*self.meta.view_name, "/data/", loc]
    }

    pub fn save(&mut self, gen: String, cache: bool) {
        remove_dir_all(join![&*self.src_loc, "/.cache"]).ok();

        status(format!("Saving '{}'", &self.meta.view_name.clone().foreground(PNK).end()));

        unsafe {
            GEN_LOC = gen;
            DATAROOT = self.data("");
            remove_dir_all(join!["/": &*GEN_LOC, &*self.meta.view_name]).ok();
        }

        make_folder(&*self.data(""));

        let meta = MFile::new(self.root("/pack.mcmeta"));
        meta.save(META_TEMPLATE.clone()
            .replace("{VERS}", &*{
                let (Ok(s) | Err(s)) = self.meta.version.clone().map(|v| v.to_string());
                s
            })
            .replace("{DESC}", &self.meta.description)).map_err(|e| {
            soft_error(e.to_string());
        }).ok();

        for nsi in 0..self.namespaces.len() {
            self.namespaces[nsi].save(cache);
            if read_src(&*join!["/", &*self.namespaces[nsi].id, "/extras"]).is_ok() {
                copy_dir_all(get_src_dir(&*join!["/", &*self.namespaces[nsi].id, "/extras"]),
                             self.namespaces[nsi].extend_path(""))
                    .expect("Could not copy 'extras' folder");
            }
        }

        let mut links: Vec<Link> = Vec::new();
        self.namespaces.iter_mut().for_each(|ns| {
            ns.links.iter_mut().for_each(|link| {
                link.functions = link.functions.clone().into_iter().filter(|flink|
                    !(unsafe {
                        !KNOWN_FUNCTIONS.contains(&qc!(flink.contains(":"), flink.to_string(), join![&*ns.id, ":", &**flink]))
                    } && {
                        warn(format_out(&*join!["No such function '", &*flink.clone().foreground(ORN).end(), "' found for link '", &*link.path.clone().foreground(BLU).end(), "'"],
                                        &*join![&*ns.id, "/event_links/", &*link.path], link.ln));
                        true
                    })
                ).collect();
            });
            links.append(&mut ns.links);
        });

        for link in links.into_iter() {
            let file = MFile::new(self.data(&*join![&*link.path, "/tags/functions/", &*link.name, ".json"]));
            let write = link.functions.clone().into_iter().map(|s| join!["\"", &*s, "\""]).collect::<Vec<String>>();
            file.save(TAG_TEMPLATE.replace("$VALUES$", &*write.join(",\n    "))).map_err(|e| {
                soft_error(e.to_string());
            }).ok();
        }

        unsafe {
            if cache {
                for fragment in O_GEN_FRAGMENTS.iter_mut() {
                    fragment.save_to_file();
                }
                self.pack_frag.save_to_file();
            }
        }
    }

    pub fn _move_clear(&self, mov: Option<String>, clear: bool) {
        if let Err(message) = read_dir(self.root("")) {
            error(join!["Failed to copy datapack: ", &*message.to_string()]);
        } else {
            let world = mov.unwrap();
            let world = join![&*world, "/datapacks/", &self.meta.view_name];
            if clear {
                let t = remove_dir_all(&world);
                if t.is_err() {
                    warn(join!["Could not clear pre-existing datapack (", &*t.unwrap_err().to_string(), ")"].foreground(RED).end());
                }
            }
            status(format!(
                "Copying {} {} {}",
                join!["./", &*self.root("").replace("\\", "/")].modifier(UNDERLINE).end(),
                "to".foreground(GRY).end(),
                &*world.replace("\\", "/").modifier(UNDERLINE).foreground(GRY).end()
            ));
            copy_dir_all(self.root(""), world).expect(&*"Failed to copy datapack".foreground(RED).end());
        }
    }

    pub fn export(&self) {
        unsafe {
            let file = MFile::new(self.get_dir(&*join!["/", &*self.meta.view_name, ".export.msk"]));
            file.save(EXPORT_FUNCTIONS.join(",")).map_err(|e| {
                soft_error(e.to_string());
            }).ok();
        }
    }
}

#[derive(Clone, Debug)]
pub struct Meta {
    pub vb: i32,
    pub version: Result<u8, String>,
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
            version: Ok(CURRENT_PACK_VERSION),
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
            "version" if extended => self.version = val.parse::<u8>().map_err(|_| val.to_string()),
            "description" if extended => self.description = val.to_string(),
            "suppress_warnings" if extended => unsafe { SUPPRESS_WARNINGS = val.to_uppercase().eq("TRUE") },
            "remgine" | "name" | "version" | "description" | "suppress_warnings" if !extended => {
                warn(
                    format_out(
                        &*["Cannot override property \'", &*property.foreground(BLU).end(), "\' in this context (value = \'", &*val.foreground(GRY).end(), "\')"].join(""),
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
                        &*["Unknown property: \'", &*property.foreground(BLU).end(), "\' (value = \'", &*val.foreground(GRY).end(), "\')"].join(""),
                        &*warns.0,
                        warns.1,
                    ),
                );
                suc = false
            }
        }
        if suc && self.vb >= 1 {
            debug(format!("Set property \'{}\' to \'{}\'", property.foreground(BLU).end(), val.foreground(AQU).end()));
        }
    }
}

pub struct Namespace {
    pub id: String,
    functions: Vec<MCFunction>,
    links: Vec<Link>,
    items: Vec<Item>,
    meta: Meta,
    loaded_files: Magnet<[MSKFiles; 3]>,
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
            loaded_files: Magnet::new(None),
        })
    }

    pub fn load_files(&mut self) {
        let mut files = [MSKFiles::new(), MSKFiles::new(), MSKFiles::new()];
        if let Ok(fn_f) = self.read_src_ns("/functions") {
            files[0] = get_msk_files_split(fn_f, 0);
        } else if self.id.ne(&"minecraft".to_string()) {
            warn(join!["No '", &*"functions".foreground(BLU).end(), "' folder found for '", &*self.id.clone().foreground(PNK).end(), "'"]);
        }

        if let Ok(el_f) = self.read_src_ns("/event_links") {
            files[1] = get_msk_files_split(el_f, 0);
        }

        if let Ok(it_f) = self.read_src_ns("/items") {
            files[2] = get_msk_files_split(it_f, 0);
        }
        self.loaded_files.attach(files);
    }

    fn read_src_ns<T: ToString>(&self, loc: T) -> std::io::Result<ReadDir> {
        read_src(join!["/", &*self.id, &*loc.to_string()])
    }

    fn build(&mut self) {
        let mut files = self.loaded_files.unattach();
        for (file, lines, _) in files[1].iter_mut() {
            self.process_link_file(file, lines)
        }
        for (file, lines, cache) in files[0].iter_mut() { // function file -> ast
            MCFunction::process_function_file(self, file, lines, cache);
        }
        for (file, lines, cache) in files[2].iter_mut() {
            self.process_item_file(file, lines, cache)
        }
        for function in self.functions.iter_mut() {
            unsafe {
                let value = join![&*self.id, ":", &*function.get_path().to_string()];
                if KNOWN_FUNCTIONS.contains(&value) {
                    error(format_out(&*join!["A function with the name '", &*function.get_path().foreground(ORN).end(), "' already exists"],
                                     &*function.get_file_loc(), function.ln));
                } else {
                    KNOWN_FUNCTIONS.push(value);
                    if function.allow_export {
                        EXPORT_FUNCTIONS.push(join![&*self.id, ":", &*function.get_path().to_string()]);
                    }
                }
            }
        }
        self.loaded_files.attach(files);
    }

    fn compile(&mut self) {
        for function in self.functions.iter_mut() { // ast -> function files
            function.test_compile();
        }
        for item in self.items.iter_mut() { // ast -> function files
            item.compile();
        }
    }

    fn process_link_file(&mut self, file: &mut String, lines: &mut Vec<String>) {
        qc!(self.meta.vb > 0, status(join!["Processing link file '", &*file.clone().foreground(BLU).end(), "'"]), ());
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

    fn process_item_file(&mut self, file: &mut String, lines: &mut Vec<String>, o_cache: &mut MskCache) {
        qc!(self.meta.vb > 0, status(join!["Processing item file '", &*file.clone().foreground(BLU).end(), "'"]), ());
        let item = Item::new(file, lines, self, o_cache);
        unsafe {
            let value = join![&*self.id, ":", &*item.function.get_path().to_string()];
            if KNOWN_FUNCTIONS.contains(&value) {
                error(format_out(&*join!["A function with the name '", &*item.function.get_path().foreground(ORN).end(), "' already exists"],
                                 &*item.function.get_file_loc(), item.function.ln));
            } else {
                KNOWN_FUNCTIONS.push(value);
                if item.function.allow_export {
                    EXPORT_FUNCTIONS.push(join![&*self.id, ":", &*item.function.get_path().to_string()]);
                }
            }
        }

        self.items.push(item);
    }

    fn extend_path(&self, loc: &str) -> String {
        join![unsafe {&*DATAROOT}, &*self.id, "/", loc]
    }

    fn save(&mut self, cache: bool) {
        if self.loaded_files.is_attached() && cache {
            for (_, _, ref mut cache) in self.loaded_files.as_mut()[0].iter_mut() {
                cache.save_to_file();
            }
        }
        let mut files: SaveFiles = vec![];
        for function in self.functions.iter_mut() {
            if cache {
                if !function.fragment.is_attached() {
                    dbg!(function);
                    continue;
                }
                files.append(&mut function.fragment.files.clone());
                unsafe {
                    O_GEN_FRAGMENTS.push(function.fragment.unattach());
                }
            } else {
                files.append(&mut function.fragment.files);
            }
        }
        for save in files {
            let file = MFile::new(join![unsafe {&*DATAROOT}, &*save.0, ".mcfunction"]);
            file.save(save.1.join("\n")).map_err(|e| {
                soft_error(e.to_string());
            }).ok();
        }

        let mut files: SaveFiles = vec![];
        for item in self.items.iter_mut() {
            item.save();
            if cache {
                if !item.cache.is_attached() {
                    dbg!(item);
                    continue;
                }
                files.append(&mut item.cache.extern_frag.files.clone());
                unsafe {
                    O_GEN_FRAGMENTS.push(item.cache.extern_frag.unattach());
                }
                item.cache.save_to_file();
            } else {
                files.append(&mut item.cache.extern_frag.files);
            }
        }
        for save in files {
            let file = MFile::new(join![unsafe {&*DATAROOT}, &*save.0]);
            file.save(save.1.join("\n")).map_err(|e| {
                soft_error(e.to_string());
            }).ok();
        }
    }
}

#[derive(Debug, Clone)]
pub struct MCFunction {
    pub fragment: Magnet<CachedFrag>,
    pub node: Magnet<Node>,
    pub file_path: String,
    pub call_path: String,
    pub call_name: String,
    pub calls: Vec<(String, usize)>,
    pub vars: Vec<(String, String)>,
    pub meta: Meta,
    pub ln: usize,
    pub ns_id: String,
    pub allow_export: bool,
    pub cached_type: CachedType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CachedType {
    Unchanged,
    Changed,
    Recompile,
}

type FileData = (Vec<(String, String)>, (bool, Option<String>), bool);

impl MCFunction {
    fn process_function_file(ns: &mut Namespace, file: &mut String, lines: &mut Vec<String>, o_cache: &mut MskCache) {
        let meta = ns.meta.clone();
        qc!(ns.meta.vb > 0, status(join!["Processing function file '", &*file.clone().foreground(BLU).end(), "'"]), ());
        let mut fns = vec![];
        let mut ln = 1usize;
        let mut data: FileData = (vec![], (false, None), true);
        let mut extern_lines = vec![];

        'lines: loop {
            if lines.len() <= 0 {
                break 'lines;
            }
            let (remove, optfn) = MCFunction::scan_function_line(file, lines, ns, ln, &mut data, &mut extern_lines);
            ln += remove;
            *lines = lines[remove..].to_vec();
            if let Some((mut gn, cache_name)) = optfn {
                if data.1.0 {
                    let path = data.1.1.unwrap_or(gn.call_name.clone());
                    let (exns, exna) = path.split_once(":").unwrap_or(("minecraft", &*path));
                    let nnode = Node {
                        node: NodeType::Alias(exns.into(), exna.into()),
                        children: vec![],
                        lines: vec![join!("function ", &*ns.id, ":", &*gn.get_path())],
                        ln,
                    };
                    gn.node.children.push(nnode);
                }
                gn.allow_export = data.2;
                data.1 = (false, None);
                data.2 = true;
                let mut fragment = CachedFrag::make_frag(cache_name, o_cache);
                fragment.update_hash(&gn.node);
                gn.fragment.attach(fragment);
                fns.push(gn);
                ns.meta = meta.clone();
            }
        }

        let mut cached_type = Recompile;
        let mut extern_frag = CachedFrag::make_frag("_EXTERN".to_string(), o_cache);
        extern_frag.files = Magnet::new(Some(vec![("_EXTERN".to_string(), extern_lines)]));
        o_cache.extern_frag.attach(extern_frag);
        unsafe {
            let mut irem = None;
            'caches: for (i, (_, i_cache)) in I_CACHED_MSK.iter().enumerate() {
                if i_cache.file_path.eq(&join![&*ns.id, "/functions/", &**file].replace("/", "$")) {
                    irem = Some(i);
                    cached_type = i_cache.compare_to(o_cache);
                    break 'caches;
                }
            }
            if let Some(i) = irem {
                I_CACHED_MSK.remove(i);
            }
        }

        fns.iter_mut().for_each(|func| func.cached_type = cached_type.clone());

        ns.functions.append(&mut fns);
    }

    fn scan_function_line(file: &String, lines: &Vec<String>, ns: &mut Namespace, ln: usize, data: &mut FileData, extern_lines: &mut Vec<String>) -> (usize, Option<(MCFunction, String)>) {
        let rem: usize;
        let mut optfn = None;
        let keys: Vec<String> = lines[0].trim().split(" ").map(|x| x.to_string()).collect::<Vec<_>>();
        let fail = "◙".to_string();
        let key_1 = keys.get(0).unwrap_or(&fail);
        match &**key_1 {
            _ if {
                if &**key_1 != "" && !key_1.starts_with("//") {
                    extern_lines.push(lines[0].clone());
                }
                false
            } => { rem = 1; }
            "fn" => {
                let key_2 = keys.get(1).unwrap_or(&fail);
                if !(MCFunction::is_valid_fn(key_2) && !key_2.contains(":")) {
                    error(format_out(
                        &*join!["Invalid function name \'", &*key_2.clone().foreground(BLU).end(), "\'"],
                        &*ns.extend_path(&*file),
                        ln,
                    ));
                }
                let mut res = MCFunction::extract_from(lines, file, &keys, ns, ln);
                res.1.vars.append(&mut data.0.clone());
                optfn = Some((res.1, key_2[..key_2.len() - 2].to_string()));
                rem = res.0;
            }
            "@set" if require::min_args_path(3, &keys, join![&*ns.id, "/functions/", &*file], ln) => {
                require::not_default_replacement(&keys[1], join![&*ns.id, "/functions/", &*file], ln);
                rem = 1;
                data.0.push((keys[1].clone(), keys[2..].join(" ")));
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
                &*join!["Unexpected token '", &foreground(ORN).end(), &*c.to_string(), END, "'"],
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
                    mcf.call_name.clone().foreground(BLU).end(),
                    ns.extend_path(&*mcf.file_path)
                ).replace("/", "\\"));
            }
            (rem, mcf)
        } else {
            death_error(format_out(
                &*["Expected '{' after \'fn ", &*keys[1], "\'"].join(""),
                &*ns.extend_path(&*mcf.file_path),
                ln,
            ), errors::AST_ERROR);
        }
    }

    fn extract_block(&mut self, lines: &Vec<String>, ns: &mut Namespace, ln: usize) -> usize {
        if lines[0].ends_with('}') {
            return 1;
        }
        let mut b = Blocker::new();
        let rem = match b.quick_block_end(lines) {
            Ok(o) => {
                if o != Blocker::NOT_FOUND {
                    lines[1..o].clone_into(&mut self.node.lines);
                    o + 1
                } else {
                    death_error(format_out("Unterminated function", &*ns.extend_path(&*self.file_path), ln), errors::AST_ERROR)
                }
            }
            Err(e) => death_error(format_out(&*e.0, &*ns.extend_path(&*self.file_path), e.1 + ln), errors::AST_ERROR),
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
            fragment: Magnet::new(None),
            node: Magnet::new(Some(Node::new(NodeType::Root, ln))),
            call_path: path_without_functions(path),
            file_path,
            call_name,
            calls: vec![],
            vars: vec![],
            meta: ns.meta.clone(),
            ln,
            ns_id: ns.id.clone(),
            allow_export: true,
            cached_type: Recompile,
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
                        &*join!("Failed to parse score function, unknown operation '", &*keys[1].clone().foreground(BLU).end(), "'"),
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
                        &*join!("Failed to parse score function, unknown operation '", &*keys[1].clone().foreground(BLU).end(), "'"),
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
                        &*join!("Failed to parse score function, unknown operation '", &*keys[1].clone().foreground(BLU).end(), "'"),
                        &*mcf.get_file_loc(), ln));
                }
            }
        }
        cds.push(command);
        return cds;
    }

    fn test_compile(&mut self) {
        // recompile => trash data ----------- 
        // changed   => test vs cache -------- read
        // unchanged => get files from cache - read
        if !self.fragment.is_attached() {
            self.cached_type = Recompile;
        }
        if self.cached_type != Recompile {
            let mut i_frag = CachedFrag::new(self.fragment.name.clone());
            self.cached_type = Recompile;
            if i_frag.read_from_file() {
                if *self.fragment == i_frag {
                    self.cached_type = Unchanged;
                    *self.fragment = i_frag;
                }
            }
        }
        // all types should either be Unchanged or Recompile
        if self.cached_type != Unchanged {
            self.compile();
            self.update_save_files();
        }
    }

    fn compile(&mut self) {
        if !self.fragment.is_attached() {
            self.fragment.attach(CachedFrag::from_mcfunction(self));
        }
        let mut node = self.node.unattach();
        node.generate(self);
        self.node.attach(node);
    }

    pub fn get_file_loc(&self) -> String {
        join![&*self.ns_id, "/functions/", &*self.file_path]
    }

    pub fn get_path(&self) -> String {
        return qc! {self.call_path.is_empty(), self.call_name.clone(), join![&*self.call_path, "/", &*self.call_name]};
    }

    fn update_save_files(&mut self) {
        let mut saves = vec![];
        self.node.clone().unattach().get_save_files(&mut saves, &mut vec![], self);
        if !self.fragment.is_attached() {
            status(join!["Cache tried to update save files on a function without a fragment (", &*self.file_path, ")"]);
            self.fragment.attach(CachedFrag::from_mcfunction(self));
        }
        saves.iter_mut().for_each(|f| {
            if f.0.starts_with("@ALIAS") {
                f.0.drain(0..6);
            } else {
                f.0 = join![&*self.ns_id, "/functions/", &*f.0]
            }
        });
        self.fragment.files = Magnet::new(Some(saves));
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

pub type MSKFiles = Vec<(String, Vec<String>, MskCache)>;
pub type CacheFiles = Vec<(Vec<u8>, MskCache)>;
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

#[derive(Debug)]
pub struct Item {
    recipe: Vec<String>,
    materials: Vec<(String, String)>,
    fn_call_path: String,
    file_name: String,
    function: MCFunction,
    cache: Magnet<MskCache>,
    cached_type: CachedType,
}

impl Item {
    fn new(name: &String, lines: &mut Vec<String>, ns: &mut Namespace, o_cache: &mut MskCache) -> Item {
        let mut cached_type = Recompile;
        unsafe {
            let mut irem = None;
            'caches: for (i, (_, i_cache)) in I_CACHED_MSK.iter().enumerate() {
                if i_cache.file_path.eq(&join![&*ns.id, "/items/", &**name].replace("/", "$")) {
                    irem = Some(i);
                    cached_type = i_cache.compare_to(o_cache);
                    break 'caches;
                }
            }
            if let Some(i) = irem {
                I_CACHED_MSK.remove(i);
            }
        }

        let mut item = Item {
            recipe: vec![],
            materials: vec![],
            fn_call_path: name.to_string(),
            function: MCFunction::new(name.to_string(), join!["item_", &*name], 0, &ns),
            file_name: name.to_string(),
            cache: Magnet::Attached(o_cache.pull_data()),
            cached_type,
        };

        if item.cached_type == Unchanged {
            let frag = CachedFrag::from_path("_EXTERN", &item.cache);
            item.cache.extern_frag.attach(frag);
            return item;
        }

        let mut ln = 0;
        while ln < lines.len() {
            let rem = item.parse_line(ln, lines, ns);
            ln += rem;
        }

        let frag = CachedFrag::new(join![&*item.cache.file_path, "/_EXTERN"]);
        item.function.fragment.attach(frag.clone());
        item.cache.extern_frag.attach(frag);

        item.function.compile();

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
                self.function.file_path = "items".to_string();
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

                if let Some(ref mut node) = nna.node.value() {
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

    pub fn compile(&mut self) {
        if self.cached_type != Unchanged {
            self.function.compile();
        }
    }

    pub fn save(&mut self) { // get cache.frag.files and cache all ready
        if self.cached_type != Unchanged {
            self.function.update_save_files();
            self.cache.extern_frag.attach(self.function.fragment.unattach());
            for file in self.cache.extern_frag.files.iter_mut() {
                file.0.push_str(".mcfunction");
            }

            let mut write_recipe = RECIPE_TEMPLATE.to_string().replace("$PATTERN$", &*self.recipe.join(",\n    "));
            let mut mats = vec![];
            for mat in &self.materials {
                if mat.1.starts_with("#") {
                    mats.push(MAT_TAG_TEMPLATE.to_string().replace("$ID$", &*mat.0).replace("$TYPE$", &mat.1[1..]));
                } else {
                    mats.push(MAT_TEMPLATE.to_string().replace("$ID$", &*mat.0).replace("$TYPE$", &*mat.1));
                }
            }
            write_recipe = write_recipe.replace("$MATERIALS$", &*mats.join(",\n    "));
            self.cache.extern_frag.files.push((join![&*self.function.ns_id, "/recipes/", &*self.fn_call_path, ".json"], vec![write_recipe]));

            let mut write_adv = qc!(self.function.meta.version.as_ref().is_ok_and(|v| v < &14) , ADV_CRAFT_TEMPLATE_119, ADV_CRAFT_TEMPLATE_120).to_string();
            write_adv = write_adv.replace("$PATH$", &*join![&*self.function.ns_id, ":", &*self.fn_call_path])
                .replace("$CALL$", &*join![&*self.function.ns_id, ":", &*self.fn_call_path]);
            self.cache.extern_frag.files.push((join![&*self.function.ns_id, "/advancements/", &*self.fn_call_path, ".json"], vec![write_adv]));
        }
    }
}