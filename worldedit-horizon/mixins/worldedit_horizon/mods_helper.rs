// The mod's row in Options > Mods.
fn we_list_mods(mods: &mut Vec<crate::mod_api::NativeMod>) {
    crate::game::we_mods_ui::list(mods);
}
