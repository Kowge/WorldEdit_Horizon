// The WorldEdit rules on their own, tested outside the game: the file is a
// module of the game in the bundle and of this crate in its tests. The paths
// are relative, so the crate tests exactly the sources that are packaged.
#[path = "../../mixins/worldedit_horizon/we_core.rs"]
mod we_core;
// The client's half: the selection as it is drawn, without the game types.
#[path = "../../mixins/worldedit_horizon/geometry.rs"]
mod geometry;

#[cfg(test)]
mod tests {
    use crate::{geometry, we_core};
    use we_core::World;

    #[test]
    fn the_client_reads_what_the_server_sends() {
        let mut selection = we_core::Selection::default();
        selection.set_pos(1, "minecraft:overworld", [10, 64, -5]);
        selection.set_pos(2, "minecraft:overworld", [20, 70, 5]);
        let payload = we_core::selection_payload(&selection);
        let read = geometry::parse(&payload).expect("the client reads the selection");
        assert_eq!(read.dimension, "minecraft:overworld");
        assert_eq!(read.min, [10, 64, -5]);
        assert_eq!(read.max, [20, 70, 5]);
        assert_eq!(read.shape, 0);
    }

    #[test]
    fn a_round_selection_travels_with_its_shape() {
        let mut selection = we_core::Selection::default();
        selection.set_pos(1, "minecraft:overworld", [0, 64, 0]);
        selection.set_pos(2, "minecraft:overworld", [10, 64, 0]);
        selection.shape = we_core::Shape::Sphere;
        let payload = we_core::selection_payload(&selection);
        let read = geometry::parse(&payload).expect("the client reads the selection");
        assert_eq!(read.shape, 1, "a sphere");
        // The box around the ball.
        assert_eq!(read.min, [-10, 54, -10]);
        assert_eq!(read.max, [10, 74, 10]);
    }

    #[test]
    fn an_empty_selection_is_no_selection() {
        let selection = we_core::Selection::default();
        let payload = we_core::selection_payload(&selection);
        assert_eq!(geometry::parse(&payload), None);
    }

    #[test]
    fn the_client_draws_the_box_the_server_sent() {
        let mut selection = we_core::Selection::default();
        selection.set_pos(1, "minecraft:overworld", [0, 64, 0]);
        selection.set_pos(2, "minecraft:overworld", [9, 69, 9]);
        let payload = we_core::selection_payload(&selection);
        let read = geometry::parse(&payload).expect("the client reads the selection");
        let lines = geometry::segments(&read);
        let frame = lines.iter().filter(|s| s.color == geometry::FRAME).count();
        assert_eq!(frame, 12, "the frame has twelve edges");
        // The grid: a rectangle on every level of every axis. The box is ten
        // by six by ten blocks: eleven, seven and eleven levels.
        let grid = lines.iter().filter(|s| s.color == geometry::GRID).count();
        assert_eq!(grid, 4 * (11 + 7 + 11));
    }

    #[test]
    fn a_wand_click_travels_with_its_block() {
        let payload = we_core::wand_payload([10, 64, -5]);
        assert_eq!(we_core::parse_wand(&payload), Some([10, 64, -5]));
        assert_eq!(we_core::parse_wand(&[1, 2, 3]), None);
        assert_eq!(we_core::parse_wand(&[]), None);
    }

    #[test]
    fn the_selection_changes_when_the_corners_do() {
        let mut selection = we_core::Selection::default();
        selection.set_pos(1, "minecraft:overworld", [0, 0, 0]);
        let before = we_core::selection_state(&selection);
        selection.set_pos(2, "minecraft:overworld", [1, 1, 1]);
        assert_ne!(before, we_core::selection_state(&selection));
    }

    #[test]
    fn the_players_own_view_travels_with_the_selection() {
        let mut selection = we_core::Selection::default();
        selection.set_pos(1, "minecraft:overworld", [0, 64, 0]);
        selection.set_pos(2, "minecraft:overworld", [9, 69, 9]);
        // The grid is on by default.
        let payload = we_core::selection_payload(&selection);
        let read = geometry::parse(&payload).expect("the client reads the selection");
        assert!(read.grid, "the grid is on by default");
        // The player turns it off: the client hears about it.
        selection.grid = false;
        let payload = we_core::selection_payload(&selection);
        let read = geometry::parse(&payload).expect("the client reads the selection");
        assert!(!read.grid, "the grid is off for this player");
        // The frame stays, the grid inside it goes.
        let lines = geometry::segments(&read);
        assert_eq!(lines.iter().filter(|s| s.color == geometry::FRAME).count(), 12);
        assert_eq!(lines.iter().filter(|s| s.color == geometry::GRID).count(), 0);
        // And the change is a change of the state, so the server tells the
        // client about it.
        let before = we_core::selection_state(&selection);
        selection.grid = true;
        assert_ne!(before, we_core::selection_state(&selection));
    }

