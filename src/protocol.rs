use wayland_client;
use wayland_client::protocol::*;

pub mod __interfaces {
    use wayland_client::protocol::__interfaces::*;
    use wayland_client::backend as wayland_backend;

    wayland_scanner::generate_interfaces!("./protocols/river-layout-v3.xml");
}
use self::__interfaces::*;

wayland_scanner::generate_client_code!("./protocols/river-layout-v3.xml");
