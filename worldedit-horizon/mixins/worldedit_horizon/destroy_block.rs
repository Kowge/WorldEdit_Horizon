// The wand breaks nothing: its left click sets the first corner.
fn we_destroy_block(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    if !*canceled && crate::game::we_glue::wand_click(game, 1, pos) {
        *canceled = true;
    }
}
