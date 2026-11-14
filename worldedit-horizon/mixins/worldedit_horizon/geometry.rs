// The selection as the client draws it: the frame around a box and the grid
// inside it, as WorldEdit CUI draws a selection, and a round shape as the
// blocks it covers, one square at a time.
//
// The pure part of the client's half: no game types, so it can be tested on
// its own. The server sends the selection in the format `parse` reads; the
// server's side of it is in the game's `we_core.rs` (`selection_payload`).

use std::collections::{BTreeMap, HashSet};

/// What the server sent: the shape, the dimension, the shape's middle and
/// sizes and the box the blocks fill.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    /// The shape's number, as the server counts them.
    pub shape: u8,
    /// The level the selection was made in.
    pub dimension: String,
    /// The corner with the smallest coordinates.
    pub min: [i32; 3],
    /// The corner with the largest coordinates.
    pub max: [i32; 3],
    /// The shape's middle.
    pub center: [i32; 3],
    /// The shape's own sizes: a cube's three sides, a sphere's radius, a
    /// cylinder's radius and height, an ellipsoid's three radii.
    pub sizes: [i32; 3],
    /// Whether the grid inside the selection is drawn.
    pub grid: bool,
}

/// A line of the drawing: its ends in the world, its colour and its width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    /// Where the line starts.
    pub start: [f64; 3],
    /// Where the line ends.
    pub end: [f64; 3],
    /// The colour, ARGB as Horizon's lines carry it.
    pub color: u32,
    /// The width in pixels.
    pub width: f32,
}

/// The shapes, as the server counts them.
#[cfg(test)]
pub const CUBE: u8 = 0;
pub const SPHERE: u8 = 1;
pub const CYLINDER: u8 = 2;
pub const ELLIPSOID: u8 = 3;

/// The colour of the lines: red, as the player asked (#CCFF0000).
pub const FRAME: u32 = 0xCCFF_0000;
/// The grid inside a box: a darker red (#CCAA0000).
pub const GRID: u32 = 0xCCAA_0000;
/// The frame's width in pixels, as CUI draws its edges.
pub const FRAME_WIDTH: f32 = 3.0;
/// The grid's width in pixels: thinner than the frame, as there are many
/// more of them.
pub const GRID_WIDTH: f32 = 1.5;
/// The outline of a round shape's blocks: between the frame and the grid.
pub const SHAPE_WIDTH: f32 = 2.0;
/// How many planes of the grid there may be on all three axes together: a
/// very big selection gets wider cells, so the lines stay few.
const MAX_GRID_PLANES: i32 = 4096;
/// The most free sides of blocks the outline of a round shape may show: past
/// that only its shadow is drawn, so the lines stay few.
const MAX_FACES: usize = 20_000;
/// The most columns of blocks a round shape may have before its blocks are
/// not drawn at all: past that only the frame of its box is drawn, so the
/// work of a shape of any size is the same little work.
const MAX_COLUMNS: usize = 65_536;
/// The most a radius of a round shape may be: the same limit the server
/// puts on its own radii. A shape bigger than that is drawn as the limit,
/// so the whole numbers of the tests stay small enough to be counted
/// exactly.
const MAX_RADIUS: i32 = 1024;

/// Reads what the server sent. None when there is no selection or when the
/// message is broken.
pub fn parse(data: &[u8]) -> Option<Selection> {
    let kind = *data.first()?;
    if kind == 0 {
        // No selection.
        return None;
    }
    if kind != 1 || data.len() < 4 {
        return None;
    }
    // The flags: the first bit says that the grid inside the selection is
    // drawn.
    let grid = data[1] & 1 != 0;
    let shape = data[2];
    let length = data[3] as usize;
    // The dimension, then the shape's middle, sizes and the box: twelve
    // numbers.
    if data.len() < 4 + length + 48 {
        return None;
    }
    let dimension = std::str::from_utf8(data.get(4..4 + length)?)
        .ok()?
        .to_owned();
    let mut numbers = [0i32; 12];
    for (i, v) in numbers.iter_mut().enumerate() {
        let at = 4 + length + i * 4;
        *v = i32::from_le_bytes(data.get(at..at + 4)?.try_into().ok()?);
    }
    Some(Selection {
        shape,
        dimension,
        min: [numbers[6], numbers[7], numbers[8]],
        max: [numbers[9], numbers[10], numbers[11]],
        center: [numbers[0], numbers[1], numbers[2]],
        sizes: [numbers[3], numbers[4], numbers[5]],
        grid,
    })
}

/// The lines of the selection: the frame and the grid of a box, the blocks of
/// a round shape, one square at a time.
pub fn segments(selection: &Selection) -> Vec<Segment> {
    let mut out = Vec::new();
    match selection.shape {
        SPHERE | CYLINDER | ELLIPSOID => {
            out.extend(outline(selection));
        }
        _ => {
            let (lo, hi) = box_bounds(selection);
            // The grid first, the frame over it: the edges of the frame stay
            // bright where the grid crosses them.
            if selection.grid {
                grid(&mut out, lo, hi);
            }
            frame(&mut out, lo, hi);
        }
    }
    out
}

