// WorldEdit's rules, with no game types: selections, shapes, patterns,
// edits with their history, and the clipboard. The glue (we_glue) maps
// them onto the game's blocks (a block is its state id, a u32).
//
// The file is a module of game in the bundle and of this crate in its
// tests, under the same name. Module payloads cannot start with inner
// doc comments, so these are ordinary comments.
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, AtomicU64, AtomicUsize, Ordering};

pub type Pos = [i32; 3];

/// The defaults of the limits (`/we limit` changes them while playing).
/// The most blocks one command may touch.
pub const MAX_BLOCKS: u64 = 500_000;
/// The largest radius of a sphere or a cylinder.
pub const MAX_RADIUS: i32 = 128;
/// How many edits a player can undo.
pub const HISTORY: usize = 20;

/// The operator level (`ops.json`, 0 to 4) a player needs for any `/we`
/// operation or a wand click: a game master, like vanilla's `/fill`.
pub const LEVEL_EDIT: u8 = 2;
/// The operator level `/we limit` needs: the limits are shared by every
/// player and a large one can freeze the server, so only an owner changes
/// them.
pub const LEVEL_LIMIT: u8 = 4;

/// The operator level the `/we` subcommand `sub` needs.
pub fn level_for(sub: &str) -> u8 {
    if sub == "limit" { LEVEL_LIMIT } else { LEVEL_EDIT }
}

/// Whether a player of operator level `permission` may run the `/we`
/// subcommand `sub` (none is the bare `/we`).
pub fn permitted(permission: u8, sub: Option<&str>) -> bool {
    permission >= sub.map_or(LEVEL_EDIT, level_for)
}

static BLOCKS_NOW: AtomicU64 = AtomicU64::new(MAX_BLOCKS);
static RADIUS_NOW: AtomicI32 = AtomicI32::new(MAX_RADIUS);
static HISTORY_NOW: AtomicUsize = AtomicUsize::new(HISTORY);

/// The most blocks one command may touch now.
pub fn max_blocks() -> u64 {
    BLOCKS_NOW.load(Ordering::Relaxed)
}
/// The largest radius or size of a shape now.
pub fn max_radius() -> i32 {
    RADIUS_NOW.load(Ordering::Relaxed)
}
/// How many edits can be undone now.
pub fn history() -> usize {
    HISTORY_NOW.load(Ordering::Relaxed)
}

/// The limits as a line.
pub fn limits() -> String {
    format!(
        "Limits: blocks {} | radius {} | history {}. Change: limit blocks|radius|history <n>, limit reset.",
        max_blocks(),
        max_radius(),
        history()
    )
}

/// Sets one limit (`blocks`, `radius`, `history`) or puts them all back
/// (`reset`); the line to tell the player.
pub fn set_limit(what: &str, value: Option<i64>) -> Result<String, String> {
    match (what, value) {
        ("reset", _) => {
            BLOCKS_NOW.store(MAX_BLOCKS, Ordering::Relaxed);
            RADIUS_NOW.store(MAX_RADIUS, Ordering::Relaxed);
            HISTORY_NOW.store(HISTORY, Ordering::Relaxed);
        }
        ("blocks", Some(v)) if (1..=20_000_000).contains(&v) => {
            BLOCKS_NOW.store(v as u64, Ordering::Relaxed)
        }
        ("radius", Some(v)) if (1..=1024).contains(&v) => {
            RADIUS_NOW.store(v as i32, Ordering::Relaxed)
        }
        ("history", Some(v)) if (1..=200).contains(&v) => {
            HISTORY_NOW.store(v as usize, Ordering::Relaxed)
        }
        ("blocks", _) => return Err("blocks: 1 to 20000000.".to_owned()),
        ("radius", _) => return Err("radius: 1 to 1024.".to_owned()),
        ("history", _) => return Err("history: 1 to 200.".to_owned()),
        _ => return Err("limit blocks|radius|history <n>, or limit reset.".to_owned()),
    }
    Ok(limits())
}

/// The blocks of a level, as the commands see them.
pub trait World {
    fn get(&self, pos: Pos) -> u32;
    /// Sets the block; whether the world changed.
    fn set(&mut self, pos: Pos, block: u32) -> bool;
}

/// A direction of the world, by compass or axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

impl Dir {
    pub fn parse(word: &str) -> Option<Dir> {
        Some(match word.to_ascii_lowercase().as_str() {
            "north" | "n" => Dir::North,
            "south" | "s" => Dir::South,
            "east" | "e" => Dir::East,
            "west" | "w" => Dir::West,
            "up" | "u" => Dir::Up,
            "down" | "d" => Dir::Down,
            _ => return None,
        })
    }
    /// The unit step: north is -z, east is +x, up is +y.
    pub fn vector(self) -> Pos {
        match self {
            Dir::North => [0, 0, -1],
            Dir::South => [0, 0, 1],
            Dir::East => [1, 0, 0],
            Dir::West => [-1, 0, 0],
            Dir::Up => [0, 1, 0],
            Dir::Down => [0, -1, 0],
        }
    }
}

fn add(a: Pos, b: Pos) -> Pos {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: Pos, b: Pos) -> Pos {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: Pos, k: i32) -> Pos {
    [a[0] * k, a[1] * k, a[2] * k]
}

/// The box between two corners, both included.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cuboid {
    pub min: Pos,
    pub max: Pos,
}

