// The client's half of the brush: the clicks in the air.
//
// There is no block there to break and none to use the wand on, so the server
// hears nothing of such a click. The client tells it, and the server paints
// along the line of sight, as far as the brush's distance.

/// The left mouse button, as GLFW counts them.
const LEFT_BUTTON: i32 = 0;

/// How far the player reaches, in blocks: a block under the view that close is
/// a click the server hears itself.
const REACH: f32 = 5.0;

/// The left mouse button went down. When there is no block under the view,
/// the server hears nothing of the click: the client tells it, and the brush
/// paints along the line of sight.
pub fn mouse_button_pre(
    game: &mut crate::game::Game<'_>,
    button: i32,
    pressed: bool,
    canceled: &mut bool,
) {
    if *canceled || !pressed || button != LEFT_BUTTON {
        return;
    }
    // A screen is open: the click is in the chat or in a chest, not in the
    // world.
    if game.screen_hooks_open() {
        return;
    }
    // The wand in the player's hand.
    let Some((item, _, _)) = game.native_held() else {
        return;
    };
    if item != crate::game::we_glue::WAND {
        return;
    }
    // A block under the view: the server hears that click itself.
    if game.native_raycast(REACH).is_some() {
        return;
    }
    // The click was in the air: the server paints along the line of sight.
    let host: &mut dyn crate::mod_api::Host = game;
    let _ = host.net_send_to_server(
        crate::game::we_core::BRUSH_CHANNEL,
        &crate::game::we_core::brush_payload(false),
    );
}
