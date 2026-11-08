// WorldEdit in the server's game: the edges between the game's types and
// the rules (we_core). Added to the game as `game::we_glue`; the helpers
// (register.rs, command.rs, destroy_block.rs, right_click.rs, lifecycle.rs)
// call it from the hook points.

use crate::game::{
    Game,
    native_server::{NativeCommand, NativeServerPlayer},
    we_core::{
        self, Clipboard, Dir, Pattern, Pos, Region, Session, Sessions, Shape, World,
    },
};
use horizon_client::text::{self, Text};
use horizon_world::registry::BlockStateId;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// The mod's id, name and version.
pub const ID: &str = "worldedit";
pub const NAME: &str = "WorldEdit";
pub const VERSION: &str = "1.1.0";

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

/// The sessions of every player. The lock is held while a command runs, so
/// nothing inside a command may take it again: that would deadlock the
/// server.
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
    // `0` is air, as in WorldEdit: `/we set 0` clears the selection.
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
    "we help",
    "we wand",
    "we brush [args:text]",
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
    "we chunkinfo",
    "we listchunks",
    "we limit [what:word] [value:integer]",
    "we pos1 [coordinates:word]",
    "we pos2 [coordinates:word]",
    "we pos",
    "we hpos1",
    "we hpos2",
    "we sel [args:text]",
    "we grid [state:word]",
    "we desel",
    "we size",
    "we trim",
    "we distr",
    "we expand <amount:integer> [direction:word]",
    "we contract <amount:integer> [direction:word]",
    "we shift <amount:integer> [direction:word]",
    "we set <pattern:word>",
    "we replace <from:word> <to:word>",
    "we walls <pattern:word>",
    "we faces <pattern:word>",
    "we count <blocks:word>",
    "we fill <pattern:word> <radius:integer> [depth:integer]",
    "we fillr <pattern:word> <radius:integer> [depth:integer]",
    "we removeabove [size:integer] [height:integer]",
    "we removebelow [size:integer] [height:integer]",
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
        let sub = usage.split_whitespace().nth(1).unwrap_or("");
        out.push(NativeCommand::op(usage, we_core::level_for(sub)));
    }
}