/// The width of a cell of the grid on each axis: one block, as the player
/// asked. Only a very big selection gets wider cells, so that the lines stay
/// few enough to be drawn.
fn steps(sizes: [i32; 3]) -> [i32; 3] {
    // Every axis has one plane more than it has blocks.
    let planes: i64 = sizes.iter().map(|s| *s as i64 + 1).sum();
    if planes <= MAX_GRID_PLANES as i64 {
        return [1, 1, 1];
    }
    // Too many planes: every axis gets its share of them, the cell grows.
    let allowed = (MAX_GRID_PLANES / 3).max(1);
    sizes.map(|size| {
        let size = size.max(1);
        ((size + allowed - 1) / allowed).max(1)
    })
}
fn line(out: &mut Vec<Segment>, a: [f64; 3], b: [f64; 3], color: u32, width: f32) {
    out.push(Segment {
        start: a,
        end: b,
        color,
        width,
    });
}

fn rect(
    out: &mut Vec<Segment>,
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
    d: [f64; 3],
    color: u32,
    width: f32,
) {
    line(out, a, b, color, width);
    line(out, b, c, color, width);
    line(out, c, d, color, width);
    line(out, d, a, color, width);
}

/// The twelve edges of the box.
fn frame(out: &mut Vec<Segment>, lo: [i32; 3], hi: [i32; 3]) {
    let corner = |x: usize, y: usize, z: usize| {
        [
            if x == 0 { lo[0] } else { hi[0] },
            if y == 0 { lo[1] } else { hi[1] },
            if z == 0 { lo[2] } else { hi[2] },
        ]
    };
    let at = |c: [i32; 3]| [c[0] as f64, c[1] as f64, c[2] as f64];
    for y in 0..2 {
        for z in 0..2 {
            line(
                out,
                at(corner(0, y, z)),
                at(corner(1, y, z)),
                FRAME,
                FRAME_WIDTH,
            );
        }
    }
    for x in 0..2 {
        for z in 0..2 {
            line(
                out,
                at(corner(x, 0, z)),
                at(corner(x, 1, z)),
                FRAME,
                FRAME_WIDTH,
            );
        }
    }
    for x in 0..2 {
        for y in 0..2 {
            line(
                out,
                at(corner(x, y, 0)),
                at(corner(x, y, 1)),
                FRAME,
                FRAME_WIDTH,
            );
        }
    }
}

/// The corners of the box the selection fills: the blocks it holds and
/// the frame that stands around them.
fn box_bounds(selection: &Selection) -> ([i32; 3], [i32; 3]) {
    let lo = selection.min;
    // The frame stands around the blocks, so it ends one block further.
    let hi = [
        selection.max[0].saturating_add(1),
        selection.max[1].saturating_add(1),
        selection.max[2].saturating_add(1),
    ];
    (lo, hi)
}

/// The frame of the box the selection fills: the twelve edges around its
/// blocks. It stands for a shape too big to draw block by block.
fn box_frame(selection: &Selection) -> Vec<Segment> {
    let (lo, hi) = box_bounds(selection);
    let mut out = Vec::with_capacity(12);
    frame(&mut out, lo, hi);
    out
}

/// The grid: the lines between the blocks, as Minecraft draws them. Every
/// cell of it is one block wide.
fn grid(out: &mut Vec<Segment>, lo: [i32; 3], hi: [i32; 3]) {
    let at = |c: [i32; 3]| [c[0] as f64, c[1] as f64, c[2] as f64];
    let sizes = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
    let steps = steps(sizes);
    for y in levels(lo[1], hi[1], steps[1]) {
        rect(
            out,
            at([lo[0], y, lo[2]]),
            at([hi[0], y, lo[2]]),
            at([hi[0], y, hi[2]]),
            at([lo[0], y, hi[2]]),
            GRID,
            GRID_WIDTH,
        );
    }
    for x in levels(lo[0], hi[0], steps[0]) {
        rect(
            out,
            at([x, lo[1], lo[2]]),
            at([x, lo[1], hi[2]]),
            at([x, hi[1], hi[2]]),
            at([x, hi[1], lo[2]]),
            GRID,
            GRID_WIDTH,
        );
    }
    for z in levels(lo[2], hi[2], steps[2]) {
        rect(
            out,
            at([lo[0], lo[1], z]),
            at([lo[0], hi[1], z]),
            at([hi[0], hi[1], z]),
            at([hi[0], lo[1], z]),
            GRID,
            GRID_WIDTH,
        );
    }
}

/// The places of the grid's lines on an axis.
fn levels(lo: i32, hi: i32, step: i32) -> Vec<i32> {
    let mut out = Vec::new();
    let mut at = lo;
    while at < hi {
        out.push(at);
        at += step;
    }
    out.push(hi);
    out
}

// The round shapes: the blocks of the shape, one square at a time.

/// A column of blocks: the lowest and the highest block of it.
#[derive(Clone, Copy, Debug)]
struct Column {
    low: i32,
    high: i32,
}

/// A free side of a block: a rectangle on one of the three planes.
#[derive(Clone, Copy, Debug)]
struct Face {
    /// The axis the side stands on: 0 = x, 1 = y, 2 = z.
    axis: usize,
    /// Where on that axis the side is.
    at: i32,
    /// The first axis of the plane and its ends.
    u: (i32, i32),
    /// The second axis of the plane and its ends.
    v: (i32, i32),
}

impl Face {
    /// The four corners of the side, around it.
    fn corners(&self) -> [[i32; 3]; 4] {
        let (u0, u1) = self.u;
        let (v0, v1) = self.v;
        let at = self.at;
        match self.axis {
            0 => [
                [at, u0, v0],
                [at, u1, v0],
                [at, u1, v1],
                [at, u0, v1],
            ],
            1 => [
                [u0, at, v0],
                [u1, at, v0],
                [u1, at, v1],
                [u0, at, v1],
            ],
            _ => [
                [u0, v0, at],
                [u1, v0, at],
                [u1, v1, at],
                [u0, v1, at],
            ],
        }
    }
}

