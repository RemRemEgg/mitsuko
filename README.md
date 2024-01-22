<h3>Warning: This branch is an in-progress rewrite of Mitsuko!</h3>

<h1>Mitsuko</h1>
<h6>Version 0.2.0 - For 1.20.2 - By RemRemEgg</h6>

---

# Overview

Mitsuko is a custom language designed to make programming in minecraft easier and faster. It is heavily based around the
vanilla commands, and it not a replacement for them. It simply adds ease of use and more functionality.

*Disclaimer*:<br>Mitsuko and RemRemEgg are in no way affiliated or endorsed by Minecraft or Mojang Studios, Mitsuko is a
fan-made project. Please don't sue me.

*Note: Items prefixed with a '»' are **going** to be added, but are not currently functioning. Items prefaced with a '«'
are going to be removed in a future version*

### TOC

* [List of Reference Files](#list-of-reference-files)
* [What is Mitsuko?](#what-is-mitsuko)
* [What is Remgine?](#what-is-remgine)
* [Compiler Options](#compiler-options)

## List of Reference Files

* [README](README.md)<br>&emsp; General information about Mitsuko
* [language-reference/File Structure](language-reference/File%20Structure.md)<br>&emsp; File structure and how to set up a
  project, `pack.mcmeta` tags
* [language-reference/Functions and Flow Control](language-reference/Functions%20and%20Flow%20Control.md)<br>&emsp; Function
  structure and examples, Flow control
* [language-reference/Event Links](language-reference/Event%20Links.md)<br>&emsp; Event link methodology and syntax
* [language-reference/Items](language-reference/Items.md)<br>&emsp; Item file layout and syntax
* [language-reference/Remgine](language-reference/Remgine.md)<br>&emsp; How Remgine interacts with Mitsuko, how to use
  it, Remgine exclusive commands
* [language-reference/Utilities](language-reference/Utilities.md)<br>&emsp; Special commands and syntax that Mitsuko
  adds
* [language-reference/Danger Zone](language-reference/Danger%20Zone.md)<br>&emsp; The Dangerous and experimental features
  of Mitsuko

## What is Mitsuko?

Hi, I'm RemRemEgg, the creator of Mitsuko. I have been programming in Minecraft for a long time, from command blocks to
datapacks, but I always found them quite annoying to use. This is why I made Mitsuko; I wanted the experience of making
a datapack to be more streamlined and accessible, so I could spend more time actually writing code instead of making a
new file for each function. Minecraft's commands are quite difficult to use on their own, so I've tried to make sure
that Mitsuko helps make them less verbose while still allowing them to be read easily.

Mitsuko is, at its core, no more than a translator. The primary code written in Mitsuko is the same as Minecraft
commands, just Mitsuko adds special functionality to make it all easier. For instance, making crafting recipes that used
nbt in the output has always been quite annoying, as you need a recipe, an advancement, and a function for each item you
want. I really don't like these limitations, hence why I have features like item files. Mitsuko still requires a good
understanding of Minecraft's commands, as the majority of code written is still just the vanilla commands. The
non-vanilla parts that are added primarily target the commands that are used most often in datapacks, like execute,
scoreboard, and function. For more information on the things that Mitsuko adds,
check [language-reference/Utilities](language-reference/Utilities.md).

## What is Remgine?

Remgine is one of the best datapacks I've made with Mitsuko. It's a back-end datapack, designed to make it easier for
anyone to detect common events, use common functions, and tie datapacks together. Remgine handles a lot of features like
entity ticking, loading, killing specific entities, and other things that I find I often use in datapacks. It also
provides a lot of functions that I often find myself using, like getting a players name, tossing an entity in the
direction they face, ray-casting, NBT testing, and much more. Remgine may not be fore everyone, I've designed Remgine to
be for what I commonly use, so there may be a lot of things that other people don't want from this datapack. Also, the
name Remgine came from mixing my name, Remmy, with Engine.

## Compiler Options

```
Usage: mitsuko [MODE] [OPTIONS]
Modes:
	help
		Prints this message
	build <pack_location> [options]
		Builds the specified datapack
		(-g | --gen-output) <location>
			Change the generation output to <location>/datapacks
		(-e | --export)
			Enable creation of export file
		(-C | --cache)
			Enable caching
```

Compiler options can stack, so doing `mitsuko build projects/pack -egC D:/datapacks` is valid.

*So, why only help and build? I'm planning on adding more modes, like one that could convert from a datapack to Mitsuko.
Maybe also one that can combine multiple datapacks together, and automatically resolve and conflicts between them. Not
sure about anything else I might want, you can let me know in
the [GitHub issues](https://github.com/RemRemEgg/mitsuko/issues), or if enough people are interested I might make a
discord for it*