// A message from the server: the selection it sent, which the client draws,
// and the schematics it asks the client to write or read.
fn we_net_from_server(
    game: &mut crate::game::Game<'_>,
    channel: &str,
    data: &[u8],
) {
    if channel == crate::game::we_core::SEL_CHANNEL {
        crate::game::we_render::set_selection(data);
    }
    if channel == crate::game::we_core::SCHEMATIC_CHANNEL {
        crate::game::we_schematics_client::from_server(game, data);
    }
}
