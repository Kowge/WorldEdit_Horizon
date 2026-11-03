// WorldEdit's commands in every player's command tree.
fn we_register_commands(commands: &mut Vec<crate::game::native_server::NativeCommand>) {
    crate::game::we_glue::commands(commands);
}
