use std::collections::HashMap;

use wayland_client::{
    backend::ObjectId,
    protocol::{
        wl_output::{self, WlOutput},
        wl_registry::{self, WlRegistry},
    },
    Dispatch, Proxy,
};

use crate::{
    layouts::{Layouts, TagData},
    protocol::{
        river_layout_manager_v3::RiverLayoutManagerV3,
        river_layout_v3::{self, RiverLayoutV3},
    },
};

#[derive(Debug)]
struct Tags {
    tags: [TagData; 32],
}

impl Tags {
    fn from_global(global: &GlobalData) -> Self {
        Self {
            tags: [TagData::new(global.default_layout, global.nmaster, global.ratio); 32],
        }
    }
}

#[derive(Debug)]
pub struct GlobalData {
    pub gap: u32,
    pub padding: u32,
    pub ratio: f32,
    pub nmaster: u32,
    pub default_layout: Layouts,
}

#[derive(Debug)]
pub struct Layout {
    global: GlobalData,
    tags: u32,
    outputs: HashMap<ObjectId, Tags>,
    proxy: Option<RiverLayoutManagerV3>,
}

impl Layout {
    pub fn new(global: GlobalData) -> Self {
        Self {
            global,
            tags: u32::MAX,
            outputs: HashMap::new(),
            proxy: None,
        }
    }
}

impl Dispatch<WlRegistry, ()> for Layout {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: <WlRegistry as Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => match interface.as_str() {
                "wl_output" => {
                    registry.bind::<WlOutput, _, Self>(name, version, qhandle, ());
                }
                "river_layout_manager_v3" => {
                    state.proxy = Some(registry.bind::<RiverLayoutManagerV3, _, Self>(
                        name,
                        version,
                        qhandle,
                        (),
                    ));
                }
                _ => (),
            },
            _ => (),
        }
    }
}

impl Dispatch<WlOutput, ()> for Layout {
    fn event(
        state: &mut Self,
        output: &WlOutput,
        event: <WlOutput as Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_output::Event::Name { name } = event {
            println!("New output: {}", name);
            state
                .outputs
                .insert(output.id(), Tags::from_global(&state.global));
            state
                .proxy
                .as_ref()
                .expect("Compositor does not support river_layout_manager_v3.")
                .get_layout(output, "basic-layout".to_owned(), qhandle, output.id());
        }
    }
}

fn tag_num(tag: u32) -> Option<usize> {
    for i in 0..32 {
        if tag >> i & 1 == 1 {
            return Some(i);
        }
    }
    None
}

fn has_tag(tag: u32, index: usize) -> bool {
    tag >> index & 1 == 1
}

impl Dispatch<RiverLayoutV3, ObjectId> for Layout {
    fn event(
        state: &mut Self,
        proxy: &RiverLayoutV3,
        event: <RiverLayoutV3 as Proxy>::Event,
        output: &ObjectId,
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            river_layout_v3::Event::NamespaceInUse => {
                eprintln!("Namespace \"basic-layout\" is already in use")
            }
            river_layout_v3::Event::LayoutDemand {
                view_count,
                usable_width,
                usable_height,
                tags,
                serial,
            } => {
                if let Some(tag) =
                    tag_num(tags).and_then(|t| state.outputs.get(output).map(|o| &o.tags[t]))
                {
                    tag.layout(
                        view_count,
                        usable_width,
                        usable_height,
                        &state.global,
                        proxy,
                        serial,
                    );
                    proxy.commit(tag.layout.name(), serial);
                }
            }
            river_layout_v3::Event::UserCommand { command } => {
                let output = state.outputs.get_mut(output).unwrap();
                let (command, value) = command.split_once(' ').unwrap_or((&command, ""));
                match command {
                    "gap" => state.global.gap = u32::from_str_radix(value, 10).unwrap_or(8),
                    "padding" => state.global.padding = u32::from_str_radix(value, 10).unwrap_or(8),
                    "layout" => pertag(state.tags, |tag| {
                        output.tags[tag].layout = Layouts::form_string(value)
                    }),
                    "main-ratio" => pertag(state.tags, |tag| {
                        output.tags[tag].ratio =
                            relative_f32(value, output.tags[tag].ratio).clamp(0.1, 0.9)
                    }),
                    "main-count" => pertag(state.tags, |tag| {
                        output.tags[tag].nmaster =
                            relative_u32(value, output.tags[tag].nmaster).max(1)
                    }),
                    _ => (),
                }
            }
            river_layout_v3::Event::UserCommandTags { tags } => state.tags = tags,
        }
    }
}

fn relative_u32(s: &str, old: u32) -> u32 {
    if s.starts_with('+') {
        old + s[1..].parse::<u32>().unwrap_or(0)
    } else if s.starts_with('-') {
        old.saturating_sub(s[1..].parse::<u32>().unwrap_or(0))
    } else {
        s.parse::<u32>().unwrap_or(1)
    }
}

fn relative_f32(s: &str, old: f32) -> f32 {
    if s.starts_with('+') {
        old + s[1..].parse::<f32>().unwrap_or(0.)
    } else if s.starts_with('-') {
        old - s[1..].parse::<f32>().unwrap_or(0.)
    } else {
        s.parse::<f32>().unwrap_or(0.5)
    }
}

/// Perform an action for every tag
fn pertag(tags: u32, mut f: impl FnMut(usize)) {
    for tag in 0..32 {
        if has_tag(tags, tag) {
            f(tag);
        }
    }
}

impl Dispatch<RiverLayoutManagerV3, ()> for Layout {
    fn event(
        _: &mut Self,
        _: &RiverLayoutManagerV3,
        _: <RiverLayoutManagerV3 as Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
    }
}
