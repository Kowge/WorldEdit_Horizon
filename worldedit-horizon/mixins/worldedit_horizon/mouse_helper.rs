// The left mouse button: the client tells the server about a click in the
// air, where there is no block to break.
fn we_mouse_button_pre(
    game: &mut crate::game::Game<'_>,
    button: i32,
    pressed: bool,
    _modifiers: u32,
    canceled: &mut bool,
) {
    crate::game::we_brush_client::mouse_button_pre(game, button, pressed, canceled);
}
