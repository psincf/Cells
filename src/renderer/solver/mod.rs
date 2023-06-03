mod camera_solver;
use camera_solver::CameraSolver;

pub mod custom_pipeline;

mod interpolation_solver;
use interpolation_solver::InterpolationSolver;

use super::Camera;
use super::Renderer;

use crate::{game::map::RATIO_POSITION, prelude::*};
use crate::window::Window;
use crate::game::gui::Gui;
use crate::new_timer_monothread_renderer;

use parking_lot::MutexGuard;
use euclid::default::Point2D;
use euclid::default::Point3D;
use euclid::default::Vector2D;

use wgpu_renderer::pipelines::generic_pipeline::Camera2D;
use wgpu_renderer::pipelines::generic_pipeline::ClearInfo;

pub struct RendererSolver<'a> {
    pub instant: std::time::Instant,
    pub update_duration: std::time::Duration,
    pub renderer: &'a mut Renderer,
    pub pre_drawable_game: MutexGuard<'a, DrawableGame>,
    pub drawable_game: MutexGuard<'a, DrawableGame>,
    pub gui: &'a Gui,
    pub window: &'a Window,
}

impl<'a> RendererSolver<'a> {
    pub fn new(renderer: &'a mut Renderer, game: &'a Game, window: &'a Window) -> RendererSolver<'a> {
        let (pre_drawable_game, drawable_game) = game.drawable.get_pre_last_and_last();
        let instant = std::time::Instant::now();
        let update_duration = drawable_game.update_duration;
        RendererSolver {
            instant,
            update_duration,
            renderer,
            pre_drawable_game,
            drawable_game,
            gui: &game.gui,
            window: window,
        }
    }

    pub fn draw(&mut self) {
        new_timer_monothread_renderer!(timer, "renderer");
        InterpolationSolver::new(self).solve();
        CameraSolver::new(self).solve();

        if !self.new_frame() { return }; // TODO: improve this ? At the creation of RendererSolver instead ?
        self.update_ssaa();
        self.update_vsync();
        self.clear();
        //self.smooth_clear();

        if self.renderer.settings.draw_matrix {
            self.draw_matrix();
        }

        #[cfg(not(feature = "shipping"))]
        self.draw_debug();

        {
            new_timer_monothread_renderer!(_timer_pipeline, "pipeline");
            if self.renderer.settings.smooth {
                self.draw_entities_smooth();
            } else {
                self.draw_entities();
            }
            self.draw_selected_entities();
            self.renderer.core.device.poll(wgpu::Maintain::Wait); //TODO: ?
        }
        
        //self.draw_mass_debug();
        {
            new_timer_monothread_renderer!(_timer_pipeline, "Gui");
            self.draw_gui();
            self.draw_gui_egui();
        }

        self.renderer.core.end_frame();

        drop(timer);
        self.renderer.benchmark.save();
        self.renderer.benchmark.clear();
    }

    #[cfg(not(feature = "shipping"))]
    fn draw_debug(&mut self) {
        if !crate::DEBUG.get() { return }
        if crate::DEBUG_SETTINGS.get().draw_matrix_simple {
            self.draw_matrix_simple();
        }
        if crate::DEBUG_SETTINGS.get().draw_matrix_physics {
            self.draw_matrix_physics();
        }
    }
    
    /*
    #[allow(dead_code)]
    fn draw_mass_debug(&mut self) {
        let size_window = self.window.window.inner_size();
        let font = wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!("../../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
        let mut glyph_brush = wgpu_glyph::GlyphBrushBuilder::using_font(font).build(&self.renderer.core.device, wgpu::TextureFormat::Bgra8Unorm);

        for entity_index in self.renderer.drawable_game_cache.local_player.entities.clone() {
            let entity = &self.renderer.drawable_game_cache.entities[entity_index];
            let position_world = entity.position;
            let position_camera = Point2D::new(self.renderer.camera.x, self.renderer.camera.y);
            let position_world2 = position_world - position_camera.to_vector();
            let position_world3 = position_world2.to_f32() / self.renderer.camera.size;
            let position_world4 = position_world3 + Vector2D::new(size_window.width as f32 / 2.0, size_window.height as f32 / 2.0);
            
            let mass_str = (entity.mass / 10_000.0).to_string();

            let section = wgpu_glyph::Section {
                screen_position: position_world4.into(),
                text: vec![wgpu_glyph::Text::new(&mass_str)],
                ..Default::default()
            };
            glyph_brush.queue(section);
        }

        let mut command_encoder = self.renderer.core.get_device().create_command_encoder(&wgpu::CommandEncoderDescriptor{ label:None });
        let size = self.window.window.inner_size();
        let mut staging_belt = wgpu::util::StagingBelt::new(1_000);
        glyph_brush.draw_queued(&self.renderer.core.device, &mut staging_belt, &mut command_encoder, &self.renderer.core.get_actual_frame().unwrap().output.view, size.width, size.height).unwrap();
        staging_belt.finish();
        self.renderer.core.queue.submit(vec![command_encoder.finish()]);
    }
    */

    fn draw_matrix(&mut self) {
        let alpha_ratio = ((RATIO_POSITION as f32) / (self.renderer.camera.size * 100.0)).max(0.0).min(1.0);
        //let size_line = (RATIO_POSITION as f32 / 20.0) as i32;
        let size_line = (self.renderer.camera.size * 5.0).min(RATIO_POSITION as f32 / 20.0) as i32;
        let mut vertex_data = wgpu_renderer::vertex_data::Vertex2DColoredi32Buffer::new();
        let color = APP.get().game.settings.matrix_color;
        let color = (color[0], color[1], color[2], color[3] * alpha_ratio);
        for x in 0..=crate::APP.get().game.map.size.width {
            vertex_data.data.extend_from_slice(&wgpu_renderer::vertex_data::colored_2d_i32::triangulate_colored_2d_i32(
                Point2D::new(x * RATIO_POSITION, 0),
                Point2D::new(x * RATIO_POSITION + size_line, crate::APP.get().game.map.size.height * RATIO_POSITION),
                1,
                color
            ));
        }

        for y in 0..=crate::APP.get().game.map.size.height {
            vertex_data.data.extend_from_slice(&wgpu_renderer::vertex_data::colored_2d_i32::triangulate_colored_2d_i32(
                Point2D::new(0, y * RATIO_POSITION),
                Point2D::new(crate::APP.get().game.map.size.width * RATIO_POSITION, y * RATIO_POSITION  + size_line),
                1,
                color
            ));
        }
        self.renderer.pipeline.draw_colored_i32(&self.renderer.core, &vertex_data, &Camera2D { x: self.renderer.camera.x, y: self.renderer.camera.y, size: self.renderer.camera.size });
    }

    #[allow(dead_code)]
    fn draw_matrix_simple(&mut self) {
        let size_field = APP.get().game.map.matrix_simple.size_field;
        let alpha_ratio = ((size_field as f32) / (self.renderer.camera.size * 100.0)).max(0.0).min(1.0);
        //let size_line = (size_field as f32 / 20.0) as i32;
        let size_line = (self.renderer.camera.size * 5.0).min(size_field as f32 / 10.0) as i32;
        let mut vertex_data = wgpu_renderer::vertex_data::Vertex2DColoredi32Buffer::new();
        for x in 0..=crate::APP.get().game.map.matrix_simple.size.width {
            vertex_data.data.extend_from_slice(&wgpu_renderer::vertex_data::colored_2d_i32::triangulate_colored_2d_i32(
                Point2D::new(x * size_field, 0),
                Point2D::new(x * size_field + size_line, crate::APP.get().game.map.matrix_simple.size.height * size_field),
                1,
                (1.0, 1.0, 1.0, 0.4 * alpha_ratio)
            ));
        }

        for y in 0..=crate::APP.get().game.map.matrix_simple.size.height {
            vertex_data.data.extend_from_slice(&wgpu_renderer::vertex_data::colored_2d_i32::triangulate_colored_2d_i32(
                Point2D::new(0, y * size_field),
                Point2D::new(crate::APP.get().game.map.matrix_simple.size.width * size_field, y * size_field  + size_line),
                1,
                (1.0, 1.0, 1.0, 0.4 * alpha_ratio)
            ));
        }
        self.renderer.pipeline.draw_colored_i32(&self.renderer.core, &vertex_data, &Camera2D { x: self.renderer.camera.x, y: self.renderer.camera.y, size: self.renderer.camera.size });
    }

    #[allow(dead_code)]
    fn draw_matrix_physics(&mut self) {
        for matrix in crate::APP.get().game.map.matrix_physics.matrix_list.iter() {
            draw_matrix_physics_fn(&mut self.renderer, matrix, [1.0, 1.0, 1.0, 1.0]);
        }

        fn draw_matrix_physics_fn(renderer: &mut crate::renderer::Renderer, matrix: &crate::game::map::matrix_physics::MatrixPhysics, color: [f32;4]) {
            let size = matrix.get_size_cell();
            let alpha_ratio = (size as f32) / (renderer.camera.size * 200.0);
            let mut vertex_data = wgpu_renderer::vertex_data::Vertex2DColoredi32Buffer::new();
            for x in 0..=matrix.get_size_matrix().width {
                vertex_data.data.extend_from_slice(&wgpu_renderer::vertex_data::colored_2d_i32::triangulate_colored_2d_i32(
                    Point2D::new(x * size, 0),
                    Point2D::new(x * size + (renderer.camera.size * 2.0) as i32, matrix.get_size_matrix().height * size),
                    1,
                    (color[0], color[1], color[2], color[3] * alpha_ratio)
                ));
            }

            for y in 0..=matrix.get_size_matrix().height {
                vertex_data.data.extend_from_slice(&wgpu_renderer::vertex_data::colored_2d_i32::triangulate_colored_2d_i32(
                    Point2D::new(0, y * size),
                    Point2D::new(matrix.get_size_matrix().width * size, y * size  + (renderer.camera.size * 2.0) as i32),
                    1,
                    (color[0], color[1], color[2], color[3] * alpha_ratio)
                ));
            }
            renderer.pipeline.clear(&renderer.core, wgpu_renderer::pipelines::generic_pipeline::ClearInfo::Depth);
            renderer.pipeline.draw_colored_i32(&renderer.core, &vertex_data, &Camera2D { x: renderer.camera.x, y: renderer.camera.y, size: renderer.camera.size });
        }
    }

    fn new_frame(&mut self) -> bool {
        self.renderer.core.new_frame().is_ok()
    }

    fn update_ssaa(&mut self) {
        if self.renderer.settings.ssaa == self.renderer.pipeline.get_ssaa_mul() { return }
        let ssaa_factor = match self.renderer.settings.ssaa {
            1 => { wgpu_renderer::pipelines::ssaa::SSAAFactor::Disabled }
            2 => { wgpu_renderer::pipelines::ssaa::SSAAFactor::X2 }
            4 => { wgpu_renderer::pipelines::ssaa::SSAAFactor::X4 }
            8 => { wgpu_renderer::pipelines::ssaa::SSAAFactor::X8 }
            _ => { panic!("SSAA") }
        };
        self.renderer.pipeline.set_ssaa(&self.renderer.core, ssaa_factor);
    }

    fn update_vsync(&mut self) {
        let present_mode = match self.renderer.settings.v_sync {
            true => wgpu::PresentMode::Fifo,
            false => wgpu::PresentMode::Mailbox,
        };
        if self.renderer.core.get_present_mode() != present_mode {
            self.renderer.core.set_present_mode(present_mode);
        }
    }

    fn draw_entities(&mut self) {
        while self.renderer.drawable_game_cache.entities.len() * std::mem::size_of::<crate::game::entity::DrawableEntity>() > self.renderer.custom_render_pipeline.get_buffer_data_size() as usize {
            let new_size = self.renderer.custom_render_pipeline.get_buffer_data_size() * 2;
            self.renderer.custom_render_pipeline.set_buffer_data_size(&self.renderer.core, &self.renderer.pipeline, new_size);
        }
        
        self.renderer.custom_render_pipeline.update_buffer_data(&self.renderer.core, &self.renderer.drawable_game_cache.entities);
        //self.renderer.custom_render_pipeline.update_buffer_data_vec_chunk(&self.renderer.core, &self.renderer.drawable_game_cache.entities);

        let vertex_data = self.renderer.custom_render_pipeline.get_original_vertex();

        self.renderer.custom_render_pipeline.update_uniform_buffer_2(&self.renderer.core, custom_pipeline::UniformBuffer2 { renderer_mode: 1, step: 1 });
        self.renderer.pipeline.draw_custom(
            &self.renderer.core,
            &self.renderer.custom_render_pipeline,
            unsafe { vertex_data.align_to().1 },
            &Camera2D { x: self.renderer.camera.x, y: self.renderer.camera.y, size: self.renderer.camera.size as f32 },
            0..self.renderer.drawable_game_cache.entities.len() as u32
        );

        self.renderer.core.device.poll(wgpu::Maintain::Wait);
    }

    fn draw_entities_smooth(&mut self) {
        while self.renderer.drawable_game_cache.entities.len() * std::mem::size_of::<crate::game::entity::DrawableEntity>() > self.renderer.custom_render_pipeline.get_buffer_data_size() as usize {
            let new_size = self.renderer.custom_render_pipeline.get_buffer_data_size() * 2;
            self.renderer.custom_render_pipeline.set_buffer_data_size(&self.renderer.core, &self.renderer.pipeline, new_size);
        }

        self.renderer.custom_render_pipeline.update_buffer_data(&self.renderer.core, &self.renderer.drawable_game_cache.entities);
        //self.renderer.custom_render_pipeline.update_buffer_data_vec_chunk(&self.renderer.core, &self.renderer.drawable_game_cache.entities);

        let vertex_data = self.renderer.custom_render_pipeline.get_original_vertex();

        self.renderer.custom_render_pipeline.update_uniform_buffer_2(&self.renderer.core, custom_pipeline::UniformBuffer2 { renderer_mode: 2, step: 1 });
        self.renderer.pipeline.draw_custom(
            &self.renderer.core,
            &self.renderer.custom_render_pipeline,
            unsafe { vertex_data.align_to().1 },
            &Camera2D { x: self.renderer.camera.x, y: self.renderer.camera.y, size: self.renderer.camera.size as f32 },
            0..self.renderer.drawable_game_cache.entities.len() as u32
        );
        self.renderer.core.device.poll(wgpu::Maintain::Wait);


        self.renderer.custom_render_pipeline.update_uniform_buffer_2(&self.renderer.core, custom_pipeline::UniformBuffer2 { renderer_mode: 2,  step: 2 });
        self.renderer.pipeline.draw_custom(
            &self.renderer.core,
            &self.renderer.custom_render_pipeline,
            unsafe { vertex_data.align_to().1 },
            &Camera2D { x: self.renderer.camera.x, y: self.renderer.camera.y, size: self.renderer.camera.size as f32 },
            0..self.renderer.drawable_game_cache.entities.len() as u32
        );
        
        self.renderer.core.device.poll(wgpu::Maintain::Wait);
    }

    fn draw_selected_entities(&mut self) {
        if self.drawable_game.state != crate::game::GameState::Editor { return }
        use epaint::emath;
        //let mut vertex = wgpu_renderer::vertex_data::Vertex2DColoredi32Buffer::new();
        let mut shapes = Vec::new();

        fn draw_inner(renderer: &mut RendererSolver, shapes: &mut Vec<epaint::ClippedShape>, index: usize, fill: epaint::Color32, color: epaint::Color32, width: f32) {
            let size_window = crate::APP.get().window.events.resize_events.size.lock().unwrap().clone();
            let entity = &renderer.renderer.drawable_game_cache.entities[index];
            let position_screen = entity.position - Vector2D::new(renderer.renderer.camera.x, renderer.renderer.camera.y);
            let position_screen = position_screen.to_f32() / renderer.renderer.camera.size as f32 + (size_window.to_vector() / 2).to_f32();
            let radius_screen = entity.get_radius() / renderer.renderer.camera.size as f32;
            let shape = epaint::ClippedShape(
                emath::Rect::EVERYTHING,
                epaint::Shape::Rect {
                    corner_radius: 0.0,
                    rect: emath::Rect::from_min_max(
                        emath::Pos2::new(position_screen.x - radius_screen, position_screen.y - radius_screen),
                        emath::Pos2::new(position_screen.x + radius_screen, position_screen.y + radius_screen),
                    ),
                    fill: fill,
                    stroke: epaint::Stroke {
                        width: width,
                        color: color,
                    }
                }
            );
            shapes.push(shape);
        }

        for selected in self.drawable_game.editor_state.selected.clone().unwrap_or_default().iter() {
            draw_inner(self, &mut shapes, *selected, epaint::Color32::TRANSPARENT,epaint::Color32::WHITE, 1.0);
            /*
            let size_window = crate::APP.get().window.events.resize_events.size.lock().unwrap().clone();
            let entity = &self.renderer.drawable_game_cache.entities[*selected];
            let position_screen = entity.position - Vector2D::new(self.renderer.camera.x, self.renderer.camera.y);
            let position_screen = position_screen.to_f32() / self.renderer.camera.size as f32 + (size_window.to_vector() / 2).to_f32();
            let radius_screen = entity.get_radius() / self.renderer.camera.size as f32;
            let shape = epaint::ClippedShape(
                emath::Rect::EVERYTHING,
                epaint::Shape::Rect {
                    corner_radius: 0.0,
                    rect: emath::Rect::from_min_max(
                        emath::Pos2::new(position_screen.x - radius_screen, position_screen.y - radius_screen),
                        emath::Pos2::new(position_screen.x + radius_screen, position_screen.y + radius_screen),
                    ),
                    fill: epaint::Color32::TRANSPARENT,
                    stroke: epaint::Stroke {
                        width: 1.0,
                        color: epaint::Color32::WHITE,
                    }
                }
            );
            shapes.push(shape);
            */
            /*
            vertex.data.extend_from_slice(&wgpu_renderer::vertex_data::colored_2d_i32::triangulate_colored_2d_i32(
                entity.position - Vector2D::new(entity.get_radius() as i32, entity.get_radius() as i32),
                entity.position + Vector2D::new(entity.get_radius() as i32, entity.get_radius() as i32),
                1_000,
                (1.0, 1.0, 1.0, 1.0)
            ));
            */
        }
        if let Some(index) = self.drawable_game.editor_state.entity_hovered {
            draw_inner(self, &mut shapes, index, epaint::Color32::TRANSPARENT,epaint::Color32::RED, 2.0);
        }

        if let Some(index) = self.drawable_game.editor_state.entity_selected {
            draw_inner(self, &mut shapes, index, epaint::Color32::TRANSPARENT,epaint::Color32::GREEN, 1.0);
        }
        if self.drawable_game.editor_state.selection.is_some() {
            let size_window = crate::APP.get().window.events.resize_events.size.lock().unwrap().clone();
            let origin = self.drawable_game.editor_state.selection.unwrap() - Vector2D::new(self.renderer.camera.x, self.renderer.camera.y);
            let origin_screen = origin.to_f32() / self.renderer.camera.size as f32 + (size_window.to_vector() / 2).to_f32();
            
            let end = APP.get().window.events.mouse_events.mouse_position;

            let shape = epaint::ClippedShape(
                emath::Rect::EVERYTHING,
                epaint::Shape::Rect {
                    corner_radius: 0.0,
                    rect: emath::Rect::from_two_pos(
                        emath::Pos2::new(origin_screen.x as f32, origin_screen.y as f32),
                        emath::Pos2::new(end.x as f32, end.y as f32),
                    ),
                    fill: epaint::Color32::TRANSPARENT,
                    stroke: epaint::Stroke {
                        width: 1.0,
                        color: epaint::Color32::WHITE,
                    }
                }
            );
            shapes.push(shape);
        }

        let mut options = epaint::TessellationOptions::default();
        options.anti_alias = false;
        let tesselate = epaint::tessellator::tessellate_shapes(shapes, options, &epaint::text::Fonts::from_definitions(1.0, Default::default()));
        let size_window = crate::APP.get().window.events.resize_events.size.lock().unwrap().clone();
        self.renderer.egui_renderer.draw_egui(
            tesselate,
            &self.renderer.core.device, &self.renderer.core.queue,
            &self.renderer.core.get_actual_frame().unwrap().output.view,
            [size_window.width as f32, size_window.height as f32]
        );
        /*
        self.renderer.pipeline.draw_colored_i32(
            &self.renderer.core,
            &vertex,
            &Camera2D { x: self.renderer.camera.x, y: self.renderer.camera.y, size: self.renderer.camera.size as f32 },
        );
        */
    }
    
    fn draw_gui(&mut self) {
        match self.drawable_game.state {
            GameState::Editor | GameState::Playing => {
                let mut imgui = self.gui.imgui.lock();
                let mut image_view = &self.renderer.core.get_actual_frame().unwrap().output.view;

                //let ssaa_factor = self.renderer.pipeline.get_ssaa_mul();
                let ssaa_factor = 1;
                let dpi = crate::APP.get().window.window.scale_factor() as f32;
                imgui.context.io_mut().display_framebuffer_scale = [ssaa_factor as f32 * dpi, ssaa_factor as f32 * dpi];
                
                if ssaa_factor > 1 {
                    image_view = self.renderer.pipeline.get_ssaa_image_view();
                }

                if let Some(draw_data) = self.gui.renderer_data.lock().imgui {
                    let window_size = crate::APP.get().window.events.resize_events.size.lock().unwrap().clone();
                    if draw_data.display_size[0] as i32 != window_size.width && draw_data.display_size[1] as i32 != window_size.height {
                        return
                    }
                    let mut command_encoder = self.renderer.core.get_device().create_command_encoder(&wgpu::CommandEncoderDescriptor{ label:None });
                    {
                        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                                attachment: &image_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: true,
                                }
                            }],
                            depth_stencil_attachment: None,
                        });
                        self.renderer.imgui_renderer.render(draw_data, self.renderer.core.get_queue(), self.renderer.core.get_device(), &mut render_pass).unwrap();
                    }
                    self.renderer.core.queue.submit(vec!(command_encoder.finish()));
                }
                if ssaa_factor > 1 { self.renderer.pipeline.update_ssaa(&self.renderer.core) };
            }
            _ => {
                
            }
        }
    }

    fn draw_gui_egui(&mut self) {
        let paints_jobs = {
            let egui = self.gui.egui.lock();
            egui.context.tessellate(self.gui.renderer_data.lock().egui.clone().unwrap_or_default().1)
        };

        let size_window = crate::APP.get().window.events.resize_events.size.lock().unwrap().clone();
        self.renderer.egui_renderer.draw_egui(
            paints_jobs,
            &self.renderer.core.device, &self.renderer.core.queue,
            &self.renderer.core.get_actual_frame().unwrap().output.view,
            [size_window.width as f32, size_window.height as f32]
        );
    }

    fn clear(&mut self) {
        let color = self.drawable_game.background_color;
        let wgpu_color = wgpu::Color {
            r: color[0] as f64,
            g: color[1] as f64,
            b: color[2] as f64,
            a: color[3] as f64,
        };
        self.renderer.pipeline.clear(&self.renderer.core, ClearInfo::All(wgpu_color) );
    }

    #[allow(dead_code)]
    fn smooth_clear(&mut self) {
        use wgpu_renderer::vertex_data::Vertex2DColoredi32;
        use wgpu_renderer::vertex_data::Vertex2DColoredi32Buffer;

        let size_window = self.window.window.inner_size();
        self.renderer.pipeline.clear(&self.renderer.core, ClearInfo::Depth );

        let mut buffer_colors = Vertex2DColoredi32Buffer::new();
        buffer_colors.data.push(Vertex2DColoredi32 {
            position: Point3D::new(0, 0, 1),
            color: (0.1, 0.1, 0.1, 0.2),
        });
        buffer_colors.data.push(Vertex2DColoredi32 {
            position: Point3D::new(1920, 0, 1),
            color: (0.1, 0.1, 0.1, 0.2),
        });
        buffer_colors.data.push(Vertex2DColoredi32 {
            position: Point3D::new(0, 1080, 1),
            color: (0.1, 0.1, 0.1, 0.2),
        });
        buffer_colors.data.push(Vertex2DColoredi32 {
            position: Point3D::new(0, 1080, 1),
            color: (0.1, 0.1, 0.1, 0.2),
        });
        buffer_colors.data.push(Vertex2DColoredi32 {
            position: Point3D::new(1920, 0, 1),
            color: (0.1, 0.1, 0.1, 0.2),
        });
        buffer_colors.data.push(Vertex2DColoredi32 {
            position: Point3D::new(1920, 1080, 1),
            color: (0.1, 0.1, 0.1, 0.2),
        });

        self.renderer.pipeline.draw_colored_i32(
            &self.renderer.core,
            &buffer_colors,
            &Camera2D{ x: size_window.width as i32 / 2, y: size_window.height as i32 / 2, size: 1.0 }
        );
    }
}