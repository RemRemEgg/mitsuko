# Remgine

*Note: I might make a GitHub project for remgine, and ill add documentation for all the functions there*

## Remgine Exclusive / Requires Remgine

These items require remgine to enabled in order to use them.

### TOC

* [rmm](#rmm)
* [Temp Scores](#temp-scores)
* [Random](#random)

### rmm

Mitsuko allows you to easily setup and call `remgine:utils/rmm` with the `rmm` command. There are three versions of the
command: `rmm`
, `rmm <value>`, `rmm set <value>`. `rmm` will just call the `remgine:utils/rmm` function. `rmm set <value>` will set
the current entity's (@s) `remgine.rmm` score to the provided value. `rmm <value>` does both, setting the value then
calling the function.

### Temp Scores

Temp scores are all stored in `remgine.temp` and are access through `$<name>`. For instance, to access the temporary
score `math` you would use `$math`. To add the temp score `a` to all players
`level`, you would use `@a:level += $a`. These temporary scores act just like normal scores and can be used in place of
any other score.

### Random

Random percentages can be used with `random <amount>`. ~~For a full list of the allowed amounts, see the remgine
documentation~~ (not yet, sorry!). i.e, `if (random 50) say well, it's a 50/50 chance!`.