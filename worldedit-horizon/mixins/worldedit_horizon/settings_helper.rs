// The buttons of the mod's page in Options > Mods.
fn we_mod_settings(mod_id: &str, buttons: &mut Vec<String>) {
    crate::game::we_mods_ui::settings(mod_id, buttons);
}
