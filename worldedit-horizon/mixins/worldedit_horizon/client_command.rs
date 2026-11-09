// The player's own command, as the client sees it before the server does:
// it is what allows the server to ask for the player's files. The command is
// not claimed: the server runs it.
fn we_client_command(command: &str) {
    crate::game::we_schematics_client::player_command(command);
}