impl Cuboid {
    pub fn new(a: Pos, b: Pos) -> Self {
        Self {
            min: [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2])],
            max: [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])],
        }
    }
    /// Blocks along each axis.
    pub fn size(&self) -> [i32; 3] {
        let d = sub(self.max, self.min);
        [d[0] + 1, d[1] + 1, d[2] + 1]
    }
    pub fn volume(&self) -> u64 {
        let s = self.size();
        s[0] as u64 * s[1] as u64 * s[2] as u64
    }
    pub fn contains(&self, p: Pos) -> bool {
        (0..3).all(|i| p[i] >= self.min[i] && p[i] <= self.max[i])
    }
    /// Every block, lowest layer first.
    pub fn positions(&self) -> Vec<Pos> {
        let mut out = Vec::with_capacity(self.volume().min(max_blocks()) as usize);
        for y in self.min[1]..=self.max[1] {
            for x in self.min[0]..=self.max[0] {
                for z in self.min[2]..=self.max[2] {
                    out.push([x, y, z]);
                }
            }
        }
        out
    }
    /// The four vertical sides.
    pub fn walls(&self) -> Vec<Pos> {
        self.positions()
            .into_iter()
            .filter(|p| {
                p[0] == self.min[0]
                    || p[0] == self.max[0]
                    || p[2] == self.min[2]
                    || p[2] == self.max[2]
            })
            .collect()
    }
    /// The six sides.
    pub fn faces(&self) -> Vec<Pos> {
        self.positions()
            .into_iter()
            .filter(|p| {
                p[0] == self.min[0]
                    || p[0] == self.max[0]
                    || p[1] == self.min[1]
                    || p[1] == self.max[1]
                    || p[2] == self.min[2]
                    || p[2] == self.max[2]
            })
            .collect()
    }
    /// Grows the box by `amount` blocks towards `dir` (a negative amount
    /// shrinks that side).
    pub fn expand(&self, dir: Dir, amount: i32) -> Option<Cuboid> {
        let v = dir.vector();
        let axis = (0..3).find(|&i| v[i] != 0)?;
        let mut c = *self;
        if v[axis] > 0 {
            c.max[axis] += amount;
        } else {
            c.min[axis] -= amount;
        }
        (c.min[axis] <= c.max[axis]).then_some(c)
    }
    /// Pulls the side facing `dir` in by `amount` blocks.
    pub fn contract(&self, dir: Dir, amount: i32) -> Option<Cuboid> {
        self.expand(dir, -amount)
    }
    pub fn shifted(&self, offset: Pos) -> Cuboid {
        Cuboid {
            min: add(self.min, offset),
            max: add(self.max, offset),
        }
    }
}

/// A player's two corners, in a dimension.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    pub dimension: String,
    pub pos1: Option<Pos>,
    pub pos2: Option<Pos>,
}

impl Selection {
    pub fn set_pos(&mut self, index: u8, dimension: &str, pos: Pos) {
        if self.dimension != dimension {
            *self = Selection {
                dimension: dimension.to_owned(),
                ..Selection::default()
            };
        }
        if index == 1 {
            self.pos1 = Some(pos);
        } else {
            self.pos2 = Some(pos);
        }
    }
    pub fn cuboid(&self) -> Option<Cuboid> {
        Some(Cuboid::new(self.pos1?, self.pos2?))
    }
    pub fn set_cuboid(&mut self, c: Cuboid) {
        self.pos1 = Some(c.min);
        self.pos2 = Some(c.max);
    }
    pub fn clear(&mut self) {
        self.pos1 = None;
        self.pos2 = None;
    }
}

/// A command that would touch more than `MAX_BLOCKS` is refused.
pub fn check_limit(count: u64) -> Result<(), String> {
    if count > max_blocks() {
        Err(format!(
            "That is {count} blocks; the limit is {}.",
            max_blocks()
        ))
    } else {
        Ok(())
    }
}

fn disc(radius: i32, inside: impl Fn(i32, i32) -> bool, hollow: bool) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for x in -radius..=radius {
        for z in -radius..=radius {
            if !inside(x, z) {
                continue;
            }
            let edge = !(inside(x + 1, z) && inside(x - 1, z) && inside(x, z + 1) && inside(x, z - 1));
            if !hollow || edge {
                out.push((x, z));
            }
        }
    }
    out
}

fn in_circle(radius: i32) -> impl Fn(i32, i32) -> bool {
    let r = radius as f64 + 0.5;
    move |x, z| ((x * x + z * z) as f64) <= r * r
}

/// A ball around `center`; hollow keeps only its outer shell.
pub fn sphere(center: Pos, radius: i32, hollow: bool) -> Result<Vec<Pos>, String> {
    if !(1..=max_radius()).contains(&radius) {
        return Err(format!("The radius is 1 to {}.", max_radius()));
    }
    let r = radius as f64 + 0.5;
    let inside = |x: i32, y: i32, z: i32| ((x * x + y * y + z * z) as f64) <= r * r;
    let mut out = Vec::new();
    for y in -radius..=radius {
        for x in -radius..=radius {
            for z in -radius..=radius {
                if !inside(x, y, z) {
                    continue;
                }
                let shell = !(inside(x + 1, y, z)
                    && inside(x - 1, y, z)
                    && inside(x, y + 1, z)
                    && inside(x, y - 1, z)
                    && inside(x, y, z + 1)
                    && inside(x, y, z - 1));
                if !hollow || shell {
                    out.push(add(center, [x, y, z]));
                }
            }
        }
    }
    check_limit(out.len() as u64)?;
    Ok(out)
}

/// A cylinder standing on `center`, `height` blocks tall. Hollow keeps its
/// side and both ends.
pub fn cylinder(center: Pos, radius: i32, height: i32, hollow: bool) -> Result<Vec<Pos>, String> {
    if !(1..=max_radius()).contains(&radius) {
        return Err(format!("The radius is 1 to {}.", max_radius()));
    }
    if height < 1 {
        return Err("The height is at least 1.".to_owned());
    }
    check_limit((2 * radius as u64 + 1).pow(2) * height as u64)?;
    let inside = in_circle(radius);
    let full = disc(radius, &inside, false);
    let rim = disc(radius, &inside, true);
    let mut out = Vec::new();
    for dy in 0..height {
        let layer = if hollow && dy != 0 && dy != height - 1 {
            &rim
        } else {
            &full
        };
        for &(x, z) in layer {
            out.push(add(center, [x, dy, z]));
        }
    }
    check_limit(out.len() as u64)?;
    Ok(out)
}

/// A hollow vertical cylinder around `center`, `height` blocks tall: only
/// its wall, one block thick, open at both ends (`//hcyl`). A height of 1
/// is a ring.
pub fn hollow_cylinder(center: Pos, radius: i32, height: i32) -> Result<Vec<Pos>, String> {
    if !(1..=max_radius()).contains(&radius) {
        return Err(format!("The radius is 1 to {}.", max_radius()));
    }
    if height < 1 {
        return Err("The height is at least 1.".to_owned());
    }
    check_limit((2 * radius as u64 + 1).pow(2) * height as u64)?;
    let inside = in_circle(radius);
    let rim = disc(radius, &inside, true);
    let mut out = Vec::with_capacity(rim.len() * height as usize);
    for dy in 0..height {
        for &(x, z) in &rim {
            out.push(add(center, [x, dy, z]));
        }
    }
    check_limit(out.len() as u64)?;
    Ok(out)
}