/// A point on a plane: the plane's axis with its place, the first and the
/// second axis of the plane.
fn point(axis: usize, at: i32, u: i32, v: i32) -> [i32; 3] {
    match axis {
        0 => [at, u, v],
        1 => [u, at, v],
        _ => [u, v, at],
    }
}

/// The outline of a round shape: the free sides of its blocks, as Minecraft
/// draws the blocks of a selection. Every line stands on the border of a
/// block, so the outline is the blocks themselves.
fn outline(selection: &Selection) -> Vec<Segment> {
    if column_count(selection) > MAX_COLUMNS as i128 {
        // Too many blocks to count: the frame of the box around the shape
        // stands for it, so the work stays the same for any size.
        return box_frame(selection);
    }
    let columns = columns(selection);
    let faces = faces(&columns);
    if faces.len() > MAX_FACES {
        // Too many blocks to show one by one: the shadow of the shape
        // instead, which stands for it in blocks as well.
        return shadow(selection, &columns);
    }
    let mut out = Vec::with_capacity(faces.len() * 4);
    // A side of a block and the side of its neighbour share an edge: it is
    // drawn once.
    let mut seen: HashSet<([i32; 3], [i32; 3])> = HashSet::with_capacity(faces.len() * 4);
    for face in faces {
        let corners = face.corners();
        for i in 0..4 {
            let a = corners[i];
            let b = corners[(i + 1) % 4];
            let key = if a < b { (a, b) } else { (b, a) };
            if seen.insert(key) {
                line(
                    &mut out,
                    [a[0] as f64, a[1] as f64, a[2] as f64],
                    [b[0] as f64, b[1] as f64, b[2] as f64],
                    FRAME,
                    SHAPE_WIDTH,
                );
            }
        }
    }
    out
}

/// The radii of a round shape: a ball and a tube have one radius, an
/// ellipsoid has three. A radius past the limit is drawn as the limit, so
/// the whole numbers of the tests stay small enough to be counted exactly.
fn radii(selection: &Selection) -> [i32; 3] {
    match selection.shape {
        ELLIPSOID => [
            selection.sizes[0].clamp(1, MAX_RADIUS),
            selection.sizes[1].clamp(1, MAX_RADIUS),
            selection.sizes[2].clamp(1, MAX_RADIUS),
        ],
        _ => {
            let r = selection.sizes[0].clamp(1, MAX_RADIUS);
            [r, r, r]
        }
    }
}

/// How many columns of blocks a round shape has: the square of its width.
/// A ball and a tube are as wide as they are long, an ellipsoid as wide as
/// it is long and as deep. It is counted before the blocks are, so a shape
/// of any size is refused before it costs anything.
fn column_count(selection: &Selection) -> i128 {
    let radii = radii(selection);
    let wide = 2 * radii[0] as i128 + 1;
    if selection.shape == ELLIPSOID {
        wide * (2 * radii[2] as i128 + 1)
    } else {
        wide * wide
    }
}

/// The columns of blocks of a round shape: for every (x, z) the blocks of the
/// shape between the lowest and the highest.
fn columns(selection: &Selection) -> BTreeMap<(i32, i32), Column> {
    let center = selection.center;
    let mut out = BTreeMap::new();
    if selection.shape == CYLINDER {
        let r = selection.sizes[0].clamp(1, MAX_RADIUS);
        let h = selection.sizes[1].max(1);
        for x in (center[0].saturating_sub(r))..=(center[0].saturating_add(r)) {
            for z in (center[2].saturating_sub(r))..=(center[2].saturating_add(r)) {
                let dx = x as i128 - center[0] as i128;
                let dz = z as i128 - center[2] as i128;
                if dx * dx + dz * dz <= (r as i128) * (r as i128) {
                    out.insert(
                        (x, z),
                        Column {
                            low: center[1],
                            high: center[1].saturating_add(h - 1),
                        },
                    );
                }
            }
        }
        return out;
    }
    // A ball or an ellipsoid: the radius of each axis.
    let radii = radii(selection);
    for x in (center[0].saturating_sub(radii[0]))..=(center[0].saturating_add(radii[0])) {
        for z in (center[2].saturating_sub(radii[2]))..=(center[2].saturating_add(radii[2])) {
            if let Some(column) = column_of(selection, x, z, radii) {
                out.insert((x, z), column);
            }
        }
    }
    out
}

