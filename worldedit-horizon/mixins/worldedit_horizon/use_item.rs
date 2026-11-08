// The wand's right click in the air: the brush erases along the line of
// sight, even where there is no block under the view.
fn we_use_item(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    claimed: &mut bool,
) {
    if *claimed {
        return;
    }
    // Only the wand.
    if stack.item.to_string() != crate::game::we_glue::WAND {
        return;
    }
    let Some(player) = game.native_acting_player() else {
        return;
    };
    if crate::game::we_glue::brush_stroke(game, &player, true) {
        // The wand's own use does not run: the brush has the click.
        *claimed = true;
    }
}