/// A pyramid standing on `center`: `size` layers, the lowest `2 * size - 1`
/// blocks wide. Hollow keeps the outline of each layer (`//hpyramid`).
pub fn pyramid(center: Pos, size: i32, hollow: bool) -> Result<Vec<Pos>, String> {
    if !(1..=max_radius()).contains(&size) {
        return Err(format!("The size is 1 to {}.", max_radius()));
    }
    let n = size as u64;
    // 1 + 9 + 25 + ... : the blocks of a filled one.
    check_limit(n * (2 * n - 1) * (2 * n + 1) / 3)?;
    let mut out = Vec::new();
    for dy in 0..size {
        let half = size - 1 - dy;
        for x in -half..=half {
            for z in -half..=half {
                if !hollow || x.abs() == half || z.abs() == half {
                    out.push(add(center, [x, dy, z]));
                }
            }
        }
    }
    check_limit(out.len() as u64)?;
    Ok(out)
}

/// One or more blocks, with weights: `stone`, `50%stone,dirt`.
#[derive(Clone, Debug, PartialEq)]
pub struct Pattern {
    entries: Vec<(f64, u32)>,
    total: f64,
}

/// Block ids by name; `None` for an unknown block.
pub type Resolver<'a> = &'a dyn Fn(&str) -> Option<u32>;

impl Pattern {
    pub fn single(block: u32) -> Self {
        Self {
            entries: vec![(1.0, block)],
            total: 1.0,
        }
    }
    pub fn parse(text: &str, resolve: Resolver<'_>) -> Result<Pattern, String> {
        let mut named: Vec<(Option<f64>, u32)> = Vec::new();
        for part in text.split(',').filter(|p| !p.is_empty()) {
            let (weight, name) = match part.split_once('%') {
                Some((w, n)) => {
                    let w: f64 = w
                        .parse()
                        .map_err(|_| format!("`{w}` is not a percentage."))?;
                    if !(w > 0.0 && w <= 100.0) {
                        return Err(format!("`{w}%` is not between 0 and 100."));
                    }
                    (Some(w), n)
                }
                None => (None, part),
            };
            let block = resolve(name).ok_or_else(|| format!("Unknown block `{name}`."))?;
            named.push((weight, block));
        }
        if named.is_empty() {
            return Err("A pattern needs a block.".to_owned());
        }
        let given: f64 = named.iter().filter_map(|e| e.0).sum();
        let open = named.iter().filter(|e| e.0.is_none()).count();
        if given > 100.0 + 1e-9 {
            return Err("The percentages add up to more than 100.".to_owned());
        }
        let share = if open > 0 {
            ((100.0 - given) / open as f64).max(0.0)
        } else {
            0.0
        };
        let entries: Vec<(f64, u32)> = named
            .into_iter()
            .map(|(w, b)| (w.unwrap_or(share), b))
            .filter(|e| e.0 > 0.0)
            .collect();
        let total = entries.iter().map(|e| e.0).sum();
        if entries.is_empty() {
            return Err("A pattern needs a block with a share.".to_owned());
        }
        Ok(Pattern { entries, total })
    }
    /// The block for `pos`: always the same one for the same position.
    pub fn pick(&self, pos: Pos) -> u32 {
        if self.entries.len() == 1 {
            return self.entries[0].1;
        }
        let mut h = (pos[0] as i64 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (pos[1] as i64 as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
            ^ (pos[2] as i64 as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
        h ^= h >> 33;
        h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
        h ^= h >> 33;
        let mut at = (h >> 11) as f64 / (1u64 << 53) as f64 * self.total;
        for &(w, b) in &self.entries {
            if at < w {
                return b;
            }
            at -= w;
        }
        self.entries[self.entries.len() - 1].1
    }
}

/// A block set: `stone,dirt`.
pub fn parse_blocks(text: &str, resolve: Resolver<'_>) -> Result<Vec<u32>, String> {
    let mut out = Vec::new();
    for name in text.split(',').filter(|n| !n.is_empty()) {
        out.push(resolve(name).ok_or_else(|| format!("Unknown block `{name}`."))?);
    }
    if out.is_empty() {
        return Err("A block is needed.".to_owned());
    }
    Ok(out)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Change {
    pub pos: Pos,
    pub old: u32,
    pub new: u32,
}

/// What one command changed, in order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Edit {
    pub changes: Vec<Change>,
}

impl Edit {
    pub fn len(&self) -> usize {
        self.changes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
    /// Puts every block back as it was, last change first, and returns the
    /// edit that undoes that.
    pub fn revert<W: World>(&self, world: &mut W) -> Edit {
        let mut out = Edit::default();
        for c in self.changes.iter().rev() {
            if world.set(c.pos, c.old) {
                out.changes.push(Change {
                    pos: c.pos,
                    old: c.new,
                    new: c.old,
                });
            }
        }
        out
    }
}

/// Writes the blocks in order and records what changed.
pub fn apply<W: World>(world: &mut W, writes: impl IntoIterator<Item = (Pos, u32)>) -> Edit {
    let mut edit = Edit::default();
    for (pos, new) in writes {
        let old = world.get(pos);
        if old != new && world.set(pos, new) {
            edit.changes.push(Change { pos, old, new });
        }
    }
    edit
}

/// `positions` filled from `pattern`.
pub fn fill<W: World>(world: &mut W, positions: &[Pos], pattern: &Pattern) -> Edit {
    apply(world, positions.iter().map(|&p| (p, pattern.pick(p))))
}

/// Blocks of `from` in `positions` turned into `to`.
pub fn replace<W: World>(world: &mut W, positions: &[Pos], from: &[u32], to: &Pattern) -> Edit {
    let hits: Vec<Pos> = positions
        .iter()
        .copied()
        .filter(|&p| from.contains(&world.get(p)))
        .collect();
    fill(world, &hits, to)
}

/// A position a player typed: `10,64,-5` or `10 64 -5`.
pub fn parse_pos(text: &str) -> Option<Pos> {
    let parts: Vec<&str> = text.split([' ', ',']).filter(|p| !p.is_empty()).collect();
    if parts.len() != 3 {
        return None;
    }
    let mut out = [0i32; 3];
    for (i, part) in parts.iter().enumerate() {
        out[i] = part.parse().ok()?;
    }
    Some(out)
}

/// The blocks of `positions` by id, the most of each first.
pub fn distribution<W: World>(world: &W, positions: &[Pos]) -> Vec<(u32, u64)> {
    let mut counts: HashMap<u32, u64> = HashMap::new();
    for &p in positions {
        *counts.entry(world.get(p)).or_insert(0) += 1;
    }
    let mut out: Vec<(u32, u64)> = counts.into_iter().collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    out
}

/// The box around the blocks that are not air in `c` (vanilla's `//trim`).
/// None when there is nothing but air.
pub fn trim_to_existing<W: World>(world: &W, c: &Cuboid, air: u32) -> Option<Cuboid> {
    let mut min = c.max;
    let mut max = c.min;
    let mut any = false;
    for p in c.positions() {
        if world.get(p) == air {
            continue;
        }
        any = true;
        for i in 0..3 {
            min[i] = min[i].min(p[i]);
            max[i] = max[i].max(p[i]);
        }
    }
    any.then_some(Cuboid { min, max })
}

/// The hole around `center` that is filled: a ball of `radius` and `depth`
/// blocks deep, where the air is (vanilla's `//fill` and `//fillr`). `/we fill`
/// fills each column down to the first block that is there, `//fillr` the
/// whole air of the ball.
pub fn fill_hole<W: World>(
    world: &W,
    center: Pos,
    radius: i32,
    depth: i32,
    pattern: &Pattern,
    recursive: bool,
    air: u32,
) -> Result<Vec<(Pos, u32)>, String> {
    if !(1..=max_radius()).contains(&radius) {
        return Err(format!("The radius is 1 to {}.", max_radius()));
    }
    if depth < 1 {
        return Err("The depth is at least 1.".to_owned());
    }
    let r = radius as f64 + 0.5;
    let bottom = center[1] - depth + 1;
    let mut writes = Vec::new();
    if recursive {
        // The whole air of the ball, down to the bottom of the hole.
        for dy in -(depth - 1)..=0 {
            let rest = r * r - (dy * dy) as f64;
            if rest < 0.0 {
                continue;
            }
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    if ((dx * dx + dz * dz) as f64) > rest {
                        continue;
                    }
                    let p = add(center, [dx, dy, dz]);
                    if world.get(p) == air {
                        writes.push((p, pattern.pick(p)));
                    }
                }
            }
        }
    } else {
        // Each column, from the top down to the first block that is there.
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let rest = r * r - (dx * dx + dz * dz) as f64;
                if rest < 0.0 {
                    continue;
                }
                let deep = ((rest.sqrt() - 0.5).floor() as i32).min(depth - 1);
                let x = center[0] + dx;
                let z = center[2] + dz;
                for y in (center[1] - deep..=center[1]).rev() {
                    let p = [x, y, z];
                    if world.get(p) != air {
                        break;
                    }
                    writes.push((p, pattern.pick(p)));
                }
            }
        }
    }
    check_limit(writes.len() as u64)?;
    Ok(writes)
}

