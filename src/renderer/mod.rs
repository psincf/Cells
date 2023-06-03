mod solver;
use solver::RendererSolver;

use crate::{utils, window::Window};
use crate::game::Game;

use euclid::default::Size2D;
use parking_lot::Mutex;
pub use wgpu_renderer::pipelines::generic_pipeline::Camera2D as Camera;
//use spin::mutex::TicketMutex as Mutex;

#[macro_export]
macro_rules! new_timer_monothread_renderer {
    ($timer: ident, $name: expr) => {
        $crate::new_timer_monothread_generic!($crate::APP.get().renderer.benchmark, $timer, $name);
    };
}

#[derive(Default)]
pub struct CacheFrameBefore {
    total_mass_local_player: f32,
    ratio_camera: f32
}

#[derive(Clone)]
pub struct RendererSettings {
    pub ssaa: i32,
    pub smooth: bool,
    pub draw_matrix: bool,
    pub v_sync: bool,
}

impl Default for RendererSettings {
    fn default() -> RendererSettings {
        RendererSettings {
            ssaa: 1,
            smooth: false,
            draw_matrix: false,
            v_sync: true,
        }
    }
}

pub struct Renderer {
    pub camera: Camera, //TODO: behind a Mutex ? and/or in Game ?
    pub camera_future: Camera, //TODO: bad
    pub core: wgpu_renderer::Renderer,
    pub pipeline: wgpu_renderer::Pipeline2D,
    pub custom_render_pipeline: solver::custom_pipeline::RenderPipeline,
    pub egui_renderer: egui_binding::wgpu_backend::EguiRendererWgpu,
    pub imgui_renderer: imgui_wgpu::Renderer,
    pub drawable_game_cache: crate::game::DrawableGame,
    pub cache_frame_before: CacheFrameBefore,
    pub camera_cache: Option<(Camera, std::time::Instant)>,
    pub benchmark: benchmark::Benchmark,
    pub settings: RendererSettings,
    pub lock_draw: Mutex<()>,
}

impl Renderer {
    pub fn new(window: &winit::window::Window, game: &Game) -> Renderer {
        let camera = Camera{
            x: 0,
            y: 0,
            size: game.settings.camera_initial,
        };
        let camera_future = camera.clone();

        let size = window.inner_size();
        let size = Size2D::new(size.width, size.height);
        let core = wgpu_renderer::Renderer::new(window, size, wgpu::PresentMode::Mailbox);
        if core.is_err() { utils::message_box::message_box(window, "No Vulkan GPU detected"); panic!(); };
        let mut core = core.unwrap();

        let mut pipeline = wgpu_renderer::Pipeline2D::new(&core);
        let custom_render_pipeline = solver::custom_pipeline::RenderPipeline::new(&core, &pipeline);
        pipeline.set_ssaa(&core, wgpu_renderer::pipelines::ssaa::SSAAFactor::Disabled);

        let egui_renderer = {
            let mut egui = game.gui.egui.lock();
            egui.context.begin_frame(Default::default());
            let _ = egui.context.end_frame();

            let mut egui_renderer = egui_binding::wgpu_backend::EguiRendererWgpu::new(&core.device, &core.queue, size.to_f32().into());
            egui_renderer.upload_texture(&egui.context, &core.device, &core.queue);
            egui_renderer
        };

        let imgui_renderer = {
            let mut imgui = game.gui.imgui.lock();
            let mut imgui_context = &mut imgui.context;
            let mut render_config = imgui_wgpu::RendererConfig::new_srgb();
            render_config.texture_format = wgpu::TextureFormat::Bgra8Unorm;
            imgui_wgpu::Renderer::new(&mut imgui_context, &core.device, &mut core.queue, render_config)
        };
        let benchmark = benchmark::Benchmark::new(1);
        let settings = RendererSettings::default();

        Renderer {
            camera,
            camera_future,
            core,
            pipeline,
            custom_render_pipeline,
            egui_renderer,
            imgui_renderer,
            drawable_game_cache: Default::default(),
            cache_frame_before: Default::default(),
            camera_cache: None,
            benchmark,
            settings,
            lock_draw: Mutex::new(()),
        }
    }

    pub fn draw(&mut self, game: &Game, window: &Window) {
        let _lock = unsafe { (*(self as *mut Renderer)).lock_draw.lock() };
        RendererSolver::new(self, game, window).draw();
    }
}