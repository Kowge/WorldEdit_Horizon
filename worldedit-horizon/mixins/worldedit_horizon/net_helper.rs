// A message from the server: the selection it sent, which the client draws.
fn we_net_from_server(channel: &str, data: &[u8]) {
    if channel == crate::game::we_core::SEL_CHANNEL {
        crate::game::we_render::set_selection(data);
    }
}
