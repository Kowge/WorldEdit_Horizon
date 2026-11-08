// A button of the mod's page in Options > Mods was pressed.
fn we_mod_settings_click(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    button: usize,
) {
    crate::game::we_mods_ui::click(game, mod_id, button);
}
