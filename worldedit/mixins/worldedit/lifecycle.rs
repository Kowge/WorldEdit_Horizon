// The world closing: selections, clipboards and history go.
fn we_world_lifecycle(phase: crate::game::native_lifecycle::WorldPhase) {
    use crate::game::native_lifecycle::WorldPhase;
    if matches!(
        phase,
        WorldPhase::WorldCloseFinal | WorldPhase::WorldCloseFailed
    ) {
        crate::game::we_glue::closed();
    }
}
