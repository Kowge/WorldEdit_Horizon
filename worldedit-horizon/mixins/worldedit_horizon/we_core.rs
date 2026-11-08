// WorldEdit's rules, with no game types: selections, shapes, patterns,
// edits with their history, and the clipboard. The glue (we_glue) maps
// them onto the game's blocks (a block is its state id, a u32).
//
// The file is a module of game in the bundle and of this crate in its
// tests, under the same name. Module payloads cannot start with inner
// doc comments, so these are ordinary comments.
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicI32, AtomicU64, AtomicUsize, Ordering};

pub type Pos = [i32; 3];

/// The defaults of the limits (`/we limit` changes them while playing).
/// The most blocks one command may touch.
pub const MAX_BLOCKS: u64 = 500_000;
/// The largest radius of a sphere or a cylinder.
pub const MAX_RADIUS: i32 = 128;
/// How many edits a player can undo.
pub const HISTORY: usize = 20;
/// How far ahead of the player a brush puts its shape when there is nothing
/// to paint.
pub const MAX_DISTANCE: i32 = 64;
/// The farthest a brush may reach when the limit allows it.
pub const MAX_DISTANCE_LIMIT: i32 = 256;

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
static DISTANCE_NOW: AtomicI32 = AtomicI32::new(MAX_DISTANCE);

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
/// How far ahead of the player a brush reaches now, in blocks.
pub fn max_distance() -> i32 {
    DISTANCE_NOW.load(Ordering::Relaxed)
}

/// The limits as a line.
pub fn limits() -> String {
    format!(
        "Limits: blocks {} | radius {} | distance {} | history {}. Change: limit blocks|radius|distance|history <n>, limit reset.",
        max_blocks(),
        max_radius(),
        max_distance(),
        history()
    )
}

/// Sets one limit (`blocks`, `radius`, `distance`, `history`) or puts them all
/// back (`reset`); the line to tell the player.
pub fn set_limit(what: &str, value: Option<i64>) -> Result<String, String> {
    match (what, value) {
        ("reset", _) => {
            BLOCKS_NOW.store(MAX_BLOCKS, Ordering::Relaxed);
            RADIUS_NOW.store(MAX_RADIUS, Ordering::Relaxed);
            HISTORY_NOW.store(HISTORY, Ordering::Relaxed);
            DISTANCE_NOW.store(MAX_DISTANCE, Ordering::Relaxed);
        }
        ("blocks", Some(v)) if (1..=20_000_000).contains(&v) => {
            BLOCKS_NOW.store(v as u64, Ordering::Relaxed)
        }
        ("radius", Some(v)) if (1..=1024).contains(&v) => {
            RADIUS_NOW.store(v as i32, Ordering::Relaxed)
        }
        ("distance", Some(v)) if (1..=MAX_DISTANCE_LIMIT as i64).contains(&v) => {
            DISTANCE_NOW.store(v as i32, Ordering::Relaxed)
        }
        ("history", Some(v)) if (1..=200).contains(&v) => {
            HISTORY_NOW.store(v as usize, Ordering::Relaxed)
        }
        ("blocks", _) => return Err("blocks: 1 to 20000000.".to_owned()),
        ("radius", _) => return Err("radius: 1 to 1024.".to_owned()),
        ("distance", _) => {
            return Err(format!("distance: 1 to {MAX_DISTANCE_LIMIT}."));
        }
        ("history", _) => return Err("history: 1 to 200.".to_owned()),
        _ => return Err("limit blocks|radius|distance|history <n>, or limit reset.".to_owned()),
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

/// The shape of a selection: a plain box, or a round shape inside it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Shape {
    #[default]
    Cube,
    Sphere,
    Cylinder,
    Ellipsoid,
}

impl Shape {
    /// The shape a word names, as in `//sel sphere`.
    pub fn parse(word: &str) -> Option<Shape> {
        Some(match word.to_ascii_lowercase().as_str() {
            "cube" | "box" => Shape::Cube,
            "sphere" | "ball" => Shape::Sphere,
            "cylinder" | "cyl" => Shape::Cylinder,
            "ellipsoid" => Shape::Ellipsoid,
            _ => return None,
        })
    }
    /// The shape's name, as a command writes it.
    pub fn name(&self) -> &'static str {
        match self {
            Shape::Cube => "cube",
            Shape::Sphere => "sphere",
            Shape::Cylinder => "cylinder",
            Shape::Ellipsoid => "ellipsoid",
        }
    }
}

/// The distance between two blocks, rounded.
fn distance(a: Pos, b: Pos) -> i32 {
    let d = sub(b, a);
    let (x, y, z) = (d[0] as f64, d[1] as f64, d[2] as f64);
    (x * x + y * y + z * z).sqrt().round() as i32
}

/// The distance between two blocks, sideways only.
fn horizontal_distance(a: Pos, b: Pos) -> i32 {
    let dx = (b[0] - a[0]) as f64;
    let dz = (b[2] - a[2]) as f64;
    (dx * dx + dz * dz).sqrt().round() as i32
}

/// A selected volume: a box, and the shape inside it (vanilla's `//sel`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    /// The box the shape sits in.
    pub min: Pos,
    pub max: Pos,
    pub shape: Shape,
    /// The shape's middle: the first corner for a sphere and an ellipsoid,
    /// the middle of the bottom for a cylinder, the box's middle for a cube.
    pub center: Pos,
    /// The shape's own sizes: a cube's three sides, a sphere's radius, a
    /// cylinder's radius and height, an ellipsoid's three radii.
    pub sizes: [i32; 3],
}