    #[test]
    fn the_view_survives_a_new_selection() {
        let mut selection = we_core::Selection::default();
        selection.set_pos(1, "minecraft:overworld", [0, 64, 0]);
        selection.set_pos(2, "minecraft:overworld", [9, 69, 9]);
        selection.grid = false;
        // The corners change: the player's own view stays.
        selection.set_pos(2, "minecraft:overworld", [19, 69, 19]);
        assert!(!selection.grid, "the grid stays off");
        // Another level: the corners go, the player's own view stays.
        selection.set_pos(1, "minecraft:the_nether", [0, 64, 0]);
        assert!(!selection.grid, "the grid stays off in another level");
    }

    #[test]
    fn the_grid_cell_is_one_block_wide() {
        let mut selection = we_core::Selection::default();
        // A wide selection: thirty by four by thirty blocks.
        selection.set_pos(1, "minecraft:overworld", [0, 64, 0]);
        selection.set_pos(2, "minecraft:overworld", [29, 67, 29]);
        let payload = we_core::selection_payload(&selection);
        let read = geometry::parse(&payload).expect("the client reads the selection");
        let lines = geometry::segments(&read);
        // Every line of the grid stands on the border of a block: no cell is
        // wider than a block, however wide the selection is.
        for l in lines.iter().filter(|s| s.color == geometry::GRID) {
            for axis in 0..3 {
                assert_eq!(
                    (l.start[axis] - l.end[axis]).abs().fract(),
                    0.0,
                    "the line runs along the blocks, not across them"
                );
            }
        }
        // The grid has a line on every border of a block: thirty one, five
        // and thirty one planes, four lines each.
        let grid = lines.iter().filter(|s| s.color == geometry::GRID).count();
        assert_eq!(grid, 4 * (31 + 5 + 31));
    }

    #[test]
    fn a_ball_travels_as_a_ball() {
        let mut selection = we_core::Selection::default();
        selection.set_pos(1, "minecraft:overworld", [10, 64, 10]);
        selection.set_pos(2, "minecraft:overworld", [16, 64, 10]);
        selection.shape = we_core::Shape::Sphere;
        let payload = we_core::selection_payload(&selection);
        let read = geometry::parse(&payload).expect("the client reads the selection");
        assert_eq!(read.shape, geometry::SPHERE);
        assert_eq!(read.center, [10, 64, 10]);
        assert_eq!(read.sizes[0], 6, "the radius");
        let lines = geometry::segments(&read);
        assert!(lines.len() > 100, "the ball is drawn as its blocks");
        // The lines stand on the border of the blocks, not in the air.
        for l in &lines {
            assert!(l.start.iter().all(|v| v.fract() == 0.0), "{:?}", l.start);
            assert!(l.end.iter().all(|v| v.fract() == 0.0), "{:?}", l.end);
        }
    }

    #[test]
    fn a_hostile_header_is_refused() {
        // An index outside the count of the file.
        assert!(!we_core::part_ok(5, 3, 10));
        // A count of a size no file may have.
        assert!(!we_core::part_ok(0, we_core::MAX_PARTS + 1, 10));
        // No part at all.
        assert!(!we_core::part_ok(0, 0, 10));
        // A part bigger than a part may be.
        assert!(!we_core::part_ok(0, 1, we_core::PART_SIZE + 1));
        // A part of a file may be.
        assert!(we_core::part_ok(0, 1, we_core::PART_SIZE));
        assert!(we_core::part_ok(
            we_core::MAX_PARTS - 1,
            we_core::MAX_PARTS,
            1
        ));
    }

    #[test]
    fn a_huge_index_does_not_make_a_huge_file() {
        let mut parts = we_core::Parts::new(3);
        // The index is outside the count of the file: it is refused.
        assert_eq!(parts.put(1_000_000_000, 3, b"x"), Err("index"));
        // The count of the file is not the one of the header.
        assert_eq!(parts.put(0, 4, b"x"), Err("count"));
        // Nothing was kept.
        assert!(parts.take().is_none());
    }

