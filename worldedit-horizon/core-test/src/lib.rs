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
}
