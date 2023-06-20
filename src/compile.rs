// bob the builder??? in this filesystem???

use crate::{Blocker, error, format_out, join, MCFunction, MCValue, Node, NodeType, qc};
use crate::compile::JSON::*;
use crate::server::*;

pub fn node_text(node: &mut Node, mcf: &mut MCFunction) {
    use NodeType::*;
    if node.lines.len() == 0 {
        return;
    }
    match &node.node {
        Command => {
            let keys = Blocker::new().split_in_same_level(" ", &node.lines[0]);
            if let Err(e) = keys {
                error(format_out(&*e, &*mcf.get_file_loc(), node.ln));
                return;
            }
            let mut keys = keys.unwrap();
            replace_local_tags(&mut keys, mcf);
            match &*keys[0] {
                "cmd" => {
                    node.lines = vec![keys.get(1..).unwrap_or(&[String::new()]).join(" ")];
                }
                "execute" => {
                    node_execute(node, &mut keys, mcf);
                }
                "create" => {
                    if keys.len() == 2 {
                        keys.push("dummy".into());
                    }
                    if keys[1].starts_with("&") {
                        keys[1].replace_range(0..1, ".");
                        keys[1].replace_range(0..0, &*mcf.ns_id);
                    }
                    node.lines[0] = join!["scoreboard objectives add ", &*keys[1..].join(" ")];
                }
                "remove" => {
                    if keys[1].starts_with("&") {
                        keys[1].replace_range(0..1, ".");
                        keys[1].replace_range(0..0, &*mcf.ns_id);
                    }
                    node.lines[0] = join!["scoreboard objectives remove ", &*keys[1]];
                }
                "rmm" if require::remgine("rmm", mcf, node.ln) => {
                    let cmd = "function remgine:utils/rmm".to_string();
                    if keys.len() == 1 {
                        return {
                            node.lines = vec![cmd];
                        };
                    }
                    return if keys[1].eq("set") && require::exact_args(4, &keys, mcf, node.ln) {
                        let power = keys[3].parse::<i8>().unwrap_or(0);
                        node.lines = vec![join!["scoreboard players set ", &*keys[2], " remgine.rmm ", &*power.to_string()]];
                    } else {
                        let power = keys[1].parse::<i8>().unwrap_or(0);
                        node.lines = vec![join!["scoreboard players set @s remgine.rmm ", &*power.to_string()], cmd];
                    };
                }
                "while" => {
                    if let Ok(conds) = Blocker::new().split_in_same_level(" && ", &keys[1][1..keys[1].len() - 1].to_string()) {
                        let mut call_cond = conds.into_iter().map(|cond| node_condition(node, cond, mcf))
                            .enumerate().map(|(_, (ccon, isif))| -> String {
                            join![qc!(isif, "if ", "unless "), &*ccon]
                        }).collect::<Vec<_>>().join(" ");
                        call_cond = join!["execute ", &*call_cond, " run"];
                        let nna = &mut node.children[0];
                        let path = join![&*mcf.get_path(), ".w", &*nna.ln.to_string()];
                        let path = path.strip_prefix("/").unwrap_or(&*path);
                        nna.lines.push(join!(&*call_cond, " function ", &*mcf.ns_id, ":", &*path));
                        node.lines = vec![call_cond];
                    } else {
                        error(format_out("Failed to parse while statement", &*mcf.get_file_loc(), node.ln));
                    }
                }
                "for" => {
                    let keys = keys[1][1..keys[1].len() - 1].split(",").map(|s| s.trim().to_string()).collect::<Vec<_>>();
                    if require::min_args_named(2, "for loop", &keys, mcf, node.ln) {
                        let target = MCValue::new(&keys[0], mcf, node.ln);
                        let mut stop = MCValue::new(&keys[1], mcf, node.ln);
                        let mut start = MCValue::Number { value: 0 };
                        if keys.len() == 3 {
                            start = stop;
                            stop = MCValue::new(&keys[2], mcf, node.ln);
                        }
                        node.lines.insert(0, target.set_equal_to(&start, mcf, node.ln));
                        let call_cond = join!["execute ", &*target.get_less_than(&stop, mcf, node.ln), " run"];
                        let nna = &mut node.children[0];
                        let path = join![&*mcf.get_path(), ".f", &*nna.ln.to_string()];
                        let path = path.strip_prefix("/").unwrap_or(&*path);
                        nna.lines.push(join!["scoreboard players add ", &*target.get(), " 1"]);
                        nna.lines.push(join!(&*call_cond, " function ", &*mcf.ns_id, ":", &*path));
                        node.lines[1] = call_cond;
                    }
                }
                f @ _ => {
                    if !f.is_empty() && !COMMANDS.contains(&f) {
                        warn(format_out(&*join!["Unknown command '", f, "'"], &*mcf.get_file_loc(), node.ln))
                    } else {
                        node.lines[0] = keys.join(" ");
                    }
                }
            }
        }
        Scoreboard => {
            let keys = Blocker::new().split_in_same_level(" ", &node.lines[0]);
            if let Err(e) = keys {
                error(format_out(&*e, &*mcf.get_file_loc(), node.ln));
                return;
            }
            let keys = keys.unwrap();
            node.lines = MCFunction::compile_score_command(&keys, mcf, node.ln);
        }
        FnCall(path) => {
            unsafe {
                if !KNOWN_FUNCTIONS.contains(path) {
                    warn(format_out(&*join!["Unknown function '", &*path.form_foreground(str::ORN), "'"], &*mcf.get_file_loc(), node.ln));
                }
            }
        }
        None => {
            node.lines = vec![];
        }
        _ => {}
    }
}

