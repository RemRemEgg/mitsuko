# Items

### TOC

* [Recipe](#recipe)
* [Materials](#materials)
* [Path](#path)
* [Item](#item)
* [Result](#result)

Item files are an easy and simple way to add custom recipes that utilise NBT crafting. The file is made up of four
parts: recipe, materials, path, item.

## Recipe

The recipe is up to 9 characters representing the shape, just like the pattern field in a normal recipe.

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

Here we define `s` as stone, `a` as any wooden plank, and `c` as copper ingots. Notice that they are not separated by
commas. Each char in the recipe needs to have an item associated with it.

## Path

The path is optional, but represents where the generated files will be placed. If no path is specified, it will use the
name of the file.

```
path : crafting/smoosher
```

This will place the advancement in `data/~/advancements/crafting/smoosher.json`, recipe in
`data/~/recipes/crafting/smoosher.json`, and function in `data/~/functions/crafting/smoosher.mcfunction`.

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

## Result

The result for this item file would like something similar to this:

```
recipe {
    "sss"
    "a a"
    "ccc"
}
materials {
    s : stone
    a : #wooden_planks
    c : copper_ingot
}
path : crafting/smoosher
item {
    give @s stone_block{display:{Name:'*JSON{text #4a2319 bold !italic : The Smoosher}'},Enchantments:[{id:knockback,lvl:10}]}
    particle wax_off ~ ~1 ~ 1 1 1 0 100
    advancment grant @s only example:obtain_smoosher
}
```