pub struct DebugSettings {
    pub draw_matrix_simple: bool,
    pub draw_matrix_physics: bool,
    pub draw_color_pression: bool,
}

impl DebugSettings {
    pub const fn new() -> DebugSettings {
        DebugSettings {
            draw_matrix_simple: false,
            draw_matrix_physics: false,
            draw_color_pression: false,
        }
    }
}