# WorldEdit for Horizon (`worldedit_horizon`)

A native mod for [Horizon](https://github.com/horizon-rust/horizon) (Minecraft: Java Edition 26.3 in Rust)
that adds WorldEdit-style building commands. It is an independent implementation, **not** the original
WorldEdit and not affiliated with EngineHub.

It is a **source mixin bundle** ([horizon-mixins](https://github.com/horizon-rust/horizon-mixins)): the
launcher compiles it into the game. There is no sandbox, so read the source before installing.

## Use

Creative mode **and** operator permission only: every `/we` command and the wand need operator level 2 or more (`ops.json`; single-player owner with cheats on), and `/we limit` (limits shared by all players) needs level 4. A creative non-operator is refused. Everything starts with `/we`; `/we help` lists the commands.

Every command is under `/we`: `/we set stone`, `/we copy`, `/we undo`. The server cuts the slashes off a command line, so `//we set stone` runs the same command, but the game marks the double slash as an unknown name in the chat box — write it with one slash.

- **Selection:** wooden axe wand (left click = first corner, right click = second), `wand`, `pos1`, `pos2`, `pos`, `hpos1`, `hpos2`, `sel`, `sel <shape> [sizes]`, `size`, `trim`, `distr`, `expand`/`contract`/`shift`/`outset`/`inset`, `chunk`
- **Edit:** `set`, `replace`, `overlay`, `naturalize`, `walls`, `faces`, `hollow`, `line`, `count`, `smooth`, `fill`, `fillr`, `removeabove`, `removebelow`, `drain`, `fixwater`, `fixlava`
- **Shapes:** `sphere`/`hsphere`, `cyl`/`hcyl`, `pyramid`/`hpyramid`
- **Selection shapes:** `sel cube`, `sel sphere`, `sel cylinder`, `sel ellipsoid` — the selection becomes a box, a ball, a tube or an ellipsoid; with sizes the volume appears around the player at once, without them it follows the two corners
- **Chunks:** `chunkinfo` (the chunk under you and how many blocks it holds), `listchunks` (the chunks the selection covers)
- **Clipboard:** `copy`, `cut`, `paste [skipair]`, `rot`, `flip`, `stack`, `move`, `clearclipboard`
- **History and limits:** `undo`, `redo`, `physics [on|off]`, `limit`

Patterns: `stone`, `50%stone,dirt`. Directions: `north south east west up down`.

## Install

- Download `worldedit_horizon-<version>.hnmod` from a release, or
- in the launcher: Mods → *Add from GitHub* with this repository's link.

## Build the `.hnmod`

```sh
python3 tools/package.py      # -> target/worldedit_horizon-<version>.hnmod (Python 3.11+)
```

Layout: `mixins/worldedit_horizon/` holds `horizon-mixin.toml`, the hook helpers and two modules
(`we_core.rs`: rules with no game types and its unit tests; `glue.rs`: the edges to the game's types).
`compatibility.json` pins the Horizon commit the mod was tested on and the hook points version (`hooks`).

The id must be the same in `[mod] id`, the folder name `mixins/<id>/`, the packaged manifest and the
id typed in the submit form at playhorizon.dev/mods/submit.

## Licence

Add a `LICENSE` file before publishing (the registry asks for a licence).
