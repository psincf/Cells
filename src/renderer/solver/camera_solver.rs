use euclid::default::Point2D;

use crate::APP;
use crate::game::GameState;

use super::Camera;
use super::RendererSolver;

pub struct CameraSolver<'a> {
    solver: *mut RendererSolver<'a>,
}

impl<'a> CameraSolver<'a> {
    pub fn new(solver: *mut RendererSolver<'a>) -> CameraSolver<'a> {
        CameraSolver {
            solver,
        }
    }
    
    pub fn solve(&mut self) { //TODO: Check overflow when compute camera position;
        let solver = unsafe { &mut *(self.solver as *mut RendererSolver) };
        let drawable_game = &solver.renderer.drawable_game_cache;
        
        solver.renderer.camera_future.size = solver.renderer.camera_future.size.min(APP.get().game.settings.max_camera); // TODO: change that -> when map recreated instead

        let local_player = &drawable_game.local_player;
        let mut position_total: Point2D<i64> = Point2D::new(
            0,
            0
        );
        let mut count = 0;
        let mut mass_total = 0.0;
        for i in 0..local_player.entities.len() {
            let entity = &drawable_game.entities[local_player.entities[i]];
            let ratio = (entity.mass / 1_000_000.0) as i64;
            position_total.x += entity.position.x as i64 * ratio;
            position_total.y += entity.position.y as i64 * ratio;
            count += ratio;
            mass_total += entity.mass;
        }
        if count != 0 {
            position_total.x /= count;
            position_total.y /= count;
        } else {
            position_total.x = solver.renderer.camera.x as i64;
            position_total.y = solver.renderer.camera.y as i64;
        }
        let ratio_camera = mass_total * (local_player.entities.len() as f32).sqrt();
        match drawable_game.state {
            GameState::Playing => {
                let _size = if solver.renderer.cache_frame_before.total_mass_local_player == 0.0 {
                    solver.renderer.camera_future.size
                } else if mass_total == 0.0 {
                    solver.renderer.camera_future.size
                } else {
                    ((ratio_camera / solver.renderer.cache_frame_before.ratio_camera).sqrt().sqrt() * solver.renderer.camera_future.size).max(1.0).min(APP.get().game.settings.max_camera)
                };

                solver.renderer.camera_future = Camera {
                    x: position_total.x as i32,
                    y: position_total.y as i32,
                    size: solver.renderer.camera_future.size,
                    //size: size,
                };
                
                solver.renderer.camera = Camera {
                    x: position_total.x as i32,
                    y: position_total.y as i32,
                    size: solver.renderer.camera_future.size,
                    //size: (mass_total / 50_000.0).sqrt().max(1.0),
                    //size: ((mass_total / solver.renderer.cache_frame_before.total_mass_local_player) * solver.renderer.camera_future.size).max(1.0).min(APP.get().game.settings.max_camera),
                };
            }

            GameState::Editor => {
                solver.renderer.camera.x = solver.renderer.camera_future.x;
                solver.renderer.camera.y = solver.renderer.camera_future.y;
                solver.renderer.camera.size = solver.renderer.camera_future.size;
            }

            _ => {

            }
        }

        if let Some((camera_old, instant_old)) = solver.renderer.camera_cache.clone() {
            let duration = solver.instant.duration_since(instant_old).as_secs_f32() * 1000.0;
            let time_half = 20.0;
            let ratio = 1.0 - (1.0 / (2.0f32.powf(duration / time_half)));
            //let time_half_2 = 200.0;
            //let ratio_2 = 1.0 - (1.0 / (2.0f32.powf(duration / time_half_2)));

            solver.renderer.camera.x = camera_old.x + (((solver.renderer.camera.x - camera_old.x) as f32) * ratio) as i32;
            solver.renderer.camera.y = camera_old.y + (((solver.renderer.camera.y - camera_old.y) as f32) * ratio) as i32;
            solver.renderer.camera.size = camera_old.size + ((solver.renderer.camera.size - camera_old.size) as f32) * ratio;
        }
        solver.renderer.camera_cache = Some((solver.renderer.camera.clone(), solver.instant));
        solver.renderer.cache_frame_before.total_mass_local_player = mass_total;
        solver.renderer.cache_frame_before.ratio_camera = ratio_camera;
    }
}