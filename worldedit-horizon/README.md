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
- **Schematics:** `schematic save|load|list|delete <name>` — the build of the selection, kept in a file of the world's own folder
- **History and limits:** `undo`, `redo`, `physics [on|off]`, `limit`

Patterns: `stone`, `50%stone,dirt`. Directions: `north south east west up down`.

## The selection is drawn on the client

The selection is drawn where you look: a frame around the box of the selection and a grid inside it. Every cell of that grid is one block wide, whatever the size of the box. A round selection (`sel sphere`, `sel cylinder`, `sel ellipsoid`) is drawn as the blocks it covers, one square at a time, so the outline stands exactly on the blocks the command would change — the same look as Minecraft's block outline (F3+B). A shape with too many blocks to show one by one is drawn as its three sections through the middle, so its axes stay together. The lines are red: `#CCFF0000` for the frame and the blocks, `#CCAA0000` for the grid inside a box. The lines are the engine's own, drawn over everything, so the blocks do not hide the selection: it is visible through the terrain. The server sends the selection to the client over `horizon:mod_net` on every change, and the client draws it.

`/we grid off` hides the grid inside the selection; the frame around it stays. The setting is yours alone: every player has their own, and it survives a new selection.

## Brushes

`/we brush sphere 5 stone` arms a brush for the player: the wand's left click then paints the brush's shape on the clicked block, in the middle of it, and its right click erases the same shape with air. The shapes are the selection's own (`sphere <radius>`, `cylinder <radius> <height>`, `cube <side>`, `ellipsoid <rx> <ry> <rz>`) and the pattern a block or a mix, as everywhere else (`stone`, `30%stone,dirt`, `0` for air). `/we brush radius <n>` sets the radius of the armed brush and `/we brush radius +1` grows it, so the reach grows without retyping the brush.

`/we brush distance <n>` makes the brush work on the block the player looks at, as far as the brush reaches (64 blocks by default, 256 with `/we limit distance 256`), instead of on the clicked block: the shape sits in the middle of that block, and nothing is painted on the way there. Where there is nothing at all — the sky — the shape goes `n` blocks ahead instead, so blocks can be put up in the air. Both the left click (paint) and the right click (erase) work that way, and `/we brush at` paints at once on the block under the view, `/we brush erase at` erases there.

The brush goes through the blocks it covered last, as if they were air: it does not shoot at its own work, so a second click does the same as the first one. A new brush keeps the distance of the old one: `/we brush sphere 4 dirt` changes the shape and the pattern, not the distance.

The brush belongs to the player alone: `/we brush` says which one is armed and `/we brush off` puts it away, so the wand selects the corners again. A stroke is one edit in the history, so `/we undo` takes it back.

## Schematics

`/we schematic save <name>` writes the build of the current selection into a file of the world's own folder: `world/worldedit_horizon/schematics/<name>.schematic`, so the world's backups carry it along. The name holds letters, digits, `_` and `-` only (up to 48 characters), so it cannot leave that folder.

`/we schematic load <name>` reads the file back into the clipboard, and `/we paste` puts the build down: its lowest corner goes where the player stands. `/we schematic list` shows the saved builds with their size, `/we schematic delete <name>` removes one.

The file holds the blocks of the build and the size of the box they were taken from, as the game's own block ids: a build is read with the same game and the same resource pack as it was saved with. The format is the mod's own, not WorldEdit's `.schematic`.

Options > Horizon Settings > Mods lists the mod: its page has the button *Open the schematics folder*, which opens the folder in the system's file manager — the open world's own (`<world>/worldedit_horizon/schematics`), where the server keeps the schematics, or the mod's own folder in the instance when no world is open. The client asks the mods for their rows and buttons through the `native_mods`, `native_settings` and `native_settings_click` hooks (`mods_ui.rs`).

## Install

- Download `worldedit_horizon-<version>.hnmod` from a release, or
- in the launcher: Mods → *Add from GitHub* with this repository's link.

## Build the `.hnmod`

```sh
python3 tools/package.py      # -> target/worldedit_horizon-<version>.hnmod (Python 3.11+)
```

Layout: `mixins/worldedit_horizon/` holds `horizon-mixin.toml`, the hook helpers and the modules
(`we_core.rs`: the rules with no game types and their unit tests; `glue.rs`: the edges to the game's types;
`geometry.rs`: the selection as the client draws it, without game types; `render.rs`: the client's half, the
selection the server sent and the lines it is drawn with; `brush_client.rs`: the brush's clicks in the air;
`mods_ui.rs`: the mod's row in Options > Mods and the button that opens the folder with the schematics).
`compatibility.json` pins the Horizon commit the mod was
tested on and the hook points version (`hooks`).

The id must be the same in `[mod] id`, the folder name `mixins/<id>/`, the packaged manifest and the
id typed in the submit form at playhorizon.dev/mods/submit.

## Licence

Add a `LICENSE` file before publishing (the registry asks for a licence).