impl Region {
    /// The box between two corners: a plain cube.
    pub fn cube(a: Pos, b: Pos) -> Region {
        let c = Cuboid::new(a, b);
        let s = c.size();
        Region {
            min: c.min,
            max: c.max,
            shape: Shape::Cube,
            center: [
                c.min[0] + (s[0] - 1) / 2,
                c.min[1] + (s[1] - 1) / 2,
                c.min[2] + (s[2] - 1) / 2,
            ],
            sizes: s,
        }
    }
    /// A shape from the two corners the player picked: for a cube they are
    /// the box's corners, for a sphere the first is its middle and the
    /// second a point on its surface, for a cylinder the first the middle of
    /// the bottom and the second a point on the top rim, for an ellipsoid
    /// the first its middle and the second one of its corners.
    pub fn from_corners(pos1: Pos, pos2: Pos, shape: Shape) -> Region {
        match shape {
            Shape::Cube => Region::cube(pos1, pos2),
            Shape::Sphere => {
                let r = distance(pos1, pos2).max(1);
                Region {
                    min: [pos1[0] - r, pos1[1] - r, pos1[2] - r],
                    max: [pos1[0] + r, pos1[1] + r, pos1[2] + r],
                    shape,
                    center: pos1,
                    sizes: [r, 0, 0],
                }
            }
            Shape::Cylinder => {
                let r = horizontal_distance(pos1, pos2).max(1);
                let h = (pos2[1] - pos1[1]).abs().max(1);
                Region {
                    min: [pos1[0] - r, pos1[1], pos1[2] - r],
                    max: [pos1[0] + r, pos1[1] + h - 1, pos1[2] + r],
                    shape,
                    center: pos1,
                    sizes: [r, h, 0],
                }
            }
            Shape::Ellipsoid => {
                let radii = [
                    (pos2[0] - pos1[0]).abs().max(1),
                    (pos2[1] - pos1[1]).abs().max(1),
                    (pos2[2] - pos1[2]).abs().max(1),
                ];
                Region {
                    min: [
                        pos1[0] - radii[0],
                        pos1[1] - radii[1],
                        pos1[2] - radii[2],
                    ],
                    max: [
                        pos1[0] + radii[0],
                        pos1[1] + radii[1],
                        pos1[2] + radii[2],
                    ],
                    shape,
                    center: pos1,
                    sizes: radii,
                }
            }
        }
    }
    /// A shape around a point, with the sizes the player gave: a cube's
    /// side, a sphere's radius, a cylinder's radius and height, an
    /// ellipsoid's three radii.
    pub fn around(shape: Shape, center: Pos, sizes: &[i32]) -> Result<Region, String> {
        let want = |n: usize, what: &str| -> Result<(), String> {
            if sizes.len() >= n {
                Ok(())
            } else {
                Err(format!("A {} needs {what}.", shape.name()))
            }
        };
        match shape {
            Shape::Cube => {
                want(1, "side")?;
                let s = sizes[0].max(1);
                let half = s / 2;
                Ok(Region::cube(
                    [center[0] - half, center[1] - half, center[2] - half],
                    [
                        center[0] + s - 1 - half,
                        center[1] + s - 1 - half,
                        center[2] + s - 1 - half,
                    ],
                ))
            }
            Shape::Sphere => {
                want(1, "radius")?;
                let r = sizes[0].max(1);
                Ok(Region::from_corners(
                    center,
                    [center[0] + r, center[1], center[2]],
                    shape,
                ))
            }
            Shape::Cylinder => {
                want(1, "radius")?;
                let r = sizes[0].max(1);
                let h = sizes.get(1).copied().unwrap_or(1).max(1);
                Ok(Region::from_corners(
                    center,
                    [center[0] + r, center[1] + h, center[2]],
                    shape,
                ))
            }
            Shape::Ellipsoid => {
                want(3, "three radii")?;
                let radii = [sizes[0].max(1), sizes[1].max(1), sizes[2].max(1)];
                Ok(Region::from_corners(
                    center,
                    [
                        center[0] + radii[0],
                        center[1] + radii[1],
                        center[2] + radii[2],
                    ],
                    shape,
                ))
            }
        }
    }
    /// Blocks along each axis of the box.
    pub fn size(&self) -> [i32; 3] {
        let d = sub(self.max, self.min);
        [d[0] + 1, d[1] + 1, d[2] + 1]
    }
    /// The blocks of the selection. For a round shape the box is only an
    /// upper bound: the count is exact, but a box too big for the limit is
    /// refused before it is counted.
    pub fn volume(&self) -> u64 {
        let s = self.size();
        let box_volume = s[0] as u64 * s[1] as u64 * s[2] as u64;
        if self.shape == Shape::Cube || box_volume > max_blocks() {
            return box_volume;
        }
        let mut n = 0u64;
        for y in self.min[1]..=self.max[1] {
            for x in self.min[0]..=self.max[0] {
                for z in self.min[2]..=self.max[2] {
                    if self.contains([x, y, z]) {
                        n += 1;
                    }
                }
            }
        }
        n
    }
    /// Whether a block is inside the selection.
    pub fn contains(&self, p: Pos) -> bool {
        match self.shape {
            Shape::Cube => (0..3).all(|i| p[i] >= self.min[i] && p[i] <= self.max[i]),
            Shape::Sphere => {
                let d = sub(p, self.center);
                let r = self.sizes[0] as i64;
                (d[0] as i64) * (d[0] as i64) + (d[1] as i64) * (d[1] as i64)
                    + (d[2] as i64) * (d[2] as i64)
                    <= r * r
            }
            Shape::Cylinder => {
                let dx = (p[0] - self.center[0]) as i64;
                let dz = (p[2] - self.center[2]) as i64;
                let r = self.sizes[0] as i64;
                dx * dx + dz * dz <= r * r && p[1] >= self.min[1] && p[1] <= self.max[1]
            }
            Shape::Ellipsoid => {
                let mut sum = 0.0f64;
                for i in 0..3 {
                    let d = (p[i] - self.center[i]) as f64 / self.sizes[i] as f64;
                    sum += d * d;
                }
                sum <= 1.0
            }
        }
    }
    /// Every block of the selection, lowest layer first.
    pub fn positions(&self) -> Vec<Pos> {
        if self.shape == Shape::Cube {
            return Cuboid {
                min: self.min,
                max: self.max,
            }
            .positions();
        }
        let mut out = Vec::new();
        for y in self.min[1]..=self.max[1] {
            for x in self.min[0]..=self.max[0] {
                for z in self.min[2]..=self.max[2] {
                    let p = [x, y, z];
                    if self.contains(p) {
                        out.push(p);
                    }
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
                !self.contains([p[0] - 1, p[1], p[2]])
                    || !self.contains([p[0] + 1, p[1], p[2]])
                    || !self.contains([p[0], p[1], p[2] - 1])
                    || !self.contains([p[0], p[1], p[2] + 1])
            })
            .collect()
    }
    /// The six sides.
    pub fn faces(&self) -> Vec<Pos> {
        self.positions()
            .into_iter()
            .filter(|p| {
                !(self.contains([p[0] - 1, p[1], p[2]])
                    && self.contains([p[0] + 1, p[1], p[2]])
                    && self.contains([p[0], p[1] - 1, p[2]])
                    && self.contains([p[0], p[1] + 1, p[2]])
                    && self.contains([p[0], p[1], p[2] - 1])
                    && self.contains([p[0], p[1], p[2] + 1]))
            })
            .collect()
    }
    /// Grown by `n` blocks on every side (a negative `n` shrinks it).
    pub fn grown(&self, n: i32) -> Option<Region> {
        match self.shape {
            Shape::Cube => {
                let c = Cuboid {
                    min: [
                        self.min[0] - n,
                        self.min[1] - n,
                        self.min[2] - n,
                    ],
                    max: [
                        self.max[0] + n,
                        self.max[1] + n,
                        self.max[2] + n,
                    ],
                };
                (c.min[0] <= c.max[0] && c.min[1] <= c.max[1] && c.min[2] <= c.max[2])
                    .then(|| Region::cube(c.min, c.max))
            }
            Shape::Sphere => {
                let r = self.sizes[0] + n;
                (r >= 1).then(|| {
                    Region::from_corners(
                        self.center,
                        [self.center[0] + r, self.center[1], self.center[2]],
                        self.shape,
                    )
                })
            }
            Shape::Cylinder => {
                let r = self.sizes[0] + n;
                let h = self.sizes[1] + n;
                (r >= 1 && h >= 1).then(|| {
                    Region::from_corners(
                        self.center,
                        [self.center[0] + r, self.center[1] + h, self.center[2]],
                        self.shape,
                    )
                })
            }
            Shape::Ellipsoid => {
                let radii = [
                    self.sizes[0] + n,
                    self.sizes[1] + n,
                    self.sizes[2] + n,
                ];
                (radii.iter().all(|r| *r >= 1)).then(|| {
                    Region::from_corners(
                        self.center,
                        [
                            self.center[0] + radii[0],
                            self.center[1] + radii[1],
                            self.center[2] + radii[2],
                        ],
                        self.shape,
                    )
                })
            }
        }
    }
    /// Grows the selection by `amount` blocks towards `dir` (a negative
    /// amount shrinks that side).
    pub fn expand(&self, dir: Dir, amount: i32) -> Option<Region> {
        let v = dir.vector();
        let axis = (0..3).find(|&i| v[i] != 0)?;
        match self.shape {
            Shape::Cube => {
                let mut c = *self;
                if v[axis] > 0 {
                    c.max[axis] += amount;
                } else {
                    c.min[axis] -= amount;
                }
                (c.min[axis] <= c.max[axis]).then(|| Region::cube(c.min, c.max))
            }
            Shape::Sphere => self.grown(amount),
            Shape::Cylinder => {
                if axis == 1 {
                    let h = self.sizes[1] + amount;
                    (h >= 1).then(|| {
                        Region::from_corners(
                            self.center,
                            [self.center[0] + self.sizes[0], self.center[1] + h, self.center[2]],
                            self.shape,
                        )
                    })
                } else {
                    let r = self.sizes[0] + amount;
                    (r >= 1).then(|| {
                        Region::from_corners(
                            self.center,
                            [self.center[0] + r, self.center[1] + self.sizes[1], self.center[2]],
                            self.shape,
                        )
                    })
                }
            }
            Shape::Ellipsoid => {
                let mut radii = self.sizes;
                radii[axis] += amount;
                (radii[axis] >= 1).then(|| {
                    Region::from_corners(
                        self.center,
                        [
                            self.center[0] + radii[0],
                            self.center[1] + radii[1],
                            self.center[2] + radii[2],
                        ],
                        self.shape,
                    )
                })
            }
        }
    }
    /// Pulls the side facing `dir` in by `amount` blocks.
    pub fn contract(&self, dir: Dir, amount: i32) -> Option<Region> {
        self.expand(dir, -amount)
    }
    /// The selection moved by `offset`.
    pub fn shifted(&self, offset: Pos) -> Region {
        Region {
            min: add(self.min, offset),
            max: add(self.max, offset),
            shape: self.shape,
            center: add(self.center, offset),
            sizes: self.sizes,
        }
    }
    /// The box of the selection.
    pub fn cuboid(&self) -> Cuboid {
        Cuboid {
            min: self.min,
            max: self.max,
        }
    }
}

/// The chunk a block is in.
pub fn chunk_of(pos: Pos) -> (i32, i32) {
    (pos[0].div_euclid(16), pos[2].div_euclid(16))
}

/// The chunks a selection covers, in reading order.
pub fn chunks_of(region: &Region) -> Vec<(i32, i32)> {
    let (min_x, min_z) = chunk_of(region.min);
    let (max_x, max_z) = chunk_of(region.max);
    let mut out = Vec::new();
    for cx in min_x..=max_x {
        for cz in min_z..=max_z {
            out.push((cx, cz));
        }
    }
    out
}

/// How many blocks of a chunk are not air.
pub fn chunk_filled<W: World>(
    world: &W,
    chunk: (i32, i32),
    bottom: i32,
    top: i32,
    air: u32,
) -> u64 {
    let mut n = 0u64;
    for y in bottom..=top {
        for x in chunk.0 * 16..=chunk.0 * 16 + 15 {
            for z in chunk.1 * 16..=chunk.1 * 16 + 15 {
                if world.get([x, y, z]) != air {
                    n += 1;
                }
            }
        }
    }
    n
}

/// A player's two corners, in a dimension.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub dimension: String,
    pub pos1: Option<Pos>,
    pub pos2: Option<Pos>,
    /// The shape of the selection: a plain box, or a round shape.
    pub shape: Shape,
    /// The player's own view of it: whether the grid inside the selection is
    /// drawn on the client. The frame around it stays either way.
    pub grid: bool,
}

