<h1>Mitsuko</h1>
<h6>Version 0.1.0 - For 1.19 - By RemRemEgg</h6>

---

# Overview

Mitsuko is a custom language designed to make programming in minecraft easier and faster. It is heavily based around the
vanilla commands, and it not a replacement for them. It simply adds ease of use and more functionality.

> Notice:
>
> Items prefixed with a '»' are *going* to be added, but are not currently functioning. Items prefaced with a '«' are going to be removed in a future version.

## TOC

### [Compiler Options](#compiler-options)

### [Files, Folders, Meta](#files-folders-meta)

* [Folder Structure](#folder-structure)
* [Folder Structure / pack.msk](#packmsk)
* [Namespaces](#namespaces)
* [Tags](#tags)
* [Functions](#function-files)
* [Items](#item-files)
* [Event Links](#event-link-files)
* [Extras Folder](#extras-folder)

### [Function Examples & Syntax](#function-examples--syntax)

* [Function Definitions](#function-definitions)
* [Comments](#comments)
* [Modifiers](#modifiers)
* [Function Code](#function-code)
* [Function Code / Execute](#execute)
* [Function Code / Calling Functions](#calling-functions)
* [Function Code / Scoreboards](#scoreboards--short-scores)
* [Function Code / Inline Blocks](#inline-tags)
* [Function Code / Repeat](#repeat)
* [Function Code / Set & Retrieve](#set--retrievals)

### [Event Links](#event-links)

* [Methodology](#methodology)
* [File Syntax](#file-syntax)

### [Item files](#items)

* [Recipe](#recipe)
* [Materials](#materials)
* [Path](#path)
* [Item](#item)

### [Other](#other)

* [Remgine Exclusive](#remgine-exclusive--requires-remgine)
* [Built-in Conditionals](#built-in-conditionals)
* [Built-in Conditionals / Score Testing](#score-testing)
* [Built-in Conditionals / Random](#random)
* [Quick JSON](#quick-json)
* ['&' and 'r&' Replacements](#-and-r-replacements)
* [Optimizations](#optimizations)
* [Danger Zone](#danger-zone)
* [Danger Zone / Suppress Warnings](#suppress-warnings)
* [Danger Zone / @ Commands](#-commands)

---

# Compiler Options

```
mitsuko.exe <project_location> [flags]
```

Flags:

* (--move | -m) <<l>location><br> Moves the finished datapack to "\<location>/datapacks/<pack_name>"
* (--clear | -c)<br> Removes the previous datapack at "\<location>/datapacks/<pack_name>" (requires move)
* (--export | -e)<br> Exports the datapack's functions to <pack_location>/<pack_name>.export.msk
* (--help | -h)<br> Prints the help message

---

# Files, Folders, Meta

## Folder Structure

```jsonpath
./
├───<name>.export.msk
├───/generated
│   └───/<name>
│       └───** compiler generated **
└───/src
    ├───pack.msk
    └───/<namespace>
        ├───»/advancements
        │   └───[folders]
        ├───/event_links
        │   └───[link_name].msk
        ├───/items
        │   └───[item_name].msk
        ├───/extras
        └───/functions
            ├───functions.msk
            ├───[file_name].msk
            └───[/sub-folders & .msk files]
```

### pack.msk

Used for overarching metadata of the datapack as well as `pack.mcmeta`. Placed in directly in the ./src/ folder of the
project, the program will error out if no file is found. This file contained a list of tags & imports for the program to
use. See "Tags" for a full list of usable tags. Example of `pack.msk`:

```
use External Datapack

name = Mitsuko Example
comments = true
version = 15
description = §3Example pack for §bmitsuko
```

#### Importing other datapacks through 'Use'

Other datapacks can be imported by putting its export file in the `imports` folder, and then adding the
line `use <name>` to pack.msk. Unlike normal programming languages, this will not add the other datapack into the
existing one, instead it will allow Mitsuko to recognise its functions. Export files are produced when the export flag
is added, and are placed in the root folder and named `<datapack_name>.export.msk`.
See [Compiler Options](#compiler-options)

## Namespaces

Namespaces are any combination of underscores, lowercase letters, and numbers. Namespaces can be inserted with
the `*{NS}` retrieval. This retrieval is locally overrideable. Attempting to use an invalid namespace will error out.

Examples of valid namespaces:

* trees
* example_ns
* v
* _alpha2
* 135

Examples of invalid namespaces:

* Trees
* example ns
* Simon F-ing Cowell
* ♂

## Tags

Tags are special functions that tell the compiler how to handle code. The normal format for tags
is `#[<name> = <value>]`.
`pack.msk` uses a different format, being `<name> = <value>`.

### Global Tags

* comments (bool) : Should comments & whitespace be transferred, default false
* debug (0..2) : Debug verbosity, default 0
* optimizations (0..?) : How far to optimize code, default 0
* recursive_replace (0..255) : How any recursions to go for internal values, default 3

### pack.msk only

* description (String) : Description field (supports §), default "A Datapack"
* name (String) : Datapack's name, default "Untitled"
* remgine (bool) : Is remgine enabled, default true
* version (int) : Datapack's Version, default is the latest release version

## Function Files

Function files get compiled into `.mcfunction`s. They are located in the `./src/<namespace>/functions/`
folder. [Syntax & Code](#function-examples--syntax)

## Item Files

Item files get compiled into an advancement, recipe, and function. These are used to pull off NBT crafting without mush
hassle. They are located in the `./src/<namespace>/items/` folder.

## Event Link Files

Event Link files get compiled into function tags. These can be used to link events from multiple datapacks together.
They are located in the `./src/<namespace>/event_links/` folder.

## »Advancement Files

N/A

## Extras Folder

This is a folder that gets copied directly into `./generated/<name>/data/~`. This can be used for files that Mitsuko
does not support, like world gen.

---

# Function Examples & Syntax

Function files contain three things: functions, comments, and modifiers.

## Function Definitions

Functions can be defined by `fn <name>() {` and the `<name>` must be a valid function path. Example of function names:

```
fn first() {}

fn __main() {}

fn 196() {}

fn sub/internal() {}
```

Notice how the `internal` function is prefaced by a `sub/`. This will cause the resulting file to
be `data/~/functions/sub/internal.mcfunction`. Functions can have any number of folders before them, but cannot be
prefaced with a different namespace.

Function paths are also dependent on the file they are located at. Any functions located within the
file `./src/~/functions/file.msk` will be at the path `example:file/<name>`. This can be combined with sub-folders and
function names to easily organize functions. A file `./src/~/functions/folder/file.msk` containing the function
declaration `fn internal/start() {}` will have the resulting path `example:folder/file/internal/start`.

If the file a function is in is called `functions.msk`, then this last path part will be ignored. Functions in the
file `./src/~/functions/folder/functions.msk` will be located at `folder/`, not `folder/functions/`. Functions placed in
the master `./src/~/functions/functions.msk` file will have no extra path.

``` 
fn sqrt($x) {}
```

Unlike most programming languages, these functions cannot accept arguments. This function will fail to compile.

## Comments

Comments can be put both inside and outside of functions but cannot be put after a command. A comment is two forward
slashes followed by any text.
```// This is a comment!```

## Modifiers

Modifiers are single lines of code that change either a single function or all functions in a file. Modifiers are an `@`
followed by data specifying the modification.

#### @alias

The `alias` modifier can be used to create an alias for a given function.

```mitsuko
// This function can be called with 'function example:short_name'
@alias example:short_name
fn really_long_function_name_that_i_dont_like() {}

// This function can be called with 'function useful_fn'
@alias
fn buried/in/folders/useful_fn() {}
```

If a name is provided, the alias will match the given namespace and name. If no namespace is given, it will default
to `minecraft`. If no name is given, the alias will be the function's name at `minecraft:`. Aliases do not override the
function, only create a new one that is linked.

#### @meta

The `meta` can can be used to temporarily change the options in `pack.mcmeta`. See [tags](#tags) for which tags can and
cannot be used in this context. Tags can only change the single function that comes after it.

Example:

``` 
fn func1() {
    // This comment is dependent on the settings in pack.msk
}

@meta comments false
fn func2() {
    // This comment will not be saved
}

fn func3() {
    // This comment is dependent on the settings in pack.msk
}
```

#### @no_export

Prevents a function from getting exported.

```
@no_export
fn private_fn() {}

fn exported_fn() {}
```

#### @set

The `@set` modifier works just like a `set` command; It will set a specific variable to a value for all succeeding
functions in a file.

Example:

```
@set AUTHOR RemRemEgg
@set SIZE 10
fn init() {
    tellraw @a *JSON{:: "This was made by *{AUTHOR}"}
}

fn tree() {
    ast @e[distance=..*{SIZE}] ast @e[distance=..*{SIZE}] ast @e[distance=..*{SIZE}] run tree_2()
}
```

## Function Code

Just like regular MCFunction, commands are put one line after another. However, Mitsuko adds some custom commands and
macros to help with writing code.

### Execute

Mitsuko provides a variety of improvements to the execute command, as well as commands that get compiled into advanced
execute commands.

#### exe

`exe` is a shorthand for `execute`. `exe ... run` => `execute ... run`

#### ast

`ast` means as <<x>selector> at @s. `ast @p run` => `execute as @p at @s run`.

`ast` can also be used inside execute commands;
`execute positioned ~ ~10 ~ ast @e[distance=..1] run` => `execute positioned ~ ~10 ~ as @e[distance=..1] at @s run`

#### if

`if` works like an `if` statement in normal programming, removing the need for `execute ... run`
. `if (predicate example:die) kill @s` => `execute if predicate example:die run kill @s`.
`if`s and their internal statements can be negated using an `!`. `if (!condition)` => `execute unless condition`.

Multiple conditions can be chained together using ` && `. `if (con1 && con2 && !con3)`
=> `execute if con1 if con2 unless con3`. Placing an `!` right before the parenthesis will invert the polarity of all
contained commands; `if !(con1 && con2 && !con3)` => `execute unless con1 unless con2 if con3`.

`if` can also be used inside execute commands. `execute positioned ~ ~10 ~ if (con1 && !con2)`
=> `execute positioned ~ ~10 ~ if con1 unless con2`.
`if` statements outside of an execute command will automatically put a `run` at the end of the conditions. This means
that chaining an `if` into an `execute` will be `if (cond) execute`. However, this can be optimized away by the
compiler. See [Optimizations](#optimizations).

### Calling Functions

Single word statements that end with `()` will be interpreted as a function call. `example:subfolder/main()`
=> `function example:subfolder/main`. Functions that start with a `#` will be turned into function tag calls. If a
function call does not start with a namespace, the current one will be added; `init()` => `function example:init`.

The current directory can be substituted with an `&`. If we are in the file `functions/outer/inner.msk` and we
call `&local()`, the result will be `function example:outer/inner/local()`. Calling just `local()` from here would
attempt to call the function `example:local`. See ['&' and 'r&' Replacements](#in-functions)

### Scoreboards & Short Scores

#### Short scores

Mitsuko introduces a new way to represent scoreboards: `<name>:<board>`. When dealing with scores, they can be
referenced through this syntax. For instance, to access the current entity's "health" score, you would use `@s:math`.
Local and Remgine boards can be accessed through local replacements. See [Local replacements & Scores](#in-scoreboards).

#### Temp Scores

[*Remgine Exclusive*] Temp scores are all stored in `remgine.temp` and are access through `$<name>`. For instance, to
access the temporary score "math" you would use `$math`.

#### Score Operations

Mitsuko also allows us to easily manipulate scores using this, by doing `<short_score> [operation] [value]`. In this
situation, `[value]` can be either a number or another short score.

Operations:

* ++ : Adds 1 to the score
* -- : Subtracts 1 from the score
* get : Gets the score (default if no operation is provided)
* reset : Resets the score
* enable : Enables the score
* = [value] : Sets the score to the given value
* += [value] : Adds the given value
* -= [value] : Subtracts the given value
* %= [value] : Modulo by the given value
* /= [value] : Divide by the given value
* *= [value] : Multiply by the given value
* \>< [value] : Swap the values
* < [value] : Min the values
* \> [value] : Max the values

*Note: Using [%=, /=, *=, ><, <, >] when `[value]` is a number requires remgine, and is suboptimal.*

#### Create / Remove

Scoreboard can be easily created & removed using two simple commands. `create <name> [type]` will create a new
scoreboard with `<name>` and `[type]`, or dummy if no type is provided. `remove <name>` removes the scoreboard. Names
can use local replacements. See [Local replacements & Scores](#in-scoreboards)

### Inline Blocks

Mitsuko add the ability to inline blocks of code.

```
fn test() {
    weather clear
    {
        kill @e
        say lmao get flexed on with Mitsuko
    }
    give @a diamond 64
}
```

This will compile two functions, the first one will be `main` and the second one will be an internal function. `main`
will then call the internal function. This is not the most helpful thing by itself, but it's quite powerful with
combined with other commands.

```
fn test() {
    weather clear
    if (predicate example:kill_everything) {
        kill @e
        say lmao get flexed on with Mitsuko
    }
    give @a diamond 64
}
```

Here we are now able to run this inline block only when the condition is met. This is helpful for having multiple
functions in the same block.

```
fn test() {
    tag @e add &temp
    ast @e[tag=&temp] {
        tag @s remove &temp
        some_other_function()
    }
    tag @e remove &temp
}
```

This allows us to call a function on `@[tag=&temp]` without having to define a separate function to call.

### For Loop

For loops follow one of two formats; `for (<score>, <stop>) {` or `for (<score>, <start>, <stop>) {`. If no `<start>` is
specified, it starts at 0; For loops go until they reach the end value (value < stop). Both `<start>` and `<stop>` can
be scoreboards or values.

```
fn test() {
    for ($i, 5) {
        tellraw @s *JSON{score :: $i}
    }
    for (iteration:&data, 18, max:&data) {
        tellraw @s *JSON{score :: iteration:&data}
    }
}
```

### While Loop

While loops continuously repeat code till the condition is not met.

```
fn test() {
    while ($temp < 15000 && random 80) {
        $temp *= #8:r&nums
        some_other_fn()
    }
}
```

### Repeat

Copies a block of code a given number of times.

```
fn test() {
    say Starting test
    repeat 1000 {
        ast @e[limit=1,sort=random] run setblock ^ ^ ^2 gold_block
        ast @e[limit=1,sort=random] tp @s ~ ~ ~ ~ ~73
    }
    say Test Done
}
```

This will repeat the two `ast` lines 1000 times.

### Set / Retrievals

The `set` command allows us at set a variable to a value, and retrievals allow us to fetch the value.

```
fn many_effects() {
    // Set "eff" to "effect give @s"
    set eff effect give @s
    // Set do not appear in the compiled code
    
    // Values can be retieved with *{<name>}
    *{eff} slowness
    *{eff} strength
    *{eff} jump_boost
    *{eff} resistance
}
```

Mitsuko comes with built-in retrieves:

| Retrieval  |        Value         |
|------------|:--------------------:|
| *{NS}      |    <<x>namespace>    |
| *{NAME}    |  <<x>datapack name>  |
| *{INT_MAX} |      2147483647      |
| *{INT_MIN} |     -2147483648      |
| *{PATH}    |    <<x>file path>    |
| *{NEAR1}   | limit=1,sort=nearest |
| *{SB}      |          §           |
| *{LN}      |   <<x>line number>   |

---

# Event Links

Event links allow you to quickly make function tags. This is mainly used for linking events from different datapacks
together.

## Methodology

One datapack, for instance Remgine, can create some blank event links for other datapacks to latch onto. One such link
it creates is `load : none`, meaning Remgine has a link called "load" with no functions assigned to it. Another
datapack, with namespace `example`, would then have the file `./src/~/event_links/remgine.msk` with the
link `load : init`. This is binding the `example:init` function to Remgine's `load` event. If the example link
was `load : init, init2, other:function` it would bind the functions `example:init`, `example:init2`,
and `other:function` to Remgine's `load` event.

Since events are just function tags, they can be called by `#<name>()`. If you had a link called `trigger`, you would
call it through `#trigger()`. Function tags can also have external namespaces attached to them.
See [Calling Functions](#calling-functions);

## File Syntax

Each link is stored on a separate line within a link file. Each link file is located in `./src/~/event_links/`. The name
of the file represents the namespace that all the links are pointing to. The links follow this
syntax: `link_name : <none | function,...>`. A file named `remgine.msk` in the `event_links` directory would cause all
its links to point to remgine. If it had a line `load : example:init`, then when remgine called its `load` group, the
current datapack would call `example:init`. Just like with code based function calls, if a function does not have a
namespace then the current one will be assumed with it. If the line was `load : init`, it would try to call `init` in
the current namespace. `none` can be put in place of the functions, in which case the link will be created with no
function calls attached to it.

For a more technical perspective, assume `remgine.msk` containing `load : example:init`. This will create a new
namespace in the current datapack named `remgine`. It will add `load.json` to `./remgine/tags/functions/`, and
the `values` array will have 'example:init' as the only item. Replace will be turned off.

# Items

Item files are an easy and simple way to add custom recipes that utilise NBT crafting. The file is made up of four
parts: recipe, materials, path, item.

## Recipe

The recipe is up to 9 characters representing the shape.

```
recipe {
    "sss"
    "a a"
    "ccc"
}
```

This recipe has 3 of the item on the top. 3 of a different item on the bottom, two different items on the sides, and
nothing in the middle. Recipes can have 1-3 lines, and each line can have 0-3 chars. At least one char needs to be
present. Blanks represent no item.

## Materials

The materials bind each of the chars in the recipe to an item or item group.

```
materials {
    s : stone
    a : #wooden_planks
    c : copper_ingot
}
```

Here we define 's' as stone, 'a' as any wooden plank, and 'c' as copper ingots. Notice that they are not separated by
commas. Each char in the recipe needs to have an item associated with it.

## Path

The path is optional, but represents where the generated files will be placed.

```
path : crafting/smoosher
```

## Item

The item is a Mitsuko function, defining what item will be crafted. The function is run as the player crafting, at
themselves.

```
item {
    give @s stone_block{display:{Name:'*JSON{text #4a2319 bold !italic : The Smoosher}'},Enchantments:[{id:knockback,lvl:10}]}
    particle wax_off ~ ~1 ~ 1 1 1 0 100
    advancment grant @s only example:obtain_smoosher
}
```

Here we are giving the player a stone block with custom nbt, making some particles, and granting an advancement. It is
up to you to `give` or otherwise grant the player the resulting item. The entire block is parsed as a function, so
standard Mitsuko rules apply.

# Other

## Remgine Exclusive / Requires Remgine

Anything that is marked as being remgine exclusive or that says it requires it will not compile if `remgine` is not
enable in pack.msk.

### rmm

Mitsuko allows you to easily call rmm with the `rmm` command. There are three versions of the command: `rmm`
, `rmm <value>`, `rmm set <value>`. `rmm` will just call the `remgine:utils/rmm` function. `rmm set <value>` will set
the current entity's (@s) `remgine.rmm` score to the provided value. `rmm <value>` does both, setting the value then
calling the function.

## Built-in Conditionals

### Score Testing

Two scores can be compared with `<short_score> <operation> <value>`. `<value>` can be either a short score or a number.
Operation can be <=, <, =, >, >=.

### Random

[*Remgine Exclusive*] Random percentages can be used with `random <amount>`. For a full list of the allowed amounts, see
the remgine documentation.

## Quick JSON

JSON objects can be quickly created using `*JSON{<type> <format...>:[events]:<data>}`

`<type>` can be one of four items:

* "text" : `<data>` is a string literal to be displayed (default)
* "score" : `<data>` is a short score. See (Short Scores)[#TODO]
* "nbt" : `<data>` is either `block <x y z> : path`, `entity <selector> : path`, or `storage <name> : path`
* "custom" : Only the format will be applied, `<data>` is appended to the format

`<format...>` is a series of arguments: [italic, bold, strike, underlined, obfuscated]. Putting one of these will enable
it, placing an `!` in front will disable it. Anything else will be interpreted as color.

`<data>` is any data to be used by the JSON, dependent on the type.

`[events]` is an optional filed dedicated to applying `hoverEvent` and `clickEvent`. Format for events
is `<event_id> <event_type> <event_data>`.
`<event_id>` is either "hover" or "click". `<event_type>` is the action to be preformed (show_text, show_item,
show_entity, suggest_command, etc.). `<event_data>` is the data passed to the event.

Examples:

```
*JSON{text aqua underlined :: "I am an underlined, aqua colored piece of text!"}
*JSON{score strike !bold !italic :: @s:&score}
*JSON{nbt bold #58af50 :: block 10 50 12 : Items}
*JSON{text : hover show_text *JSON{score :: @s:&score} : "Hover to see your score!"}
```

Section breaks (§) can be used to apply formatting where Minecraft allows it. However, Quick JSON does not like dealing
with them. Section breaks can be inserted with the `*{SB}` retrieval. Nesting Quick JSON Objects is experimental, the
Lexer gets pretty angry. See [Danger: @NOLEX](#nolex)

## '&' and 'r&' Replacements

In many places, the `&` can be used to reference the datapack itself, or a local part of it.

### In Functions

When calling a function, placing an ampersand in front the of the function's name will cause it to use the path of the
current file. If you are in a file called `players` and you want to reference a function in the same file, you can
use `&other_function()` rather than the full path, `players:other_function()`. Ampersands cannot be used when defining a
function, as the path of the current file is automatically appended. See [Calling Functions](#calling-functions).

### In Tags

Selectors will replace any instance of `tag=&<tag>` with `tag=<namespace>.<tag>`. For instance, `@s[tag=&admin]` in
the `example` namespace will result in `@s[tag=example.admin]`.

The `tag` command is able to add/remove local tags. `tag @s add &cutie` => `tag @s add example.cutie`.

Adding tags to summon commands will be able to replace locally. `summon pig ~ ~ ~ {Tags:[&oinker]}`
=> `summon pig ~ ~ ~ {Tags:[example.oinker]}`

### In Scoreboards

Scoreboard create/remove and operations will replace `&` with the current namespace. `create &temp`
=> `scoreboard objectives add example.temp dummy`. `@s:&temp ++` => `scoreboard players add @s example.temp 1`.
See [Scoreboards](#scoreboards--short-scores)

### 'r&' Replacement

In tags and scoreboards, using `r&` rather than `&` will use `remgine` as the namespace. `@s[tag=&admin]`
=> `@s[tag=example.admin]`, `@s[tag=r&admin]` => `@s[tag=remgine.admin]`.

## Optimizations

Mitsuko supports three levels of optimizations: Off [0], Mild [1], Strong [2], Experimental [3]. The level can be set
through the "optimization_level" tag. See [Tags](#tags)

**Off**: Will not optimize code.

**Mild**: Certain parts of code will get collapsed. 'execute run', 'run execute', and 'as @s' will be removed. In-line
blocks that only contain a single command will have the command inlined.

**Strong**: Optimizes score operations by making static variables. Will inline small functions.

**Experimental**: Dangerous!

---

## Danger Zone

Welcome to the Danger Zone! (*spooky thunder*) In here you will find abilities that are highly experimental and should
not be used unless you know what you're doing.

### Suppress Warnings

Adding `suppress_warnings = true` to `pack.mcmeta` will suppress any further warnings. Remember, the warnings are there
for a reason.

### @ Commands

#### @NOLEX

Prevents the current line from being seen by the lexer from the Mitsuko lexer plugin. Ignored by the compiler.

#### @DEBUG

Prints to the console when this line is hit

#### @DBG_ERROR

Prints to the console when this line is hit and halts execution

#### @TREE

Displays a tree view of the next node
