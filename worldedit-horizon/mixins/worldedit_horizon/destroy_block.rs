// The wand breaks nothing: its left click sets the first corner, or paints
// with the brush when one is armed.
fn we_destroy_block(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    if *canceled {
        return;
    }
    // A brush paints instead of selecting.
    if crate::game::we_glue::brush_click(game, pos) {
        *canceled = true;
        return;
    }
    if crate::game::we_glue::wand_click(game, 1, pos) {
        *canceled = true;
    }
}
