use clap::ValueEnum;

use crate::{layout::GlobalData, protocol::river_layout_v3::RiverLayoutV3};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Layouts {
    #[default]
    Tile,
    Column,
    Rows,
    CenteredMaster,
    Dwindle,
}

#[derive(Debug)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Layouts {
    pub fn name(&self) -> String {
        match self {
            Layouts::Tile => "Tile",
            Layouts::Column => "Column",
            Layouts::Rows => "Rows",
            Layouts::CenteredMaster => "Centered Master",
            Layouts::Dwindle => "Dwindle",
        }
        .to_owned()
    }

    pub fn form_string(str: impl AsRef<str>) -> Self {
        match str.as_ref() {
            "tile" => Self::Tile,
            "column" => Self::Column,
            "rows" => Self::Rows,
            "centered-master" => Self::CenteredMaster,
            "dwindle" => Self::Dwindle,
            _ => {
                eprintln!("Unknown layout: {}", str.as_ref());
                Self::Tile
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl Rect {
    fn split_vertial(&self, gap: u32, fraction: f32) -> (Rect, Rect) {
        let left = ((self.width - gap) as f32 * fraction) as u32;
        (
            rect(self.x, self.y, left, self.height),
            rect(
                self.x + left as i32 + gap as i32,
                self.y,
                self.width - left - gap,
                self.height,
            ),
        )
    }

    fn split_horizontal(&self, gap: u32, fraction: f32) -> (Rect, Rect) {
        let top = ((self.height - gap) as f32 * fraction) as u32;
        (
            rect(self.x, self.y, self.width, top),
            rect(
                self.x,
                self.y + top as i32 + gap as i32,
                self.width,
                self.height - top - gap,
            ),
        )
    }
}

fn rect(x: i32, y: i32, width: u32, height: u32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TagData {
    pub layout: Layouts,
    pub nmaster: u32,
    pub ratio: f32,
}

impl TagData {
    pub fn new(layout: Layouts, nmaster: u32, ratio: f32) -> Self {
        Self {
            layout,
            nmaster,
            ratio,
        }
    }

    pub fn layout(
        &self,
        view_count: u32,
        width: u32,
        height: u32,
        win: &GlobalData,
        proxy: &RiverLayoutV3,
        serial: u32,
    ) {
        let area = rect(
            win.padding as i32,
            win.padding as i32,
            width - win.padding * 2,
            height - win.padding * 2,
        );
        match self.layout {
            Layouts::Tile => tile(view_count, area, self, win, proxy, serial),
            Layouts::Column => column(view_count, area, win, proxy, serial),
            Layouts::Rows => rows(view_count, area, win, proxy, serial),
            Layouts::CenteredMaster => centered_master(view_count, area, self, win, proxy, serial),
            Layouts::Dwindle => dwindle(
                view_count,
                area,
                Direction::Left,
                self.ratio,
                win,
                proxy,
                serial,
            ),
        }
    }
}

fn tile(
    view_count: u32,
    area: Rect,
    tag: &TagData,
    win: &GlobalData,
    proxy: &RiverLayoutV3,
    serial: u32,
) {
    if view_count <= tag.nmaster {
        rows(view_count, area, win, proxy, serial)
    } else {
        let (left, right) = area.split_vertial(win.gap, tag.ratio);
        rows(tag.nmaster, left, win, proxy, serial);
        rows(view_count - tag.nmaster, right, win, proxy, serial);
    };
}

fn centered_master(
    view_count: u32,
    area: Rect,
    tag: &TagData,
    win: &GlobalData,
    proxy: &RiverLayoutV3,
    serial: u32,
) {
    if view_count <= tag.nmaster + 1 {
        tile(view_count, area, tag, win, proxy, serial)
    } else {
        let main_width = ((area.width as f32) * tag.ratio) as u32;
        let outside = view_count - tag.nmaster;
        let left = outside / 2;
        let right = outside - left;
        let outside_width = (area.width - main_width) / 2 - win.gap;
        rows(
            tag.nmaster,
            rect(
                area.x + outside_width as i32 + win.gap as i32,
                area.y,
                main_width,
                area.height,
            ),
            win,
            proxy,
            serial,
        );
        rows(
            left,
            rect(area.x, area.y, outside_width, area.height),
            win,
            proxy,
            serial,
        );
        rows(
            right,
            rect(
                area.x + area.width as i32 - outside_width as i32,
                area.y,
                outside_width,
                area.height,
            ),
            win,
            proxy,
            serial,
        );
    };
}

fn dwindle(
    view_count: u32,
    area: Rect,
    direction: Direction,
    fraction: f32,
    win: &GlobalData,
    proxy: &RiverLayoutV3,
    serial: u32,
) {
    if view_count == 1 {
        proxy.push_view_dimensions(area.x, area.y, area.width, area.height, serial)
    } else {
        match direction {
            Direction::Up => {
                let (top, bottom) = area.split_horizontal(win.gap, fraction);
                proxy.push_view_dimensions(top.x, top.y, top.width, top.height, serial);
                dwindle(
                    view_count - 1,
                    bottom,
                    Direction::Right,
                    0.5,
                    win,
                    proxy,
                    serial,
                );
            }
            Direction::Right => {
                let (left, right) = area.split_vertial(win.gap, fraction);
                proxy.push_view_dimensions(right.x, right.y, right.width, right.height, serial);
                dwindle(
                    view_count - 1,
                    left,
                    Direction::Down,
                    0.5,
                    win,
                    proxy,
                    serial,
                );
            }
            Direction::Down => {
                let (top, bottom) = area.split_horizontal(win.gap, fraction);
                proxy.push_view_dimensions(bottom.x, bottom.y, bottom.width, bottom.height, serial);
                dwindle(
                    view_count - 1,
                    top,
                    Direction::Left,
                    0.5,
                    win,
                    proxy,
                    serial,
                );
            }
            Direction::Left => {
                let (left, right) = area.split_vertial(win.gap, fraction);
                proxy.push_view_dimensions(left.x, left.y, left.width, left.height, serial);
                dwindle(
                    view_count - 1,
                    right,
                    Direction::Up,
                    0.5,
                    win,
                    proxy,
                    serial,
                );
            }
        }
    }
}

fn column(view_count: u32, area: Rect, win: &GlobalData, proxy: &RiverLayoutV3, serial: u32) {
    let window_width = ((area.width + win.gap) / view_count) - win.gap;
    for i in 0..view_count {
        proxy.push_view_dimensions(
            area.x + (i * (window_width + win.gap)) as i32,
            area.y,
            window_width,
            area.height,
            serial,
        )
    }
}

fn rows(view_count: u32, area: Rect, win: &GlobalData, proxy: &RiverLayoutV3, serial: u32) {
    let window_height = ((area.height + win.gap) / view_count) - win.gap;
    for i in 0..view_count {
        proxy.push_view_dimensions(
            area.x,
            area.y + (i * (window_height + win.gap)) as i32,
            area.width,
            window_height,
            serial,
        )
    }
}