pub fn node_execute(node: &mut Node, keys: &mut Vec<String>, mcf: &mut MCFunction) {
    for i in 0.. {
        if i >= keys.len() {
            break;
        }
        match &*keys[i] {
            "ast" if require::min_args(2, &keys[i..].to_vec(), mcf, node.ln) => {
                keys[i] = "as".into();
                keys[i + 1].push_str(" at @s");
            }
            "if" if keys[i + 1].starts_with("(") || keys[i + 1].starts_with("!(") => {
                let inverse = keys[i + 1].starts_with("!(");
                let ilifs = keys.remove(i + 1);
                let ilifs = ilifs[(1 + inverse as usize)..ilifs.len() - 1].to_string();
                if let Ok(conds) = Blocker::new().split_in_same_level(" && ", &ilifs) {
                    keys[i] = conds.into_iter().map(|cond| node_condition(node, cond, mcf))
                        .enumerate().map(|(_, (ccon, isif))| -> String {
                        join![qc!(isif != inverse, "if ", "unless "), &*ccon]
                    }).collect::<Vec<_>>().join(" ");
                } else {
                    error(format_out("Failed to parse if statement", &*mcf.get_file_loc(), node.ln));
                }
            }
            _ => {}
        }
    }
    node.lines[0] = keys.join(" ");
}

pub fn node_condition(node: &mut Node, mut cond: String, mcf: &mut MCFunction) -> (String, bool) {
    let isif = !cond.starts_with('!');
    qc!(!isif, cond.remove(0), 'w');
    let keys = Blocker::new().split_in_same_level(" ", &cond);
    if let Err(e) = keys {
        error(format_out(&*e, &*mcf.get_file_loc(), node.ln));
        return (cond, isif);
    }
    let mut keys = keys.unwrap();
    replace_local_tags(&mut keys, mcf);
    require::min_args(1, &keys, mcf, node.ln);
    match &*keys[0] {
        "random" if require::remgine("random", mcf, node.ln) &&
            require::min_args(2, &keys, mcf, node.ln) => {
            cond = join!["predicate remgine:random/", &*keys[1]];
        }
        _ if MCFunction::is_score_path(&keys[0], mcf, node.ln) &&
            require::exact_args(3, &keys, mcf, node.ln) => {
            let target = MCFunction::compile_score_path(&keys[0], mcf, node.ln).join(" ");
            if keys[2].contains("..") {
                require::keyword("=", &keys[1], mcf, node.ln);
                cond = join!["score ", &*target, " matches ", &*keys[2]];
                return (cond, isif);
            }
            let target2 = MCValue::new(&keys[2], mcf, node.ln);
            match &*keys[1] {
                ">=" | ">" | "=" | "<" | "<=" if !target2.is_number() => {
                    cond = join!["score ", &*target, " ", &*keys[1], " ", &*target2.get()];
                }
                ">=" | ">" | "<" | "<=" => {
                    let gt = keys[1].contains(">");
                    let eq = keys[1].contains("=");
                    if let MCValue::Number { mut value } = target2 {
                        value += qc!(!eq, (gt as i32 * 2) - 1, 0);
                        cond = join!["score ", &*target, " matches ", qc!(!gt, "..", ""), &*value.to_string(), qc!(gt, "..", "")];
                    }
                }
                "=" => {
                    cond = join!["score ", &*target, " matches ", &*target2.get()];
                }
                _ => {
                    error(format_out(
                        &*join!("Failed to parse score test, unknown operation '", &*keys[1].form_foreground(str::BLU), "'"),
                        &*mcf.get_file_loc(), node.ln));
                }
            }
        }
        _ => {
            cond = keys.join(" ")
        }
    }
    (cond, isif)
}