/// The blocks above (`up`) or below (`down`) the player, in a square of
/// `size` blocks around them and `height` blocks deep (vanilla's
/// `//removeabove` and `//removebelow`).
pub fn remove_column<W: World>(
    world: &W,
    center: Pos,
    size: i32,
    height: i32,
    up: bool,
    air: u32,
) -> Vec<Pos> {
    let (from, to) = if up {
        (center[1] + 1, center[1] + height)
    } else {
        (center[1] - height, center[1] - 1)
    };
    let mut out = Vec::new();
    for y in from..=to {
        for dx in -size..=size {
            for dz in -size..=size {
                let p = [center[0] + dx, y, center[2] + dz];
                if world.get(p) != air {
                    out.push(p);
                }
            }
        }
    }
    out
}

impl Cuboid {
    /// Grown by `n` blocks on every side (a negative `n` shrinks it).
    pub fn grown(&self, n: i32) -> Option<Cuboid> {
        let c = Cuboid {
            min: [self.min[0] - n, self.min[1] - n, self.min[2] - n],
            max: [self.max[0] + n, self.max[1] + n, self.max[2] + n],
        };
        (0..3).all(|i| c.min[i] <= c.max[i]).then_some(c)
    }
}

/// The blocks on the straight line from `a` to `b`, both included.
pub fn line(a: Pos, b: Pos) -> Result<Vec<Pos>, String> {
    let d = sub(b, a);
    let n = d[0].abs().max(d[1].abs()).max(d[2].abs());
    check_limit(n as u64 + 1)?;
    let mut out: Vec<Pos> = Vec::with_capacity(n as usize + 1);
    for i in 0..=n {
        let at = |k: usize| {
            if n == 0 {
                a[k]
            } else {
                (a[k] as f64 + d[k] as f64 * i as f64 / n as f64).round() as i32
            }
        };
        let p = [at(0), at(1), at(2)];
        if out.last() != Some(&p) {
            out.push(p);
        }
    }
    Ok(out)
}

/// `overlay`: a block of `pattern` on top of the highest block of each column.
pub fn overlay_writes<W: World>(
    world: &W,
    region: &Cuboid,
    air: u32,
    pattern: &Pattern,
) -> Vec<(Pos, u32)> {
    let mut out = Vec::new();
    for x in region.min[0]..=region.max[0] {
        for z in region.min[2]..=region.max[2] {
            for y in (region.min[1]..=region.max[1]).rev() {
                if world.get([x, y, z]) != air {
                    let p = [x, y + 1, z];
                    out.push((p, pattern.pick(p)));
                    break;
                }
            }
        }
    }
    out
}

/// `naturalize`: each column's top block `top`, the next three `soil`, the
/// rest of its solid run `deep`.
pub fn naturalize_writes<W: World>(
    world: &W,
    region: &Cuboid,
    air: u32,
    (top, soil, deep): (u32, u32, u32),
) -> Vec<(Pos, u32)> {
    let mut out = Vec::new();
    for x in region.min[0]..=region.max[0] {
        for z in region.min[2]..=region.max[2] {
            let mut depth = 0;
            for y in (region.min[1]..=region.max[1]).rev() {
                let p = [x, y, z];
                if world.get(p) == air {
                    if depth > 0 {
                        break;
                    }
                    continue;
                }
                out.push((p, [top, soil, soil, soil].get(depth).copied().unwrap_or(deep)));
                depth += 1;
            }
        }
    }
    out
}