/// The column of a ball or an ellipsoid at (x, z): the blocks between the
/// lowest and the highest one that are inside the shape.
fn column_of(
    selection: &Selection,
    x: i32,
    z: i32,
    radii: [i32; 3],
) -> Option<Column> {
    let center = selection.center;
    if selection.shape == SPHERE {
        let r = selection.sizes[0].max(1) as i128;
        let dx = x as i128 - center[0] as i128;
        let dz = z as i128 - center[2] as i128;
        let rest = r * r - dx * dx - dz * dz;
        if rest < 0 {
            return None;
        }
        let dy = (rest as f64).sqrt().floor() as i32;
        return Some(Column {
            low: center[1].saturating_sub(dy),
            high: center[1].saturating_add(dy),
        });
    }
    // An ellipsoid: the same test as the server's, so the outline is of the
    // very blocks the server would change.
    let at = |d: i32| [x, center[1].saturating_add(d), z];
    if !inside_ellipsoid(center, radii, at(0)) {
        // The middle of the column is not inside: there is no column here.
        return None;
    }
    // How high the column goes: an estimate first, then the exact test of
    // the blocks at its ends.
    let dx = (x as f64 - center[0] as f64) / radii[0] as f64;
    let dz = (z as f64 - center[2] as f64) / radii[2] as f64;
    let rest = 1.0 - dx * dx - dz * dz;
    let mut dy = if rest <= 0.0 {
        0
    } else {
        ((radii[1] as f64) * rest.sqrt()).floor() as i32
    };
    // The estimate may be one block too low or too high: the exact test
    // says which blocks are there.
    while dy > 0 && !inside_ellipsoid(center, radii, at(dy)) {
        dy -= 1;
    }
    while dy < radii[1] && inside_ellipsoid(center, radii, at(dy + 1)) {
        dy += 1;
    }
    if !inside_ellipsoid(center, radii, at(dy)) {
        return None;
    }
    Some(Column {
        low: center[1].saturating_sub(dy),
        high: center[1].saturating_add(dy),
    })
}

/// Whether a block is inside an ellipsoid: the server's own test, so that
/// the outline and the blocks the server changes are the same. The numbers
/// are whole, so a block on the border is inside, as it is for the server.
fn inside_ellipsoid(center: [i32; 3], radii: [i32; 3], p: [i32; 3]) -> bool {
    let d = [
        p[0] as i128 - center[0] as i128,
        p[1] as i128 - center[1] as i128,
        p[2] as i128 - center[2] as i128,
    ];
    let r = [
        radii[0] as i128,
        radii[1] as i128,
        radii[2] as i128,
    ];
    let r2 = [r[0] * r[0], r[1] * r[1], r[2] * r[2]];
    let left = d[0] * d[0] * r2[1] * r2[2]
        + d[1] * d[1] * r2[0] * r2[2]
        + d[2] * d[2] * r2[0] * r2[1];
    left <= r2[0] * r2[1] * r2[2]
}

/// Whether a block is inside a round shape: the server's own test, so that
/// the outline and the blocks the server changes are the same.
#[cfg(test)]
fn inside(selection: &Selection, p: [i32; 3]) -> bool {
    let center = selection.center;
    let d = [
        p[0] as i128 - center[0] as i128,
        p[1] as i128 - center[1] as i128,
        p[2] as i128 - center[2] as i128,
    ];
    match selection.shape {
        SPHERE => {
            let r = selection.sizes[0].max(1) as i128;
            d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= r * r
        }
        ELLIPSOID => inside_ellipsoid(center, radii(selection), p),
        _ => {
            let r = selection.sizes[0].max(1) as i128;
            d[0] * d[0] + d[2] * d[2] <= r * r
                && p[1] >= selection.min[1]
                && p[1] <= selection.max[1]
        }
    }
}

/// The free sides of the blocks of a shape: every side that has no block of
/// the shape behind it.
fn faces(columns: &BTreeMap<(i32, i32), Column>) -> Vec<Face> {
    let mut out = Vec::new();
    for ((x, z), column) in columns {
        // The top and the bottom of the column.
        out.push(Face {
            axis: 1,
            at: column.high + 1,
            u: (*x, *x + 1),
            v: (*z, *z + 1),
        });
        out.push(Face {
            axis: 1,
            at: column.low,
            u: (*x, *x + 1),
            v: (*z, *z + 1),
        });
        // The sides: against the neighbouring columns.
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let neighbour = columns.get(&(x + dx, z + dz)).copied();
            for (low, high) in gaps(*column, neighbour) {
                if dx != 0 {
                    out.push(Face {
                        axis: 0,
                        at: if dx > 0 { *x + 1 } else { *x },
                        u: (low, high + 1),
                        v: (*z, *z + 1),
                    });
                } else {
                    out.push(Face {
                        axis: 2,
                        at: if dz > 0 { *z + 1 } else { *z },
                        u: (*x, *x + 1),
                        v: (low, high + 1),
                    });
                }
            }
        }
    }
    out
}

/// The blocks of a column that the neighbouring column does not have.
fn gaps(column: Column, neighbour: Option<Column>) -> Vec<(i32, i32)> {
    let Some(neighbour) = neighbour else {
        return vec![(column.low, column.high)];
    };
    let mut out = Vec::new();
    if column.low < neighbour.low {
        let high = (neighbour.low - 1).min(column.high);
        if column.low <= high {
            out.push((column.low, high));
        }
    }
    if column.high > neighbour.high {
        let low = (neighbour.high + 1).max(column.low);
        if low <= column.high {
            out.push((low, column.high));
        }
    }
    out
}

