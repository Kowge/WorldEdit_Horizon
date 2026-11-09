// The client made a level: another server, another dimension. What the last
// server asked for is not allowed any more.
fn we_client_level_load(_dimension: &str) {
    crate::game::we_schematics_client::level_changed();
}