/// Every block of `positions` that is not one of `except`, turned into `to`.
pub fn replace_except<W: World>(
    world: &mut W,
    positions: &[Pos],
    except: &[u32],
    to: &Pattern,
) -> Edit {
    let hits: Vec<Pos> = positions
        .iter()
        .copied()
        .filter(|&p| !except.contains(&world.get(p)))
        .collect();
    fill(world, &hits, to)
}

/// `smooth`: each column's top block moves to the average height of its
/// 3x3 neighbourhood, `iterations` times. Blocks are added below a raised
/// surface (the block under the old top) and removed from a lowered one.
pub fn smooth_writes<W: World>(
    world: &W,
    region: &Cuboid,
    air: u32,
    iterations: u32,
) -> Vec<(Pos, u32)> {
    let w = (region.max[0] - region.min[0] + 1) as usize;
    let d = (region.max[2] - region.min[2] + 1) as usize;
    let floor = region.min[1] - 1;
    let mut start = vec![floor; w * d];
    for i in 0..w {
        for k in 0..d {
            let (x, z) = (region.min[0] + i as i32, region.min[2] + k as i32);
            for y in (region.min[1]..=region.max[1]).rev() {
                if world.get([x, y, z]) != air {
                    start[i * d + k] = y;
                    break;
                }
            }
        }
    }
    let mut now = start.clone();
    for _ in 0..iterations {
        let before = now.clone();
        for i in 0..w {
            for k in 0..d {
                if start[i * d + k] == floor {
                    continue;
                }
                let (mut sum, mut n) = (0, 0);
                for di in -1i32..=1 {
                    for dk in -1i32..=1 {
                        let (a, b) = (i as i32 + di, k as i32 + dk);
                        if a >= 0 && b >= 0 && (a as usize) < w && (b as usize) < d {
                            sum += before[a as usize * d + b as usize];
                            n += 1;
                        }
                    }
                }
                now[i * d + k] = ((sum as f64 / n as f64).round() as i32)
                    .clamp(region.min[1], region.max[1]);
            }
        }
    }
    let mut out = Vec::new();
    for i in 0..w {
        for k in 0..d {
            let (from, to) = (start[i * d + k], now[i * d + k]);
            if from == floor || from == to {
                continue;
            }
            let (x, z) = (region.min[0] + i as i32, region.min[2] + k as i32);
            let top = world.get([x, from, z]);
            if to > from {
                let under = if from > region.min[1] {
                    world.get([x, from - 1, z])
                } else {
                    top
                };
                for y in from..to {
                    out.push(([x, y, z], under));
                }
                out.push(([x, to, z], top));
            } else {
                for y in (to + 1)..=from {
                    out.push(([x, y, z], air));
                }
                out.push(([x, to, z], top));
            }
        }
    }
    out
}

/// How many blocks of `of` are in `positions`.
pub fn count<W: World>(world: &W, positions: &[Pos], of: &[u32]) -> u64 {
    positions
        .iter()
        .filter(|&&p| of.contains(&world.get(p)))
        .count() as u64
}

/// A copied region: its blocks relative to where the player stood.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Clipboard {
    pub blocks: Vec<(Pos, u32)>,
}

impl Clipboard {
    pub fn copy<W: World>(world: &W, region: &Cuboid, origin: Pos) -> Clipboard {
        Clipboard {
            blocks: region
                .positions()
                .into_iter()
                .map(|p| (sub(p, origin), world.get(p)))
                .collect(),
        }
    }
    /// Turns the clipboard about the vertical axis by a multiple of 90
    /// degrees (clockwise seen from above). Block properties such as
    /// `facing` stay as they were.
    pub fn rotate(&mut self, degrees: i32) -> Result<(), String> {
        if degrees % 90 != 0 {
            return Err("Rotate by a multiple of 90 degrees.".to_owned());
        }
        for _ in 0..(degrees.rem_euclid(360) / 90) {
            for (p, _) in &mut self.blocks {
                *p = [-p[2], p[1], p[0]];
            }
        }
        Ok(())
    }
    /// Mirrors the clipboard across the axis of `dir` through the player.
    pub fn flip(&mut self, dir: Dir) {
        let v = dir.vector();
        for (p, _) in &mut self.blocks {
            for i in 0..3 {
                if v[i] != 0 {
                    p[i] = -p[i];
                }
            }
        }
    }
    /// The writes that put the clipboard at `at`; blocks equal to `skip`
    /// (air, for `paste -a`) are left out.
    pub fn writes(&self, at: Pos, skip: Option<u32>) -> Vec<(Pos, u32)> {
        let mut out: Vec<(Pos, u32)> = self
            .blocks
            .iter()
            .filter(|(_, b)| Some(*b) != skip)
            .map(|&(p, b)| (add(at, p), b))
            .collect();
        out.sort_by_key(|(p, _)| (p[1], p[0], p[2]));
        out
    }
}

/// The writes of `stack`: the region repeated `times` along `dir`.
pub fn stack_writes<W: World>(
    world: &W,
    region: &Cuboid,
    dir: Dir,
    times: i32,
) -> Result<Vec<(Pos, u32)>, String> {
    if times < 1 {
        return Err("Stack at least once.".to_owned());
    }
    check_limit(region.volume().saturating_mul(times as u64))?;
    let v = dir.vector();
    let s = region.size();
    let step = [v[0] * s[0], v[1] * s[1], v[2] * s[2]];
    let source: Vec<(Pos, u32)> = region
        .positions()
        .into_iter()
        .map(|p| (p, world.get(p)))
        .collect();
    let mut out = Vec::with_capacity(source.len() * times as usize);
    for i in 1..=times {
        let offset = scale(step, i);
        out.extend(source.iter().map(|&(p, b)| (add(p, offset), b)));
    }
    Ok(out)
}

/// The writes of `move`: the region is cleared to `fill`, then put
/// `distance` blocks along `dir`.
pub fn move_writes<W: World>(
    world: &W,
    region: &Cuboid,
    dir: Dir,
    distance: i32,
    fill: u32,
) -> Result<Vec<(Pos, u32)>, String> {
    if distance < 1 {
        return Err("Move by at least 1.".to_owned());
    }
    check_limit(region.volume())?;
    let offset = scale(dir.vector(), distance);
    let source: Vec<(Pos, u32)> = region
        .positions()
        .into_iter()
        .map(|p| (p, world.get(p)))
        .collect();
    let mut out: Vec<(Pos, u32)> = source.iter().map(|&(p, _)| (p, fill)).collect();
    out.extend(source.iter().map(|&(p, b)| (add(p, offset), b)));
    Ok(out)
}

