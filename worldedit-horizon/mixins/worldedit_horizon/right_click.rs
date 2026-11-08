// The wand's right click sets the second corner, and the block and the
// item do nothing else.
fn we_right_click_block(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    if !*canceled && crate::game::we_glue::wand_click(game, 2, pos) {
        *canceled = true;
    }
}
