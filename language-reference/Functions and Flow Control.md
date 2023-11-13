# Function Examples & Syntax, Flow Control

### TOC

* [Function Definitions](#function-definitions)
* [Comments](#comments)
* [Modifiers](#modifiers)
* [Function Code](#function-code)
* [Function Code § Inline Blocks](#inline-blocks)
* [Flow Control § For](#for-loop)
* [Flow Control § While](#while-loop)
* [Flow Control § Repeat](#repeat)

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
be `/data/~/functions/sub/internal.mcfunction`. Functions can have any number of folders before them, but cannot be
prefaced with a different namespace.

Function paths are also dependent on the file they are located at. Any functions located within the
file `~/functions/file.msk` will be at the path `example:file/<name>`. This can be combined with sub-folders and
function names to easily organize functions. A file `~/functions/folder/file.msk` containing the function
declaration `fn internal/start() {}` will have the resulting path `<namespace>:folder/file/internal/start`.

If the file a function is in is called `functions.msk`, then this last path part will be ignored. Functions in the
file `~/functions/folder/functions.msk` will be located at `folder/`, not `folder/functions/`. Functions placed in the
master `~/functions/functions.msk` file will have no extra path.

``` 
fn sqrt($x) {}
```

Unlike most programming languages, these functions cannot accept arguments. This function will fail to compile.

*Note: with the recent introduction of macro functions to minecraft, this may change!*

## Comments

Comments can be put both inside and outside of functions but cannot be put after a command. A comment is two forward
slashes followed by any text.
```// This is a comment!```

## Modifiers

Modifiers are single lines of code that change either a single function or all functions in a file. Modifiers are an `@`
followed by data specifying the modification.

### @alias

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
function, only create a new one that calls the defined function.

### @meta

The `meta` modifier can can be used to temporarily change the options in `pack.mcmeta`.
See [tags](File%20Structure.md#tags) for which tags can and cannot be used in this context. Tags can only change the
single function that comes after it.

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

### @no_export

Prevents a function from getting exported, assuming that the export [flag](../README.md#compiler-options) is enabled.

```
@no_export
fn private_fn() {}

fn exported_fn() {}
```

### @set

The `@set` modifier works just like a `set` command; It will set a specific variable to a value for all subsequent
functions in a file.

Example:

```
@set AUTHOR RemRemEgg
@set SIZE 10
fn init() {
    tellraw @a *JSON{:: "This was made by *{AUTHOR}"}
}

fn did_i_hear_a() {
    ast @e[distance=..*{SIZE}] ast @e[distance=..*{SIZE}] ast @e[distance=..*{SIZE}] run say ROCK AND STONE!
}
```

## Function Code

Just like regular MCFunction, commands are put one line after another. However, Mitsuko adds some custom commands,
macros, and features to help with writing code. This section uses other concepts
like [Quick JSON](Utilities.md#quick-json)
and [& / r& Replacements](Utilities.md#-and-r-replacements).

### Inline Blocks

Mitsuko adds the ability to inline blocks of code.

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

This allows us to call a function on `@e[tag=&temp]` without having to define a separate function to call.

## Flow Control

Does what it says on the tin! This section is dedicated to the 3 main ways that Mitsuko adds common flow control to
MCFunction.

### For Loop

For loops follow one of two formats; `for (<score>, <stop>) {` or `for (<score>, <start>, <stop>) {`. If no `<start>` is
specified, it starts at 0; For loops go until they reach the end value (value < stop). Both `<start>` and `<stop>` can
be scoreboards or values. The `<start>` value can be omitted with an `_`, making the current value the start value

```
fn test() {
    for ($i, 5) {
        tellraw @s *JSON{score :: $i}
    }
    for (iteration:&data, 18, max:&data) {
        tellraw @s *JSON{score :: iteration:&data}
    }
    for ($test, _, 18) {
        setblock ~ ~ ~ gold_block
    }
}
```

The first loop would print out the numbers from 0 all the way to `$i - 1`, and the second would print from
`iteration:&data` to `max:&data - 1`.

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