const HELP: &str = "WorldEdit (creative mode and operator level 2; /we limit level 4): every command is /we <command>, as /we set stone. \
wand | brush <shape> [sizes] <pattern> | brush radius <n> | brush distance <n> | brush at | brush erase at (off) | pos1 | pos2 | pos | hpos1 | hpos2 | sel (clear, or sel cube|sphere|cylinder|ellipsoid [sizes]) | grid on|off (the grid inside the selection, for you alone) | size | trim | distr | chunk | chunkinfo | listchunks | \
expand/contract/shift <n> <dir> | outset/inset <n> | set <pattern> (0 = air) | replace [!]<from> <to> | overlay <pat> | naturalize | line <pat> | hollow | chunk | walls <pattern> | faces <pattern> | \
count <blocks> | fill/fillr <pattern> <radius> [depth] | removeabove/removebelow [size] [height] | sphere/hsphere <pattern> <r> | cyl <pattern> <r> [h] | hcyl <pattern> <r> [h] | \
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
    let Some(first) = all.next() else {
        return;
    };
    // Every command is under `/we`: the server cuts the slashes of a command
    // line away, so `//we set stone` reaches the mod as `we set stone`.
    if first != "we" {
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
        let before = we_core::selection_state(&session.selection);
        let result = run(game, player, session, &args);
        // The client draws the selection: it needs to hear about every change.
        if before != we_core::selection_state(&session.selection) {
            let payload = we_core::selection_payload(&session.selection);
            let host: &mut dyn crate::mod_api::Host = game;
            let _ = host.net_send_to_player(player.uuid, we_core::SEL_CHANNEL, &payload);
        }
        result
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

/// A number in a word, as `+2` or `-1` in `/we brush radius`.
fn a_number(text: &str) -> Result<i32, String> {
    text.parse::<i32>()
        .map_err(|_| format!("`{text}` is not a number."))
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

/// The block the player looks at (vanilla's `//hpos1` and `//hpos2`).
fn targeted(game: &Game<'_>) -> Result<Pos, String> {
    let hit = game
        .native_raycast(64.0)
        .ok_or_else(|| "Look at a block first.".to_owned())?;
    Ok([hit.block_x, hit.block_y, hit.block_z])
}

fn region(session: &Session, player: &NativeServerPlayer) -> Result<Region, String> {
    let c = session
        .selection
        .region()
        .ok_or_else(|| "Select two corners first (/we wand, or /we pos1 and /we pos2).".to_owned())?;
    if session.selection.dimension != player.dimension {
        return Err("Your selection is in another dimension.".to_owned());
    }
    Ok(c)
}

fn describe(c: &Region) -> String {
    let s = c.size();
    let shape = match c.shape {
        Shape::Cube => format!("{} x {} x {} = {} blocks", s[0], s[1], s[2], c.volume()),
        Shape::Sphere => format!("sphere r={} = {} blocks", c.sizes[0], c.volume()),
        Shape::Cylinder => format!(
            "cylinder r={} h={} = {} blocks",
            c.sizes[0],
            c.sizes[1],
            c.volume()
        ),
        Shape::Ellipsoid => format!(
            "ellipsoid {}x{}x{} = {} blocks",
            c.sizes[0],
            c.sizes[1],
            c.sizes[2],
            c.volume()
        ),
    };
    format!("{shape}, {:?} to {:?}", c.min, c.max)
}

fn pattern(game: &Game<'_>, text: &str) -> Result<Pattern, String> {
    Pattern::parse(text, &|name| block_id(game, name))
}

/// The message when the player has no brush.
const NO_BRUSH: &str = "No brush. `/we brush sphere 5 stone` arms one.";

/// `/we brush ...`: the brush the player paints with.
fn brush_command(
    game: &mut Game<'_>,
    player: &NativeServerPlayer,
    session: &mut Session,
    text: &str,
) -> Result<String, String> {
    if text.is_empty() {
        return Ok(match &session.brush {
            Some(b) => format!("The brush is {}.", b.describe()),
            None => NO_BRUSH.to_owned(),
        });
    }
    if text.eq_ignore_ascii_case("off") {
        session.brush = None;
        return Ok("The brush is off. The wand selects the corners again.".to_owned());
    }
    let mut words = text.split_whitespace();
    let first = words.next().unwrap_or_default();
    let rest: Vec<&str> = words.collect();
    // The size of the armed brush: `brush radius 7`.
    if first.eq_ignore_ascii_case("radius") || first.eq_ignore_ascii_case("size") {
        let Some(brush) = session.brush.clone() else {
            return Err(NO_BRUSH.to_owned());
        };
        let Some(word) = rest.first() else {
            return Ok(format!(
                "The brush is {} (radius {}).",
                brush.describe(),
                brush.radius()
            ));
        };
        let radius = if let Some(delta) = word.strip_prefix('+') {
            brush.radius() + a_number(delta)?
        } else if let Some(delta) = word.strip_prefix('-') {
            brush.radius() - a_number(delta)?
        } else {
            a_number(word)?
        };
        let brush = brush.with_radius(radius)?;
        let line = format!("The brush is {}.", brush.describe());
        session.brush = Some(brush);
        return Ok(line);
    }
    // How far ahead of the player the brush works: `brush distance 20`.
    if first.eq_ignore_ascii_case("distance") {
        let Some(brush) = session.brush.clone() else {
            return Err(NO_BRUSH.to_owned());
        };
        let Some(word) = rest.first() else {
            return Ok(format!("The brush is {}.", brush.describe()));
        };
        let distance = if word.eq_ignore_ascii_case("off") || *word == "0" {
            None
        } else {
            Some(a_number(word)?)
        };
        let brush = brush.with_distance(distance)?;
        let line = match brush.distance {
            Some(_) => format!("The brush is {}.", brush.describe()),
            None => format!("The brush is {}. It works where you use it.", brush.describe()),
        };
        session.brush = Some(brush);
        return Ok(line);
    }
    // Paint now, at the block under the view: `brush at`, `brush erase at`.
    if first.eq_ignore_ascii_case("at") || first.eq_ignore_ascii_case("erase") {
        let erase = first.eq_ignore_ascii_case("erase");
        if erase && !rest.iter().any(|w| w.eq_ignore_ascii_case("at")) {
            return Err("`/we brush erase at` erases at the block under your view.".to_owned());
        }
        let Some(brush) = session.brush.clone() else {
            return Err(NO_BRUSH.to_owned());
        };
        let Some(target) = brush_target(game, player, &brush, None) else {
            return Err("Look at a block first.".to_owned());
        };
        let edit = paint_brush(game, player, &brush, target, erase)?;
        let n = edit.len();
        // The command already holds the lock of the sessions.
        session.record(edit);
        return Ok(format!(
            "{n} blocks {} at {target:?}.",
            if erase { "erased" } else { "painted" }
        ));
    }
    // A new brush: `brush sphere 5 stone`.
    let brush = {
        let g: &Game<'_> = game;
        we_core::Brush::parse(text, &|name| block_id(g, name))?
    };
    let line = format!(
        "The brush is {}. Click a block to paint it; `/we brush off` stops it.",
        brush.describe()
    );
    session.brush = Some(brush);
    Ok(line)
}

/// Where the brush works: the block it was used on, `distance` blocks ahead
/// of the player, or the block under the view.
fn brush_target(
    game: &Game<'_>,
    player: &NativeServerPlayer,
    brush: &we_core::Brush,
    used: Option<Pos>,
) -> Option<Pos> {
    // A distance ahead of the player wins.
    if let Some(distance) = brush.distance {
        let eye = [
            player.position[0],
            player.position[1] + 1.62,
            player.position[2],
        ];
        return Some(we_core::ahead(
            eye,
            we_core::look(f64::from(player.yaw), f64::from(player.pitch)),
            f64::from(distance),
        ));
    }
    if let Some(pos) = used {
        return Some(pos);
    }
    // The block under the view, up to 64 blocks away.
    game.native_raycast(64.0)
        .map(|hit| [hit.block_x, hit.block_y, hit.block_z])
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
        "brush" => brush_command(game, player, session, &args[1..].join(" ")),
        "pos1" | "pos2" => {
            let index = if sub == "pos1" { 1 } else { 2 };
            // With a position of its own, as in vanilla: `//pos1 10,64,-5`.
            let pos = match args.get(1) {
                Some(text) => we_core::parse_pos(text)
                    .ok_or_else(|| format!("`{text}` is not a position (x,y,z)."))?,
                None => origin,
            };
            session.selection.set_pos(index, &dimension, pos);
            let mut line = format!("Position {index} set to {pos:?}.");
            // With both corners the selection is known: say what it is now,
            // so that a shape the player chose is not left to guesswork.
            if let Some(c) = session.selection.region() {
                line.push(' ');
                line.push_str(&describe(&c));
            }
            Ok(line)
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
            session.selection.set_region(&next);
            Ok(describe(&next))
        }
        "chunk" => {
            let x = origin[0].div_euclid(16) * 16;
            let z = origin[2].div_euclid(16) * 16;
            let c = Region::cube([x, -64, z], [x + 15, 319, z + 15]);
            session.selection.dimension = dimension.clone();
            session.selection.set_region(&c);
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
        "desel" => {
            session.selection.clear();
            Ok("Selection cleared.".to_owned())
        }
        "sel" => {
            let Some(word) = args.get(1).copied() else {
                session.selection.clear();
                return Ok("Selection cleared.".to_owned());
            };
            let Some(shape) = Shape::parse(word) else {
                return Err(format!(
                    "`{word}` is not a shape (cube, sphere, cylinder, ellipsoid)."
                ));
            };
            let sizes: Vec<i32> = args[2..].iter().filter_map(|a| a.parse().ok()).collect();
            session.selection.dimension = dimension.clone();
            if sizes.is_empty() {
                // The shape for the corners the player sets next.
                session.selection.shape = shape;
                // With the corners already set, the shape applies to them at once.
                if let Some(r) = session.selection.region() {
                    return Ok(describe(&r));
                }
                return Ok(format!(
                    "The selection is a {} now. Set its corners with /we pos1 and /we pos2.",
                    shape.name()
                ));
            }
            let c = Region::around(shape, origin, &sizes)?;
            session.selection.set_region(&c);
            Ok(describe(&c))
        }
        "grid" => {
            // The player's own view of the selection: the grid inside it can
            // be turned off, the frame around it stays either way.
            let on = match args.get(1).copied() {
                Some("on") => true,
                Some("off") => false,
                Some(word) => return Err(format!("`{word}` is not on or off.")),
                None => !session.selection.grid,
            };
            session.selection.grid = on;
            Ok(format!(
                "The grid inside the selection is {} for you.",
                if on { "on" } else { "off" }
            ))
        }
        "chunkinfo" => {
            let chunk = we_core::chunk_of(origin);
            let filled = {
                let w = GameWorld {
                    game,
                    dimension: dimension.clone(),
                };
                we_core::chunk_filled(&w, chunk, -64, 319, air)
            };
            Ok(format!(
                "Chunk [{}, {}] at {}..{}, {}..{} (y -64..319): {filled} blocks that are not air.",
                chunk.0,
                chunk.1,
                chunk.0 * 16,
                chunk.0 * 16 + 15,
                chunk.1 * 16,
                chunk.1 * 16 + 15
            ))
        }
        "listchunks" => {
            let c = region(session, player)?;
            let chunks = we_core::chunks_of(&c);
            let shown: Vec<String> = chunks
                .iter()
                .take(64)
                .map(|(cx, cz)| format!("[{cx}, {cz}]"))
                .collect();
            let mut line = format!(
                "{} chunks in the selection: {}",
                chunks.len(),
                shown.join(", ")
            );
            if chunks.len() > shown.len() {
                line.push_str(" ...");
            }
            Ok(line)
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
            session.selection.set_region(&next);
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
        "fill" | "fillr" => {
            let pattern = pattern(game, args.get(1).copied().unwrap_or_default())?;
            let radius = number(args, 2)?;
            let depth = if args.len() > 3 { number(args, 3)? } else { 1 };
            let writes = {
                let w = GameWorld {
                    game,
                    dimension: dimension.clone(),
                };
                we_core::fill_hole(&w, origin, radius, depth, &pattern, sub == "fillr", air)?
            };
            let edit = {
                let mut w = GameWorld { game, dimension };
                we_core::apply(&mut w, writes)
            };
            let n = edit.len();
            session.record(edit);
            Ok(format!("{n} blocks filled."))
        }
        "removeabove" | "removebelow" => {
            let size = if args.len() > 1 {
                number(args, 1)?.clamp(0, 32)
            } else {
                1
            };
            let height = if args.len() > 2 {
                number(args, 2)?.clamp(1, 256)
            } else {
                64
            };
            let positions = {
                let w = GameWorld {
                    game,
                    dimension: dimension.clone(),
                };
                we_core::remove_column(&w, origin, size, height, sub == "removeabove", air)
            };
            we_core::check_limit(positions.len() as u64)?;
            let edit = {
                let mut w = GameWorld { game, dimension };
                we_core::fill(&mut w, &positions, &Pattern::single(air))
            };
            let n = edit.len();
            session.record(edit);
            Ok(format!("{n} blocks removed."))
        }
        "hpos1" | "hpos2" => {
            let index = if sub == "hpos1" { 1 } else { 2 };
            let target = targeted(game)?;
            session.selection.set_pos(index, &dimension, target);
            Ok(format!("Position {index} set to {target:?}."))
        }
        "pos" => {
            let target = targeted(game)?;
            session.selection.set_pos(1, &dimension, origin);
            session.selection.set_pos(2, &dimension, target);
            Ok(format!(
                "Position 1 set to {origin:?}, position 2 to {target:?}."
            ))
        }
        "trim" => {
            let c = region(session, player)?;
            let trimmed = {
                let w = GameWorld {
                    game,
                    dimension: dimension.clone(),
                };
                we_core::trim_to_existing(&w, &c, air)
            }
            .ok_or_else(|| "The selection holds nothing but air.".to_owned())?;
            session.selection.set_region(&trimmed);
            Ok(describe(&trimmed))
        }
        "distr" => {
            let c = region(session, player)?;
            we_core::check_limit(c.volume())?;
            let dist = {
                let w = GameWorld {
                    game,
                    dimension: dimension.clone(),
                };
                we_core::distribution(&w, &c.positions())
            };
            let total: u64 = dist.iter().map(|(_, n)| n).sum();
            let mut lines = Vec::new();
            for (id, n) in dist.iter().take(10) {
                let name = game.res.facts().get(BlockStateId(*id)).name.to_string();
                lines.push(format!("{n} {name}"));
            }
            Ok(format!("{total} blocks: {}.", lines.join(", ")))
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
                    .set_region(&c.shifted([v[0] * n, v[1] * n, v[2] * n]));
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
    // With a brush armed, the right click erases instead of setting a corner.
    if index == 2 {
        let brush = with_sessions(|sessions| sessions.get(player.uuid).brush.clone());
        if let Some(brush) = brush {
            return brush_erase(game, &player, &brush, pos);
        }
    }
    set_corner(game, player.uuid, &player.dimension, index, pos);
    true
}

/// A click of the wand with a brush armed: it paints the brush on the block
/// at `pos`. It says whether the click was the brush's, so that the block is
/// not broken then.
pub fn brush_click(game: &mut Game<'_>, pos: [i32; 3]) -> bool {
    let Some(player) = wand_player(game) else {
        return false;
    };
    let Some(brush) = with_sessions(|sessions| sessions.get(player.uuid).brush.clone()) else {
        return false;
    };
    let Some(target) = brush_target(game, &player, &brush, Some(pos)) else {
        return false;
    };
    match paint_brush(game, &player, &brush, target, false) {
        Ok(edit) => {
            let n = edit.len();
            with_sessions(|sessions| sessions.get(player.uuid).record(edit));
            game.native_action_bar(
                player.uuid,
                Text::literal(format!("Brush: {n} blocks painted.")),
            );
        }
        Err(why) => fail(game, why),
    }
    true
}

/// The right click with a brush armed: it erases the brush's shape with air.
fn brush_erase(
    game: &mut Game<'_>,
    player: &NativeServerPlayer,
    brush: &we_core::Brush,
    pos: [i32; 3],
) -> bool {
    let Some(target) = brush_target(game, player, brush, Some(pos)) else {
        return false;
    };
    match paint_brush(game, player, brush, target, true) {
        Ok(edit) => {
            let n = edit.len();
            with_sessions(|sessions| sessions.get(player.uuid).record(edit));
            game.native_action_bar(
                player.uuid,
                Text::literal(format!("Brush: {n} blocks erased.")),
            );
        }
        Err(why) => fail(game, why),
    }
    true
}

/// The brush on the block at `target`: with its pattern, or with air when it
/// erases. It gives the edit back, so the caller can put it into the
/// history: the history is behind a lock, and a command already holds it.
fn paint_brush(
    game: &mut Game<'_>,
    player: &NativeServerPlayer,
    brush: &we_core::Brush,
    target: Pos,
    erase: bool,
) -> Result<we_core::Edit, String> {
    let positions = brush
        .positions(target)
        .map_err(|why| format!("The brush does not fit: {why}"))?;
    let pattern = if erase {
        let air = block_id(game, "air").unwrap_or(0);
        we_core::Pattern::single(air)
    } else {
        brush.pattern.clone()
    };
    let dimension = player.dimension.clone();
    let edit = {
        let mut w = GameWorld { game, dimension };
        we_core::fill(&mut w, &positions, &pattern)
    };
    Ok(edit)
}

/// Sets the corner `index` of the player's selection, tells them over the
/// hotbar and sends the new selection to their client, which draws it.
fn set_corner(game: &mut Game<'_>, player: u128, dimension: &str, index: u8, pos: [i32; 3]) {
    let (size, payload) = with_sessions(|sessions| {
        let selection = &mut sessions.get(player).selection;
        selection.set_pos(index, dimension, pos);
        let size = selection.region().map(|c| c.volume());
        (size, we_core::selection_payload(selection))
    });
    let mut line = format!("Position {index} set to {pos:?}");
    if let Some(volume) = size {
        line.push_str(&format!(" ({volume} blocks)"));
    }
    game.native_action_bar(player, Text::literal(line));
    // The client draws the selection.
    let host: &mut dyn crate::mod_api::Host = game;
    let _ = host.net_send_to_player(player, we_core::SEL_CHANNEL, &payload);
}

/// A client tells the server that the wand was used on a block: the server
/// owns the selection, so it sets the corner itself. A dedicated server
/// never hears the click itself, only the client's message.
pub fn wand_request(game: &mut Game<'_>, pos: [i32; 3]) {
    // Only the wand asks.
    let Some((item, _, _)) = game.native_held() else {
        return;
    };
    if item != WAND {
        return;
    }
    let host: &mut dyn crate::mod_api::Host = game;
    let _ = host.net_send_to_server(we_core::WAND_CHANNEL, &we_core::wand_payload(pos));
}

/// The server got a wand's click from a client: it checks the player and
/// sets the second corner.
pub fn wand_message(game: &mut Game<'_>, player: u128, data: &[u8]) {
    let Some(pos) = we_core::parse_wand(data) else {
        return;
    };
    let Some(p) = game.native_server_players().into_iter().find(|p| p.uuid == player) else {
        return;
    };
    // Only an operator in creative mode may use WorldEdit.
    if !we_core::permitted(p.permission, None) || p.game_mode != 1 {
        return;
    }
    // The click must be within reach: a client may ask for any block.
    if !within_reach(p.position, pos) && !brush_armed(player) {
        return;
    }
    // The wand in the player's hand.
    let host: &mut dyn crate::mod_api::Host = game;
    let Some(stacks) = host.player_stacks(player) else {
        return;
    };
    if !stacks.iter().flatten().take(9).any(|s| s.item.to_string() == WAND) {
        return;
    }
    // With a brush armed, the click erases instead of setting the corner.
    let brush = with_sessions(|sessions| sessions.get(player).brush.clone());
    if let Some(brush) = brush {
        brush_erase(game, &p, &brush, pos);
        return;
    }
    set_corner(game, player, &p.dimension, 2, pos);
}

/// Whether the player has a brush armed.
fn brush_armed(player: u128) -> bool {
    with_sessions(|sessions| sessions.get(player).brush.is_some())
}

/// Whether the block at `pos` is within the player's reach: six blocks, so
/// a click at the far side of the block the player looks at counts too.
fn within_reach(at: [f64; 3], pos: [i32; 3]) -> bool {
    let d: f64 = (0..3)
        .map(|i| at[i] - f64::from(pos[i]))
        .map(|d| d * d)
        .sum();
    d <= 36.0
}

#[allow(dead_code)]
fn unused(_: &str) {
    let _ = (ID, NAME, VERSION);
}
