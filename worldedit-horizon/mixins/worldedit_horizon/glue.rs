// WorldEdit in the server's game: the edges between the game's types and
// the rules (we_core). Added to the game as `game::we_glue`; the helpers
// (register.rs, command.rs, destroy_block.rs, right_click.rs, lifecycle.rs)
// call it from the hook points.

use crate::game::{
    Game,
    native_server::{NativeCommand, NativeServerPlayer},
    we_core::{
        self, Clipboard, Cuboid, Dir, Pattern, Pos, Session, Sessions, World,
    },
};
use horizon_client::text::{self, Text};
use horizon_world::registry::BlockStateId;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// The mod's id, name and version.
pub const ID: &str = "worldedit";
pub const NAME: &str = "WorldEdit";
pub const VERSION: &str = "0.6.2";

/// The item that selects: left click sets the first corner, right click the
/// second.
pub const WAND: &str = "minecraft:wooden_axe";
/// WorldEdit works only in creative mode (`/gamemode creative`).
fn creative(player: &NativeServerPlayer) -> bool {
    player.game_mode == 1
}
const CREATIVE_ONLY: &str = "WorldEdit works only in creative mode (/gamemode creative).";
const NOT_OPERATOR: &str = "WorldEdit needs operator permission level 2 or more.";
const NOT_ADMIN: &str = "/we limit needs operator permission level 4 or more.";

/// Whether the player may edit: an operator (level 2 or more) in creative mode.
fn may_edit(player: &NativeServerPlayer) -> bool {
    we_core::permitted(player.permission, None) && creative(player)
}

/// Whether edits tell neighbouring blocks (`/we physics on`): then torches,
/// plants and the like pop off as items. Off by default, as in WorldEdit.
static PHYSICS: AtomicBool = AtomicBool::new(false);

fn write_flags() -> i32 {
    use horizon_world::level::flags;
    if PHYSICS.load(Ordering::Relaxed) {
        flags::UPDATE_ALL
    } else {
        flags::UPDATE_CLIENTS | flags::UPDATE_KNOWN_SHAPE | flags::UPDATE_SUPPRESS_DROPS
    }
}

static SESSIONS: Mutex<Option<Sessions>> = Mutex::new(None);

fn with_sessions<R>(f: impl FnOnce(&mut Sessions) -> R) -> R {
    let mut guard = SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(Sessions::default))
}

/// The world closed: every player's selection, clipboard and history go.
pub fn closed() {
    let mut guard = SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    *guard = None;
}

/// The level's blocks, written through the game with neighbour updates.
struct GameWorld<'a, 'g> {
    game: &'a mut Game<'g>,
    dimension: String,
}

impl World for GameWorld<'_, '_> {
    fn get(&self, p: Pos) -> u32 {
        self.game.world.block(p[0], p[1], p[2]).0
    }
    fn set(&mut self, p: Pos, block: u32) -> bool {
        self.game.native_set_block(
            &self.dimension,
            p,
            BlockStateId(block),
            write_flags(),
        )
    }
}

fn block_id(game: &Game<'_>, name: &str) -> Option<u32> {
    let name = name.to_ascii_lowercase();
    // `0` is air, as in WorldEdit: `//set 0` clears the selection.
    let name = if name == "0" { "air".to_owned() } else { name };
    let full = if name.contains(':') {
        name
    } else {
        format!("minecraft:{name}")
    };
    game.pack.assets.registry.default_state(&full).map(|s| s.0)
}

fn feet(player: &NativeServerPlayer) -> Pos {
    [
        player.position[0].floor() as i32,
        player.position[1].floor() as i32,
        player.position[2].floor() as i32,
    ]
}

fn reply(game: &mut Game<'_>, line: String) {
    game.native_feedback(Text::literal(line));
}

fn fail(game: &mut Game<'_>, line: String) {
    game.native_feedback(Text::literal(line).with_color(text::RED));
}