impl Default for Selection {
    fn default() -> Self {
        Selection {
            dimension: String::new(),
            pos1: None,
            pos2: None,
            shape: Shape::Cube,
            grid: true,
        }
    }
}

impl Selection {
    pub fn set_pos(&mut self, index: u8, dimension: &str, pos: Pos) {
        if self.dimension != dimension {
            // Another level: the corners go, the player's own view stays.
            let grid = self.grid;
            *self = Selection {
                dimension: dimension.to_owned(),
                grid,
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
    /// The selection as a volume, with its shape.
    pub fn region(&self) -> Option<Region> {
        Some(Region::from_corners(self.pos1?, self.pos2?, self.shape))
    }
    pub fn set_cuboid(&mut self, c: Cuboid) {
        self.pos1 = Some(c.min);
        self.pos2 = Some(c.max);
        self.shape = Shape::Cube;
    }
    /// Takes a volume as the selection: the corners are the ones that make
    /// the shape again.
    pub fn set_region(&mut self, r: &Region) {
        self.shape = r.shape;
        match r.shape {
            Shape::Cube => {
                self.pos1 = Some(r.min);
                self.pos2 = Some(r.max);
            }
            Shape::Sphere => {
                self.pos1 = Some(r.center);
                self.pos2 = Some([r.center[0] + r.sizes[0], r.center[1], r.center[2]]);
            }
            Shape::Cylinder => {
                self.pos1 = Some([r.center[0], r.min[1], r.center[2]]);
                // The height is the number of rows, so the second corner sits
                // one row above the top of the cylinder.
                self.pos2 = Some([r.center[0] + r.sizes[0], r.max[1] + 1, r.center[2]]);
            }
            Shape::Ellipsoid => {
                self.pos1 = Some(r.center);
                self.pos2 = Some([
                    r.center[0] + r.sizes[0],
                    r.center[1] + r.sizes[1],
                    r.center[2] + r.sizes[2],
                ]);
            }
        }
    }
    pub fn clear(&mut self) {
        self.pos1 = None;
        self.pos2 = None;
        self.shape = Shape::Cube;
    }
}

/// The channel the selection travels to the client on (`mod_id:name`).
pub const SEL_CHANNEL: &str = "worldedit_horizon:sel";

/// The channel the wand's clicks travel to the server on (`mod_id:name`).
pub const WAND_CHANNEL: &str = "worldedit_horizon:wand";

/// The channel the brush's clicks in the air travel to the server on.
pub const BRUSH_CHANNEL: &str = "worldedit_horizon:brush";

/// A brush's click in the air: whether it erases.
pub fn brush_payload(erase: bool) -> Vec<u8> {
    vec![u8::from(erase)]
}

/// The click of a brush in the air. None when the message is broken.
pub fn parse_brush(data: &[u8]) -> Option<bool> {
    match data.first()? {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}

/// A wand's click: the block it was made on, three little-endian coordinates.
pub fn wand_payload(pos: Pos) -> Vec<u8> {
    let mut out = Vec::with_capacity(12);
    for v in pos {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

/// The block a wand's click was made on. None when the message is broken.
pub fn parse_wand(data: &[u8]) -> Option<Pos> {
    if data.len() != 12 {
        return None;
    }
    let mut pos = [0i32; 3];
    for (i, v) in pos.iter_mut().enumerate() {
        *v = i32::from_le_bytes(data.get(i * 4..i * 4 + 4)?.try_into().ok()?);
    }
    Some(pos)
}

/// The shape as a byte, so the client knows what it draws.
fn shape_id(shape: Shape) -> u8 {
    match shape {
        Shape::Cube => 0,
        Shape::Sphere => 1,
        Shape::Cylinder => 2,
        Shape::Ellipsoid => 3,
    }
}

/// The selection as the client draws it: the kind, the flags, the shape, the
/// dimension, the shape's middle and sizes and the box the blocks fill. The
/// client reads it in its `geometry.rs`.
pub fn selection_payload(selection: &Selection) -> Vec<u8> {
    let Some(region) = selection.region() else {
        // No selection: the client draws nothing.
        return vec![0];
    };
    let dimension = selection.dimension.as_bytes();
    let dimension = &dimension[..dimension.len().min(255)];
    let mut out = Vec::with_capacity(4 + dimension.len() + 48);
    out.push(1);
    // The flags: the first bit says that the grid inside the selection is
    // drawn.
    out.push(u8::from(selection.grid));
    out.push(shape_id(region.shape));
    out.push(dimension.len() as u8);
    out.extend_from_slice(dimension);
    for v in region
        .center
        .iter()
        .chain(region.sizes.iter())
        .chain(region.min.iter())
        .chain(region.max.iter())
    {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

/// The corners, the shape and the player's own view of the selection: the
/// client's drawing changes when these change.
pub fn selection_state(selection: &Selection) -> (Option<Pos>, Option<Pos>, Shape, bool) {
    (
        selection.pos1,
        selection.pos2,
        selection.shape,
        selection.grid,
    )
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

/// The direction a player looks at, as a unit vector: the yaw 0 is south
/// (+z) and the pitch is positive downwards, as in the game.
pub fn look(yaw: f64, pitch: f64) -> [f64; 3] {
    let (y, p) = (yaw.to_radians(), pitch.to_radians());
    [-y.sin() * p.cos(), -p.sin(), y.cos() * p.cos()]
}

/// The block `distance` blocks ahead of the eye.
pub fn ahead(eye: [f64; 3], dir: [f64; 3], distance: f64) -> Pos {
    [
        (eye[0] + dir[0] * distance).floor() as i32,
        (eye[1] + dir[1] * distance).floor() as i32,
        (eye[2] + dir[2] * distance).floor() as i32,
    ]
}

/// What a ray found: the blocks it went through and whether it ended on a
/// block that is not air.
#[derive(Clone, Debug, PartialEq)]
pub struct Ray {
    /// The blocks the ray went through, in order. The block the eye is in is
    /// not among them.
    pub blocks: Vec<Pos>,
    /// Whether the ray ended on a block that is not air: it hit something.
    pub solid: bool,
}

impl Ray {
    /// The block the ray met: the first one that is not air. None when it met
    /// nothing.
    pub fn hit(&self) -> Option<Pos> {
        if self.solid {
            self.blocks.last().copied()
        } else {
            None
        }
    }
}

/// The blocks a ray goes through from the eye along `dir`, up to `distance`
/// blocks away. It stops at the first block that is not air, and that block is
/// the last one. When it hits nothing, it goes on to the distance.
///
/// The blocks in `own` are as if they were air: the brush goes through the
/// blocks it covered last, so that it does not shoot at its own work.
pub fn ray_blocks(
    world: &impl World,
    eye: [f64; 3],
    dir: [f64; 3],
    distance: f64,
    air: u32,
    own: &HashSet<Pos>,
) -> Ray {
    let mut blocks = Vec::new();
    if distance <= 0.0 {
        return Ray {
            blocks,
            solid: false,
        };
    }
    let mut pos = [
        eye[0].floor() as i32,
        eye[1].floor() as i32,
        eye[2].floor() as i32,
    ];
    // On every axis: the way the ray goes, the length of one block on it and
    // where the next block starts.
    let mut step = [0i32; 3];
    let mut delta = [f64::INFINITY; 3];
    let mut next = [f64::INFINITY; 3];
    for i in 0..3 {
        if dir[i] > 0.0 {
            step[i] = 1;
            delta[i] = 1.0 / dir[i];
            next[i] = (f64::from(pos[i]) + 1.0 - eye[i]) / dir[i];
        } else if dir[i] < 0.0 {
            step[i] = -1;
            delta[i] = -1.0 / dir[i];
            next[i] = (f64::from(pos[i]) - eye[i]) / dir[i];
        }
    }
    let mut solid = false;
    loop {
        // The next block the ray reaches.
        let t = next[0].min(next[1]).min(next[2]);
        if !t.is_finite() || t > distance {
            break;
        }
        // The axis the ray crosses first.
        if next[0] <= next[1] && next[0] <= next[2] {
            pos[0] += step[0];
            next[0] += delta[0];
        } else if next[1] <= next[2] {
            pos[1] += step[1];
            next[1] += delta[1];
        } else {
            pos[2] += step[2];
            next[2] += delta[2];
        }
        blocks.push(pos);
        // The first block that is not air ends the ray. The blocks the brush
        // covered last are as if they were air.
        if world.get(pos) != air && !own.contains(&pos) {
            solid = true;
            break;
        }
    }
    Ray { blocks, solid }
}

/// A brush: the shape it leaves and the pattern it paints with.
#[derive(Clone, Debug, PartialEq)]
pub struct Brush {
    /// The shape the brush leaves.
    pub shape: Shape,
    /// The shape's sizes: a cube's side, a sphere's radius, a cylinder's
    /// radius and height, an ellipsoid's three radii.
    pub sizes: [i32; 3],
    /// The pattern it paints with.
    pub pattern: Pattern,
    /// The pattern as it was typed, for the messages.
    pub text: String,
    /// How far ahead of the player the brush works, in blocks. Without it
    /// the brush works on the block it was used on.
    pub distance: Option<i32>,
}

impl Brush {
    /// A brush from `shape sizes... pattern`, as in `sphere 5 stone`.
    pub fn parse(text: &str, resolve: Resolver<'_>) -> Result<Brush, String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() < 3 {
            return Err(
                "A brush is a shape, its sizes and a pattern, as in `sphere 5 stone`.".to_owned(),
            );
        }
        let shape = Shape::parse(words[0]).ok_or_else(|| {
            format!(
                "`{}` is not a brush shape (cube sphere cylinder ellipsoid).",
                words[0]
            )
        })?;
        let wanted = match shape {
            Shape::Cube | Shape::Sphere => 1,
            Shape::Cylinder => 2,
            Shape::Ellipsoid => 3,
        };
        let pattern_text = words[words.len() - 1];
        let sizes = &words[1..words.len() - 1];
        if sizes.len() != wanted {
            return Err(format!(
                "A {} brush takes {}, as in `{}`.",
                shape.name(),
                match wanted {
                    1 => "one size",
                    2 => "a radius and a height",
                    _ => "three radii",
                },
                example(shape)
            ));
        }
        let mut out = [0i32; 3];
        for (i, word) in sizes.iter().enumerate() {
            let n: i32 = word
                .parse()
                .map_err(|_| format!("`{word}` is not a number."))?;
            if !(1..=max_radius()).contains(&n) {
                return Err(format!("A brush size is 1 to {}.", max_radius()));
            }
            out[i] = n;
        }
        let pattern = Pattern::parse(pattern_text, resolve)?;
        Ok(Brush {
            shape,
            sizes: out,
            pattern,
            text: pattern_text.to_owned(),
            distance: None,
        })
    }

    /// The blocks the brush covers when it is used on `center`: the shape
    /// sits in the middle of the block it was used on.
    pub fn positions(&self, center: Pos) -> Result<Vec<Pos>, String> {
        match self.shape {
            Shape::Cube => {
                let side = self.sizes[0];
                check_limit(side as u64 * side as u64 * side as u64)?;
                let low = -(side / 2);
                let mut out = Vec::with_capacity((side * side * side) as usize);
                for x in low..=low + side - 1 {
                    for y in low..=low + side - 1 {
                        for z in low..=low + side - 1 {
                            out.push(add(center, [x, y, z]));
                        }
                    }
                }
                Ok(out)
            }
            Shape::Sphere => {
                let r = self.sizes[0];
                // The limit is checked before the ball is built: a big brush
                // must not cost a big allocation.
                check_limit((2 * r as u64 + 1).pow(3))?;
                if r == 0 {
                    // A radius of zero is the block itself.
                    return Ok(vec![center]);
                }
                sphere(center, r, false)
            }
            Shape::Cylinder => {
                let (radius, height) = (self.sizes[0], self.sizes[1]);
                check_limit((2 * radius as u64 + 1).pow(2) * height as u64)?;
                let inside = in_circle(radius);
                let disc = disc(radius, &inside, false);
                let low = -(height / 2);
                let mut out = Vec::with_capacity(disc.len() * height as usize);
                for dy in low..=low + height - 1 {
                    for &(x, z) in &disc {
                        out.push(add(center, [x, dy, z]));
                    }
                }
                Ok(out)
            }
            Shape::Ellipsoid => {
                let [rx, ry, rz] = self.sizes;
                check_limit((2 * rx as u64 + 1) * (2 * ry as u64 + 1) * (2 * rz as u64 + 1))?;
                let (rx, ry, rz) = (rx as f64 + 0.5, ry as f64 + 0.5, rz as f64 + 0.5);
                let mut out = Vec::new();
                for y in -self.sizes[1]..=self.sizes[1] {
                    for x in -self.sizes[0]..=self.sizes[0] {
                        for z in -self.sizes[2]..=self.sizes[2] {
                            let inside = (x as f64 / rx).powi(2)
                                + (y as f64 / ry).powi(2)
                                + (z as f64 / rz).powi(2);
                            if inside <= 1.0 {
                                out.push(add(center, [x, y, z]));
                            }
                        }
                    }
                }
                Ok(out)
            }
        }
    }

    /// The brush's radius: a sphere's and a cylinder's radius, a cube's half
    /// side, an ellipsoid's longest radius.
    pub fn radius(&self) -> i32 {
        match self.shape {
            Shape::Cube => (self.sizes[0] - 1) / 2,
            Shape::Sphere | Shape::Cylinder => self.sizes[0],
            Shape::Ellipsoid => self.sizes[0].max(self.sizes[1]).max(self.sizes[2]),
        }
    }

    /// The same brush with another radius: a cube keeps its shape, an
    /// ellipsoid keeps its proportions.
    pub fn with_radius(&self, radius: i32) -> Result<Brush, String> {
        if !(0..=max_radius()).contains(&radius) {
            return Err(format!("A brush radius is 0 to {}.", max_radius()));
        }
        let mut sizes = self.sizes;
        match self.shape {
            Shape::Cube => sizes[0] = radius * 2 + 1,
            Shape::Sphere | Shape::Cylinder => sizes[0] = radius,
            Shape::Ellipsoid => {
                let was = self.radius().max(1);
                for s in sizes.iter_mut() {
                    *s = ((*s as i64 * radius as i64).div_euclid(was as i64)) as i32;
                }
            }
        }
        Ok(Brush {
            shape: self.shape,
            sizes,
            pattern: self.pattern.clone(),
            text: self.text.clone(),
            distance: self.distance,
        })
    }

    /// The same brush, working `distance` blocks ahead of the player
    /// instead of on the block it was used on. Without a distance the brush
    /// works where it was used.
    pub fn with_distance(&self, distance: Option<i32>) -> Result<Brush, String> {
        if let Some(d) = distance {
            if !(1..=max_distance()).contains(&d) {
                return Err(format!("The distance is 1 to {} blocks.", max_distance()));
            }
        }
        Ok(Brush {
            shape: self.shape,
            sizes: self.sizes,
            pattern: self.pattern.clone(),
            text: self.text.clone(),
            distance,
        })
    }

    /// The same brush, working the way `other` worked: a new brush keeps the
    /// distance of the old one.
    pub fn with_distance_of(&self, other: &Brush) -> Result<Brush, String> {
        match other.distance {
            Some(distance) => self.with_distance(Some(distance.min(max_distance()))),
            None => Ok(self.clone()),
        }
    }

    /// The brush as the messages write it.
    pub fn describe(&self) -> String {
        let shape = match self.shape {
            Shape::Cube => format!("cube {}", self.sizes[0]),
            Shape::Sphere => format!("sphere r={}", self.sizes[0]),
            Shape::Cylinder => format!("cylinder r={} h={}", self.sizes[0], self.sizes[1]),
            Shape::Ellipsoid => format!(
                "ellipsoid {}x{}x{}",
                self.sizes[0], self.sizes[1], self.sizes[2]
            ),
        };
        let mut out = format!("{shape}, {}", self.text);
        if let Some(distance) = self.distance {
            out.push_str(&format!(", {distance} blocks ahead"));
        }
        out
    }
}

/// How a shape's brush is written, for the messages.
fn example(shape: Shape) -> &'static str {
    match shape {
        Shape::Cube => "cube 5 stone",
        Shape::Sphere => "sphere 5 stone",
        Shape::Cylinder => "cylinder 3 4 stone",
        Shape::Ellipsoid => "ellipsoid 4 2 4 stone",
    }
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

/// The box around the blocks that are not air in `r` (vanilla's `//trim`).
/// None when there is nothing but air. The shape stays, its box shrinks.
pub fn trim_to_existing<W: World>(world: &W, r: &Region, air: u32) -> Option<Region> {
    let mut min = r.max;
    let mut max = r.min;
    let mut any = false;
    for p in r.positions() {
        if world.get(p) == air {
            continue;
        }
        any = true;
        for i in 0..3 {
            min[i] = min[i].min(p[i]);
            max[i] = max[i].max(p[i]);
        }
    }
    if !any {
        return None;
    }
    // The shape keeps its own bounds: only the box around it shrinks.
    let min = [
        min[0].max(r.min[0]),
        min[1].max(r.min[1]),
        min[2].max(r.min[2]),
    ];
    let max = [
        max[0].min(r.max[0]),
        max[1].min(r.max[1]),
        max[2].min(r.max[2]),
    ];
    Some(Region {
        min,
        max,
        shape: r.shape,
        center: r.center,
        sizes: r.sizes,
    })
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
    region: &Region,
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
    region: &Region,
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
    region: &Region,
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
    /// The size of the box the blocks came from, when the clipboard holds a
    /// schematic: a paste then puts the box in front of the player, in the
    /// middle of the way they look.
    pub size: Option<[i32; 3]>,
}

impl Clipboard {
    pub fn copy<W: World>(world: &W, region: &Region, origin: Pos) -> Clipboard {
        Clipboard {
            blocks: region
                .positions()
                .into_iter()
                .map(|p| (sub(p, origin), world.get(p)))
                .collect(),
            size: None,
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

/// The magic at the start of a schematic file.
const SCHEMATIC_MAGIC: &[u8; 4] = b"WEHS";
/// The format of a schematic file this version writes and reads.
const SCHEMATIC_FORMAT: u32 = 1;
/// The bytes of the header of a schematic file: the magic, the format, the
/// size of the box and the number of blocks.
const SCHEMATIC_HEADER: usize = 24;

/// The mod's own folder in the world's folder.
pub const DATA_FOLDER: &str = "worldedit_horizon";
/// The folder of the schematics in it.
pub const SCHEMATICS_FOLDER: &str = "schematics";
/// The extension of a schematic file.
pub const SCHEMATIC_EXTENSION: &str = "schematic";

/// The folder the schematics live in, under the world's own folder.
pub fn schematic_dir(world: &std::path::Path) -> std::path::PathBuf {
    world.join(DATA_FOLDER).join(SCHEMATICS_FOLDER)
}

/// The file of a schematic, with its extension.
pub fn schematic_file(name: &str) -> String {
    format!("{name}.{SCHEMATIC_EXTENSION}")
}

/// Where a schematic's box goes: the blocks in front of the player, the box
/// in the middle of the way they look, its floor on the ground.
pub fn in_front(feet: Pos, size: [i32; 3], dir: Dir) -> Pos {
    let v = dir.vector();
    let mut at = feet;
    at[0] = if v[0] > 0 {
        feet[0] + 1
    } else if v[0] < 0 {
        feet[0] - size[0]
    } else {
        feet[0] - (size[0] - 1) / 2
    };
    at[2] = if v[2] > 0 {
        feet[2] + 1
    } else if v[2] < 0 {
        feet[2] - size[2]
    } else {
        feet[2] - (size[2] - 1) / 2
    };
    at[1] = feet[1];
    at
}

/// The size of the box and the number of blocks of a schematic file, from
/// its header alone.
pub fn schematic_info(data: &[u8]) -> Option<([i32; 3], usize)> {
    if data.len() < SCHEMATIC_HEADER || &data[0..4] != SCHEMATIC_MAGIC {
        return None;
    }
    let mut size = [0i32; 3];
    for (i, s) in size.iter_mut().enumerate() {
        let at = 8 + i * 4;
        *s = i32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
    }
    let count = u32::from_le_bytes([data[20], data[21], data[22], data[23]]) as usize;
    Some((size, count))
}

/// The channel the client and the server talk about the schematics on.
pub const SCHEMATIC_CHANNEL: &str = "worldedit_horizon:schematic";
/// The most bytes of a schematic file in one message: the game's own limit
/// is 30 000.
pub const PART_SIZE: usize = 24_000;
/// The most messages of one batch: the game sends at most 32 a tick.
pub const BATCH: usize = 30;

/// A message of the channel: a header line and the bytes after it.
pub fn message(header: &str, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(header.len() + 1 + payload.len());
    out.extend_from_slice(header.as_bytes());
    out.push(b'\n');
    out.extend_from_slice(payload);
    out
}

/// The header of a message and its bytes.
pub fn split_message(data: &[u8]) -> Option<(&str, &[u8])> {
    let at = data.iter().position(|b| *b == b'\n')?;
    let header = std::str::from_utf8(&data[..at]).ok()?;
    Some((header, &data[at + 1..]))
}

/// A file in parts, each of them small enough for one message.
pub fn parts(data: &[u8]) -> Vec<Vec<u8>> {
    if data.is_empty() {
        return vec![Vec::new()];
    }
    data.chunks(PART_SIZE).map(|part| part.to_vec()).collect()
}

/// The name of a schematic: only the characters that cannot leave the
/// schematics folder.
pub fn schematic_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("The schematic needs a name.".to_owned());
    }
    if name.len() > 48 {
        return Err("The name is too long: 48 characters at most.".to_owned());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err("The name may hold letters, digits, `_` and `-` only.".to_owned());
    }
    Ok(name.to_owned())
}

/// A saved build: its blocks, relative to the lowest corner of the region it
/// was taken from.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Schematic {
    /// The blocks of the build, relative to the region's lowest corner.
    pub blocks: Vec<(Pos, u32)>,
    /// The size of the region the build was taken from.
    pub size: [i32; 3],
}

impl Schematic {
    /// The build of `region`, as it is read from `world`.
    pub fn of<W: World>(world: &W, region: &Region) -> Schematic {
        let min = region.min;
        Schematic {
            blocks: region
                .positions()
                .into_iter()
                .map(|p| (sub(p, min), world.get(p)))
                .collect(),
            size: region.size(),
        }
    }
    /// The clipboard that pastes this build: its lowest corner goes where
    /// the player stands.
    pub fn to_clipboard(&self) -> Clipboard {
        Clipboard {
            blocks: self.blocks.clone(),
            size: Some(self.size),
        }
    }
    /// The bytes of the file: the header, then every block.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(SCHEMATIC_HEADER + self.blocks.len() * 16);
        out.extend_from_slice(SCHEMATIC_MAGIC);
        out.extend_from_slice(&SCHEMATIC_FORMAT.to_le_bytes());
        for s in self.size {
            out.extend_from_slice(&s.to_le_bytes());
        }
        out.extend_from_slice(&(self.blocks.len() as u32).to_le_bytes());
        for (p, block) in &self.blocks {
            for v in p {
                out.extend_from_slice(&v.to_le_bytes());
            }
            out.extend_from_slice(&block.to_le_bytes());
        }
        out
    }
    /// The build of a file, as it was written.
    pub fn decode(data: &[u8]) -> Result<Schematic, String> {
        if data.len() < SCHEMATIC_HEADER {
            return Err("The file is too short to be a schematic.".to_owned());
        }
        if &data[0..4] != SCHEMATIC_MAGIC {
            return Err("This is not a WorldEdit schematic.".to_owned());
        }
        let format = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        if format != SCHEMATIC_FORMAT {
            return Err(format!("The schematic's format {format} is not known."));
        }
        let mut size = [0i32; 3];
        for (i, s) in size.iter_mut().enumerate() {
            let at = 8 + i * 4;
            *s = i32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
        }
        if size.iter().any(|s| *s <= 0) {
            return Err("The schematic's size is not right.".to_owned());
        }
        let count = u32::from_le_bytes([data[20], data[21], data[22], data[23]]) as usize;
        if count.checked_mul(16) != Some(data.len() - SCHEMATIC_HEADER) {
            return Err("The schematic is cut short.".to_owned());
        }
        if count as u64 > max_blocks() {
            return Err("The schematic holds more blocks than the limit allows.".to_owned());
        }
        let mut blocks = Vec::with_capacity(count);
        for i in 0..count {
            let at = SCHEMATIC_HEADER + i * 16;
            let p = [
                i32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]),
                i32::from_le_bytes([data[at + 4], data[at + 5], data[at + 6], data[at + 7]]),
                i32::from_le_bytes([data[at + 8], data[at + 9], data[at + 10], data[at + 11]]),
            ];
            let block = u32::from_le_bytes([data[at + 12], data[at + 13], data[at + 14], data[at + 15]]);
            if (0..3).any(|k| p[k] < 0 || p[k] >= size[k]) {
                return Err("The schematic holds a block outside its box.".to_owned());
            }
            blocks.push((p, block));
        }
        Ok(Schematic { blocks, size })
    }
}

/// The writes of `stack`: the region repeated `times` along `dir`.
pub fn stack_writes<W: World>(
    world: &W,
    region: &Region,
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
    region: &Region,
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
    /// The brush the player armed with `/we brush`, if any.
    pub brush: Option<Brush>,
    /// The blocks the brush covered last: the brush goes through them as if
    /// they were air, so that it does not shoot at its own work.
    pub last_stroke: HashSet<Pos>,
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
        let c = Region::cube([0, 0, 0], [2, 2, 2]);
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
        let c = Region::cube([0, 0, 0], [2, 5, 2]);
        let writes = smooth_writes(&w, &c, 0, 1);
        apply(&mut w, writes);
        assert_eq!(w.get([1, 3, 1]), 0, "the spike came down");
        assert_ne!(w.get([0, 1, 0]), 0, "its sides came up");
        let flat = smooth_writes(&w, &Region::cube([5, 0, 5], [6, 3, 6]), 0, 3);
        assert!(flat.is_empty());
    }

    #[test]
    fn copy_rotate_flip_and_paste() {
        let mut w = Map::default();
        w.0.insert([1, 0, 0], 7);
        let c = Region::cube([0, 0, 0], [1, 0, 0]);
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
    fn schematics_are_saved_and_read_back() {
        let mut w = Map::default();
        w.0.insert([1, 0, 0], 7);
        w.0.insert([1, 1, 0], 9);
        let c = Region::cube([1, 0, 0], [2, 1, 1]);
        let s = Schematic::of(&w, &c);
        assert_eq!(s.size, [2, 2, 2]);
        assert!(s.blocks.contains(&([0, 0, 0], 7)));
        assert!(s.blocks.contains(&([0, 1, 0], 9)));
        let data = s.encode();
        let back = Schematic::decode(&data).unwrap();
        assert_eq!(back, s);
        // The clipboard pastes the build where the player stands.
        let clip = back.to_clipboard();
        assert!(clip.writes([10, 0, 10], None).contains(&([10, 0, 10], 7)));
    }

    #[test]
    fn broken_schematics_are_refused() {
        let mut w = Map::default();
        w.0.insert([0, 0, 0], 1);
        let s = Schematic::of(&w, &Region::cube([0, 0, 0], [0, 0, 0]));
        let data = s.encode();
        assert!(Schematic::decode(&[]).is_err());
        assert!(Schematic::decode(b"not a schematic at all, but long").is_err());
        assert!(Schematic::decode(&data[..data.len() - 1]).is_err());
        // A block outside the box is refused.
        let mut bad = data.clone();
        bad[24..28].copy_from_slice(&99i32.to_le_bytes());
        assert!(Schematic::decode(&bad).is_err());
    }

    #[test]
    fn schematic_names_stay_in_their_folder() {
        assert_eq!(schematic_name("house").unwrap(), "house");
        assert_eq!(schematic_name(" my_house-2 ").unwrap(), "my_house-2");
        assert!(schematic_name("").is_err());
        assert!(schematic_name("../secret").is_err());
        assert!(schematic_name("a/b").is_err());
        assert!(schematic_name("a\\b").is_err());
        assert!(schematic_name(&"x".repeat(49)).is_err());
    }

    #[test]
    fn a_schematic_goes_in_front_of_the_player() {
        // A 3×3×2 build: its near face a block ahead of the player, the box
        // in the middle of the way they look, its floor on the ground.
        assert_eq!(in_front([10, 64, 10], [3, 2, 3], Dir::North), [9, 64, 7]);
        assert_eq!(in_front([10, 64, 10], [3, 2, 3], Dir::South), [9, 64, 11]);
        assert_eq!(in_front([10, 64, 10], [3, 2, 3], Dir::East), [11, 64, 9]);
        assert_eq!(in_front([10, 64, 10], [3, 2, 3], Dir::West), [7, 64, 9]);
    }

    #[test]
    fn the_schematics_channel_splits_its_messages() {
        let data = message("write house 0 2", &[1, 2, 3]);
        let (header, payload) = split_message(&data).unwrap();
        assert_eq!(header, "write house 0 2");
        assert_eq!(payload, &[1, 2, 3]);
        assert!(split_message(b"no line").is_none());
        let parts = parts(&[0u8; 50_000]);
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].len(), PART_SIZE);
        assert_eq!(parts[2].len(), 2_000);
    }

    #[test]
    fn a_schematic_file_tells_its_size() {
        let mut w = Map::default();
        w.0.insert([0, 0, 0], 1);
        let s = Schematic::of(&w, &Region::cube([0, 0, 0], [1, 1, 1]));
        let data = s.encode();
        let (size, count) = schematic_info(&data).unwrap();
        assert_eq!(size, [2, 2, 2]);
        assert_eq!(count, 8);
        assert!(schematic_info(b"not a schematic").is_none());
    }

    #[test]
    fn a_loaded_schematic_knows_its_box() {
        let mut w = Map::default();
        w.0.insert([1, 0, 0], 7);
        let c = Region::cube([1, 0, 0], [2, 1, 1]);
        let clip = Schematic::of(&w, &c).to_clipboard();
        assert_eq!(clip.size, Some([2, 2, 2]));
        // A copy keeps the place it was copied from.
        assert_eq!(Clipboard::copy(&w, &c, [0, 0, 0]).size, None);
    }

    #[test]
    fn stack_and_move() {
        let mut w = Map::default();
        w.0.insert([0, 0, 0], 5);
        let c = Region::cube([0, 0, 0], [0, 1, 0]);
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
        let c = Region::cube([0, 0, 0], [999, 999, 999]);
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
        let c = Region::cube([0, 0, 0], [6, 6, 6]);
        let t = trim_to_existing(&w, &c, 0).unwrap();
        assert_eq!((t.min, t.max), ([1, 2, 3], [4, 5, 6]));
        let empty = Region::cube([20, 20, 20], [21, 21, 21]);
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

    #[test]
    fn a_cube_around_the_player_is_a_box() {
        let r = Region::around(Shape::Cube, [0, 0, 0], &[3]).unwrap();
        assert_eq!(r.shape, Shape::Cube);
        assert_eq!(r.size(), [3, 3, 3]);
        assert_eq!(r.volume(), 27);
        assert!(r.contains([-1, -1, -1]) && r.contains([1, 1, 1]));
        assert!(!r.contains([2, 0, 0]));
    }

    #[test]
    fn a_sphere_is_the_ball_around_the_player() {
        let r = Region::around(Shape::Sphere, [0, 0, 0], &[2]).unwrap();
        assert_eq!(r.shape, Shape::Sphere);
        assert_eq!(r.sizes, [2, 0, 0]);
        // The lattice points in a ball of radius 2.
        assert_eq!(r.volume(), 33);
        assert!(r.contains([2, 0, 0]) && r.contains([1, 1, 1]));
        assert!(!r.contains([3, 0, 0]) && !r.contains([2, 2, 0]));
        assert_eq!(r.positions().len(), 33);
    }

    #[test]
    fn a_cylinder_is_a_tube_up_from_the_player() {
        let r = Region::around(Shape::Cylinder, [0, 0, 0], &[2, 3]).unwrap();
        assert_eq!(r.shape, Shape::Cylinder);
        assert_eq!(r.sizes, [2, 3, 0]);
        // 13 columns of three blocks.
        assert_eq!(r.volume(), 39);
        assert!(r.contains([0, 2, 0]) && r.contains([2, 0, 0]));
        assert!(!r.contains([0, 3, 0]) && !r.contains([3, 0, 0]));
    }

    #[test]
    fn an_ellipsoid_stretches_the_ball() {
        let r = Region::around(Shape::Ellipsoid, [0, 0, 0], &[2, 1, 2]).unwrap();
        assert_eq!(r.shape, Shape::Ellipsoid);
        assert_eq!(r.sizes, [2, 1, 2]);
        // The same 13 columns, but only the middle one is three blocks high.
        assert_eq!(r.volume(), 15);
        assert!(r.contains([0, 1, 0]) && r.contains([2, 0, 0]));
        assert!(!r.contains([0, 2, 0]));
    }

    #[test]
    fn the_selection_keeps_its_shape() {
        let mut s = Selection::default();
        s.set_pos(1, "minecraft:overworld", [0, 0, 0]);
        s.set_pos(2, "minecraft:overworld", [3, 0, 0]);
        s.shape = Shape::Sphere;
        let r = s.region().unwrap();
        assert_eq!(r.shape, Shape::Sphere);
        assert_eq!(r.sizes, [3, 0, 0]);
        assert_eq!(r.volume(), 123);
        // The corners the shape needs are the ones that make it again.
        let mut s2 = Selection::default();
        s2.set_region(&r);
        assert_eq!(s2.region().unwrap(), r);
    }

    #[test]
    fn a_round_selection_grows_by_its_radius() {
        let r = Region::around(Shape::Sphere, [0, 0, 0], &[2]).unwrap();
        let bigger = r.grown(1).unwrap();
        assert_eq!(bigger.sizes, [3, 0, 0]);
        let smaller = r.grown(-1).unwrap();
        assert_eq!(smaller.sizes, [1, 0, 0]);
        assert!(r.grown(-2).is_none());
        let up = r.expand(Dir::Up, 2).unwrap();
        assert_eq!(up.sizes, [4, 0, 0]);
    }

    #[test]
    fn chunks_are_counted_from_the_blocks() {
        assert_eq!(chunk_of([0, 0, 0]), (0, 0));
        assert_eq!(chunk_of([15, 0, 15]), (0, 0));
        assert_eq!(chunk_of([-1, 0, 0]), (-1, 0));
        assert_eq!(chunk_of([16, 0, -16]), (1, -1));
        let r = Region::cube([0, 0, 0], [31, 0, 15]);
        assert_eq!(chunks_of(&r), vec![(0, 0), (1, 0)]);
        let mut w = Map::default();
        w.0.insert([0, 0, 0], 1);
        w.0.insert([15, 319, 15], 1);
        assert_eq!(chunk_filled(&w, (0, 0), -64, 319, 0), 2);
    }

    #[test]
    fn every_shape_survives_the_corners() {
        for (shape, sizes) in [
            (Shape::Cube, vec![3]),
            (Shape::Sphere, vec![2]),
            (Shape::Cylinder, vec![2, 3]),
            (Shape::Ellipsoid, vec![2, 1, 2]),
        ] {
            let r = Region::around(shape, [0, 0, 0], &sizes).unwrap();
            let mut s = Selection::default();
            s.set_region(&r);
            assert_eq!(s.region().unwrap(), r, "{shape:?} does not survive");
        }
    }

    #[test]
    fn a_brush_is_a_shape_its_sizes_and_a_pattern() {
        let b = Brush::parse("sphere 5 stone", &names).unwrap();
        assert_eq!(b.shape, Shape::Sphere);
        assert_eq!(b.sizes, [5, 0, 0]);
        assert_eq!(b.text, "stone");
        assert_eq!(b.describe(), "sphere r=5, stone");
        let c = Brush::parse("cylinder 3 4 stone", &names).unwrap();
        assert_eq!(c.sizes, [3, 4, 0]);
        let e = Brush::parse("ellipsoid 4 2 4 dirt", &names).unwrap();
        assert_eq!(e.sizes, [4, 2, 4]);
        // A brush needs a shape, its sizes and a pattern.
        assert!(Brush::parse("sphere stone", &names).is_err());
        assert!(Brush::parse("cube 2 3 stone", &names).is_err());
        assert!(Brush::parse("pyramid 2 stone", &names).is_err());
        assert!(Brush::parse("sphere 0 stone", &names).is_err());
        assert!(Brush::parse("sphere 5 nosuchblock", &names).is_err());
    }

    #[test]
    fn a_brush_paints_around_the_block_it_was_used_on() {
        let b = Brush::parse("sphere 1 stone", &names).unwrap();
        let p = b.positions([0, 0, 0]).unwrap();
        // The block itself and the eighteen around it: the corners of the
        // three by three by three box are outside the ball.
        assert_eq!(p.len(), 19);
        assert!(p.contains(&[0, 0, 0]) && p.contains(&[1, 0, 0]) && p.contains(&[-1, 0, 0]));
        assert!(!p.contains(&[1, 1, 1]));
        let c = Brush::parse("cube 3 stone", &names).unwrap();
        let p = c.positions([10, 20, 30]).unwrap();
        assert_eq!(p.len(), 27);
        assert!(p.contains(&[10, 20, 30]) && p.contains(&[11, 21, 31]) && p.contains(&[9, 19, 29]));
        // A cylinder stands in the middle of the block it was used on.
        let cyl = Brush::parse("cylinder 1 3 stone", &names).unwrap();
        let p = cyl.positions([0, 0, 0]).unwrap();
        // The disc of radius 1 has nine columns, three blocks high.
        assert_eq!(p.len(), 27);
        assert!(p.contains(&[0, 0, 0]) && p.contains(&[0, 1, 0]) && p.contains(&[0, -1, 0]));
        assert!(!p.contains(&[0, 2, 0]));
        // An ellipsoid is a ball stretched.
        let e = Brush::parse("ellipsoid 2 1 2 stone", &names).unwrap();
        let p = e.positions([0, 0, 0]).unwrap();
        assert!(p.contains(&[0, 0, 0]) && p.contains(&[0, 1, 0]) && p.contains(&[2, 0, 0]));
        assert!(!p.contains(&[0, 2, 0]) && !p.contains(&[3, 0, 0]));
    }

    #[test]
    fn a_brush_that_is_too_big_is_refused() {
        // The limit is checked before the blocks are built.
        let b = Brush::parse("sphere 128 stone", &names).unwrap();
        assert!(b.positions([0, 0, 0]).is_err());
        let c = Brush::parse("cube 100 stone", &names).unwrap();
        assert!(c.positions([0, 0, 0]).is_err());
        let d = Brush::parse("cylinder 100 100 stone", &names).unwrap();
        assert!(d.positions([0, 0, 0]).is_err());
    }

    #[test]
    fn a_brush_stroke_can_be_undone() {
        let mut w = Map::default();
        let mut s = Session::default();
        let b = Brush::parse("cube 3 stone", &names).unwrap();
        let edit = fill(&mut w, &b.positions([0, 0, 0]).unwrap(), &b.pattern);
        assert_eq!(edit.len(), 27);
        s.record(edit);
        assert_eq!(w.get([0, 0, 0]), 1);
        let (edits, blocks) = s.undo(&mut w, 1);
        assert_eq!((edits, blocks), (1, 27));
        assert_eq!(w.get([0, 0, 0]), 0);
    }

    #[test]
    fn a_brush_grows_and_shrinks_with_its_radius() {
        let b = Brush::parse("sphere 5 stone", &names).unwrap();
        assert_eq!(b.radius(), 5);
        let bigger = b.with_radius(7).unwrap();
        assert_eq!(bigger.sizes, [7, 0, 0]);
        assert_eq!(bigger.describe(), "sphere r=7, stone");
        // A cube's radius is its half side.
        let c = Brush::parse("cube 5 stone", &names).unwrap();
        assert_eq!(c.radius(), 2);
        assert_eq!(c.with_radius(3).unwrap().sizes, [7, 0, 0]);
        // An ellipsoid keeps its proportions.
        let e = Brush::parse("ellipsoid 4 2 4 stone", &names).unwrap();
        assert_eq!(e.radius(), 4);
        assert_eq!(e.with_radius(6).unwrap().sizes, [6, 3, 6]);
        // A radius of zero is the block itself.
        let one = b.with_radius(0).unwrap();
        assert_eq!(one.positions([1, 2, 3]).unwrap(), vec![[1, 2, 3]]);
        // A radius outside the limit is refused.
        assert!(b.with_radius(1000).is_err());
    }

    #[test]
    fn a_brush_looks_where_the_player_looks() {
        // The yaw 0 is south (+z) and the pitch is positive downwards.
        let south = look(0.0, 0.0);
        assert!((south[2] - 1.0).abs() < 1e-9);
        assert!(south[0].abs() < 1e-9 && south[1].abs() < 1e-9);
        let west = look(90.0, 0.0);
        assert!((west[0] + 1.0).abs() < 1e-9);
        let down = look(0.0, 90.0);
        assert!((down[1] + 1.0).abs() < 1e-9);
        // The block ahead of the eye.
        let eye = [0.0, 64.0, 0.0];
        assert_eq!(ahead(eye, south, 10.0), [0, 64, 10]);
        assert_eq!(ahead(eye, down, 5.0), [0, 59, 0]);
    }

    #[test]
    fn a_brush_can_work_far_ahead_of_the_player() {
        let b = Brush::parse("sphere 2 stone", &names).unwrap();
        assert_eq!(b.distance, None);
        let far = b.with_distance(Some(20)).unwrap();
        assert_eq!(far.distance, Some(20));
        assert!(far.describe().contains("20 blocks ahead"));
        // The distance stays when the size changes.
        assert_eq!(far.with_radius(3).unwrap().distance, Some(20));
        // And it can be taken away again.
        assert_eq!(far.with_distance(None).unwrap().distance, None);
        // A distance out of reach is refused.
        assert!(b.with_distance(Some(300)).is_err());
    }

    #[test]
    fn the_distance_has_a_limit_of_its_own() {
        let b = Brush::parse("sphere 2 stone", &names).unwrap();
        // The limit goes up to 256 blocks.
        assert!(set_limit("distance", Some(256)).is_ok());
        assert!(b.with_distance(Some(200)).is_ok());
        // And no further.
        assert!(set_limit("distance", Some(300)).is_err());
        assert!(set_limit("distance", Some(0)).is_err());
        // The old limit comes back.
        set_limit("reset", None).unwrap();
        assert_eq!(max_distance(), MAX_DISTANCE);
    }

    #[test]
    fn a_new_brush_keeps_the_distance_of_the_old_one() {
        let old = Brush::parse("sphere 2 stone", &names)
            .unwrap()
            .with_distance(Some(20))
            .unwrap();
        let new = Brush::parse("sphere 4 dirt", &names).unwrap();
        let new = new.with_distance_of(&old).unwrap();
        // The distance stays.
        assert_eq!(new.distance, Some(20));
        // The shape and the pattern are the new ones.
        assert_eq!(new.sizes, [4, 0, 0]);
        assert_eq!(new.text, "dirt");
        // A brush without a distance gives none.
        let plain = Brush::parse("sphere 4 dirt", &names).unwrap();
        assert_eq!(plain.with_distance_of(&plain).unwrap().distance, None);
    }

    #[test]
    fn a_ray_stops_at_the_first_block_it_meets() {
        let mut w = Map::default();
        w.set([0, 64, 5], 1);
        // The eye is in the middle of the block [0, 64, 0].
        let ray = ray_blocks(&w, [0.5, 64.5, 0.5], [0.0, 0.0, 1.0], 20.0, 0, &HashSet::new());
        assert!(ray.solid);
        // The block the eye is in is not in the way.
        assert!(!ray.blocks.contains(&[0, 64, 0]));
        // The ray ends on the block it meets.
        assert_eq!(ray.blocks.last(), Some(&[0, 64, 5]));
        // The blocks between the eye and the stone.
        assert_eq!(ray.blocks.len(), 5);
    }

    #[test]
    fn a_ray_through_the_sky_goes_on_to_the_distance() {
        let w = Map::default();
        let ray = ray_blocks(&w, [0.5, 64.5, 0.5], [0.0, 0.0, 1.0], 3.0, 0, &HashSet::new());
        // It meets nothing.
        assert!(!ray.solid);
        // And it stops at the distance.
        assert_eq!(ray.blocks.last(), Some(&[0, 64, 3]));
    }

    #[test]
    fn a_ray_goes_through_the_blocks_the_brush_covered_last() {
        let mut w = Map::default();
        // The ball the brush put up in the air.
        for x in 0..=4 {
            for y in 64..=68 {
                for z in 13..=17 {
                    w.set([x, y, z], 1);
                }
            }
        }
        let own: HashSet<Pos> = (0..=4)
            .flat_map(|x| (64..=68).map(move |y| (x, y)))
            .flat_map(|(x, y)| (13..=17).map(move |z| [x, y, z]))
            .collect();
        let ray = ray_blocks(&w, [0.5, 64.5, 0.5], [0.0, 0.0, 1.0], 20.0, 0, &own);
        // The brush does not shoot at its own work: it meets nothing.
        assert!(!ray.solid);
        // Without that, the ball would end the ray.
        let plain = ray_blocks(&w, [0.5, 64.5, 0.5], [0.0, 0.0, 1.0], 20.0, 0, &HashSet::new());
        assert!(plain.solid);
        assert_eq!(plain.blocks.last(), Some(&[0, 64, 13]));
    }

    #[test]
    fn a_brush_click_says_which_button_it_was() {
        assert_eq!(parse_brush(&brush_payload(false)), Some(false));
        assert_eq!(parse_brush(&brush_payload(true)), Some(true));
        assert_eq!(parse_brush(&[]), None);
        assert_eq!(parse_brush(&[7]), None);
    }
}
