# Utilities

### TOC

* [Execute](#execute)
* [Execute § exe](#exe)
* [Execute § ast](#ast)
* [Execute § if](#if)
* [Calling Functions](#calling-functions)
* [Scoreboards & Short Scores](#scoreboards--short-scores)
* [Scoreboards & Short Scores § Score Operations](#score-operations)
* [Set and Retrievals](#set--retrievals)
* [Built-in Conditionals](#built-in-conditionals)
* [Quick JSON](#quick-json)
* ['&' and 'r&' Replacements](#-and-r-replacements)

## Execute

Mitsuko provides a variety of improvements to the execute command, as well as commands that get compiled into advanced
execute commands.

### exe

`exe` is a shorthand for `execute`. `exe ... run` => `execute ... run`

### ast

`ast <selector>` means `as <selector> at @s`. `ast @p run` => `execute as @p at @s run`.

`ast` can also be used inside execute commands;
`exe positioned ~ ~10 ~ ast @e[distance=..1] run` => `execute positioned ~ ~10 ~ as @e[distance=..1] at @s run`

### if

`if` works like an `if` statement in normal programming, removing the need for `execute ... run`
. `if (predicate example:die) kill @s` => `execute if predicate example:die run kill @s`.
`if`s and their internal statements can be negated using an `!`. `if (!condition)` => `execute unless condition`.

Multiple conditions can be chained together using ` && `. `if (con1 && con2 && !con3)`
=> `execute if con1 if con2 unless con3`. Placing an `!` right before the parenthesis will invert the polarity of all
contained commands; `if !(con1 && con2 && !con3)` => `execute unless con1 unless con2 if con3`.

`if` can also be used inside execute commands. `execute positioned ~ ~10 ~ if (con1 && !con2)`
=> `execute positioned ~ ~10 ~ if con1 unless con2`.
`if` statements outside of an execute command will automatically put a `run` at the end of the conditions. This means
that chaining an `if` into an `execute` will be `if (cond) execute`. However, this can be [optimized](Danger%20Zone.md#optimizations) away by the
compiler.

## Calling Functions

Statements without spaces that end with `()` will be interpreted as a function call. `example:subfolder/main()`
=> `function example:subfolder/main`. Functions that start with a `#` will be turned into function tag calls. If a
function call does not start with a namespace, the current one will be added; `init()` => `function <namespace>:init`.
`#groups/reload()` => `function #<namespace>:groups/reload`

The current directory can be substituted with an `&`. If we are in the file `functions/outer/inner.msk` and we
call `&local()`, the result will be `function <namespace>:outer/inner/local()`. Calling just `local()` from here would
attempt to call the function `<namespace>:local`. See ['&' and 'r&' Replacements](#-and-r-replacements)

## Scoreboards & Short Scores

### Short scores

Mitsuko introduces a new way to represent scoreboards: `<entity>:<board>`. When dealing with scores, they can be
referenced through this syntax. For instance, to access the current entity's `health` score, you would use `@s:health`.
Local and Remgine boards can be accessed through [local replacements](#-and-r-replacements).
[Temporary scores](Remgine.md#temp-scores) (remgine exclusive) act just like normal scores, but have an even easier
syntax.

### Score Operations

Mitsuko also allows us to easily manipulate scores using this, by doing `<short_score> [<operation> <value>]`. In this
situation, `<value>` can be either a number or another short score. If no operation is provided, it defaults to `get`.

Operations:

* `++` : Adds 1 to the score
* `--` : Subtracts 1 from the score
* `get` : Gets the score (default if no operation is provided)
* `reset` : Resets the score
* `enable` : Enables the score
* `= <value>` : Sets the score to the given value
* `+= <value>` : Adds the given value
* `-= <value>` : Subtracts the given value
* `%= <value>` : Modulo by the given value
* `/= <value>` : Divide by the given value
* `*= <value>` : Multiply by the given value
* `>< <value>` : Swap the values
* `< <value>` : Min the values
* `> <value>` : Max the values

*Note: Using [%=, /=, *=, ><, <, >] when `<value>` is a number requires remgine, and is suboptimal.*

#### Create / Remove

Scoreboard can be easily created & removed using two simple commands. `create <name> [type]` will create a new
scoreboard with `<name>` and `[type]`, or `dummy` if no type is provided. `remove <name>` removes the scoreboard. Names
can use [local replacements](#-and-r-replacements)

## Set / Retrievals

The `set` command allows us to set a variable to a value, and retrievals allow us to fetch the value.

```
fn many_effects() {
    // Set "eff" to "effect give @s"
    set eff effect give @s
    // Set does not appear in the compiled code
    
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

## Built-in Conditionals

These are conditions that can be used in statements that are built into Mitsuko. For the remgine exclusive one,
see [random](Remgine.md#random).

### Score Testing

Two scores can be compared with `<short_score> <operation> <value>`. `<value>` can be either a short score or a number.
`<operation>` can be <=, <, =, >, >=.

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
show_entity, suggest_command, etc.). `<event_data>` is the data passed to the event. These can quickly make the JSON
bloated and confusing to read.

Examples:

```
*JSON{text aqua underlined :: "I am an underlined, aqua colored piece of text!"}
*JSON{score strike !bold !italic :: @s:&score}
*JSON{nbt bold #58af50 :: block 10 50 12 : Items}
*JSON{text : hover show_text *JSON{score :: @s:&score} : "Hover to see your score!"}
```

Section breaks (§) can be used to apply formatting where Minecraft allows it. However, Quick JSON does not like dealing
with them, and is prone to throwing errors when they are used. Section breaks can be inserted with the `*{SB}`
retrieval. Nesting Quick JSON Objects is experimental, the Lexer can get pretty angry. If you really need to use quick
JSON and the lexer won't let you, See [Danger Zone: @NOLEX](Danger%20Zone.md#nolex).

## '&' and 'r&' Replacements

In many places, an `&` can be used to reference the datapack itself, or a local part of it.

### In Functions

When [calling](#calling-functions) a function, placing an ampersand in front the of the function's name will cause it to
use the path of the current file. If you are in a file called `players` and you want to reference a function in the same
file, you can use `&other_function()` rather than the full path, `players:other_function()`. Ampersands cannot be used
when defining a function, as the path of the current file is automatically appended.

### In Tags

Selectors will replace any instance of `tag=&<tag>` with `tag=<namespace>.<tag>`. For instance, `@s[tag=&admin]` in
the `example` namespace will result in `@s[tag=example.admin]`.

The `tag` command is able to add/remove local tags. `tag @s add &cutie` => `tag @s add <namespace>.cutie`.

Adding tags to summon commands will be able to replace locally. `summon pig ~ ~ ~ {Tags:[&oinker]}`
=> `summon pig ~ ~ ~ {Tags:[<namespace>.oinker]}`

### In Scoreboards

Scoreboard [create/remove](#create--remove) and operations will replace `&` with the current namespace. `create &temp`
=> `scoreboard objectives add <namespace>.temp dummy`. Scoreboard [operations](#score-operations) can also accept `&`
, `@s:&temp ++` => `scoreboard players add @s <namespace>.temp 1`.

### 'r&' Replacement

In tags and scoreboards, using `r&` rather than `&` will use `remgine` as the namespace. `@s[tag=&admin]`
=> `@s[tag=<namespace>.admin]`, but `@s[tag=r&admin]` => `@s[tag=remgine.admin]`.