pub fn is_fn_call(call: &str, mcf: &mut MCFunction) -> Option<String> {
    let mut call = call.clone();
    if call.len() < 2 { return None; };
    let local = call.starts_with("&");
    let tag = call.starts_with("#");
    call = call[(local as usize + tag as usize)..].into();
    return if MCFunction::is_valid_fn(&*call) {
        call = call.trim_end_matches("()");
        let (ns, name) = call.split_once(":").unwrap_or((&*mcf.ns_id, call));
        let path = qc!(local, path_without_functions(mcf.file_path.clone()), "".into());
        Some(join![qc!(tag, "#", ""), ns, ":", &*path, qc!(local && path.len() > 0, "/", ""), name])
    } else {
        None
    };
}

pub fn replacements(text: &String, node: &Node, mcf: &mut MCFunction, ln: usize) -> String {
    let mut text = text.replace("*{NS}", &*mcf.ns_id)
        .replace("*{NAME}", &*mcf.meta.view_name)
        .replace("*{INT_MAX}", "2147483647")
        .replace("*{INT_MIN}", "-2147483648")
        .replace("*{PATH}", &*mcf.get_file_loc())
        .replace("*{NEAR1}", "limit=1,sort=nearest")
        .replace("*{LN}", &*(node.ln + ln).to_string());
    parse_json_all(&mut text, mcf, node.ln + ln);
    text
}

