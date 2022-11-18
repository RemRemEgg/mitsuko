<h1>Mitsuko</h1>
<h6>Version 0.0.0 - For 1.19 - By RemRemEgg</h6>

---
# Overview
*This is in very early development*


Mitsuko is a compiler for Minecraft's commands, from a custom language.
The input language is inspired by JavaScript and Rust, but modified to be able to fit into a data pack easily.
I'm not good at writing compilers, so the code is a bit slow and restrictive (and it would put an italian restaurant to shame).

This is mainly an exercise for me to practice Rust, but I'm also trying to make sure that it can be a decent, working program.
Feel free to open a ticket with any ideas you have for features once I get an actual release out.

> Note:
> 
> Items prefixed with a '»' are *going* to be added, but are not currently functioning. Items prefaced with a '«' are going to be removed in a future version.

---
# Files, Folders, Meta
## Folder Structure
```jsonpath
<root>
├───generated
│   └───** compiler generated **
└───src
    ├───pack.msk
    └───<namespace>
        ├───»advancements
        │   └───[folders]
        ├───event_links
        │   └───[link_name].msk
        ├───extras
        └───functions
            ├───functions.msk
            ├───[file_name].msk
            └───[sub-folders & .msk files]
```


### pack.msk
Used for overarching metadata of the datapack as well as `pack.mcmeta`. 
Placed in directly in the ./src/ folder of the project, the program will error out if no file is found.
See "Tags" for a full list of usable tags.
Example `pack.msk`:

```
name = Mitsuko Example
comments = true
version = 11
description = §3Example pack for §bmitsuko
```
## Namespaces
Namespaces are any combination of underscores, lowercase letters, and numbers. 
Namespaces can be auto-filled in code with a `${NS}` retrieval. This retrieval is locally overrideable. 
Attempting to use an invalid namespace will error out.

Examples of valid namespaces:
* trees
* other_ns
* v
* _alpha2
* 135

Examples of invalid namespaces:
* Trees
* other ns
* guns&roses
* Simon F-ing Cowell
* ♂

## Tags
Tags are special functions that tell the compiler how to handle code. 
The normal format for tags is `#[<name> = <value>]`.
`pack.msk` uses a different format, being `<name> = <value>`.
### List of tags
* comments (bool) : Should comments & whitespace be transferred, default false
* debug (0..2) : Debug verbosity, default 0
* »optimizations (0..?) : How far to optimize code, default 0
* recursive_replace (0..255) : How any recursions to go for internal values, default 3

### pack.msk only
* description (String) : Description field (supports §), default "A Datapack"
* name (String) : Datapack's name, default "Untitled"
* remgine (bool) : Is remgine enabled, default true
* version (int) : Datapack's Version, default is latest release version

## Function Files
Function files are the files that get compiled into `.mcfunction`s. 
They are located in the ./src/<<x>namespace>/

---
# Actual Code