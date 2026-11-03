// A player's command on the server: WorldEdit's own, when it is one.
fn we_perform_command(
    game: &mut crate::game::Game<'_>,
    player: &crate::game::native_server::NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    if !*ran {
        crate::game::we_glue::perform(game, player, command, ran);
    }
}
