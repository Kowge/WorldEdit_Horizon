# WorldEdit for Horizon (`worldedit_horizon`)

A native mod for [Horizon](https://github.com/horizon-rust/horizon) (Minecraft: Java Edition 26.3 in Rust)
that adds WorldEdit-style building commands. It is an independent implementation, **not** the original
WorldEdit and not affiliated with EngineHub.

It is a **source mixin bundle** ([horizon-mixins](https://github.com/horizon-rust/horizon-mixins)): the
launcher compiles it into the game. There is no sandbox, so read the source before installing.

## Use

Creative mode **and** operator permission only: every `/we` command and the wand need operator level 2 or more (`ops.json`; single-player owner with cheats on), and `/we limit` (limits shared by all players) needs level 4. A creative non-operator is refused. Everything starts with `/we`; `/we help` lists the commands.

Every command is under `/we`: `/we set stone`, `/we copy`, `/we undo`. The server cuts the slashes off a command line, so `//we set stone` runs the same command, but the game marks the double slash as an unknown name in the chat box — write it with one slash.

- **Selection:** wooden axe wand (left click = first corner, right click = second), `wand`, `pos1`, `pos2`, `pos`, `hpos1`, `hpos2`, `sel`, `sel <shape> [sizes]`, `grid on|off`, `size`, `trim`, `distr`, `expand`/`contract`/`shift`/`outset`/`inset`, `chunk`
- **Edit:** `set`, `replace`, `overlay`, `naturalize`, `walls`, `faces`, `hollow`, `line`, `count`, `smooth`, `fill`, `fillr`, `removeabove`, `removebelow`, `drain`, `fixwater`, `fixlava`
- **Shapes:** `sphere`/`hsphere`, `cyl`/`hcyl`, `pyramid`/`hpyramid`
- **Selection shapes:** `sel cube`, `sel sphere`, `sel cylinder`, `sel ellipsoid` — the selection becomes a box, a ball, a tube or an ellipsoid; with sizes the volume appears around the player at once, without them it follows the two corners
- **Chunks:** `chunkinfo` (the chunk under you and how many blocks it holds), `listchunks` (the chunks the selection covers)
- **Clipboard:** `copy`, `cut`, `paste [skipair]`, `rot`, `flip`, `stack`, `move`, `clearclipboard`
- **History and limits:** `undo`, `redo`, `physics [on|off]`, `limit`

Patterns: `stone`, `50%stone,dirt`. Directions: `north south east west up down`.

## The selection is drawn on the client

The selection is drawn where you look: a frame around the box of the selection and a grid inside it. Every cell of that grid is one block wide, whatever the size of the box. A round selection (`sel sphere`, `sel cylinder`, `sel ellipsoid`) is drawn as the blocks it covers, one square at a time, so the outline stands exactly on the blocks the command would change — the same look as Minecraft's block outline (F3+B). A shape with too many blocks to show one by one is drawn as its three sections through the middle, so its axes stay together. The lines are red: `#CCFF0000` for the frame and the blocks, `#CCAA0000` for the grid inside a box. The lines are the engine's own, drawn over everything, so the blocks do not hide the selection: it is visible through the terrain. The server sends the selection to the client over `horizon:mod_net` on every change, and the client draws it.

`/we grid off` hides the grid inside the selection; the frame around it stays. The setting is yours alone: every player has their own, and it survives a new selection.

## Brushes

`/we brush sphere 5 stone` arms a brush for the player: the wand's left click then paints the brush's shape on the clicked block, in the middle of it, and its right click erases the same shape with air. The shapes are the selection's own (`sphere <radius>`, `cylinder <radius> <height>`, `cube <side>`, `ellipsoid <rx> <ry> <rz>`) and the pattern a block or a mix, as everywhere else (`stone`, `30%stone,dirt`, `0` for air). `/we brush radius <n>` sets the radius of the armed brush and `/we brush radius +1` grows it, so the reach grows without retyping the brush.

`/we brush distance <n>` makes the brush work along the line of sight instead of on the clicked block: it paints every block from the player's eyes to the first one that is not air, so the wand cuts a tunnel into whatever it looks at. Where there is nothing at all — the sky — the brush's shape goes `n` blocks ahead, so blocks can be put up in the air. Both the left click (paint) and the right click (erase) work that way, and `/we brush at` paints along the same line at once, `/we brush erase at` erases there.

The brush belongs to the player alone: `/we brush` says which one is armed and `/we brush off` puts it away, so the wand selects the corners again. A stroke is one edit in the history, so `/we undo` takes it back.

## Install

- Download `worldedit_horizon-<version>.hnmod` from a release, or
- in the launcher: Mods → *Add from GitHub* with this repository's link.

## Build the `.hnmod`

```sh
python3 tools/package.py      # -> target/worldedit_horizon-<version>.hnmod (Python 3.11+)
```

Layout: `mixins/worldedit_horizon/` holds `horizon-mixin.toml`, the hook helpers and four modules
(`we_core.rs`: the rules with no game types and their unit tests; `glue.rs`: the edges to the game's types;
`geometry.rs`: the selection as the client draws it, without game types; `render.rs`: the client's half, the
selection the server sent and the lines it is drawn with). `compatibility.json` pins the Horizon commit the mod was
tested on and the hook points version (`hooks`).

The id must be the same in `[mod] id`, the folder name `mixins/<id>/`, the packaged manifest and the
id typed in the submit form at playhorizon.dev/mods/submit.

## Licence

Add a `LICENSE` file before publishing (the registry asks for a licence).
