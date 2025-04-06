use clap::{command, Parser};
use layout::{GlobalData, Layout};
use layouts::Layouts;
use wayland_client::Connection;

mod protocol;
mod layout;
mod layouts;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Set the number of views in the main are of the layout. Not implemented for all layouts.
    #[arg(short='n', long, default_value_t = 1)]
    main_count: u32,

    /// Set the initial ratio of the main area to total layout area. The ratio is clamped between
    /// 0.1 and 0.9. Not implemented for all layouts.
    #[arg(short='r', long, default_value_t = 0.5)]
    main_ratio: f32,

    /// Set the gap between windows.
    #[arg(short, long, default_value_t = 8)]
    gap: u32,

    /// Set the padding around the edge of the screen.
    #[arg(short, long, default_value_t = 8)]
    padding: u32,

    /// The default layout to use for tags.
    #[arg(short, long, value_enum, default_value_t = Layouts::Tile)]
    layout: Layouts,
}

fn main() {
    let cli = Args::parse();
    let global = GlobalData {
        gap: cli.gap,
        padding: cli.padding,
        ratio: cli.main_ratio,
        nmaster: cli.main_count,
        default_layout: cli.layout,
    };

    let conn = Connection::connect_to_env().unwrap();

    let display = conn.display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let _registry = display.get_registry(&qh, ());

    let mut layout = Layout::new(global);

    event_queue.roundtrip(&mut layout).unwrap();

    loop {
        event_queue.blocking_dispatch(&mut layout).unwrap();
    }
}
