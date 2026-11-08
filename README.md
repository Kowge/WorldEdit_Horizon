<p align="center">
  <img src="logo.png" width="128px" alt="WorldEdit for Horizon" />
</p>
<div align="center">

# 🪓 WorldEdit for Horizon

**A WorldEdit-style building toolkit for the [Horizon](https://playhorizon.dev) Minecraft client — native mod (`.hnmod`)**

![version](https://img.shields.io/badge/version-1.2.0-blue) ![game](https://img.shields.io/badge/Horizon_Minecraft-26.3-green) ![hooks](https://img.shields.io/badge/native%20hooks-v58-lightgrey) ![mode](https://img.shields.io/badge/works%20in-creative-orange) ![ops](https://img.shields.io/badge/operator%20level%202%2B-orange)

</div>

---

## ✨ Features

- 🪓 **Wooden axe wand** — left click = pos 1, right click = pos 2. Outside creative it is a normal axe.
- 🎨 **Creative only, operators only** — every command works only in `/gamemode creative` and only for operators (permission level 2); `/we limit` needs level 4.
- 📍 **Exact corners** — `/we pos1 10,64,-5` and `/we pos2 10 64 -5` set a corner by coordinates; `/we hpos1` and `/we hpos2` take the block you look at.
- 📐 **Selection tools** — `/we trim` cuts the selection down to the blocks that are really there, `/we distr` counts what is inside.
- 🕳️ **Digging** — `/we fill` and `/we fillr` fill the hole around you, `/we removeabove` and `/we removebelow` clear a column up or down.
- 🧱 **Fill, replace, walls, faces, hollow, overlay, naturalize, line.**
- ⚪ **Shapes** — spheres, cylinders, pyramids, solid or hollow (`hcyl`, `hsphere`, `hpyramid`).
- 🔷 **Shapes for the selection** — `/we sel sphere 5`, `/we sel cylinder 3 4`, `/we sel ellipsoid 4 2 4`: the selection becomes a ball, a cylinder or an ellipsoid instead of a box.
- 🟥 **The selection is drawn for you** — a red frame around the box and a grid inside it, one cell per block. A round shape is drawn as the blocks it covers, so the outline stands on the blocks the command would change (like F3+B). The lines are drawn over everything, so the terrain does not hide the selection.
- 🔲 **Your own view** — `/we grid on` and `/we grid off` turn the grid inside the selection off and on; the frame stays. The setting is yours alone.
- 🖌 **Brushes** — `/we brush sphere 5 stone` arms a brush: the wand's left click paints its shape on the block you click, or along your view when a distance is set. `/we brush off` puts the wand back to selecting corners.
- 🗺 **Chunks** — `/we chunkinfo` tells what the chunk under you holds, `/we listchunks` lists the chunks the selection covers.
- 🌊 **Terrain tools** — `drain`, `fixwater`, `fixlava`, `smooth`.
- 📋 **Clipboard** — copy, cut, paste, rotate, flip, stack, move.
- 💾 **Schematics** — `/we schematic save <name>` writes the build of the selection into a file of the world's folder; `/we schematic load` reads it back and `/we paste` puts it down.
- ↩️ **Undo / redo** with configurable history.
- 🎲 **Patterns** — `stone`, `0` (air), `stone,dirt`, `30%stone,dirt`.
- 🔧 **In-game limits** — `/we limit` changes block, radius, distance and history limits live.
- 🔇 **No item drops** — edits do not pop torches/flowers (toggle with `/we physics`).

## 📦 Installation

#### Method 1: Automatic (Recommended)

1. Copy the URL of this GitHub repository from your browser's address bar.
2. Open the **Horizon** launcher, go to **Mods** -> **Install from Link**.
3. Simply paste this GitHub link directly inside the launcher, and the mod will install automatically!

#### Method 2: Manual

1. Download [worldedit_horizon](https://github.com/Kowge/WorldEdit_Horizon/releases).
2. Open the Horizon launcher → **Mods** → **Add mods** and pick the file.
3. Enable **Settings → Native mods (experimental)** and press **Build** (the first build takes a while).
4. Launch the **Native mods** instance, enter a world and switch to creative: `/gamemode creative`.

> ⚠️ Native mods are compiled into the game and have full access to your computer. Build only code you trust.

## 🚀 Quick start

```text
/gamemode creative
/we wand                    # get the axe
(left click, right click)   # pick two corners
/we set stone               # fill
/we set 0                   # delete
/we expand 5                # grow where you look
/we hcyl stone 5 3          # hollow cylinder r=5 h=3
/we sel sphere 5            # a round selection
/we undo                    # undo
```

---

# 📖 Commands

All commands start with `/we`. They work only in creative mode (`/gamemode creative`) and for operators (permission level 2). `/we limit` needs permission level 4.

> Write it with one slash — `/we set stone`. The double slash (`//we set stone`) runs as well (the server cuts the slashes off), but the chat box marks it as an unknown command.

## Selection

| Command | Description |
|---|---|
| `/we` | Show help |
| `/we wand` | Give the wooden axe: left click = pos 1, right click = pos 2. Outside creative it is a normal axe |
| `/we pos1 [coordinates]`, `/we pos2 [coordinates]` | Set a corner at the block you stand on, or by coordinates (`10,64,-5` or `10 64 -5`) |
| `/we hpos1`, `/we hpos2` | Set a corner at the block you are looking at |
| `/we pos` | Both corners at once: where you stand and the block you look at |
| `/we trim` | Trim the selection down to the blocks that are really there |
| `/we distr` | Show what the selection holds and how much of it |
| `/we sel` (or `desel`) | Clear the selection |
| `/we sel <shape> [sizes]` | Shape of the selection: `cube`, `sphere`, `cylinder`, `ellipsoid`. With sizes the shape appears around you at once, without them it follows the `pos1`/`pos2` corners |
| `/we grid on` / `/we grid off` | The grid inside the selection: on or off. The frame always stays. Your setting alone |
| `/we brush <shape> [sizes] <pattern>` | Arm a brush: the wand's left click paints it on the block you click, the right click erases it. `/we brush off` puts it away |
| `/we brush radius <n>` | The radius of the armed brush: `7` sets it, `+1` grows it, `-1` shrinks it |
| `/we brush distance <n>` | The brush works on the block you look at, up to 64 blocks away; where there is none — the sky — its shape goes `n` blocks ahead. `off` puts it back |
| `/we brush at` | Paint now on the block under your view (up to 64 blocks away). `/we brush erase at` erases there |
| `/we size` | Show selection size and block count |
| `/we expand <n> [direction]` | Grow the selection by n blocks. No direction = the way you look |
| `/we contract <n> [direction]` | Shrink the selection from that side |
| `/we outset <n>` | Grow the selection by n blocks on all sides |
| `/we inset <n>` | Shrink the selection by n blocks on all sides |
| `/we chunk` | Select the whole chunk you stand in (height -64 to 319) |
| `/we shift <n> [direction]` | Move the selection itself |

Directions: `north south east west up down` (`n s e w u d`).

## Selection shape

`/we sel <shape> [sizes]` changes the shape of the selection: a plain box (`cube`) or a round shape inside it.

- With sizes the volume appears around you: `/we sel sphere 5` — a ball of radius 5, `/we sel cylinder 3 4` — a cylinder of radius 3 and height 4, `/we sel ellipsoid 4 2 4` — an ellipsoid with those radii, `/we sel cube 5` — a 5×5×5 cube. Without sizes the shape waits for the corners: `/we pos1` and `/we pos2`, then `/we sel sphere`.

How the sizes work:

- a cube is its side;
- a sphere is a radius with its middle in the first corner;
- a cylinder is a horizontal radius and a height in rows, its bottom in the first corner;
- an ellipsoid is three radii with its middle in the first corner.

## Brushes

- `/we brush sphere 5 stone` arms a brush. The wand's left click then paints its shape on the block you click: the shape sits in the middle of that block. The wand's **right click erases** the same shape with air.
- The shapes are the selection's own: `sphere <radius>`, `cylinder <radius> <height>`, `cube <side>`, `ellipsoid <rx> <ry> <rz>`.
- The pattern is a block or a mix, as everywhere else: `stone`, `30%stone,dirt`, and `0` for air.
- `/we brush radius 7` sets the radius of the armed brush, `/we brush radius +1` grows it and `/we brush radius -1` shrinks it, so you can reach further without retyping the whole brush. A radius of 0 paints the single block you click.
- **Work on what you look at**: `/we brush distance 20` makes the brush work on the block under your view, as far as the brush reaches (64 blocks by default, 256 with `/we limit distance 256`), instead of on the block you click: the shape sits in the middle of that block and nothing is painted on the way there. Where there is nothing at all — the sky — the shape goes 20 blocks ahead instead, so you can put blocks up in the air. Both buttons work that way: the left click paints, the right click erases. `/we brush distance off` puts it back.
- The brush goes through the blocks it covered last, as if they were air: it does not shoot at its own work, so a second click does the same as the first one.
- A new brush keeps the distance of the old one: `/we brush sphere 4 dirt` changes the shape and the pattern, not the distance.
- `/we brush at` paints right now on the block under your view, without a click; `/we brush erase at` erases there.
- The brush belongs to you alone. `/we brush` says which one is armed, `/we brush off` puts it away and the wand selects the corners again.
- A stroke is one edit: `/we undo` takes it back.

## How the selection is drawn

- The selection is drawn on the client: a red frame around the box (`#CCFF0000`) and a grid inside it (`#CCAA0000`), one cell per block whatever the size of the selection is.
- A round shape is drawn as the blocks it covers, square by square, so the outline stands exactly on the blocks the command would change (like F3+B).
- A shape with too many blocks is drawn as its three sections through the middle, so its axes do not slip apart.
- The lines are drawn over everything, so the terrain does not hide the selection.
- `/we grid off` turns only the inner grid lines off; the frame stays. The setting is yours alone and survives a new selection.

## Chunks

| Command | Description |
|---|---|
| `/we chunkinfo` | The chunk you stand in and how many blocks it holds (height -64..319) |
| `/we listchunks` | The chunks the selection covers |

## Editing

| Command | Description |
|---|---|
| `/we set <pattern>` | Fill the selection. `/we set 0` deletes the blocks |
| `/we replace <from> <to>` | Replace some blocks with others |
| `/we walls <pattern>` | The 4 vertical sides only |
| `/we faces <pattern>` | All 6 sides (hollow box) |
| `/we replace !<blocks> <to>` | A leading `!` replaces everything except the listed blocks |
| `/we overlay <pattern>` | Put a layer on top of the highest block of each column |
| `/we naturalize` | Top becomes grass, 3 layers of dirt, stone below |
| `/we hollow` | Hollow out the selection, keeping a 1-block shell |
| `/we line <pattern>` | A line between pos 1 and pos 2 |
| `/we drain` | Remove water, lava and water plants in the selection |
| `/we fixwater` | Turn flowing water in the selection into source blocks |
| `/we fixlava` | The same for lava |
| `/we smooth [n]` | Smooth the terrain in the selection, n passes (1–50, default 1) |
| `/we count <blocks>` | Count blocks in the selection |
| `/we fill <pattern> <radius> [depth]` | Fill the hole around you: each column down to the first block |
| `/we fillr <pattern> <radius> [depth]` | The same, but the whole air of the ball |
| `/we removeabove [size] [height]` | Remove the blocks above you (a `size` × `height` square) |
| `/we removebelow [size] [height]` | Remove the blocks below you |

## Shapes at your feet

No selection needed.

| Command | Description |
|---|---|
| `/we sphere <pattern> <radius> [true]` | Ball; `true` = hollow |
| `/we hsphere <pattern> <radius>` | Hollow sphere |
| `/we cyl <pattern> <radius> [height] [true]` | Cylinder (height defaults to 1); `true` = hollow with caps |
| `/we hcyl <pattern> <radius> [height]` | Hollow cylinder, open top and bottom; height 1 = ring |
| `/we pyramid <pattern> <size>` | Pyramid with `size` layers |
| `/we hpyramid <pattern> <size>` | Hollow pyramid |

## Clipboard

| Command | Description |
|---|---|
| `/we copy` | Copy the selection (relative to you) |
| `/we cut` | Copy and delete |
| `/we paste [true]` | Paste at your feet; `true` = skip air |
| `/we rot <degrees>` | Rotate the clipboard (multiples of 90) |
| `/we flip [direction]` | Flip the clipboard |
| `/we clearclipboard` | Clear the clipboard |
| `/we stack <n> [direction]` | Repeat the selection n times |
| `/we move <n> [direction]` | Move blocks and the selection |

## Schematics

| Command | Description |
|---|---|
| `/we schematic save <name>` | Write the build of the selection into a file of the world's folder (`world/worldedit_horizon/schematics/<name>.wehs`) |
| `/we schematic load <name>` | Read the file back into the clipboard; `/we paste` puts it down |
| `/we schematic list` | The saved builds and their size |
| `/we schematic delete <name>` | Remove a saved build |

- The build's **lowest corner goes where you stand** when you paste it, so the same build always lands the same way.
- The name holds letters, digits, `_` and `-` only (up to 48 characters): it cannot point outside the schematics folder.
- The file holds the blocks as the game's own block ids: read a build with the same game and the same resource pack as you saved it with.

## History

| Command | Description |
|---|---|
| `/we undo [n]` | Undo n last edits (up to 20) |
| `/we redo [n]` | Redo undone edits |

## Limits

| Command | Description |
|---|---|
| `/we limit` | Show current limits |
| `/we limit blocks <n>` | Max blocks per command (1–20,000,000) |
| `/we limit radius <n>` | Max shape radius/size (1–1024) |
| `/we limit distance <n>` | How far a brush reaches (1–256, 64 by default) |
| `/we limit history <n>` | How many edits undo keeps (1–200) |
| `/we limit reset` | Restore defaults |

Limits are shared by all players and reset when the game restarts; only an operator of level 4 changes them.

## Physics

| Command | Description |
|---|---|
| `/we physics` | Show the physics mode |
| `/we physics off` | (default) neighbours are not updated: no dropped items from torches, flowers etc. after `/we set 0` |
| `/we physics on` | Normal updates: dependent blocks pop off and drop items |

## Notes

`drain` does not remove water inside waterlogged blocks (e.g. stairs). `smooth` works on the highest non-air block of each selection column. `regen` (regenerating an area) is not implemented: mods have no access to the world generator.

## Patterns and limits

- `stone` = one block; `0` = air; `stone,dirt` = even mix; `30%stone,dirt` = 30% stone, the rest dirt. The `minecraft:` prefix is optional.
- By default at most 500,000 blocks per command, shape radius and size up to 128, a brush reaches 64 blocks and history keeps 20 edits, and it resets when you leave the world. Only an operator of level 4 changes the limits (`/we limit`). A brush may reach 256 blocks: `/we limit distance 256`.

---

## ⚠️ Known limitations

- `regen` is not implemented: mods have no access to the world generator.
- `rotate` is named `/we rot` because vanilla already has `/rotate`.
- Block orientation (`facing`) is not rotated by `rot` / `flip`.
- Limits, physics mode and history are not saved between game launches.

## 🛠 Building

Built as a Horizon native mod (mixin bundle). Logic lives in `we_core.rs` (pure Rust, unit-tested with `cargo test`), game glue in `glue.rs`, and the drawing of the selection in `geometry.rs` and `render.rs`.

```bash
python3 native/tools/native.py check worldedit     # check against Horizon source
python3 native/tools/native.py package worldedit   # produce worldedit-<version>.hnmod
```

## 📄 License

MIT