    #[test]
    fn a_file_is_put_together_from_its_parts() {
        let mut parts = we_core::Parts::new(2);
        assert_eq!(parts.put(0, 2, b"ab"), Ok(false));
        assert_eq!(parts.put(1, 2, b"cd"), Ok(true));
        assert_eq!(parts.take(), Some(b"abcd".to_vec()));
        // The file is gone: it is not there twice.
        assert_eq!(parts.take(), None);
    }

    #[test]
    fn a_file_does_not_grow_past_its_size() {
        let mut parts = we_core::Parts::new(we_core::MAX_PARTS);
        let part = vec![0; we_core::PART_SIZE];
        let mut refused = false;
        for index in 0..we_core::MAX_PARTS {
            if parts.put(index, we_core::MAX_PARTS, &part).is_err() {
                refused = true;
                break;
            }
        }
        assert!(refused, "the file stops at its size");
        assert!(parts.take().is_none());
    }

    #[test]
    fn the_client_does_only_what_the_player_asked_for() {
        let mut allowed = we_core::Allowed::default();
        // Nothing is allowed before the player asks.
        assert!(!allowed.allows("read", "house"));
        assert!(!allowed.allows("list", ""));
        // The player asks for a file.
        allowed.ask("read", "house");
        assert!(allowed.allows("read", "house"));
        // The server may not ask for another one.
        assert!(!allowed.allows("read", "castle"));
        // The allowance is spent once.
        assert!(allowed.take("read", "house"));
        assert!(!allowed.take("read", "house"));
    }

    #[test]
    fn the_write_of_a_file_is_allowed_until_it_is_written() {
        let mut allowed = we_core::Allowed::default();
        allowed.ask("write", "house");
        // The parts of the file come one by one: the write is allowed
        // until the file is there.
        assert!(allowed.allows("write", "house"));
        // The file is written: the server may not write it again.
        allowed.clear("write", "house");
        assert!(!allowed.allows("write", "house"));
        // Another file is not allowed either.
        assert!(!allowed.allows("write", "castle"));
    }

    #[test]
    fn the_listing_of_the_files_is_allowed_once() {
        let mut allowed = we_core::Allowed::default();
        allowed.ask("list", "");
        assert!(allowed.allows("list", ""));
        assert!(allowed.take("list", ""));
        assert!(!allowed.allows("list", ""));
    }

    /// A world of blocks, for the commands that change them.
    #[derive(Default)]
    struct Ground(std::collections::HashMap<we_core::Pos, u32>);

    impl we_core::World for Ground {
        fn get(&self, p: we_core::Pos) -> u32 {
            self.0.get(&p).copied().unwrap_or(0)
        }
        fn set(&mut self, p: we_core::Pos, block: u32) -> bool {
            let old = self.get(p);
            self.0.insert(p, block);
            old != block
        }
        fn is_solid(&self, block: u32) -> bool {
            block != 0
        }
        fn is_liquid(&self, _block: u32) -> bool {
            false
        }
    }

