//use crate::game::entity::EntityColor;

pub fn from_hsv_to_rgb(hsv: (f32, u8, u8)) -> (u8, u8, u8) {
    //let h = hsv.0;
    let s = hsv.1;
    let v = hsv.2;

    //let h_float = (h as f32) / 255.0;
    //let h_360 = h_float * 360.0;
    let mut h_360 = hsv.0 % 360.0; if h_360 < 0.0 { h_360 = 360.0 - h_360.abs(); }
    let s_float = (s as f32) / 255.0;
    let v_float = (v as f32) / 255.0;

    let mut r = 0;
    let mut g = 0;
    let mut b = 0;

    if h_360 <= 60.0 || h_360 >= 300.0 { r = 255; }
    if h_360 >= 60.0 && h_360 <= 120.0 { r = (((1.0 - (h_360 - 60.0) / 60.0)) * 255.0) as u8; }
    
    if h_360 >= 0.0 && h_360 <= 60.0 { g = (((h_360 - 0.0) / 60.0) * 255.0) as u8; }
    if h_360 >= 60.0 && h_360 <= 180.0 { g = 255 as u8; }
    if h_360 >= 180.0 && h_360 <= 240.0 { g = (((1.0 - (h_360 - 180.0) / 60.0)) * 255.0) as u8; }
    
    if h_360 >= 120.0 && h_360 <= 180.0 { b = (((h_360 - 120.0) / 60.0) * 255.0) as u8; }
    if h_360 >= 180.0 && h_360 <= 300.0 { b = 255 as u8; }
    if h_360 >= 300.0 && h_360 <= 360.0 { b = (((1.0 - (h_360 - 300.0) / 60.0)) * 255.0) as u8; }

    if h_360 >= 240.0 && h_360 <= 300.0 { r = (((h_360 - 240.0) / 60.0) * 255.0) as u8; }

    let max = r.max(g).max(b);
    r = (r + ((max - r) as f32 * (1.0 - s_float)) as u8) as u8;
    g = (g + ((max - g) as f32 * (1.0 - s_float)) as u8) as u8;
    b = (b + ((max - b) as f32 * (1.0 - s_float)) as u8) as u8;

    r = ((r as f32) * v_float) as u8;
    g = ((g as f32) * v_float) as u8;
    b = ((b as f32) * v_float) as u8;
    return (r, g, b);
}

/*
const SMALL: u8 = 15;
const MEDIUM: u8 = 128;
const BIG: u8 = 240;

pub const DEFAULT_RED_COLORS: [[u8;4];5] = [
    [BIG, SMALL, SMALL, 255],
    [BIG, SMALL, MEDIUM, 255],
    [BIG, SMALL, BIG, 255],
    [BIG, MEDIUM, SMALL, 255],
    [BIG, BIG, SMALL, 255],
];

pub const DEFAULT_RED_ENTITY_COLORS: [EntityColor;5] = [
    EntityColor { center: DEFAULT_RED_COLORS[0], edge: DEFAULT_RED_COLORS[0] },
    EntityColor { center: DEFAULT_RED_COLORS[1], edge: DEFAULT_RED_COLORS[1] },
    EntityColor { center: DEFAULT_RED_COLORS[2], edge: DEFAULT_RED_COLORS[2] },
    EntityColor { center: DEFAULT_RED_COLORS[3], edge: DEFAULT_RED_COLORS[3] },
    EntityColor { center: DEFAULT_RED_COLORS[4], edge: DEFAULT_RED_COLORS[4] },
];
*/