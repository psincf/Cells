use crate::APP;
use crate::window::Window;

use crate::prelude::*;
use crate::new_timer_monothread;

use parking_lot::Mutex;
use euclid::default::{Point2D, Size2D};
use winit::event::Event;

use std::rc::Rc;
use std::cell::RefCell;


#[derive(Clone)]
pub enum Action {
    None,
    Editor,
    LeaveEditor,
    Options,
    GraphicsOptions,
    Map(Option<String>),
    Quit
}

#[derive(Clone)]
pub enum GUIState {
    Open(Rc<RefCell<Action>>),
    Closed,
}
#[derive(Clone)]
pub struct GUIState2 {
    pub size_width_editor: f32,
}

#[derive(Default)]
pub struct GuiDebugInfo {
    total_mass: bool,
    entity_index_choosen: i32,
}

pub struct GuiComponent {
    position: Point2D<i32>,
    size: Size2D<i32>,
}

pub struct Egui {
    pub context: egui::CtxRef,
    pub winit_backend: egui_binding::winit_backend::WinitBackend,
}

pub struct Imgui {
    pub context: imgui::Context,
    pub platform: imgui_winit_support::WinitPlatform,
    pub ui: Option<imgui::Ui<'static>>,
}

impl Drop for Imgui {
    fn drop(&mut self) {
        // Necessary because these field need to be dropped BEFORE context..
        // But because we unsafely change lifetime to 'static, we have to do it ourselves
        self.ui.take();
    }
}

pub struct GuiRendererData {
    pub imgui: Option<&'static imgui::DrawData>,
    pub egui: Option<(egui::Output, Vec<egui::paint::ClippedShape>)>,
}

pub enum GuiAction {
    Resize(u32, u32),
    Fullscreen,
    NotFullscreen,
}

pub struct Gui {
    pub egui: Mutex<Egui>,
    pub imgui: Mutex<Imgui>,
    pub list: Vec<GuiComponent>,
    pub state: Rc<RefCell<GUIState>>,
    pub state_2: GUIState2,
    pub events: Mutex<Vec<winit::event::Event<'static, ()>>>,
    pub renderer_data: Mutex<GuiRendererData>,
    pub debug_info: RefCell<GuiDebugInfo>, //TODO: bad
    actions: RefCell<Vec<GuiAction>>,
}

impl Drop for Gui {
    fn drop(&mut self) {
        // Necessary because these field need to be dropped BEFORE context..
        // But because we unsafely change lifetime to 'static, we have to do it ourselves
        self.renderer_data.lock().imgui.take();
    }
}

impl Gui {
    pub fn new(window: &Window) -> Gui {
        let mut context = imgui::Context::create();
        context.set_ini_filename(None);
        let style = context.style_mut();
        style.alpha = 1.0;
        style.window_rounding = 0.0;
        style.window_padding = [0.0, 0.0];
        style.window_border_size = 0.0;
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut context);
        platform.attach_window(context.io_mut(), &window.window, imgui_winit_support::HiDpiMode::Default);

        let draw_data = None;
        let ui = None;

        let imgui = Mutex::new(Imgui {
            context,
            platform,
            ui,
        });
        
        let list = Vec::new();
        let state = Rc::new(RefCell::new(GUIState::Closed));
        let state_2 = GUIState2 {
            size_width_editor: 400.0,
        };
        let debug_info = RefCell::new(GuiDebugInfo::default());

        let egui_context = egui::CtxRef::default();

        let egui = Mutex::new(Egui {
            context: egui_context,
            winit_backend: egui_binding::winit_backend::WinitBackend::new(),
        });

        let renderer_data = Mutex::new(GuiRendererData {
            imgui: draw_data,
            egui: None,
        });

        let mut default_fonts = egui::FontDefinitions::default();
        default_fonts.family_and_size.insert(egui::TextStyle::Button, (egui::FontFamily::Monospace, 13.0));
        default_fonts.family_and_size.insert(egui::TextStyle::Body, (egui::FontFamily::Monospace, 13.0));
        default_fonts.family_and_size.insert(egui::TextStyle::Heading, (egui::FontFamily::Monospace, 13.0));
        egui.lock().context.set_fonts(default_fonts);

