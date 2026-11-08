// The selection's lines, over everything: the blocks do not hide them.
//
// The engine keeps two lists of lines: the ones it draws with the depth test
// (the blocks hide them) and the ones it draws over everything. The
// selection belongs to the second list, so the player sees it through the
// terrain.
fn we_gizmos(
    game: &crate::game::Game<'_>,
    gizmos: &mut crate::gizmos::Gizmos,
) {
    crate::game::we_render::draw_on_top(game.dim.id(), gizmos);
}