    /// The ball as it was made before the fix: the reference of the blocks.
    fn old_ball(center: we_core::Pos, radius: i32, hollow: bool) -> Vec<we_core::Pos> {
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
                        out.push([center[0] + x, center[1] + y, center[2] + z]);
                    }
                }
            }
        }
        out
    }

    #[test]
    fn the_ball_is_the_ball_of_before() {
        for radius in [1, 2, 5, 16] {
            for hollow in [false, true] {
                let mut now = we_core::sphere([10, 20, 30], radius, hollow).expect("the ball");
                let mut before = old_ball([10, 20, 30], radius, hollow);
                now.sort();
                before.sort();
                assert_eq!(now, before, "radius {radius}, hollow {hollow}");
            }
        }
    }

    #[test]
    fn the_ball_is_refused_at_the_radius_it_was_refused_before() {
        // The ball of before was made and then refused, when it was too
        // big: the new one is refused at the same radius.
        for radius in 44..=52 {
            let before = old_ball([0, 0, 0], radius, false).len() as u64;
            let was_ok = radius <= we_core::max_radius() && before <= we_core::max_blocks();
            let now = we_core::sphere([0, 0, 0], radius, false);
            assert_eq!(now.is_ok(), was_ok, "radius {radius}, {before} blocks");
        }
        // The hollow ball is refused by the radius of it as well.
        assert!(we_core::sphere([0, 0, 0], we_core::max_radius() + 1, true).is_err());
    }

    #[test]
    fn a_ball_too_big_for_the_limit_is_refused_before_it_is_made() {
        // The radius limit is lifted, as the report of the defect has it.
        we_core::set_limit("radius", Some(1024)).expect("the radius limit");
        let start = std::time::Instant::now();
        let too_big = we_core::sphere([0, 0, 0], 1024, false);
        let spent = start.elapsed();
        // Only the radius limit is put back, so that the other limits
        // the tests use are left alone.
        we_core::set_limit("radius", Some(we_core::MAX_RADIUS as i64)).expect("the radius limit");
        let message = too_big.expect_err("the ball is too big");
        assert!(message.contains("the limit is"), "{message}");
        assert!(spent.as_secs() < 5, "the ball is counted, not made");
        // The radius beyond the limit is refused as well.
        assert!(we_core::sphere([0, 0, 0], 1024, false).is_err());
    }

    #[test]
    fn the_fill_is_only_as_deep_as_the_ball() {
        let world = Ground::default();
        let stone = we_core::Pattern::single(1);
        let normal = we_core::fill_hole(&world, [0, 10, 0], 10, 10, &stone, true, 0)
            .expect("the hole is filled");
        let deep = we_core::fill_hole(&world, [0, 10, 0], 10, i32::MAX, &stone, true, 0)
            .expect("the hole is filled");
        assert_eq!(normal, deep, "the ball is only ten blocks deep");
        let plain = we_core::fill_hole(&world, [0, 10, 0], 10, 10, &stone, false, 0)
            .expect("the hole is filled");
        let plain_deep = we_core::fill_hole(&world, [0, 10, 0], 10, i32::MAX, &stone, false, 0)
            .expect("the hole is filled");
        assert_eq!(plain, plain_deep, "the ball is only ten blocks deep");
    }

    #[test]
    fn a_ball_too_big_for_the_limit_is_refused_by_the_fill() {
        let world = Ground::default();
        let stone = we_core::Pattern::single(1);
        let too_big = we_core::fill_hole(&world, [0, 0, 0], 60, 10, &stone, true, 0);
        let message = too_big.expect_err("the ball is too big");
        assert!(message.contains("the limit is"), "{message}");
    }

    #[test]
    fn a_round_selection_is_not_changed_outside_its_shape() {
        let region = we_core::Region::around(we_core::Shape::Sphere, [0, 0, 0], &[5])
            .expect("the ball of the selection");
        let mut world = Ground::default();
        // The ground is one block below the middle of the ball, and it
        // reaches the corners of the box.
        for x in -5..=5 {
            for z in -5..=5 {
                world.set([x, -1, z], 1);
                world.set([x, -2, z], 1);
            }
        }
        let stone = we_core::Pattern::single(2);
        let top = we_core::overlay_writes(&world, &region, 0, &stone);
        assert!(!top.is_empty(), "the ball stands on the ground");
        for (p, _) in &top {
            assert!(region.contains(*p), "{p:?} is outside the ball");
            assert!(
                p[0] * p[0] + p[2] * p[2] <= 25,
                "the corners of the box are not in the ball"
            );
        }
        let nat = we_core::naturalize_writes(&world, &region, 0, (3, 4, 5));
        assert!(!nat.is_empty(), "the ball stands on the ground");
        for (p, _) in &nat {
            assert!(region.contains(*p), "{p:?} is outside the ball");
            assert!(
                p[0] * p[0] + p[2] * p[2] <= 25,
                "the corners of the box are not in the ball"
            );
        }
        // The depth is counted over the blocks of the ball only.
        assert!(nat.contains(&([0, -1, 0], 3)), "the top of the ground");
        assert!(nat.contains(&([0, -2, 0], 4)), "the soil under it");
    }

    #[test]
    fn an_ellipsoid_holds_the_blocks_on_its_surface() {
        let r = we_core::Region::around(we_core::Shape::Ellipsoid, [0, 0, 0], &[5, 5, 5])
            .expect("the ellipsoid");
        assert!(r.contains([4, 3, 0]), "the block is on the surface");
        assert!(r.contains([4, -3, 0]), "the block is on the surface");
        assert!(!r.contains([4, 4, 0]), "the block is outside");
    }
}