/// One player's selection, clipboard and history.
#[derive(Debug, Default)]
pub struct Session {
    pub selection: Selection,
    pub clipboard: Option<Clipboard>,
    undo: Vec<Edit>,
    redo: Vec<Edit>,
}

impl Session {
    /// Remembers an edit to undo; a new edit forgets what was undone.
    pub fn record(&mut self, edit: Edit) {
        if edit.is_empty() {
            return;
        }
        self.undo.push(edit);
        if self.undo.len() > history() {
            self.undo.remove(0);
        }
        self.redo.clear();
    }
    /// Undoes up to `times` edits; how many edits and blocks.
    pub fn undo<W: World>(&mut self, world: &mut W, times: usize) -> (usize, usize) {
        Self::step(&mut self.undo, &mut self.redo, world, times)
    }
    pub fn redo<W: World>(&mut self, world: &mut W, times: usize) -> (usize, usize) {
        Self::step(&mut self.redo, &mut self.undo, world, times)
    }
    fn step<W: World>(
        from: &mut Vec<Edit>,
        to: &mut Vec<Edit>,
        world: &mut W,
        times: usize,
    ) -> (usize, usize) {
        let (mut edits, mut blocks) = (0, 0);
        for _ in 0..times {
            let Some(edit) = from.pop() else { break };
            let back = edit.revert(world);
            blocks += back.len();
            edits += 1;
            if !back.is_empty() {
                to.push(back);
            }
        }
        (edits, blocks)
    }
}

/// Every player's session, by UUID.
#[derive(Debug, Default)]
pub struct Sessions {
    by_player: HashMap<u128, Session>,
}

