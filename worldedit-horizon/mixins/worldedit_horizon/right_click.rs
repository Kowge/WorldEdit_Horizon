// The wand's right click sets the second corner, and the block and the
// item do nothing else.
//
// A dedicated server never hears the click itself: the client tells it,
// and the server sets the corner and sends the selection back.
fn we_right_click_block(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    if *canceled {
        return;
    }
    if crate::game::we_glue::wand_click(game, 2, pos) {
        *canceled = true;
        return;
    }
    crate::game::we_glue::wand_request(game, pos);
}
