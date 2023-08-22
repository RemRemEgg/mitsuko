# Event Links

### TOC

* [Methodology](#methodology)
* [File Syntax](#file-syntax)

Event links allow you to quickly make function tags. This is mainly used for linking events from different datapacks
together.

## Methodology

One datapack, for instance Remgine, can create some blank event links for other datapacks to latch onto. One such link
it creates is `load : none`, meaning Remgine has a link called "load" with no functions assigned to it. Another
datapack, with namespace `example`, would then have the file `~/event_links/remgine.msk` with the link `load : init`.
This is binding the `example:init` function to Remgine's `load` event. If the example link
was `load : init, init2, other:function` it would bind the functions `example:init`, `example:init2`,
and `other:function` to Remgine's `load` event.

Since events are just function tags, they can be [called](Utilities.md#calling-functions) by `#<name>()`. If you had a
link called `trigger`, you would call it through `#trigger()`. Function tags can also have external namespaces attached
to them.

## File Syntax

Each link is stored on a separate line within a link file. Each link file is located in `~/event_links/`. The name of
the file represents the namespace that all the links are pointing to. The links follow this
syntax: `link_name : <none | function>+`. A file named `remgine.msk` in the `event_links` directory would cause all its
links to point to remgine. If it had a line `load : example:init`, then when remgine called its `load` group, the
current datapack would call `example:init`. Just like with code based function calls, if a function does not have a
namespace then the current one will be assumed. If the line was `load : init`, it would try to call `init` in the
current namespace. `none` can be put in place of the functions, in which case the link will be created with no function
calls attached to it.

For a more technical perspective, assume `remgine.msk` containing `load : example:init`. This will create a new
namespace in the current datapack named `remgine`. It will add `load.json` to `/data/remgine/tags/functions/`, and
the `values` array will have `example:init` as the only item. The `replace` field will be set to `false`.

But what if I want to call a function called `none` from an event link? You can either:

- add a separate function called `none_link` that calls `none` and then link `none_link`
- be reasonable, and rename the function