/// The commands, in every player's command tree, typed `/we set 0`.
const USAGES: &[&str] = &[
    "we",
    "we wand",
    "we drain",
    "we fixwater",
    "we fixlava",
    "we smooth [iterations:integer]",
    "we physics [state:word]",
    "we overlay <pattern:word>",
    "we naturalize",
    "we line <pattern:word>",
    "we hollow",
    "we outset <amount:integer>",
    "we inset <amount:integer>",
    "we chunk",
    "we limit [what:word] [value:integer]",
    "we pos1",
    "we pos2",
    "we sel",
    "we desel",
    "we size",
    "we expand <amount:integer> [direction:word]",
    "we contract <amount:integer> [direction:word]",
    "we shift <amount:integer> [direction:word]",
    "we set <pattern:word>",
    "we replace <from:word> <to:word>",
    "we walls <pattern:word>",
    "we faces <pattern:word>",
    "we count <blocks:word>",
    "we sphere <pattern:word> <radius:integer> [hollow:boolean]",
    "we hsphere <pattern:word> <radius:integer>",
    "we cyl <pattern:word> <radius:integer> [height:integer] [hollow:boolean]",
    "we hcyl <pattern:word> <radius:integer> [height:integer]",
    "we pyramid <pattern:word> <size:integer>",
    "we hpyramid <pattern:word> <size:integer>",
    "we copy",
    "we cut",
    "we paste [skipair:boolean]",
    "we rot <degrees:integer>",
    "we flip [direction:word]",
    "we clearclipboard",
    "we stack <count:integer> [direction:word]",
    "we move <distance:integer> [direction:word]",
    "we undo [times:integer]",
    "we redo [times:integer]",
];

pub fn commands(out: &mut Vec<NativeCommand>) {
    for usage in USAGES {
        // Operators only: the level a player needs to see and run it.
        let sub = usage.split_whitespace().nth(1);
        out.push(NativeCommand::op(usage, we_core::level_for(sub.unwrap_or(""))));
    }
}

const HELP: &str = "WorldEdit (creative mode and operator level 2; /we limit level 4), commands start with /we: wand | pos1 | pos2 | sel (clear) | size | \
expand/contract/shift <n> <dir> | set <pattern> (0 = air) | replace [!]<from> <to> | overlay <pat> | naturalize | line <pat> | hollow | outset/inset <n> | chunk | walls <pattern> | faces <pattern> | \
count <blocks> | sphere/hsphere <pattern> <r> | cyl <pattern> <r> [h] | hcyl <pattern> <r> [h] | \
pyramid/hpyramid <pattern> <size> | copy | cut | paste [skipair] | rot <deg> | flip <dir> | \
clearclipboard | stack <n> <dir> | move <n> <dir> | undo [n] | redo [n] | physics [on|off] | drain | fixwater | fixlava | smooth [n] | limit [blocks|radius|history <n>|reset]. \
Patterns: stone, 50%stone,dirt. Directions: north south east west up down.";

/// A player's command, when it is WorldEdit's.
pub fn perform(
    game: &mut Game<'_>,
    player: &NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    let mut all = command.split_whitespace();
    if all.next() != Some("we") {
        return;
    }
    let args: Vec<&str> = all.collect();
    *ran = true;
    if !we_core::permitted(player.permission, None) {
        return fail(game, NOT_OPERATOR.to_owned());
    }
    if !creative(player) {
        return fail(game, CREATIVE_ONLY.to_owned());
    }
    let outcome = with_sessions(|sessions| {
        let session = sessions.get(player.uuid);
        run(game, player, session, &args)
    });
    match outcome {
        Ok(line) => reply(game, line),
        Err(line) => fail(game, line),
    }
}

fn number(args: &[&str], at: usize) -> Result<i32, String> {
    args.get(at)
        .and_then(|a| a.parse().ok())
        .ok_or_else(|| "A number is needed.".to_owned())
}