/// The shadow of a shape: its border seen from the three sides, in blocks.
/// It stands for the shape when its blocks are too many to show one by one.
///
/// All three sections go through the middle of the shape, so they meet in
/// one point: the shape stands right around its own middle.
fn shadow(selection: &Selection, columns: &BTreeMap<(i32, i32), Column>) -> Vec<Segment> {
    let mut out = Vec::new();
    if columns.is_empty() {
        return out;
    }
    // On the ground: the blocks of every x, from the lowest z to the highest.
    let mut ground: BTreeMap<i32, (i32, i32)> = BTreeMap::new();
    // The two sides: the blocks of every x and of every z, from the lowest y
    // to the highest.
    let mut side_x: BTreeMap<i32, (i32, i32)> = BTreeMap::new();
    let mut side_z: BTreeMap<i32, (i32, i32)> = BTreeMap::new();
    let mut bottom = i32::MAX;
    let mut top = i32::MIN;
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_z = i32::MAX;
    let mut max_z = i32::MIN;
    for ((x, z), column) in columns {
        ground
            .entry(*x)
            .and_modify(|r| {
                r.0 = r.0.min(*z);
                r.1 = r.1.max(*z);
            })
            .or_insert((*z, *z));
        side_x
            .entry(*x)
            .and_modify(|r| {
                r.0 = r.0.min(column.low);
                r.1 = r.1.max(column.high);
            })
            .or_insert((column.low, column.high));
        side_z
            .entry(*z)
            .and_modify(|r| {
                r.0 = r.0.min(column.low);
                r.1 = r.1.max(column.high);
            })
            .or_insert((column.low, column.high));
        bottom = bottom.min(column.low);
        top = top.max(column.high);
        min_x = min_x.min(*x);
        max_x = max_x.max(*x);
        min_z = min_z.min(*z);
        max_z = max_z.max(*z);
    }
    // The third side: the blocks of every y, from the lowest z to the
    // highest. It is the other way round from the two above, so it is
    // counted on its own.
    let side_y = projection_yz(columns);
    if selection.shape == CYLINDER {
        // A cylinder stands on the ground: its circle is at its bottom and
        // at its top.
        staircase(&mut out, 1, bottom, &ground);
        staircase(&mut out, 1, top + 1, &ground);
    } else {
        // A ball or an ellipsoid: the circle in the ground plane goes
        // through the middle of the shape, like the other two sections.
        staircase(&mut out, 1, (bottom + top) / 2, &ground);
    }
    // The shadow on the side at z, at the middle of the shape.
    staircase(&mut out, 2, (min_z + max_z) / 2, &side_x);
    // The shadow on the side at x, at the middle of the shape.
    staircase(&mut out, 0, (min_x + max_x) / 2, &side_y);
    out
}

/// The blocks of the shape seen from the side of x: for every y, the blocks
/// from the lowest z to the highest.
///
/// The columns give the shape as columns of blocks, so the answer is found
/// by sweeping them by their height: a column is there from its lowest
/// block to its highest.
fn projection_yz(columns: &BTreeMap<(i32, i32), Column>) -> BTreeMap<i32, (i32, i32)> {
    let mut out = BTreeMap::new();
    let mut bottom = i32::MAX;
    let mut top = i32::MIN;
    // The columns that start and the ones that end at every height.
    let mut starts: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
    let mut ends: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
    for ((_, z), column) in columns {
        starts.entry(column.low).or_default().push(*z);
        ends.entry(column.high + 1).or_default().push(*z);
        bottom = bottom.min(column.low);
        top = top.max(column.high);
    }
    if columns.is_empty() {
        return out;
    }
    // The z of the columns that are there now, and how many of them.
    let mut here: BTreeMap<i32, i32> = BTreeMap::new();
    for y in bottom..=top {
        if let Some(list) = ends.get(&y) {
            for z in list {
                if let Some(n) = here.get_mut(z) {
                    *n -= 1;
                    if *n <= 0 {
                        here.remove(z);
                    }
                }
            }
        }
        if let Some(list) = starts.get(&y) {
            for z in list {
                *here.entry(*z).or_insert(0) += 1;
            }
        }
        if let (Some(low), Some(high)) = (here.keys().next(), here.keys().next_back()) {
            out.insert(y, (*low, *high));
        }
    }
    out
}

/// The border of a shape seen from one side: a staircase of blocks.
fn staircase(
    out: &mut Vec<Segment>,
    axis: usize,
    at: i32,
    ranges: &BTreeMap<i32, (i32, i32)>,
) {
    let mut previous: Option<(i32, i32, i32)> = None;
    for (&i, &(low, high)) in ranges {
        // The top and the bottom of this slice. The lines stand on the
        // borders of the blocks, so the top is one block above the highest
        // one and the bottom is at the lowest one.
        let top = high.saturating_add(1);
        border(out, axis, at, i, top, i + 1, top);
        border(out, axis, at, i, low, i + 1, low);
        match previous {
            None => {
                // The first slice: its side.
                border(out, axis, at, i, low, i, top);
            }
            Some((previous_index, previous_low, previous_high)) => {
                // The step to this slice.
                let _ = previous_index;
                if previous_high != high {
                    let previous_top = previous_high.saturating_add(1);
                    border(
                        out,
                        axis,
                        at,
                        i,
                        previous_top.min(top),
                        i,
                        previous_top.max(top),
                    );
                }
                if previous_low != low {
                    border(
                        out,
                        axis,
                        at,
                        i,
                        previous_low.min(low),
                        i,
                        previous_low.max(low),
                    );
                }
            }
        }
        previous = Some((i, low, high));
    }
    if let Some((i, low, high)) = previous {
        // The last slice: its side.
        border(
            out,
            axis,
            at,
            i + 1,
            low,
            i + 1,
            high.saturating_add(1),
        );
    }
}