        Gui {
            egui,
            imgui,
            list,
            state,
            state_2,
            events: Mutex::new(Vec::new()),
            renderer_data,
            debug_info,
            actions: RefCell::new(Vec::new()),
        }
    }

    pub fn update(&self, game: &Game) { // TODO: refactor this
        new_timer_monothread!(_t, "GUI");
        self.handle_events_2();
        let mut egui = self.egui.lock();
        let egui_input = egui.winit_backend.take();
        egui.context.begin_frame(egui_input);
        //self.egui_test(&mut egui.context);

        let size = *APP.get().window.events.resize_events.size.lock().unwrap();
        let size = (size.to_f32() / APP.get().window.window.scale_factor() as f32).to_i32();
        if size.width == 0 && size.height == 0 { return }

        let mut imgui = self.imgui.lock();
        imgui.ui = None;

        imgui.platform.prepare_frame(unsafe { &mut *(&imgui.context as *const _ as *mut imgui::Context) }.io_mut(), &crate::APP.get().window.window).unwrap();
        
        let ui = unsafe { &mut *(&mut imgui.context as *mut imgui::Context) }.frame();

        #[cfg(all(feature = "serialize"))]
        self.editor(&ui);

        #[cfg(not(feature = "shipping"))]
        self.update_debug_menu(&ui, size, game);
        self.update_ingame_menu(&ui, size);

        imgui.ui = Some(ui);
        
        if let Some(ui) = imgui.ui.take() {
            update_hovered(&ui);
            self.renderer_data.lock().imgui = Some(ui.render());
        }
        let end_frame = egui.context.end_frame();
        self.renderer_data.lock().egui = Some(end_frame);

        drop(imgui);
        drop(egui);
        for action in self.actions.borrow_mut().iter() {
            let parker = std::sync::Arc::new(crossbeam_utils::sync::Parker::new());
            match action {
                GuiAction::Resize(width, height) => {
                    for _ in 0..2 {
                        let parker = std::sync::Arc::new(crossbeam_utils::sync::Parker::new());
                        let actual_size = crate::APP.get().window.events.resize_events.size.lock().unwrap().clone();
                        if actual_size.width == *width as i32 && actual_size.height == *height as i32 {
                            continue
                        }
                        *crate::APP.get().window.events.resize_events.fullscreen.lock().unwrap() = false;
                        let _lock = crate::APP.get().renderer.lock_draw.lock();
                        crate::APP.get().window.events.parker.lock().unwrap().push(parker.clone());

                        crate::APP.get().window.window.set_maximized(false);
                        crate::APP.get().window.window.set_decorations(true);
                        crate::APP.get().window.window.set_inner_size(winit::dpi::PhysicalSize::new(*width, *height));
                        self.renderer_data.lock().imgui = None;

                        parker.unparker().unpark();

                        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
                        crate::APP.get().window.events.barrier.lock().unwrap().push(barrier.clone());
                        barrier.wait();
                    }
                }
                GuiAction::Fullscreen => {
                    if *crate::APP.get().window.events.resize_events.fullscreen.lock().unwrap() == true { continue }
                    *crate::APP.get().window.events.resize_events.fullscreen.lock().unwrap() = true;
                    let _lock = crate::APP.get().renderer.lock_draw.lock();
                    crate::APP.get().window.events.parker.lock().unwrap().push(parker.clone());

                    crate::APP.get().window.window.set_decorations(false);
                    crate::APP.get().window.window.set_maximized(true);
                    self.renderer_data.lock().imgui = None;

                    parker.unparker().unpark();

                    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
                    crate::APP.get().window.events.barrier.lock().unwrap().push(barrier.clone());
                    barrier.wait();
                }
                GuiAction::NotFullscreen => {
                    if *crate::APP.get().window.events.resize_events.fullscreen.lock().unwrap() == false { continue }
                    *crate::APP.get().window.events.resize_events.fullscreen.lock().unwrap() = false;
                    let _lock = crate::APP.get().renderer.lock_draw.lock();
                    crate::APP.get().window.events.parker.lock().unwrap().push(parker.clone());

                    crate::APP.get().window.window.set_maximized(false);
                    crate::APP.get().window.window.set_decorations(true);
                    self.renderer_data.lock().imgui = None;

                    parker.unparker().unpark();

                    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
                    crate::APP.get().window.events.barrier.lock().unwrap().push(barrier.clone());
                    barrier.wait();
                }
            }
        }
        self.actions.borrow_mut().clear();
    }

    #[cfg(not(feature = "shipping"))]
    pub fn update_debug_menu(&self, ui: &imgui::Ui, size_window: Size2D<i32>, game: &Game) {
        new_timer_monothread!(_t, "imgui_debug");
        if *crate::DEBUG.get() == false { return }
        let mut debug_info = self.debug_info.borrow_mut();
        let debug_settings = crate::DEBUG_SETTINGS.get_mut();
        
        let benchmark_timer = game.benchmark.get_average_in_order();
        let entities_len = game.entities.len();

        let window = imgui::Window::new(imgui::im_str!("Debug window 1"));
        let color = ui.push_style_color(imgui::StyleColor::WindowBg, [0.05, 0.05, 0.05, 1.0]);
        window
            .position([0.0, 0.0], imgui::Condition::FirstUseEver)
            .title_bar(true)
            .resizable(false)
            .movable(false)
            .bg_alpha(0.5)
            .size([350.0, 600.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                ui.text(imgui::im_str!("Benchmark"));
                ui.spacing();
                for timer in benchmark_timer {
                    let mut time = (timer.1.duration.as_secs_f32() * 1_000.0).to_string(); time.truncate(5);
                    let space = String::from("  "); 
                    let space = space.repeat(timer.1.level + 1);
                    ui.text(space + &timer.0 + " " + &time + " ms");
                }
                ui.spacing();
                ui.text("num_threads: ".to_owned() + &game.threadpool.num_threads().to_string());
                ui.spacing();
                ui.text("entities amount: ".to_owned() + &entities_len.to_string());
                ui.spacing();

                let speed = ((game.step.duration_vec.first().unwrap().as_secs_f32()) / game.step.last_duration.as_secs_f32()) * 100.0;
                let speed = speed.trunc() + (speed.fract() * 100.0).round() / 100.0;
                ui.text("speed: ".to_owned() + &(speed).to_string() + "%" );
                
                ui.checkbox(imgui::im_str!("total_mass"), &mut debug_info.total_mass );
                if debug_info.total_mass {
                    ui.same_line(100.0);
                    ui.text(": ".to_owned() + &game.total_mass().to_string());
                }

                let mut full_speed = game.step.full_speed;
                ui.checkbox(imgui::im_str!("full_speed"), &mut full_speed );
                crate::APP.get_mut().game.step.full_speed = full_speed;
                
                let update_time = game.step.last_duration.as_millis() as f32;
                ui.text("tick_duration: ".to_owned() + &update_time.to_string() + " ms");
                /*
                ui.input_float(imgui::im_str!("update_tick_fast"), &mut update_time).build();
                *crate::APP.get_mut().game.step.duration_vec.first_mut().unwrap() = std::time::Duration::from_micros((update_time * 1_000.0) as u64);
                */

                
                ui.separator();
                ui.spacing();
                ui.text(imgui::im_str!("Benchmark renderer"));
                ui.spacing();
                for timer in crate::APP.get().renderer.benchmark.get_in_order() {
                    let mut time = (timer.1.duration.as_secs_f32() * 1_000.0).to_string(); time.truncate(5);
                    let space = String::from("  "); 
                    let space = space.repeat(timer.1.level + 1);
                    ui.text(space + &timer.0 + " " + time.as_str() + " ms");
                }
            });
        {
            let window = imgui::Window::new(imgui::im_str!("Debug window 2"));
            window
                .position([size_window.width as f32 - 350.0, 0.0], imgui::Condition::Always)
                .title_bar(true)
                .resizable(false)
                .movable(false)
                .bg_alpha(0.5)
                .size([350.0, 600.0], imgui::Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text("Map");
                    let mut size_map_width = game.map.size.width;
                    let mut size_map_height = game.map.size.height;
                    ui.spacing();
                    if ui.input_int(imgui::im_str!("size_map_x"), &mut size_map_width).enter_returns_true(true).build() {
                        let ratio = size_map_width as f32 / crate::APP.get().game.map.size.width as f32;
                        crate::APP.get_mut().game.map.size = Size2D::new(size_map_width, size_map_height);
                        for entity in crate::APP.get_mut().game.entities.core.iter_mut() {
                            let entity_position = &mut crate::APP.get_mut().game.entities.position[entity.index.main];
                            entity_position.x = crate::utils::i32_mul_f32_2(entity_position.x, ratio);

                            entity_position.x = entity_position.x.min(crate::APP.get().game.map.max().width);
                            crate::APP.get_mut().game.map.delete_entity(&game.entities, entity);
                        }
                        crate::APP.get_mut().game.map = crate::game::map::Map::new(crate::game::MapInfo {
                            size: Size2D::new(size_map_width, size_map_height)
                        });
                        for entity in crate::APP.get_mut().game.entities.core.iter_mut() {
                            let entity_position = &mut crate::APP.get_mut().game.entities.position[entity.index.main];
                            *entity_position = entity_position.min(crate::APP.get().game.map.max().to_vector().to_point());
                            crate::APP.get_mut().game.map.add_entity(&game.entities, entity);
                            crate::APP.get_mut().game.entities.update_drawable(entity.index.main);
                        }
                    }

                    if ui.input_int(imgui::im_str!("size_map_y"), &mut size_map_height).enter_returns_true(true).build() {
                        let ratio = size_map_height as f32 / crate::APP.get().game.map.size.height as f32;
                        crate::APP.get_mut().game.map.size = Size2D::new(size_map_width, size_map_height);
                        for entity in crate::APP.get_mut().game.entities.core.iter_mut() {
                            let entity_position = &mut crate::APP.get_mut().game.entities.position[entity.index.main];
                            entity_position.y = crate::utils::i32_mul_f32_2(entity_position.y, ratio);

                            entity_position.y = entity_position.y.min(crate::APP.get().game.map.max().height);
                            crate::APP.get_mut().game.map.delete_entity(&game.entities, entity);
                        }
                        crate::APP.get_mut().game.map = crate::game::map::Map::new(crate::game::MapInfo {
                            size: Size2D::new(size_map_width, size_map_height)
                        });
                        for entity in crate::APP.get_mut().game.entities.core.iter_mut() {
                            let entity_position = &mut crate::APP.get_mut().game.entities.position[entity.index.main];
                            *entity_position = entity_position.min(crate::APP.get().game.map.max().to_vector().to_point());
                            crate::APP.get_mut().game.map.add_entity(&game.entities, entity);
                            crate::APP.get_mut().game.entities.update_drawable(entity.index.main);
                        }
                    }

                    ui.text("MatrixSimple");
                    let mut size_field_simple = game.map.matrix_simple.size_field;
                    ui.spacing();
                    ui.input_int(imgui::im_str!("size_field_simple"), &mut size_field_simple).enter_returns_true(true).build();
                    crate::APP.get_mut().game.map.matrix_simple.rebuild(&mut crate::APP.get_mut().game, crate::APP.get_mut().game.map.size * crate::APP.get_mut().game.map.size_field, size_field_simple);
                    
                    
                    ui.spacing();
                    ui.text("MatrixPhysics");
                    let mut size_field_initial = game.map.matrix_physics.size_field_initial();
                    let mut size_field_ratio = game.map.matrix_physics.size_field_ratio();
                    let mut count = game.map.matrix_physics.count();
                    ui.spacing();
                    ui.input_int(imgui::im_str!("size_field_initial"), &mut size_field_initial).enter_returns_true(true).build();
                    ui.input_int(imgui::im_str!("size_field_ratio"), &mut size_field_ratio).enter_returns_true(true).build();
                    ui.input_int(imgui::im_str!("count"), &mut count).enter_returns_true(true).build();
                    crate::APP.get_mut().game.map.matrix_physics.rebuild(&mut crate::APP.get_mut().game, crate::APP.get_mut().game.map.size * crate::APP.get_mut().game.map.size_field, size_field_initial, size_field_ratio, count);
                    
                    ui.text("Debug settings");
                    ui.spacing();
                    ui.checkbox(imgui::im_str!("draw_matrix_simple"), &mut debug_settings.draw_matrix_simple );
                    ui.checkbox(imgui::im_str!("draw_matrix_physics"), &mut debug_settings.draw_matrix_physics );
                    ui.checkbox(imgui::im_str!("draw_color_pression"), &mut debug_settings.draw_color_pression );
                }
            );
            
            
        }
        color.pop(&ui);

        
    }

    fn update_ingame_menu(&self, ui: &imgui::Ui, size_window: Size2D<i32>) {
        new_timer_monothread!(_t, "imgui_menu");
        let game = &mut APP.get_mut().game;
        let state = self.state.borrow().clone();
        match state {
            GUIState::Closed => {
                let window = imgui::Window::new(imgui::im_str!("Hello world2"));
                window
                    .position([size_window.width as f32 / 2.0 - 10.0, 0.0], imgui::Condition::Always)
                    .title_bar(false)
                    .resizable(false)
                    .movable(false)
                    .draw_background(false)
                    //.focused(true)
                    .size([40.0, 40.0], imgui::Condition::Always)
                    .build(&ui, || {
                        if ui.arrow_button(imgui::im_str!("arrow_button_menu"), imgui::Direction::Down) {
                            *self.state.borrow_mut() = GUIState::Open(Rc::new(RefCell::new(Action::None)));
                        }
                        update_hovered(ui);
                    });
            },
            GUIState::Open(action) => {
                let window = imgui::Window::new(imgui::im_str!("Hello world2"));
                window
                    .position([size_window.width as f32 / 2.0 - 125.0, 0.0], imgui::Condition::Always)
                    .title_bar(false)
                    .resizable(false)
                    .movable(false)
                    .draw_background(true)
                    .size([250.0, 350.0], imgui::Condition::Always)
                    .build(&ui, || {
                        ui.indent_by(115.0);
                        if ui.arrow_button(imgui::im_str!("arrow_button_menu"), imgui::Direction::Down) {
                            *self.state.borrow_mut() = GUIState::Closed;
                        }
                        ui.unindent_by(5.0);
                        ui.text("MENU");
                        ui.unindent_by(110.0);
                        ui.separator();
                        ui.spacing();
                        
                        ui.indent_by(50.0);
                        if ui.button(imgui::im_str!("Options"), [150.0, 50.0]) { *action.borrow_mut() = Action::Options }
                        ui.unindent_by(50.0);

                        ui.spacing();
                        
                        ui.indent_by(50.0);
                        match game.state {
                            crate::game::GameState::Playing => { if ui.button(imgui::im_str!("Editor"), [150.0, 50.0]) { *action.borrow_mut() = Action::Editor; game.editor_state.reinit() } },
                            crate::game::GameState::Editor => { if ui.button(imgui::im_str!("Leave Editor"), [150.0, 50.0]) { *action.borrow_mut() = Action::LeaveEditor; game.editor_state.reinit() } },
                            _ => {}
                        }
                        ui.unindent_by(50.0);

                        ui.spacing();
                        
                        ui.indent_by(50.0);
                        if ui.button(imgui::im_str!("Graphics"), [150.0, 50.0]) { *action.borrow_mut() = Action::GraphicsOptions }
                        ui.unindent_by(50.0);

                        ui.spacing();

                        ui.indent_by(50.0);
                        if ui.button(imgui::im_str!("Maps"), [150.0, 50.0]) { *action.borrow_mut() = Action::Map(None) }
                        ui.unindent_by(50.0);

                        ui.spacing();

                        ui.indent_by(50.0);
                        if ui.button(imgui::im_str!("Quit"), [150.0, 50.0]) { *action.borrow_mut() = Action::Quit };
                        ui.unindent_by(50.0);

                        let mut graphics_settings = crate::APP.get().renderer.settings.clone();
                        let action_choosen = action.borrow().clone();
                        match action_choosen {
                            Action::Editor => {
                                game.state = crate::game::GameState::Editor;
                            }

                            Action::LeaveEditor => {
                                game.state = crate::game::GameState::Playing;
                            }

                            Action::Options => {
                                let mut thread_count = game.threadpool.num_threads().max(1) as i32; //TODO: Better singlethread handling
                                ui.open_popup(imgui::im_str!("Options"));
                                ui.popup_modal(imgui::im_str!("Options")).always_auto_resize(true).resizable(false).movable(false).build(|| {
                                    ui.separator();
                                    ui.spacing();
                                    ui.indent_by(20.0);

                                    ui.text("Compute threads count: ");
                                    ui.same_line_with_spacing(0.0, 0.0);
                                    ui.set_next_item_width(80.0);
                                    ui.input_int(imgui::im_str!(" "), &mut thread_count).step(1).enter_returns_true(true).build();
                                    thread_count = thread_count.max(1);
                                    if thread_count == 1 { thread_count = 0; }
                                    game.threadpool.set_threads(thread_count as usize);
                                    
                                    ui.checkbox(imgui::im_str!("Draw matrix"), &mut graphics_settings.draw_matrix);
                                    crate::APP.get_mut().renderer.settings.draw_matrix = graphics_settings.draw_matrix;
                                    
                                    let width_editor = unsafe { &mut *(&self.state_2.size_width_editor as *const _ as *mut GUIState2) };
                                    imgui::Drag::new(imgui::im_str!("Size Editor")).range(100.0..=(crate::APP.get().window.events.resize_events.size.lock().unwrap().width as f32 / 2.0 - 150.0)).build(ui, &mut width_editor.size_width_editor);
                                    //ui.input_float(imgui::im_str!("Size Editor"), &mut width_editor.size_width_editor).build();
                                    
                                    ui.unindent_by(20.0);
                                    if ui.button(imgui::im_str!("Return"), [300.0, 20.0]) { *action.borrow_mut() = Action::None };
                                });
                            }

                            Action::GraphicsOptions => {
                                ui.open_popup(imgui::im_str!("Graphics Option"));
                                ui.popup_modal(imgui::im_str!("Graphics Option")).always_auto_resize(true).resizable(false).movable(false).build(|| {
                                    ui.separator();
                                    ui.spacing();
                                    ui.indent_by(20.0);

                                    /*
                                    let renderer_mode = if !graphics_settings.smooth { imgui::im_str!("Basic") } else { imgui::im_str!("Smooth") };

                                    imgui::ComboBox::new(imgui::im_str!("Renderer mode")).preview_value(renderer_mode).build(ui, || {
                                        let style = ui.push_style_var(imgui::StyleVar::ItemSpacing([0.0, 1.0]));
                                        if imgui::Selectable::new(imgui::im_str!("Basic")).build(ui) { crate::APP.get_mut().renderer.settings.smooth = false };
                                        if imgui::Selectable::new(imgui::im_str!("Smooth")).build(ui) { crate::APP.get_mut().renderer.settings.smooth = true };
                                        style.pop(ui);
                                    });
                                    */

                                    #[cfg(not(feature = "shipping"))]
                                    (|| {
                                        let mut fullscreen = *crate::APP.get().window.events.resize_events.fullscreen.lock().unwrap();
                                        ui.checkbox(imgui::im_str!("Fullscreen"), &mut fullscreen);
                                        if fullscreen {
                                            self.actions.borrow_mut().push(GuiAction::Fullscreen);
                                        } else {
                                            self.actions.borrow_mut().push(GuiAction::NotFullscreen);
                                        }

                                        let all_resolutions = [
                                            (640, 480),
                                            (800, 600),
                                            (1024, 768),
                                            (1280, 720),
                                            (1366, 768),
                                            (1600, 1200),
                                            (1920, 1080),
                                        ];
                                        
                                        let resolutions_compatible: Vec<&(u32, u32)> = all_resolutions
                                            .iter()
                                            .filter( |&r| {
                                                let window_size = crate::APP.get().window.window.primary_monitor().unwrap().size();
                                                r.0 <= window_size.width && r.1 <= window_size.height
                                            })
                                            .collect();

                                        let actual_resolution = crate::APP.get().window.window.inner_size();
                                        let actual_resolution_string = String::from(actual_resolution.width.to_string() + " x " + &actual_resolution.height.to_string());
                                        
                                        imgui::ComboBox::new(imgui::im_str!("Resolution")).preview_value(&imgui::ImString::from(actual_resolution_string)).build(ui, || {
                                            let style = ui.push_style_var(imgui::StyleVar::ItemSpacing([0.0, 1.0]));
                                            for &res in resolutions_compatible.iter() {
                                                let res_string = String::from(res.0.to_string() + " x " + &res.1.to_string());
                                                if imgui::Selectable::new(&imgui::ImString::from(res_string)).build(ui) {
                                                    self.actions.borrow_mut().push(GuiAction::Resize(res.0, res.1));
                                                };
                                            }
                                            style.pop(ui);
                                        });

                                        let v_sync = match graphics_settings.v_sync {
                                            true => imgui::im_str!("V_Sync"),
                                            false => imgui::im_str!("Mailbox"),
                                        };

                                        imgui::ComboBox::new(imgui::im_str!("Sync")).preview_value(v_sync).build(ui, || {
                                            let style = ui.push_style_var(imgui::StyleVar::ItemSpacing([0.0, 1.0]));
                                            if imgui::Selectable::new(imgui::im_str!("V_Sync")).build(ui) { crate::APP.get_mut().renderer.settings.v_sync = true };
                                            if imgui::Selectable::new(imgui::im_str!("Mailbox")).build(ui) { crate::APP.get_mut().renderer.settings.v_sync = false };
                                            style.pop(ui);
                                        });
                                    })();

                                    let ssaa_string = match graphics_settings.ssaa {
                                        1 => imgui::im_str!("SSAA Disabled"),
                                        2 => imgui::im_str!("SSAA X2"),
                                        4 => imgui::im_str!("SSAA X4"),
                                        _ => panic!("ssaa gui")
                                    };

                                    imgui::ComboBox::new(imgui::im_str!("SSAA")).preview_value(ssaa_string).build(ui, || {
                                        let style = ui.push_style_var(imgui::StyleVar::ItemSpacing([0.0, 1.0]));
                                        if imgui::Selectable::new(imgui::im_str!("SSAA Disabled")).build(ui) { crate::APP.get_mut().renderer.settings.ssaa = 1 };
                                        if imgui::Selectable::new(imgui::im_str!("SSAA X2")).build(ui) { crate::APP.get_mut().renderer.settings.ssaa = 2 };
                                        if imgui::Selectable::new(imgui::im_str!("SSAA X4")).build(ui) { crate::APP.get_mut().renderer.settings.ssaa = 4 };
                                        style.pop(ui);
                                    });
                                    
                                    ui.unindent_by(20.0);
                                    if ui.button(imgui::im_str!("Return"), [300.0, 20.0]) { *action.borrow_mut() = Action::None };
                                });
                            }
                            Action::Map(map_id) => {
                                ui.open_popup(imgui::im_str!("Maps"));
                                ui.popup_modal(imgui::im_str!("Maps")).resizable(false).movable(false).build( || {
                                    ui.separator();
                                    ui.spacing();
                                    ui.spacing();

                                    ui.indent_by(20.0);
                                    let maps = crate::game::map::premade::PREMADE_MAPS;
                                    for map in maps.iter() {
                                        if ui.button(imgui::ImString::new(*map).as_ref(), [150.0, 20.0]) { *action.borrow_mut() = Action::Map(Some(map.to_string())) };
                                    }
                                    ui.unindent_by(20.0);

                                    ui.spacing();
                                    ui.spacing();
                                    if ui.button(imgui::im_str!("Return"), [190.0, 20.0]) { *action.borrow_mut() = Action::None };

                                    if let Some(map_chosed) = map_id {
                                        ui.open_popup(imgui::im_str!("Confirmation map"));
                                        ui.popup(imgui::im_str!("Confirmation map"), || {
                                            if ui.button(imgui::im_str!("Confirmation"), [150.0, 20.0]) {
                                                crate::game::map::premade::create_premade_map(game, &map_chosed);
                                                game.step.changed_map = true;
                                                *self.state.borrow_mut() = GUIState::Closed;
                                            }
                                            if ui.button(imgui::im_str!("Return"), [150.0, 20.0]) { *action.borrow_mut() = Action::Map(None) };
                                            if !ui.is_any_item_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Left) { *action.borrow_mut() = Action::Map(None) }
                                        });
                                    }
                                });
                            }
                            
                            Action::Quit => {
                                ui.open_popup(imgui::im_str!("Confirmation quit"));
                                ui.popup(imgui::im_str!("Confirmation quit"), || {
                                    if ui.button(imgui::im_str!("Confirmation"), [150.0, 20.0]) {
                                        crate::APP.get().app_runner_infos.stop();
                                        crate::APP.get().window.event_loop_proxy.as_ref().unwrap().send_event(()).unwrap();
                                    }
                                    if ui.button(imgui::im_str!("Cancel"), [150.0, 20.0]) { *action.borrow_mut() = Action::None };
                                });
                                if !ui.is_any_item_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Left) { *action.borrow_mut() = Action::None }
                            }

                            _ => {}
                        }
                        update_hovered(ui);
                    })
                ;
            }
        }
    }

    pub fn handle_events(&self, _window: &winit::window::Window, event: Event<()>) {
        if event == winit::event::Event::MainEventsCleared { return }
        if event == winit::event::Event::RedrawEventsCleared { return }
        if let winit::event::Event::NewEvents(_) = event { return }
        
        self.events.lock().push( unsafe { std::mem::transmute(event) } );
        //self.handle_events_2();
        /*
        let mut imgui = self.imgui.lock();
        let io = unsafe { &mut *(imgui.context.io_mut() as *mut imgui::Io) };
        imgui.platform.handle_event(io, window, &event);
        drop(imgui);

        let mut egui = self.egui.lock();
        egui.winit_backend.update(window, &event, &crate::APP.get().window.events.input_events);
        */
    }

    pub fn handle_events_2(&self) {
        let window = &APP.get().window.window;
        let mut events = self.events.lock();

        for event in events.clone() {
            let mut imgui = self.imgui.lock();
            let io = unsafe { &mut *(imgui.context.io_mut() as *mut imgui::Io) };
            imgui.platform.handle_event(io, window, &event);
            drop(imgui);

            let mut _egui = self.egui.lock();
            //egui.winit_backend.update(window, &event);
        }

        events.clear();
    }

    pub fn is_on_gui(&self, position: Point2D<i32>) -> bool {
        let mut result = false;
        for component in self.list.iter() {
            if position.x >= component.position.x && position.x < component.position.x + component.size.width {
                if position.y >= component.position.y && position.y < component.position.y + component.size.height {
                    result = true;
                }
            }
        }
        return result;
    }

    #[cfg(all(feature = "serialize"))]
    fn editor(&self, ui: &imgui::Ui) {
        new_timer_monothread!(_t, "imgui_2");

        let mut debug_info = self.debug_info.borrow_mut();
        let game = &mut crate::APP.get_mut().game;
        let game_bis = &mut crate::APP.get_mut().game;
        if game.state != crate::game::GameState::Editor { return }

        let window = imgui::Window::new(imgui::im_str!("Inspection"));
        window
            .position([0.0, 0.0], imgui::Condition::FirstUseEver)
            .title_bar(true)
            .resizable(false)
            .movable(false)
            .horizontal_scrollbar(true)
            .bg_alpha(1.0)
            .size([self.state_2.size_width_editor, APP.get().window.window.inner_size().height as f32], imgui::Condition::Always)
            .build(ui, || {
                /*
                let local_entities = &game.players[game.settings.local_player].entities;
                let mut local_entities_info = local_entities.to_value();
                reflect::imgui_impl::inspect(ui, &mut local_entities_info, None, None);
                */

                imgui::TabBar::new(&imgui::ImString::new("Tabbar")).build(ui, || {
                    imgui::TabItem::new(&imgui::ImString::new("New")).build(ui, || {
                        game.editor_state.tab = 0;
                        ui.checkbox(imgui::im_str!("Create on click"), &mut game.editor_state.new_entity_on_click);
                        use reflect::Reflect;
                        let settings = settings_editor();
                        let mut entity_info = game.editor_state.new_entity.to_value();
                        reflect::imgui_impl::inspect(ui, &mut entity_info, None, Some(settings));
                        let mut new_entity_info: EntityInfo = serde::de::Deserialize::deserialize(entity_info).unwrap();
                        new_entity_info.validate(game_bis);

                        game.editor_state.new_entity = new_entity_info;
                    });
                    imgui::TabItem::new(&imgui::ImString::new("Edit")).build(ui, || {
                        game.editor_state.tab = 1;
                        ui.input_int(imgui::im_str!("Select entity"), &mut debug_info.entity_index_choosen).build();
                        ui.spacing();
                        if let Some(entity) = game.entities.core.get_mut(debug_info.entity_index_choosen as usize) {
                            game.editor_state.entity_selected = Some(debug_info.entity_index_choosen as usize);
                            use reflect::Reflect;
                            //let entity_info = EntityInfo::from_entity(game_bis, entity.index.main);
                            let entity_info = crate::game::entity::EntityInfo::from_entity(game_bis, entity.index.main);
                            
                            let settings = settings_editor();
        
                            let mut entity_info = entity_info.to_value();
                            reflect::imgui_impl::inspect(ui, &mut entity_info, None, Some(settings));
                            let mut new_entity_info: EntityInfo = serde::de::Deserialize::deserialize(entity_info).unwrap();
                            //game_bis.entities.update_from_factory(entity.index.main, new_entity_info);
                            new_entity_info.validate(game_bis);
                            game_bis.entities.update(entity.index.main, new_entity_info);
                        } else {
                            game.editor_state.entity_selected = None;
                        }
                    });
                    imgui::TabItem::new(&imgui::ImString::new("Selection")).build(ui, || {
                        game.editor_state.tab = 2;
                        let selected = game.editor_state.selected.clone().unwrap_or_default();
                        if !selected.is_empty() {
                            ui.align_text_to_frame_padding();
                            imgui::TreeNode::new(&imgui::ImString::new("Selected".to_string())).build(ui, || {
                                game.editor_state.entity_hovered = None;
                                ui.indent();
                                for entity in selected.iter() {
                                    if ui.button(&imgui::ImString::new(entity.to_string()), [150.0, 19.0]) {
                                        debug_info.entity_index_choosen = *entity as i32;
                                        game.editor_state.entity_selected = Some(debug_info.entity_index_choosen as usize);
                                    }
                                    if ui.is_item_hovered() {
                                        game.editor_state.entity_hovered = Some(*entity);
                                    }
                                }
                                ui.unindent();
                            });
                        }
                    });
                });
                /*
                let new_selected = serde::de::Deserialize::deserialize(selected_in_order.to_value()).unwrap();
                *game.editor_state.selected.as_mut().unwrap_or(&mut Vec::new()) = new_selected;
                */
            }
        );

        let window = imgui::Window::new(imgui::im_str!("Inspection Settings"));
        window
            .position([crate::APP.get().window.window.inner_size().width as f32 - self.state_2.size_width_editor, 0.0], imgui::Condition::Always)
            .title_bar(true)
            .resizable(false)
            .movable(false)
            .horizontal_scrollbar(true)
            .bg_alpha(1.0)
            .size([self.state_2.size_width_editor, APP.get().window.window.inner_size().height as f32], imgui::Condition::Always)
            .build(ui, || {
                ui.spacing();
                /*
                let mut settings_reflect = reflect::ReflectedStructSerializer::serialize(&game.settings);
                reflect::imgui::inspect(ui, &mut settings_reflect, None, None);
                let new_settings = settings_reflect.deserialize();
                game.settings = new_settings;
                */

                use reflect::Reflect;
                let mut settings_value = game.settings.to_value();
                reflect::imgui_impl::inspect(ui, &mut settings_value, None, Some(settings_editor()));
                let new_settings = serde::de::Deserialize::deserialize(settings_value).unwrap();
                game.settings = new_settings;
                game.settings.validate(game_bis);
            }
        );
    }

    #[cfg(not(feature = "shipping"))]
    #[allow(dead_code)]
    fn egui_test(&self, egui_context: &egui::CtxRef) {
        new_timer_monothread!(_t, "egui");
        use reflect::Reflect;
        use egui::Widget;
        let game = &mut crate::APP.get_mut().game;
        let game_bis = &mut crate::APP.get_mut().game;
        egui_context.set_style(egui_style());
        egui::Window::new("BBB").resizable(true).scroll(true).show(egui_context, |ui| {
            let mut debug_info = self.debug_info.borrow_mut();
            egui::widgets::DragValue::i32(&mut debug_info.entity_index_choosen).ui(ui);
            if let Some(entity) = game.entities.core.get_mut(debug_info.entity_index_choosen as usize) {
                let entity_info = EntityInfo::from_entity(game_bis, entity.index.main);
                let mut entity_info = entity_info.to_value();
                reflect::egui_impl::inspect(ui, &mut entity_info, None, Some(settings_editor_egui()));
                let new_entity_info = serde::de::Deserialize::deserialize(entity_info).unwrap();
                game_bis.entities.update(entity.index.main, new_entity_info);
            }
        });
        egui::Window::new("AAA").resizable(true).scroll(true).show(egui_context, |ui| {
            let mut settings_value = game.settings.to_value();
            reflect::egui_impl::inspect(ui, &mut settings_value, None, Some(settings_editor_egui()));
            let new_settings = serde::de::Deserialize::deserialize(settings_value).unwrap();
            game.settings = new_settings;
        });
    }
}

