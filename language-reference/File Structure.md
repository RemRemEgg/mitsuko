# File Structure

### TOC

* [Folder Structure](#folder-structure)
* [Folder Structure § pack.msk](#packmsk)
* [Namespaces](#namespaces)
* [Tags](#tags)
* [Function Files](#function-files)
* [Item Files](#item-files)
* [Event Link Files](#event-link-files)
* [Advancement Files](#advancement-files)
* [Extras Folder](#extras-folder)

*Note: When dealing with file paths, I will often use a `~` to represent the path `/src/<namespace>/`,
so `~/functions/file.msk` represents `/src/<namespace>/functions/file.msk`. I also use `data/~/`
to mean `/generated/<name>/data/<namespace>/`, so `data/~/functions/file.mcfunction` means
`/generated/<name>/data/<namespace>/functions/file.mcfunction`*

## Folder Structure

```jsonpath
./
├───<name>.export.msk
├───/.cache
│   └───(compiler generated)
├───/generated
│   └───/<name>
│       └─── (compiler generated)
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

Used for overarching metadata of the datapack as well as `pack.mcmeta`. Placed in directly in the `/src/` folder of the
project, the program will error out if no file is found. This file contains a list of tags & imports for the program to
use. See [Tags](#tags) for a full list of usable tags. Example of `pack.msk`:

```
name = Mitsuko Example
comments = true
version = 15
description = §3Example pack for §bmitsuko
remgine = true

use Remgine
use Another External Datapack
```

#### Importing other datapacks through 'Use'

Other datapacks can be imported by putting its export file in Mitsuko's `imports` folder, and then adding the
line `use <name>` to pack.msk. Unlike normal programming languages, this will not add the other datapack into the
existing one, instead it will allow Mitsuko to recognise its functions. Export files are produced when the export flag
is added, and are placed in the root folder and named `<datapack_name>.export.msk`.
See [Compiler Options](../README.md#compiler-options)

*Note: the current import/export system is a little off, should be reworked in the near future*

## Namespaces

Namespaces are any combination of underscores, lowercase letters, and numbers. Namespaces can be inserted with
the `*{NS}` [retrieval](Utilities.md#set--retrievals). This retrieval is locally overrideable. Attempting to use an invalid namespace will
error out.

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

Tags are special functions that tell the compiler how to handle code. The format for tags within function files
is `#[<name> = <value>]`, and in `pack.msk` it is `<name> = <value>`.

### Global Tags (Can be used with the @meta [modifier](Functions%20and%20Flow%20Control.md#modifiers))

* comments (bool) : Should the comments & whitespace be transferred to the `.mcfunction` file, default false
* debug (0..2) : Debug verbosity, default 0
* optimizations (0..3) : How much to [optimize](Danger%20Zone.md#optimizations) code, default 0
* recursive_replace (0..255) : How any recursions to use for
  nested [retrievals](Utilities.md#set--retrievals), default 3

### pack.msk only

* description (String) : Description field (supports §), default "A Datapack"
* name (String) : Datapack's name, default "Untitled"
* remgine (bool) : Is remgine enabled, default false
* version (int) : Datapack's Version, default is the latest release version

## Function Files

Function files get compiled into a variety of `.mcfunction`s. They are located in the `/src/<namespace>/functions/`
folder. [language-reference/Functions and Flow Control](Functions and Flow Control.md)

## Item Files

Item files get compiled into an advancement, recipe, and function. These are used to pull off NBT crafting without much
hassle. They are located in the `/src/<namespace>/items/` folder. [language-reference/Items](Items.md)

## Event Link Files

Event Link files get compiled into function tags. These can be used to link events from multiple datapacks together.
They are located in the `/src/<namespace>/event_links/` folder. [language-reference/Event Links](Event Links.md)

## »Advancement Files

Todo! Should allow ease of use advancements and advancements as triggers

## Extras Folder

This is a folder that gets copied directly into `/generated/<name>/data/`. This can be used for files that Mitsuko does
not support, like world gen. i.e:

```
./src
└───/namespace
    └───/extras
        └───/worldgen
            └───/../
```

Would cause the generated output to have

```
./generated
└───/pack
    └───/data
        └───/namespace
            └───/worldgen
                └───/../
```