// The mod's row in Options > Mods and the button on its page: the folder
// with the schematics. The client asks the mods it knows for their rows
// (`native_mods`), for the buttons of a page (`native_settings`) and tells
// them of a pressed button (`native_settings_click`); the helpers in
// mods_helper.rs, settings_helper.rs and settings_click_helper.rs call in
// here.

use crate::mod_api::Host;
use std::path::PathBuf;

/// The id the mod is listed under.
pub const ID: &str = "worldedit_horizon";

/// The row the mod has in Options > Mods.
pub fn list(mods: &mut Vec<crate::mod_api::NativeMod>) {
    mods.push(crate::mod_api::NativeMod {
        id: ID.to_owned(),
        title: format!("WorldEdit {}", crate::game::we_glue::VERSION),
        description: "Build with the /we commands. The schematics you save are in a folder of their own.".to_owned(),
        keys: Vec::new(),
    });
}

/// The buttons of the mod's page: the folder with the schematics.
pub fn settings(mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id == ID {
        buttons.push("Open the schematics folder".to_owned());
    }
}

/// A button of the mod's page was pressed: the first one opens the folder.
pub fn click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    if mod_id == ID && button == 0 {
        let dir = schematics_dir(game);
        let _ = std::fs::create_dir_all(&dir);
        crate::game::packs::open_folder(&dir);
    }
}

/// The folder with the schematics: the open world's own, where the server
/// keeps them, or the mod's own folder in the instance when no world is
/// open.
fn schematics_dir(game: &mut crate::game::Game<'_>) -> PathBuf {
    let host: &mut dyn Host = game;
    match host.world_dir() {
        Some(world) => crate::game::we_core::schematic_dir(&world),
        None => game.native_data_dir(ID),
    }
}
