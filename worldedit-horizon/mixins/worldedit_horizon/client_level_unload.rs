// The client dropped its level: it left the server, or it changed the
// dimension. What the last server asked for is not allowed any more.
fn we_client_level_unload(_dimension: &str) {
    crate::game::we_schematics_client::level_changed();
}