/// The way the player looks: up or down when steep, else the compass side.
fn facing(player: &NativeServerPlayer) -> Dir {
    if player.pitch > 45.0 {
        Dir::Down
    } else if player.pitch < -45.0 {
        Dir::Up
    } else {
        let yaw = player.yaw.rem_euclid(360.0);
        if !(45.0..315.0).contains(&yaw) {
            Dir::South
        } else if yaw < 135.0 {
            Dir::West
        } else if yaw < 225.0 {
            Dir::North
        } else {
            Dir::East
        }
    }
}

fn direction(args: &[&str], at: usize, player: &NativeServerPlayer) -> Result<Dir, String> {
    let Some(word) = args.get(at).copied() else {
        return Ok(facing(player));
    };
    Dir::parse(word)
        .ok_or_else(|| format!("`{word}` is not a direction (north south east west up down)."))
}

fn flag(args: &[&str], at: usize) -> bool {
    args.get(at).is_some_and(|a| *a == "true")
}

fn region(session: &Session, player: &NativeServerPlayer) -> Result<Cuboid, String> {
    let c = session
        .selection
        .cuboid()
        .ok_or_else(|| "Select two corners first (/we wand, or /we pos1 and /we pos2).".to_owned())?;
    if session.selection.dimension != player.dimension {
        return Err("Your selection is in another dimension.".to_owned());
    }
    Ok(c)
}

fn describe(c: &Cuboid) -> String {
    let s = c.size();
    format!(
        "{} x {} x {} = {} blocks, {:?} to {:?}",
        s[0],
        s[1],
        s[2],
        c.volume(),
        c.min,
        c.max
    )
}

fn pattern(game: &Game<'_>, text: &str) -> Result<Pattern, String> {
    Pattern::parse(text, &|name| block_id(game, name))
}