/// A line on a plane: its axis with its place, and its ends in the two
/// axes of the plane.
fn border(
    out: &mut Vec<Segment>,
    axis: usize,
    at: i32,
    u0: i32,
    v0: i32,
    u1: i32,
    v1: i32,
) {
    let a = point(axis, at, u0, v0);
    let b = point(axis, at, u1, v1);
    if a == b {
        return;
    }
    line(
        out,
        [a[0] as f64, a[1] as f64, a[2] as f64],
        [b[0] as f64, b[1] as f64, b[2] as f64],
        FRAME,
        SHAPE_WIDTH,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selection() -> Selection {
        Selection {
            shape: CUBE,
            dimension: "minecraft:overworld".to_owned(),
            min: [0, 64, 0],
            max: [9, 69, 9],
            center: [4, 66, 4],
            sizes: [10, 6, 10],
            grid: true,
        }
    }

    #[test]
    fn the_frame_has_twelve_edges() {
        let lines = segments(&selection());
        let frame = lines.iter().filter(|s| s.color == FRAME).count();
        assert_eq!(frame, 12, "the frame alone has twelve edges");
    }

    #[test]
    fn the_frame_stands_around_the_blocks() {
        let lines = segments(&selection());
        let mut far = [f64::MIN; 3];
        let mut near = [f64::MAX; 3];
        for s in lines.iter().filter(|s| s.color == FRAME) {
            for axis in 0..3 {
                if s.start[axis] > far[axis] {
                    far[axis] = s.start[axis];
                }
                if s.end[axis] > far[axis] {
                    far[axis] = s.end[axis];
                }
                if s.start[axis] < near[axis] {
                    near[axis] = s.start[axis];
                }
                if s.end[axis] < near[axis] {
                    near[axis] = s.end[axis];
                }
            }
        }
        // The frame ends one block further than the last block.
        assert_eq!(near, [0.0, 64.0, 0.0]);
        assert_eq!(far, [10.0, 70.0, 10.0]);
    }

    #[test]
    fn the_grid_has_a_line_on_every_level() {
        let lines = segments(&selection());
        let grid = lines.iter().filter(|s| s.color == GRID).count();
        // The box is ten by six by ten blocks: eleven, seven and eleven
        // levels, four sides each.
        assert_eq!(grid, 4 * (11 + 7 + 11));
    }

    #[test]
    fn the_grid_can_be_turned_off_and_the_frame_stays() {
        let mut s = selection();
        s.grid = false;
        let lines = segments(&s);
        assert_eq!(lines.iter().filter(|s| s.color == GRID).count(), 0);
        assert_eq!(lines.iter().filter(|s| s.color == FRAME).count(), 12);
    }

    #[test]
    fn the_lines_are_red() {
        // #CCFF0000: red, not the purple of CUI.
        assert_eq!(FRAME, 0xCCFF_0000);
        assert_eq!(FRAME & 0x00FF_0000, 0x00FF_0000, "the red channel is full");
        assert_eq!(FRAME & 0x0000_FF00, 0, "no green");
        assert_eq!(FRAME & 0x0000_00FF, 0, "no blue");
    }

    /// Whether every line stands on the border of a block: its ends are
    /// whole blocks.
    fn on_the_grid(lines: &[Segment]) -> bool {
        lines.iter().all(|s| {
            s.start.iter().all(|v| v.fract() == 0.0) && s.end.iter().all(|v| v.fract() == 0.0)
        })
    }

    #[test]
    fn a_ball_is_drawn_as_its_blocks() {
        let mut s = selection();
        s.shape = SPHERE;
        s.center = [10, 64, 10];
        s.sizes = [6, 0, 0];
        let lines = segments(&s);
        assert!(lines.len() > 100, "{} lines", lines.len());
        assert!(on_the_grid(&lines), "the lines stand on the blocks");
        // The outline stays around the ball: the radius of six blocks.
        let mut far = [f64::MIN; 3];
        let mut near = [f64::MAX; 3];
        for l in &lines {
            for axis in 0..3 {
                far[axis] = far[axis].max(l.start[axis]);
                near[axis] = near[axis].min(l.start[axis]);
            }
        }
        assert!(near[0] >= 4.0 && far[0] <= 17.0, "{near:?} {far:?}");
        // The ball of six blocks has blocks from 4 to 16 around its middle.
        assert!(near[0] <= 4.0 && far[0] >= 16.0, "{near:?} {far:?}");
    }

    #[test]
    fn the_blocks_of_a_ball_are_the_ones_the_server_changes() {
        // The outline shows the same blocks as the server's own test.
        let mut s = selection();
        s.shape = SPHERE;
        s.center = [0, 64, 0];
        s.sizes = [4, 0, 0];
        let columns = columns(&s);
        let mut blocks = 0;
        for column in columns.values() {
            blocks += (column.high - column.low + 1) as usize;
        }
        // The count of a ball of four: the server's own number.
        let mut inside = 0;
        for x in -4..=4 {
            for y in -4..=4 {
                for z in -4..=4 {
                    if x * x + y * y + z * z <= 16 {
                        inside += 1;
                    }
                }
            }
        }
        assert_eq!(blocks, inside, "the same blocks as the server's test");
    }

    #[test]
    fn a_tube_is_drawn_as_its_blocks() {
        let mut s = selection();
        s.shape = CYLINDER;
        s.center = [10, 64, 10];
        s.sizes = [3, 4, 0];
        let lines = segments(&s);
        assert!(on_the_grid(&lines), "the lines stand on the blocks");
        // The tube of three blocks around its middle.
        let mut far = f64::MIN;
        let mut near = f64::MAX;
        for l in &lines {
            far = far.max(l.start[0]);
            near = near.min(l.start[0]);
        }
        assert!(near >= 7.0 && far <= 14.0, "{near} {far}");
    }

    #[test]
    fn a_wide_selection_keeps_a_cell_of_a_block() {
        let mut s = selection();
        s.min = [0, 0, 0];
        s.max = [999, 99, 999];
        let lines = segments(&s);
        // A thousand blocks on a side is still a cell of a block: every line
        // of the grid stands on the border of a block.
        let grid: Vec<_> = lines.iter().filter(|l| l.color == GRID).collect();
        assert_eq!(grid.len(), 4 * (1001 + 101 + 1001));
        for l in &grid {
            for axis in 0..3 {
                assert_eq!((l.start[axis] - l.end[axis]).abs().fract(), 0.0, "{l:?}");
            }
        }
    }

    #[test]
    fn a_huge_selection_gets_wider_cells() {
        let mut s = selection();
        s.min = [0, 0, 0];
        s.max = [4000, 4000, 4000];
        let lines = segments(&s);
        // Too many blocks to draw a cell for each of them: the cells grow,
        // so the lines stay few.
        assert!(lines.len() <= 4 * MAX_GRID_PLANES as usize + 16, "{} lines", lines.len());
    }

    #[test]
    fn a_cell_of_the_grid_is_a_block() {
        let mut s = selection();
        s.min = [0, 0, 0];
        s.max = [39, 19, 39];
        let lines = segments(&s);
        // Forty by twenty by forty blocks: the grid has a line on every
        // border of a block, so a cell is one block wide.
        let grid: Vec<_> = lines.iter().filter(|l| l.color == GRID).collect();
        assert_eq!(grid.len(), 4 * (41 + 21 + 41));
        // Every line of it runs along the border of the blocks.
        for l in &grid {
            for axis in 0..3 {
                assert_eq!((l.start[axis] - l.end[axis]).abs().fract(), 0.0, "{l:?}");
            }
        }
    }

    #[test]
    fn a_big_ball_gets_its_shadow_instead_of_its_blocks() {
        let mut s = selection();
        s.shape = SPHERE;
        s.center = [0, 64, 0];
        s.sizes = [100, 0, 0];
        let lines = segments(&s);
        assert!(on_the_grid(&lines), "the lines stand on the blocks");
        // Too many blocks to show one by one: the shadow stands for it.
        assert!(lines.len() < 3000, "{} lines", lines.len());
    }

    #[test]
    fn the_sections_of_a_big_ball_meet_in_its_middle() {
        let mut s = selection();
        s.shape = SPHERE;
        s.center = [0, 64, 0];
        s.sizes = [50, 0, 0];
        let lines = segments(&s);
        // The three sections of the shadow: the circle in the ground plane,
        // the one at z and the one at x. All of them go through the middle
        // of the ball, so the axes of the shape do not slip apart.
        let mut lowest = f64::MAX;
        let mut highest = f64::MIN;
        let mut lowest_x = f64::MAX;
        let mut highest_x = f64::MIN;
        let mut lowest_z = f64::MAX;
        let mut highest_z = f64::MIN;
        for l in &lines {
            for axis in 0..3 {
                let a = l.start[axis];
                let b = l.end[axis];
                match axis {
                    0 => {
                        lowest_x = lowest_x.min(a).min(b);
                        highest_x = highest_x.max(a).max(b);
                    }
                    1 => {
                        lowest = lowest.min(a).min(b);
                        highest = highest.max(a).max(b);
                    }
                    _ => {
                        lowest_z = lowest_z.min(a).min(b);
                        highest_z = highest_z.max(a).max(b);
                    }
                }
            }
        }
        // The ball of fifty blocks reaches from -50 to 50 around its middle.
        assert!(lowest_x >= -51.0 && highest_x <= 51.0, "{lowest_x} {highest_x}");
        assert!(lowest >= 13.0 && highest <= 115.0, "{lowest} {highest}");
        assert!(lowest_z >= -51.0 && highest_z <= 51.0, "{lowest_z} {highest_z}");
        // And the shape is even around its middle: the axes do not slip.
        // The lines stand on the borders of the blocks, so the far ends are
        // one block past the last one.
        let middle = |low: f64, high: f64, center: f64| {
            assert!(
                ((low + high) / 2.0 - center).abs() <= 1.0,
                "the shape is not even: {low} {high} around {center}"
            );
        };
        middle(lowest_x, highest_x, s.center[0] as f64);
        middle(lowest, highest, s.center[1] as f64);
        middle(lowest_z, highest_z, s.center[2] as f64);
    }

    #[test]
    fn a_broken_message_is_no_selection() {
        assert_eq!(parse(&[]), None);
        assert_eq!(parse(&[0]), None);
        assert_eq!(parse(&[1]), None);
        assert_eq!(parse(&[1, 0, 200, 1, 2]), None);
        assert_eq!(parse(&[2, 0, 0]), None);
    }

    /// Whether a line stands between two places, in any order.
    fn has_edge(lines: &[Segment], a: [f64; 3], b: [f64; 3]) -> bool {
        lines
            .iter()
            .any(|l| (l.start == a && l.end == b) || (l.start == b && l.end == a))
    }

    #[test]
    fn a_huge_ball_gets_the_frame_of_its_box_instead_of_its_blocks() {
        let mut s = selection();
        s.shape = SPHERE;
        s.center = [0, 64, 0];
        s.sizes = [100_000, 0, 0];
        s.min = [-100_000, -99_936, -100_000];
        s.max = [100_000, 200_000, 100_000];
        let lines = segments(&s);
        // Too many blocks to count: the frame of the box around the shape
        // stands for it, so the work stays the same for any size.
        assert!(lines.len() <= 64, "{} lines", lines.len());
        assert_eq!(lines.iter().filter(|l| l.color == FRAME).count(), 12);
    }

    #[test]
    fn a_ball_of_ten_blocks_is_still_drawn_as_its_blocks() {
        let mut s = selection();
        s.shape = SPHERE;
        s.center = [0, 64, 0];
        s.sizes = [10, 0, 0];
        let lines = segments(&s);
        // The blocks of the ball, one square at a time.
        assert!(lines.len() > 1000, "{} lines", lines.len());
        assert!(on_the_grid(&lines), "the lines stand on the blocks");
        // The outline stands around the ball: the blocks are from -10 to 10
        // and the border is one block further.
        let mut near = [f64::MAX; 3];
        let mut far = [f64::MIN; 3];
        for l in &lines {
            for axis in 0..3 {
                near[axis] = near[axis].min(l.start[axis]).min(l.end[axis]);
                far[axis] = far[axis].max(l.start[axis]).max(l.end[axis]);
            }
        }
        assert!(near[0] <= -10.0 && far[0] >= 11.0, "{near:?} {far:?}");
        assert!(near[1] <= 54.0 && far[1] >= 75.0, "{near:?} {far:?}");
    }

    #[test]
    fn an_ellipsoid_reaches_the_blocks_on_its_border() {
        let mut s = selection();
        s.shape = ELLIPSOID;
        s.center = [0, 0, 0];
        s.sizes = [5, 5, 5];
        let columns = columns(&s);
        // The column at x = 4 holds the border of the shape: the server
        // counts the block at y = 3 as inside, so the outline holds it too.
        let column = columns.get(&(4, 0)).expect("the column at x = 4");
        assert_eq!(column.high, 3, "the block at [4,3,0] is inside");
        assert_eq!(column.low, -3, "the block at [4,-3,0] is inside");
        // And the outline draws the borders of those very blocks.
        let lines = segments(&s);
        assert!(
            has_edge(&lines, [4.0, 4.0, 0.0], [5.0, 4.0, 0.0]),
            "the top of the block at [4,3,0]"
        );
        assert!(
            has_edge(&lines, [4.0, -3.0, 0.0], [5.0, -3.0, 0.0]),
            "the bottom of the block at [4,-3,0]"
        );
    }

    #[test]
    fn a_block_outside_an_ellipsoid_is_not_inside() {
        let mut s = selection();
        s.shape = ELLIPSOID;
        s.center = [0, 0, 0];
        s.sizes = [5, 5, 5];
        // On the border of the shape: inside.
        assert!(inside(&s, [4, 3, 0]), "the border of the shape is inside");
        // Clearly outside: not inside.
        assert!(!inside(&s, [4, 4, 0]), "the corner is outside");
        assert!(!inside(&s, [6, 0, 0]), "the far side is outside");
    }

    #[test]
    fn the_shadow_of_a_shape_stands_on_the_borders_of_its_blocks() {
        // One block: the shadow of it is the three sides of its box.
        let mut s = selection();
        s.shape = SPHERE;
        s.center = [0, 0, 0];
        s.sizes = [1, 0, 0];
        let mut columns = BTreeMap::new();
        columns.insert((0, 0), Column { low: 0, high: 0 });
        let lines = shadow(&s, &columns);
        // The top of a slice is one block above its highest block: the
        // border of the block above it.
        assert!(
            has_edge(&lines, [0.0, 0.0, 1.0], [1.0, 0.0, 1.0]),
            "the top of the slice"
        );
        // The bottom of it is at its lowest block.
        assert!(
            has_edge(&lines, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
            "the bottom of the slice"
        );
        // And the side of the slice runs from the bottom to the top.
        assert!(
            has_edge(&lines, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
            "the side of the slice"
        );
        // The top and the bottom are lines of their own: a slice of one
        // block has a top and a bottom, not one line.
        assert_eq!(lines.len(), 12, "three sections of four lines");
    }

    #[test]
    fn the_steps_of_a_shadow_stand_on_the_borders_of_its_blocks() {
        // Two slices: the first one is three blocks high, the second one is
        // a single block.
        let mut ranges = BTreeMap::new();
        ranges.insert(0, (0, 2));
        ranges.insert(1, (0, 0));
        let mut lines = Vec::new();
        staircase(&mut lines, 1, 0, &ranges);
        // The top of the first slice is one block above its highest block.
        assert!(
            has_edge(&lines, [0.0, 0.0, 3.0], [1.0, 0.0, 3.0]),
            "the top of the first slice"
        );
        // The bottom of it is at its lowest block.
        assert!(
            has_edge(&lines, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
            "the bottom of the first slice"
        );
        // The top of the second slice is one block above its only block.
        assert!(
            has_edge(&lines, [1.0, 0.0, 1.0], [2.0, 0.0, 1.0]),
            "the top of the second slice"
        );
        // The step between them runs from the top of the first slice to
        // the top of the second one.
        assert!(
            has_edge(&lines, [1.0, 0.0, 1.0], [1.0, 0.0, 3.0]),
            "the step between the slices"
        );
    }
}