#[cfg(all(feature = "serialize"))]
fn settings_editor() -> reflect::imgui_impl::InspectSettings {
    let mut settings = reflect::imgui_impl::InspectSettings::default();
    let atomic_num = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));
    settings.add_default_struct(ThrowEntityInfo::default());
    settings.add_default_struct(crate::game::entity::EntityGravityInfo::default());
    settings.add_default_struct(crate::game::entity::EntityCharacteristics::default());
    /*
    settings.add_default_struct(crate::game::entity::factory::ThrowEntityInfo::default());
    settings.add_default_struct(crate::game::entity::factory::EntityCharacteristics::default());
    */
    
    settings.add_default_value(1_000i32);
    //settings.add_default_value_special(1_000i32, "inertia");

    let atomic_num_clone = atomic_num.clone();
    settings.add_behaviour_struct(Point2D::new(0i32, 0i32), move |ui, value| {
        if let reflect::Value::I32(x) = value.fields[0].value {
            if let reflect::Value::I32(y) = value.fields[1].value {
                let mut values = [x, y];
                let mut id = atomic_num_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed).to_string();
                id.insert_str(0, "##");
                ui.input_int2(&imgui::ImString::new(&id), &mut values).enter_returns_true(true).build();
                value.fields[0].value = reflect::Value::I32(values[0]);
                value.fields[1].value = reflect::Value::I32(values[1]);
            }
        }
    });

    let atomic_num_clone = atomic_num.clone();
    settings.add_behaviour_struct(euclid::default::Vector2D::new(0.0f32, 0.0f32), move |ui, value| {
        if let reflect::Value::F32(x) = value.fields[0].value {
            if let reflect::Value::F32(y) = value.fields[1].value {
                let mut values = [x, y];
                let mut id = atomic_num_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed).to_string();
                id.insert_str(0, "##");
                ui.input_float2(&imgui::ImString::new(&id), &mut values).enter_returns_true(true).build();
                value.fields[0].value = reflect::Value::F32(values[0]);
                value.fields[1].value = reflect::Value::F32(values[1]);
            }
        }
    });

    let atomic_num_clone = atomic_num.clone();
    settings.add_behaviour_struct(0.0f32..10.0, move |ui, value| {
        if let reflect::Value::F32(x) = value.fields[0].value {
            if let reflect::Value::F32(y) = value.fields[1].value {
                let mut values = [x, y];
                let mut id = atomic_num_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed).to_string();
                id.insert_str(0, "##");
                ui.input_float2(&imgui::ImString::new(&id), &mut values).enter_returns_true(true).build();
                value.fields[0].value = reflect::Value::F32(values[0]);
                value.fields[1].value = reflect::Value::F32(values[1]);
            }
        }
    });

    let atomic_num_clone = atomic_num.clone();
    settings.add_behaviour_struct(0.0f32..=10.0, move |ui, value| {
        if let reflect::Value::F32(x) = value.fields[0].value {
            if let reflect::Value::F32(y) = value.fields[1].value {
                let mut values = [x, y];
                let mut id = atomic_num_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed).to_string();
                id.insert_str(0, "##");
                ui.input_float2(&imgui::ImString::new(&id), &mut values).enter_returns_true(true).build();
                value.fields[0].value = reflect::Value::F32(values[0]);
                value.fields[1].value = reflect::Value::F32(values[1]);
            }
        }
    });

    let atomic_num_clone = atomic_num.clone();
    settings.add_behaviour_struct(0i32..=10, move |ui, value| {
        if let reflect::Value::I32(x) = value.fields[0].value {
            if let reflect::Value::I32(y) = value.fields[1].value {
                let mut values = [x, y];
                let mut id = atomic_num_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed).to_string();
                id.insert_str(0, "##");
                ui.input_int2(&imgui::ImString::new(&id), &mut values).enter_returns_true(true).build();
                value.fields[0].value = reflect::Value::I32(values[0]);
                value.fields[1].value = reflect::Value::I32(values[1]);
            }
        }
    });

    settings.add_behaviour_field_value(0usize, "texture".to_string(), |_, _| {});
    settings.add_behaviour_field_value([0.0f32;4], "background_color".to_string(), |_, _| {});
    settings.add_behaviour_field_value([0.0f32;4], "matrix_color".to_string(), |_, _| {});
    settings.add_behaviour_field_value(0.0f32, "camera_initial".to_string(), |_, _| {});
    settings.add_behaviour_field_value(vec![crate::game::entity::EntitySpecial::WASM("".to_string())], "special".to_string(), |_, _| {});
    
    return settings
}