fn run(
    game: &mut Game<'_>,
    player: &NativeServerPlayer,
    session: &mut Session,
    args: &[&str],
) -> Result<String, String> {
    let Some(&sub) = args.first() else {
        return Ok(HELP.to_owned());
    };
    let dimension = player.dimension.clone();
    let origin = feet(player);
    let air = block_id(game, "air").unwrap_or(0);
    match sub {
        "help" => Ok(HELP.to_owned()),
        "physics" => {
            match args.get(1).copied() {
                Some("on") => PHYSICS.store(true, Ordering::Relaxed),
                Some("off") => PHYSICS.store(false, Ordering::Relaxed),
                Some(_) => return Err("physics on|off.".to_owned()),
                None => {}
            }
            Ok(format!(
                "Physics are {}: neighbours {} updated, so dropped items {}.",
                if PHYSICS.load(Ordering::Relaxed) { "on" } else { "off" },
                if PHYSICS.load(Ordering::Relaxed) { "are" } else { "are not" },
                if PHYSICS.load(Ordering::Relaxed) { "can appear" } else { "do not appear" },
            ))
        }
        "limit" if !we_core::permitted(player.permission, Some("limit")) => {
            Err(NOT_ADMIN.to_owned())
        }
        "limit" => match args.get(1) {
            None => Ok(we_core::limits()),
            Some(what) => we_core::set_limit(what, args.get(2).and_then(|v| v.parse().ok())),
        },
        "wand" => match game.native_give_items(WAND, 1) {
            0 => Err("Your inventory is full.".to_owned()),
            _ => Ok("Left click a block for the first corner, right click for the second.".to_owned()),
        },
        "pos1" | "pos2" => {
            let index = if sub == "pos1" { 1 } else { 2 };
            session.selection.set_pos(index, &dimension, origin);
            Ok(format!("Position {index} set to {origin:?}."))
        }
        "drain" | "fixwater" | "fixlava" => {
            let c = region(session, player)?;
            we_core::check_limit(c.volume())?;
            let source = match sub {
                "fixwater" => block_id(game, "water"),
                "fixlava" => block_id(game, "lava"),
                _ => None,
            };
            let mut known: std::collections::HashMap<u32, bool> = Default::default();
            let mut hits = Vec::new();
            for p in c.positions() {
                let id = game.world.block(p[0], p[1], p[2]).0;
                let hit = *known.entry(id).or_insert_with(|| {
                    let name = game.res.facts().get(BlockStateId(id)).name.to_string();
                    match sub {
                        "fixwater" => name == "minecraft:water",
                        "fixlava" => name == "minecraft:lava",
                        _ => matches!(
                            name.as_str(),
                            "minecraft:water"
                                | "minecraft:lava"
                                | "minecraft:bubble_column"
                                | "minecraft:kelp"
                                | "minecraft:kelp_plant"
                                | "minecraft:seagrass"
                                | "minecraft:tall_seagrass"
                        ),
                    }
                });
                if hit && Some(id) != source {
                    hits.push(p);
                }
            }
            let to = Pattern::single(source.unwrap_or(air));
            let edit = {
                let mut w = GameWorld { game, dimension };
                we_core::fill(&mut w, &hits, &to)
            };
            let n = edit.len();
            session.record(edit);
            Ok(format!("{n} blocks changed."))
        }
        "smooth" => {
            let c = region(session, player)?;
            we_core::check_limit(c.volume())?;
            let times = if args.len() > 1 {
                number(args, 1)?.clamp(1, 50) as u32
            } else {
                1
            };
            let edit = {
                let mut w = GameWorld { game, dimension };
                let writes = we_core::smooth_writes(&w, &c, air, times);
                we_core::apply(&mut w, writes)
            };
            let n = edit.len();
            session.record(edit);
            Ok(format!("{n} blocks changed."))
        }
        "outset" | "inset" => {
            let n = number(args, 1)?;
            let c = region(session, player)?;
            let n = if sub == "inset" { -n } else { n };
            let next = c
                .grown(n)
                .ok_or_else(|| "The selection would be empty.".to_owned())?;
            session.selection.set_cuboid(next);
            Ok(describe(&next))
        }
        "chunk" => {
            let x = origin[0].div_euclid(16) * 16;
            let z = origin[2].div_euclid(16) * 16;
            let c = Cuboid::new([x, -64, z], [x + 15, 319, z + 15]);
            session.selection.set_pos(1, &dimension, c.min);
            session.selection.set_cuboid(c);
            Ok(describe(&c))
        }
        "line" => {
            let (Some(a), Some(b)) = (session.selection.pos1, session.selection.pos2) else {
                return Err("Set //pos1 and //pos2 first.".to_owned());
            };
            let pattern = pattern(game, args.get(1).copied().unwrap_or_default())?;
            let positions = we_core::line(a, b)?;
            let edit = {
                let mut w = GameWorld { game, dimension };
                we_core::fill(&mut w, &positions, &pattern)
            };
            let n = edit.len();
            session.record(edit);
            Ok(format!("{n} blocks changed."))
        }
        "hollow" => {
            let c = region(session, player)?;
            we_core::check_limit(c.volume())?;
            let inner = c
                .grown(-1)
                .ok_or_else(|| "The selection is too thin to hollow.".to_owned())?;
            let edit = {
                let mut w = GameWorld { game, dimension };
                we_core::fill(&mut w, &inner.positions(), &Pattern::single(air))
            };
            let n = edit.len();
            session.record(edit);
            Ok(format!("{n} blocks removed."))
        }
        "overlay" | "naturalize" => {
            let c = region(session, player)?;
            we_core::check_limit(c.volume())?;
            let p = if sub == "overlay" {
                Some(pattern(game, args.get(1).copied().unwrap_or_default())?)
            } else {
                None
            };
            let ids = (
                block_id(game, "grass_block").unwrap_or(air),
                block_id(game, "dirt").unwrap_or(air),
                block_id(game, "stone").unwrap_or(air),
            );
            let writes = {
                let w = GameWorld {
                    game,
                    dimension: dimension.clone(),
                };
                match &p {
                    Some(p) => we_core::overlay_writes(&w, &c, air, p),
                    None => we_core::naturalize_writes(&w, &c, air, ids),
                }
            };
            let edit = {
                let mut w = GameWorld { game, dimension };
                we_core::apply(&mut w, writes)
            };
            let n = edit.len();
            session.record(edit);
            Ok(format!("{n} blocks changed."))
        }
        "sel" | "desel" => {
            session.selection.clear();
            Ok("Selection cleared.".to_owned())
        }
        "size" => Ok(describe(&region(session, player)?)),
        "expand" | "contract" | "shift" => {
            let amount = number(args, 1)?;
            let dir = direction(args, 2, player)?;
            let c = region(session, player)?;
            let next = match sub {
                "expand" => c.expand(dir, amount),
                "contract" => c.contract(dir, amount),
                _ => {
                    let v = dir.vector();
                    Some(c.shifted([v[0] * amount, v[1] * amount, v[2] * amount]))
                }
            }
            .ok_or_else(|| "The selection would be empty.".to_owned())?;
            session.selection.set_cuboid(next);
            Ok(describe(&next))
        }
        "set" | "walls" | "faces" => {
            let c = region(session, player)?;
            we_core::check_limit(c.volume())?;
            let pattern = pattern(game, args.get(1).copied().unwrap_or_default())?;
            let positions = match sub {
                "set" => c.positions(),
                "walls" => c.walls(),
                _ => c.faces(),
            };
            let edit = {
                let mut w = GameWorld { game, dimension };
                we_core::fill(&mut w, &positions, &pattern)
            };
            let n = edit.len();
            session.record(edit);
            Ok(format!("{n} blocks changed."))
        }
        "replace" => {
            let c = region(session, player)?;
            we_core::check_limit(c.volume())?;
            let text = args.get(1).copied().unwrap_or_default();
            let (inverse, text) = match text.strip_prefix('!') {
                Some(rest) => (true, rest),
                None => (false, text),
            };
            let from = we_core::parse_blocks(text, &|n| block_id(game, n))?;
            let to = pattern(game, args.get(2).copied().unwrap_or_default())?;
            let edit = {
                let mut w = GameWorld { game, dimension };
                if inverse {
                    we_core::replace_except(&mut w, &c.positions(), &from, &to)
                } else {
                    we_core::replace(&mut w, &c.positions(), &from, &to)
                }
            };
            let n = edit.len();
            session.record(edit);
            Ok(format!("{n} blocks replaced."))
        }
        "count" => {
            let c = region(session, player)?;
            we_core::check_limit(c.volume())?;
            let of = we_core::parse_blocks(args.get(1).copied().unwrap_or_default(), &|n| {
                block_id(game, n)
            })?;
            let w = GameWorld { game, dimension };
            Ok(format!(
                "Counted {}.",
                we_core::count(&w, &c.positions(), &of)
            ))
        }
        "sphere" | "hsphere" | "cyl" | "hcyl" | "pyramid" | "hpyramid" => {
            let pattern = pattern(game, args.get(1).copied().unwrap_or_default())?;
            let size = number(args, 2)?;
            let height = || match args.get(3) {
                Some(a) if *a != "true" && *a != "false" => number(args, 3),
                _ => Ok(1),
            };
            let positions = match sub {
                "sphere" => we_core::sphere(origin, size, flag(args, 3))?,
                "hsphere" => we_core::sphere(origin, size, true)?,
                "cyl" => we_core::cylinder(origin, size, height()?, flag(args, 4))?,
                "hcyl" => we_core::hollow_cylinder(origin, size, height()?)?,
                "pyramid" => we_core::pyramid(origin, size, false)?,
                _ => we_core::pyramid(origin, size, true)?,
            };
            let edit = {
                let mut w = GameWorld { game, dimension };
                we_core::fill(&mut w, &positions, &pattern)
            };
            let n = edit.len();
            session.record(edit);
            Ok(format!("{n} blocks changed."))
        }
        "copy" | "cut" => {
            let c = region(session, player)?;
            we_core::check_limit(c.volume())?;
            let mut w = GameWorld { game, dimension };
            let clip = Clipboard::copy(&w, &c, origin);
            let n = clip.blocks.len();
            session.clipboard = Some(clip);
            if sub == "cut" {
                let edit = we_core::fill(&mut w, &c.positions(), &Pattern::single(air));
                session.record(edit);
                Ok(format!("{n} blocks cut."))
            } else {
                Ok(format!("{n} blocks copied."))
            }
        }
        "paste" => {
            let clip = session
                .clipboard
                .as_ref()
                .ok_or_else(|| "The clipboard is empty (//copy).".to_owned())?;
            let skip = flag(args, 1).then_some(air);
            let writes = clip.writes(origin, skip);
            we_core::check_limit(writes.len() as u64)?;
            let edit = {
                let mut w = GameWorld { game, dimension };
                we_core::apply(&mut w, writes)
            };
            let n = edit.len();
            session.record(edit);
            Ok(format!("{n} blocks pasted."))
        }
        "rot" | "rotate" => {
            let degrees = number(args, 1)?;
            let clip = session
                .clipboard
                .as_mut()
                .ok_or_else(|| "The clipboard is empty (//copy).".to_owned())?;
            clip.rotate(degrees)?;
            Ok(format!("Clipboard rotated {degrees} degrees."))
        }
        "flip" => {
            let dir = direction(args, 1, player)?;
            let clip = session
                .clipboard
                .as_mut()
                .ok_or_else(|| "The clipboard is empty (//copy).".to_owned())?;
            clip.flip(dir);
            Ok("Clipboard flipped.".to_owned())
        }
        "clearclipboard" => {
            session.clipboard = None;
            Ok("Clipboard cleared.".to_owned())
        }
        "stack" | "move" => {
            let n = number(args, 1)?;
            let dir = direction(args, 2, player)?;
            let c = region(session, player)?;
            we_core::check_limit(c.volume())?;
            let mut w = GameWorld { game, dimension };
            let writes = if sub == "stack" {
                we_core::stack_writes(&w, &c, dir, n)?
            } else {
                we_core::move_writes(&w, &c, dir, n, air)?
            };
            let edit = we_core::apply(&mut w, writes);
            let changed = edit.len();
            session.record(edit);
            if sub == "move" {
                let v = dir.vector();
                session
                    .selection
                    .set_cuboid(c.shifted([v[0] * n, v[1] * n, v[2] * n]));
            }
            Ok(format!("{changed} blocks changed."))
        }
        "undo" | "redo" => {
            let times = if args.len() > 1 {
                number(args, 1)?.max(1) as usize
            } else {
                1
            };
            let mut w = GameWorld { game, dimension };
            let (edits, blocks) = if sub == "undo" {
                session.undo(&mut w, times)
            } else {
                session.redo(&mut w, times)
            };
            if edits == 0 {
                Err(format!("Nothing to {sub}."))
            } else {
                let done = if sub == "undo" { "undone" } else { "redone" };
                Ok(format!("{edits} edits {done}, {blocks} blocks."))
            }
        }
        other => Err(format!("Unknown WorldEdit command `{other}` (/we).")),
    }
}

/// The wand in the acting player's hand: the player, when it is one with
/// the level for it.
fn wand_player(game: &Game<'_>) -> Option<NativeServerPlayer> {
    let (item, _, _) = game.native_held()?;
    if item != WAND {
        return None;
    }
    game.native_acting_player().filter(may_edit)
}

/// A click of the wand on the block at `pos`: sets corner `index`, tells the
/// player over the hotbar, and says whether it was the wand's (the click
/// then does nothing else).
pub fn wand_click(game: &mut Game<'_>, index: u8, pos: [i32; 3]) -> bool {
    let Some(player) = wand_player(game) else {
        return false;
    };
    let size = with_sessions(|sessions| {
        let selection = &mut sessions.get(player.uuid).selection;
        selection.set_pos(index, &player.dimension, pos);
        selection.cuboid().map(|c| c.volume())
    });
    let mut line = format!("Position {index} set to {pos:?}");
    if let Some(volume) = size {
        line.push_str(&format!(" ({volume} blocks)"));
    }
    game.native_action_bar(player.uuid, Text::literal(line));
    true
}

#[allow(dead_code)]
fn unused(_: &str) {
    let _ = (ID, NAME, VERSION);
}
