// A message from a client: the wand's click, which the server checks.
fn we_net_from_client(
    game: &mut crate::game::Game<'_>,
    player: u128,
    channel: &str,
    data: &[u8],
) {
    if channel == crate::game::we_core::WAND_CHANNEL {
        crate::game::we_glue::wand_message(game, player, data);
    }
}