impl Sessions {
    pub fn get(&mut self, uuid: u128) -> &mut Session {
        self.by_player.entry(uuid).or_default()
    }
    pub fn forget(&mut self, uuid: u128) {
        self.by_player.remove(&uuid);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn permission_levels() {
        // A creative non-operator is refused everywhere.
        for sub in [None, Some("set"), Some("undo"), Some("limit")] {
            assert!(!permitted(0, sub));
            assert!(!permitted(1, sub));
        }
        // A game master edits but cannot change the shared limits.
        assert!(permitted(2, None));
        assert!(permitted(2, Some("set")));
        assert!(permitted(2, Some("paste")));
        assert!(!permitted(2, Some("limit")));
        // A game master edits the world but not the shared limits.
        assert!(!permitted(3, Some("limit")));
        assert!(permitted(3, Some("replace")));
        // Only an owner changes the limits.
        assert!(permitted(4, Some("limit")));
        assert!(permitted(4, Some("replace")));
    }
    use super::*;

    #[derive(Default)]
    struct Map(HashMap<Pos, u32>);

    impl World for Map {
        fn get(&self, p: Pos) -> u32 {
            self.0.get(&p).copied().unwrap_or(0)
        }
        fn set(&mut self, p: Pos, b: u32) -> bool {
            let old = self.get(p);
            self.0.insert(p, b);
            old != b
        }
    }

    fn names(n: &str) -> Option<u32> {
        match n {
            "air" => Some(0),
            "stone" => Some(1),
            "dirt" => Some(2),
            "glass" => Some(3),
            _ => None,
        }
    }

    #[test]
    fn cuboid_counts_and_sides() {
        let c = Cuboid::new([3, 5, 3], [0, 0, 0]);
        assert_eq!(c.size(), [4, 6, 4]);
        assert_eq!(c.volume(), 96);
        assert_eq!(c.positions().len(), 96);
        assert_eq!(c.walls().len(), 96 - 2 * 2 * 6);
        assert_eq!(c.faces().len(), 96 - 2 * 4 * 2);
    }

    #[test]
    fn expand_and_contract_move_one_side() {
        let c = Cuboid::new([0, 0, 0], [3, 3, 3]);
        let e = c.expand(Dir::Up, 2).unwrap();
        assert_eq!((e.min, e.max), ([0, 0, 0], [3, 5, 3]));
        let e = c.expand(Dir::West, 1).unwrap();
        assert_eq!(e.min, [-1, 0, 0]);
        let k = c.contract(Dir::East, 2).unwrap();
        assert_eq!(k.max, [1, 3, 3]);
        assert!(c.contract(Dir::East, 4).is_none());
    }

    #[test]
    fn set_replace_count_and_undo() {
        let mut w = Map::default();
        let c = Cuboid::new([0, 0, 0], [2, 2, 2]);
        let stone = Pattern::single(1);
        let mut s = Session::default();
        let e = fill(&mut w, &c.positions(), &stone);
        assert_eq!(e.len(), 27);
        s.record(e);
        let dirt = Pattern::single(2);
        let e = replace(&mut w, &c.positions(), &[1], &dirt);
        assert_eq!(e.len(), 27);
        s.record(e);
        assert_eq!(count(&w, &c.positions(), &[2]), 27);
        assert_eq!(s.undo(&mut w, 1), (1, 27));
        assert_eq!(count(&w, &c.positions(), &[1]), 27);
        assert_eq!(s.redo(&mut w, 1), (1, 27));
        assert_eq!(count(&w, &c.positions(), &[2]), 27);
        assert_eq!(s.undo(&mut w, 5), (2, 54));
        assert_eq!(count(&w, &c.positions(), &[0]), 27);
        assert_eq!(s.undo(&mut w, 1), (0, 0));
    }

    #[test]
    fn a_new_edit_forgets_the_undone() {
        let mut w = Map::default();
        let mut s = Session::default();
        s.record(fill(&mut w, &[[0, 0, 0]], &Pattern::single(1)));
        s.undo(&mut w, 1);
        s.record(fill(&mut w, &[[0, 0, 0]], &Pattern::single(2)));
        assert_eq!(s.redo(&mut w, 1), (0, 0));
    }

    #[test]
    fn history_is_bounded() {
        let mut w = Map::default();
        let mut s = Session::default();
        for i in 0..(HISTORY as u32 + 5) {
            s.record(fill(&mut w, &[[i as i32, 0, 0]], &Pattern::single(1)));
        }
        assert_eq!(s.undo(&mut w, 100).0, HISTORY);
    }

    #[test]
    fn patterns_parse_weigh_and_are_stable() {
        let p = Pattern::parse("stone", &names).unwrap();
        assert_eq!(p.pick([1, 2, 3]), 1);
        assert!(Pattern::parse("bedrock", &names).is_err());
        assert!(Pattern::parse("80%stone,40%dirt", &names).is_err());
        assert!(Pattern::parse("x%stone", &names).is_err());
        let p = Pattern::parse("25%stone,dirt", &names).unwrap();
        let mut stone = 0;
        for x in 0..40 {
            for z in 0..40 {
                let b = p.pick([x, 0, z]);
                assert_eq!(b, p.pick([x, 0, z]));
                assert!(b == 1 || b == 2);
                stone += (b == 1) as u32;
            }
        }
        assert!((300..500).contains(&stone), "stone share {stone}/1600");
    }

    #[test]
    fn spheres_and_cylinders() {
        let ball = sphere([0, 0, 0], 3, false).unwrap();
        assert!(ball.contains(&[3, 0, 0]));
        assert!(!ball.contains(&[3, 3, 3]));
        let shell = sphere([0, 0, 0], 3, true).unwrap();
        assert!(!shell.contains(&[0, 0, 0]));
        assert!(shell.len() < ball.len());
        assert!(sphere([0, 0, 0], 0, false).is_err());
        assert!(sphere([0, 0, 0], MAX_RADIUS + 1, false).is_err());
        let can = cylinder([0, 10, 0], 2, 4, false).unwrap();
        assert_eq!(can.len(), 21 * 4);
        let pipe = cylinder([0, 10, 0], 3, 4, true).unwrap();
        assert!(!pipe.contains(&[0, 11, 0]));
        assert!(pipe.contains(&[0, 10, 0]) && pipe.contains(&[0, 13, 0]));
        assert!(cylinder([0, 0, 0], 2, 0, false).is_err());
    }

    #[test]
    fn hollow_cylinders_are_open_walls() {
        let ring = hollow_cylinder([0, 10, 0], 3, 1).unwrap();
        assert!(!ring.contains(&[0, 10, 0]), "no middle");
        assert!(ring.contains(&[3, 10, 0]) && ring.contains(&[0, 10, -3]));
        assert!(ring.iter().all(|p| p[1] == 10));
        let tube = hollow_cylinder([0, 10, 0], 3, 4).unwrap();
        assert_eq!(tube.len(), ring.len() * 4);
        assert!(!tube.contains(&[0, 10, 0]) && !tube.contains(&[0, 13, 0]), "no caps");
        assert!(tube.contains(&[3, 13, 0]));
        // Every block of a ring touches the outside.
        let inside = |x: i32, z: i32| ((x * x + z * z) as f64) <= 3.5 * 3.5;
        for p in &ring {
            assert!(inside(p[0], p[2]));
        }
        assert!(hollow_cylinder([0, 0, 0], 0, 1).is_err());
        assert!(hollow_cylinder([0, 0, 0], 2, 0).is_err());
        assert!(hollow_cylinder([0, 0, 0], MAX_RADIUS + 1, 1).is_err());
    }

    #[test]
    fn pyramids_narrow_by_one_each_layer() {
        let solid = pyramid([0, 0, 0], 3, false).unwrap();
        assert_eq!(solid.len(), 25 + 9 + 1);
        assert!(solid.contains(&[2, 0, 2]) && solid.contains(&[0, 2, 0]));
        assert!(!solid.contains(&[2, 1, 2]));
        let shell = pyramid([0, 0, 0], 3, true).unwrap();
        assert_eq!(shell.len(), 16 + 8 + 1);
        assert!(!shell.contains(&[0, 0, 0]) && !shell.contains(&[0, 1, 0]));
        assert!(shell.contains(&[2, 0, 0]) && shell.contains(&[1, 1, 1]));
        assert!(pyramid([0, 0, 0], 0, false).is_err());
        assert!(pyramid([0, 0, 0], MAX_RADIUS, false).is_err(), "too many blocks");
    }

    #[test]
    fn lines_overlay_naturalize_and_grown() {
        let l = line([0, 0, 0], [4, 2, 0]).unwrap();
        assert_eq!((l[0], *l.last().unwrap()), ([0, 0, 0], [4, 2, 0]));
        assert_eq!(l.len(), 5);
        assert_eq!(line([1, 1, 1], [1, 1, 1]).unwrap().len(), 1);
        let c = Cuboid::new([0, 0, 0], [2, 2, 2]);
        assert_eq!(c.grown(1).unwrap().size(), [5, 5, 5]);
        assert!(c.grown(-2).is_none());
        let mut w = Map::default();
        for x in 0..3 {
            for z in 0..3 {
                for y in 0..3 {
                    w.0.insert([x, y, z], 1);
                }
            }
        }
        let top = overlay_writes(&w, &c, 0, &Pattern::single(3));
        assert_eq!(top.len(), 9);
        assert!(top.iter().all(|(p, b)| p[1] == 3 && *b == 3));
        let nat = naturalize_writes(&w, &c, 0, (5, 6, 7));
        assert_eq!(nat.len(), 27);
        assert!(nat.contains(&([1, 2, 1], 5)) && nat.contains(&([1, 0, 1], 6)));
        let e = replace_except(&mut w, &c.positions(), &[1], &Pattern::single(2));
        assert_eq!(e.len(), 0);
        let e = replace_except(&mut w, &c.positions(), &[9], &Pattern::single(2));
        assert_eq!(e.len(), 27);
    }

    #[test]
    fn smooth_levels_a_spike() {
        let mut w = Map::default();
        for x in 0..3 {
            for z in 0..3 {
                w.0.insert([x, 0, z], 1);
            }
        }
        for y in 1..=3 {
            w.0.insert([1, y, 1], 1);
        }
        let c = Cuboid::new([0, 0, 0], [2, 5, 2]);
        let writes = smooth_writes(&w, &c, 0, 1);
        apply(&mut w, writes);
        assert_eq!(w.get([1, 3, 1]), 0, "the spike came down");
        assert_ne!(w.get([0, 1, 0]), 0, "its sides came up");
        let flat = smooth_writes(&w, &Cuboid::new([5, 0, 5], [6, 3, 6]), 0, 3);
        assert!(flat.is_empty());
    }

    #[test]
    fn copy_rotate_flip_and_paste() {
        let mut w = Map::default();
        w.0.insert([1, 0, 0], 7);
        let c = Cuboid::new([0, 0, 0], [1, 0, 0]);
        let mut clip = Clipboard::copy(&w, &c, [0, 0, 0]);
        clip.rotate(90).unwrap();
        let writes = clip.writes([10, 0, 10], None);
        assert!(writes.contains(&([10, 0, 11], 7)));
        clip.rotate(270).unwrap();
        clip.flip(Dir::East);
        assert!(clip.writes([0, 0, 0], None).contains(&([-1, 0, 0], 7)));
        assert_eq!(clip.writes([0, 0, 0], Some(0)).len(), 1);
        assert!(clip.rotate(45).is_err());
        let e = apply(&mut w, clip.writes([20, 0, 0], None));
        assert_eq!(e.len(), 1);
    }

    #[test]
    fn stack_and_move() {
        let mut w = Map::default();
        w.0.insert([0, 0, 0], 5);
        let c = Cuboid::new([0, 0, 0], [0, 1, 0]);
        let writes = stack_writes(&w, &c, Dir::East, 2).unwrap();
        apply(&mut w, writes);
        assert_eq!((w.get([1, 0, 0]), w.get([2, 0, 0])), (5, 5));
        let writes = move_writes(&w, &c, Dir::Up, 1, 0).unwrap();
        apply(&mut w, writes);
        assert_eq!((w.get([0, 0, 0]), w.get([0, 1, 0])), (0, 5));
        assert!(stack_writes(&w, &c, Dir::East, 0).is_err());
    }

    #[test]
    fn big_regions_are_refused() {
        assert!(check_limit(MAX_BLOCKS).is_ok());
        assert!(check_limit(MAX_BLOCKS + 1).is_err());
        let w = Map::default();
        let c = Cuboid::new([0, 0, 0], [999, 999, 999]);
        assert!(stack_writes(&w, &c, Dir::Up, 1).is_err());
    }

    #[test]
    fn selection_resets_in_another_dimension() {
        let mut s = Selection::default();
        s.set_pos(1, "minecraft:overworld", [0, 0, 0]);
        s.set_pos(2, "minecraft:overworld", [1, 1, 1]);
        assert!(s.cuboid().is_some());
        s.set_pos(1, "minecraft:the_nether", [5, 5, 5]);
        assert!(s.cuboid().is_none());
    }

    #[test]
    fn positions_are_read_with_commas_or_spaces() {
        assert_eq!(parse_pos("10,64,-5"), Some([10, 64, -5]));
        assert_eq!(parse_pos("10 64 -5"), Some([10, 64, -5]));
        assert_eq!(parse_pos("10,64"), None);
        assert_eq!(parse_pos("a,64,-5"), None);
        assert_eq!(parse_pos(""), None);
    }

    #[test]
    fn the_distribution_counts_the_most_first() {
        let mut w = Map::default();
        w.0.insert([0, 0, 0], 1);
        w.0.insert([1, 0, 0], 1);
        w.0.insert([2, 0, 0], 2);
        let c = Cuboid::new([0, 0, 0], [2, 0, 0]);
        let d = distribution(&w, &c.positions());
        assert_eq!(d, vec![(1, 2), (2, 1)]);
    }

    #[test]
    fn trimming_keeps_the_blocks_that_are_there() {
        let mut w = Map::default();
        w.0.insert([1, 2, 3], 1);
        w.0.insert([4, 5, 6], 1);
        let c = Cuboid::new([0, 0, 0], [6, 6, 6]);
        let t = trim_to_existing(&w, &c, 0).unwrap();
        assert_eq!((t.min, t.max), ([1, 2, 3], [4, 5, 6]));
        let empty = Cuboid::new([20, 20, 20], [21, 21, 21]);
        assert!(trim_to_existing(&w, &empty, 0).is_none());
    }

    #[test]
    fn a_hole_is_filled_down_to_the_ground() {
        let mut w = Map::default();
        // The ground at y = 0, air above it.
        for x in -2..=2 {
            for z in -2..=2 {
                w.0.insert([x, 0, z], 1);
            }
        }
        let stone = Pattern::single(1);
        let writes = fill_hole(&w, [0, 2, 0], 2, 3, &stone, false, 0).unwrap();
        // The disc of radius 2 is 21 columns, and the ball narrows with the
        // height: the outer columns are one block deep, the middle two.
        let columns: std::collections::HashSet<(i32, i32)> =
            writes.iter().map(|(p, _)| (p[0], p[2])).collect();
        assert_eq!(columns.len(), 21);
        assert!(writes.contains(&([0, 2, 0], 1)) && writes.contains(&([0, 1, 0], 1)));
        assert!(writes.iter().all(|(p, _)| p[1] >= 1), "not through the ground");
        // The deeper fill stops at the ground too.
        let deep = fill_hole(&w, [0, 2, 0], 1, 5, &stone, false, 0).unwrap();
        assert!(deep.iter().all(|(p, _)| p[1] >= 1), "not through the ground");
        // The recursive fill takes the whole air of the ball.
        let all = fill_hole(&w, [0, 2, 0], 2, 5, &stone, true, 0).unwrap();
        assert!(all.iter().any(|(p, _)| *p == [0, 1, 0]));
        assert!(all.iter().all(|(p, _)| p[1] >= 1), "not through the ground");
        assert!(fill_hole(&w, [0, 2, 0], 0, 1, &stone, false, 0).is_err());
        assert!(fill_hole(&w, [0, 2, 0], 2, 0, &stone, false, 0).is_err());
    }

    #[test]
    fn blocks_above_and_below_the_player_go() {
        let mut w = Map::default();
        for y in 0..5 {
            w.0.insert([0, y, 0], 1);
        }
        w.0.insert([1, 1, 1], 1);
        let up = remove_column(&w, [0, 1, 0], 0, 3, true, 0);
        assert_eq!(up, vec![[0, 2, 0], [0, 3, 0], [0, 4, 0]]);
        let down = remove_column(&w, [0, 2, 0], 1, 2, false, 0);
        assert_eq!(down.len(), 3, "the square around the player");
        assert!(down.contains(&[0, 0, 0]) && down.contains(&[0, 1, 0]));
        assert!(down.contains(&[1, 1, 1]), "the square is one block wide");
    }
}
