# Danger Zone

### TOC

* [Suppress Warnings](#suppress-warnings)
* [@Commands](#commands)
* [Optimizations](#optimizations)

Welcome to the Danger Zone! (*spooky thunder*) In here you will find abilities that are highly experimental and should
not be used unless you know what you're doing.

## Suppress Warnings

Suppress warnings is a [pack.mcmeta tag](File%20Structure.md#packmsk-only), Adding `suppress_warnings = true`
to `pack.mcmeta` will suppress any further warnings. Remember, the warnings are there for a reason.

## @Commands

Not to be confused with [modifiers](Functions%20and%20Flow%20Control.md#modifiers)! @Commands acts like normal commands;
they are put into the code part of a function block. They are not compiled into the final result, instead changing what
the compiler does at that specific line.

#### @NOLEX

Prevents the current line from being seen by the lexer from the Mitsuko lexer plugin. Ignored by the compiler.

#### @DEBUG [name]

Prints to the console when this line is hit

#### @DBG_ERROR [name]

Prints to the console when this line is hit and halts execution

#### @TREE

Displays a tree view of the next node AST, before it's compiled

## Optimizations

Mitsuko supports three levels of optimizations: Off [0], Mild [1], Strong [2], Experimental [3]. The level can be set
through the `optimization_level` [tag](File Structure.md#tags).

**Off**: Will not optimize code.

**Mild**: Certain parts of code will get collapsed. 'execute run', 'run execute', and 'as @s' will be removed. In-line
blocks that only contain a single command will have the command inlined.

»**Strong**: Mild optimizations + Optimizes score operations by making static variables. Will inline small functions.

»**Experimental**: Dangerous!