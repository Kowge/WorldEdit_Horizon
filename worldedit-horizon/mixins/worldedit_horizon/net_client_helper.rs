// A message from a client: the wand's click, which the server checks, the
// brush's click in the air, and the schematics the client keeps.
fn we_net_from_client(
    game: &mut crate::game::Game<'_>,
    player: u128,
    channel: &str,
    data: &[u8],
) {
    if channel == crate::game::we_core::WAND_CHANNEL {
        crate::game::we_glue::wand_message(game, player, data);
    }
    if channel == crate::game::we_core::BRUSH_CHANNEL {
        crate::game::we_glue::brush_message(game, player, data);
    }
    if channel == crate::game::we_core::SCHEMATIC_CHANNEL {
        crate::game::we_glue::schematic_message(game, player, data);
    }
}