fn parse_json_all(text: &mut String, mcf: &mut MCFunction, ln: usize) {
    if text.starts_with("//") {
        return;
    }
    let mut pos = text.match_indices("*JSON{").map(|s| s.0).collect::<Vec<_>>();
    pos.reverse();
    for p in pos {
        if let Ok(out) = Blocker::new().find_size(text, p + 5) {
            let mut input = Blocker::new().split_in_same_level(":", &text[(p + 6)..out - 1].to_string())
                .unwrap_or(vec!["text".into(), "".into(), "\"\"".into()]);
            input.insert(0, "*JSON".into());
            if !require::min_args(4, &input, mcf, ln) {
                return;
            }
            input.remove(0);
            let mut data = JSONData::new();
            let options = input[0].split(" ").collect::<Vec<_>>();
            let json = options.first().unwrap_or(&"text").clone();
            for (idx, mut opt) in options.into_iter().enumerate() {
                let mut set = true;
                if opt.starts_with("!") {
                    set = false;
                    opt = &opt[1..];
                }
                match opt {
                    "italic" => data.italic = Some(set),
                    "bold" => data.bold = Some(set),
                    "strike" | "strikethrough" => data.strike = Some(set),
                    "underline" | "underlined" => data.underline = Some(set),
                    "obfuscated" | "mystify" => data.obfuscated = Some(set),
                    _ if idx != 0 && data.color.is_none() && !opt.eq("") => data.color = Some(opt.to_string()),
                    _ => {}
                }
            }
            let mut events = Blocker::new().split_in_same_level(" ", &input[1]).unwrap_or(vec![]).into_iter();
            while let Some(event) = events.next() {
                match &*event {
                    "hover" => {
                        let style = events.next().unwrap_or("show_text".into()).to_string();
                        let content = events.next().unwrap_or(r#"{"text":""}"#.into()).to_string();
                        data.event_hover = Some((style, content));
                    }
                    "click" => {
                        let style = events.next().unwrap_or("suggest_command".into()).to_string();
                        let content = events.next().unwrap_or("/".into()).to_string();
                        data.event_click = Some((style, content));
                    }
                    "" => {}
                    _ => {
                        warn(format_out(&*join!["Error parsing JSON, unknown event: '", &*event, "'"], &*mcf.get_file_loc(), ln));
                    }
                }
            }
            let json = match json {
                "score" => { Score(data, MCFunction::compile_score_path(&input[2..].join(":").trim().into(), mcf, ln)) }
                "custom" => { Custom(data, input[2..].join(":").to_string()) }
                "nbt" => { NBT(data, input[2..].join(":").trim().to_string()) }
                _ => { Text(data, input[2..].join(":").trim().to_string()) }
            };
            let json = json.to_string();
            text.replace_range(p..out, &*json);
        }
    }
}

struct JSONData {
    italic: Option<bool>,
    bold: Option<bool>,
    strike: Option<bool>,
    underline: Option<bool>,
    obfuscated: Option<bool>,
    color: Option<String>,

    event_hover: Option<(String, String)>,
    event_click: Option<(String, String)>,
}

impl JSONData {
    fn new() -> JSONData {
        JSONData {
            italic: None,
            bold: None,
            strike: None,
            underline: None,
            obfuscated: None,
            color: None,
            event_hover: None,
            event_click: None,
        }
    }

    fn append_data<'a>(&self, json: &'a mut String) -> &'a mut String {
        if let Some(b) = self.italic { json.push_str(&*join![r#","italic":""#, &*b.to_string(), "\""]); }
        if let Some(b) = self.bold { json.push_str(&*join![r#","bold":""#, &*b.to_string(), "\""]); }
        if let Some(b) = self.strike { json.push_str(&*join![r#","strikethrough":""#, &*b.to_string(), "\""]); }
        if let Some(b) = self.underline { json.push_str(&*join![r#","underlined":""#, &*b.to_string(), "\""]); }
        if let Some(b) = self.obfuscated { json.push_str(&*join![r#","obfuscated":""#, &*b.to_string(), "\""]); }
        if let Some(b) = self.color.clone() { json.push_str(&*join![r#","color":""#, &*b, "\""]); }
        if let Some((t, d)) = &self.event_hover { json.push_str(&*join![r#","hoverEvent":{"action":""#, &*t, r#"","contents":"#, &*d, r#"}"#]); }
        if let Some((t, d)) = &self.event_click { json.push_str(&*join![r#","clickEvent":{"action":""#, &*t, r#"","value":"#, &*d, r#"}"#]); }
        json
    }
}

enum JSON {
    Text(JSONData, String),
    Score(JSONData, [String; 2]),
    Custom(JSONData, String),
    NBT(JSONData, String),
}

impl JSON {
    fn to_string(&self) -> String {
        match self {
            Text(data, text) => {
                let mut json = join![r#"{"text":"#, &*text];
                data.append_data(&mut json).push_str("}");
                json
            }
            Score(data, path) => {
                let mut json = join![r#"{"score":{"name":""#, &*path[0], r#"","objective":""#, &*path[1], "\"}"];
                data.append_data(&mut json).push_str("}");
                json
            }
            Custom(data, text) => {
                let mut json = join!["{", &*text];
                data.append_data(&mut json).push_str("}");
                json
            }
            NBT(data, path) => {
                let (typ, path) = path.split_once(" ").unwrap_or(("entity", "_ : _"));
                let (target, path) = path.split_once(" : ").unwrap_or(("_", "_"));
                let mut json = join![r#"{"nbt":""#, path, r#"",""#, typ, r#"":""#, target, "\""];
                data.append_data(&mut json).push_str("}");
                json
            }
        }
    }
}

pub fn finish_lines(lines: &mut Vec<String>, mcf: &mut MCFunction) {
    lines.iter_mut().for_each(|line| {
        *line = line.replace("*{SB}", "§");
        qc!(mcf.meta.opt_level >= 1, optimize_line(line), ());
    });
}

pub fn optimize_line(line: &mut String) {
    *line = line.replace(" positioned as @s ", " positioned as @s[] ")
        .replace(" as @s ", " ")
        .replace(" @s[] ", " @s ")
        .replace(" run execute ", " ")
        .replace("execute run ", "")
        .replace(" run run ", " run ")
        .replace("execute execute ", "execute ");
}

pub fn replace_local_tags(keys: &mut Vec<String>, mcf: &mut MCFunction) {
    keys.iter_mut().for_each(|key| {
        if key.len() > 8 && key.starts_with('@') && key.as_bytes()[2] == '[' as u8 && key.ends_with(']') && key.contains("tag") {
            let b = key[3..key.len() - 1].to_string();
            if let Ok(options) = Blocker::new().split_in_same_level(",", &b) {
                let ops = options.into_iter().map(|o| -> String {
                    let mut t = o.clone();
                    t.retain(|c| !c.is_whitespace());
                    if t.starts_with("tag=") && t.contains("&") {
                        o.replace("r&", "remgine.").replace("&", &*join!(&*mcf.ns_id, "."))
                    } else {
                        o
                    }
                }).collect::<Vec<String>>();
                *key = join![&key[0..3], &*ops.join(","), "]"];
            }
        }
        if key.len() > 9 && key.starts_with('{') && key.ends_with('}') && key.contains("Tags:[") {
            let b = key[1..key.len() - 1].to_string();
            if let Ok(options) = Blocker::new().split_in_same_level(",", &b) {
                let ops = options.into_iter().map(|o| -> String {
                    if o.starts_with("Tags:[") {
                        if let Ok(mut tags) = Blocker::new().split_in_same_level(",", &o[6..o.len() - 1].to_string()) {
                            tags.iter_mut().for_each(|t|
                                *t = t.replace("r&", "remgine.").replace("&", &*join![&*mcf.ns_id, "."]));
                            join!["Tags:[", &*tags.join(","), "]"]
                        } else { o }
                    } else { o }
                }).collect::<Vec<String>>();
                *key = join!["{", &*ops.join(","), "}"];
            }
        }
    });
}

pub mod require {
    use std::fmt::Display;
    use crate::{error, format_out, join, MCFunction};
    use crate::server::FancyText;

    pub fn min_args<T: Display>(count: usize, keys: &Vec<T>, mcf: &mut MCFunction, ln: usize) -> bool {
        if keys.len() < count {
            error(format_out(&*format!("Not enough arguments for '{}' ({} expected, found {})", keys[0], count, keys.len()), &*mcf.get_file_loc(), ln))
        }
        keys.len() >= count
    }

    pub fn min_args_named<T: Display>(count: usize, name: &str, keys: &Vec<T>, mcf: &mut MCFunction, ln: usize) -> bool {
        if keys.len() < count {
            error(format_out(&*format!("Not enough arguments for '{}' ({} expected, found {})", name, count, keys.len()), &*mcf.get_file_loc(), ln))
        }
        keys.len() >= count
    }

    pub fn min_args_path<T: Display>(count: usize, keys: &Vec<T>, path: String, ln: usize) -> bool {
        if keys.len() < count {
            error(format_out(&*format!("Not enough arguments for '{}' ({} expected, found {})", keys[0], count, keys.len()), &*path, ln))
        }
        keys.len() >= count
    }

    pub fn exact_args<T: Display>(count: usize, keys: &Vec<T>, mcf: &mut MCFunction, ln: usize) -> bool {
        if keys.len() != count {
            error(format_out(&*format!("Wrong number of arguments for '{}' ({} expected, found {})", keys[0], count, keys.len()), &*mcf.get_file_loc(), ln))
        }
        keys.len() == count
    }

    pub fn remgine(item: &str, mcf: &mut MCFunction, ln: usize) -> bool {
        if !mcf.meta.remgine {
            error(format_out(&*join!("Remgine is required to use [", &*item.form_foreground(str::AQU), "]"), &*mcf.get_file_loc(), ln))
        }
        mcf.meta.remgine
    }

    pub fn keyword(word: &str, test: &String, mcf: &mut MCFunction, ln: usize) -> bool {
        if !test.eq(word) {
            error(format_out(&*format!("Expected keyword '{}' got {}", word, test), &*mcf.get_file_loc(), ln))
        }
        test.eq(word)
    }
}