#[cfg(all(feature = "serialize", not(feature = "shipping")))]
fn settings_editor_egui() -> reflect::egui_impl::InspectSettings {
    let mut settings = reflect::egui_impl::InspectSettings::default();
    settings.add_default_struct(ThrowEntityInfo::default());
    settings.add_default_struct(crate::game::entity::EntityGravityInfo::default());
    /*
    settings.add_default_value(1_000i32);
    settings.add_default_value_special(1_000i32, "inertia");
    */
    return settings
}

fn egui_style() -> egui::Style {
    let mut style = egui::Style::default();
    style.visuals.window_corner_radius = 0.0;
    //style.body_text_style = egui::TextStyle::Monospace;

    style.visuals.widgets.active.corner_radius = 0.0;
    style.visuals.widgets.hovered.corner_radius = 0.0;
    style.visuals.widgets.inactive.corner_radius = 0.0;
    style.visuals.widgets.noninteractive.corner_radius = 0.0;
    style.visuals.window_shadow.extrusion = 0.0;

    return style;
}

fn update_hovered(ui: &imgui::Ui) {
    if ui.is_window_hovered_with_flags(imgui::WindowHoveredFlags::all()) { crate::APP.get_mut().game.editor_state.hovered = true; }
    if ui.is_any_item_hovered() { crate::APP.get_mut().game.editor_state.hovered = true; }
}