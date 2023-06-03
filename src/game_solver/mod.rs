mod events_solver;
mod entity_solver;
mod apply_cache_solver;
mod map_solver;
mod drawing_buffer_solver;
mod smooth_wait_solver;

use events_solver::EventsSolver;
use entity_solver::EntitySolver;
use apply_cache_solver::ApplyCacheSolver;
use map_solver::MapSolver;
use drawing_buffer_solver::DrawingBufferSolver;
use smooth_wait_solver::SmoothWaitSolver;

use crate::renderer::Camera;
use crate::prelude::*;
use crate::window::Events;

#[macro_export]
macro_rules! new_timer_monothread {
    ($timer: ident, $name: expr) => {
        $crate::new_timer_monothread_generic!($crate::APP.get().game.benchmark, $timer, $name);
    };
}

pub struct GameSolver<'a> {
    game: &'a mut Game,
    events: &'a mut Events,
    camera: Camera,
    instant: std::time::Instant,
}

impl<'a> GameSolver<'a> {
    pub fn new(game: &'a mut Game, events: &'a mut Events, camera: Camera) -> GameSolver<'a> {
        GameSolver {
            game,
            events,
            camera,
            instant: std::time::Instant::now(),
        }
    }

    pub fn solve(&mut self) {
        new_timer_monothread!(_t, "total");
        match self.game.state {
            GameState::MainMenu => {
                
            }
            GameState::Editor => {
                self.check_events();
                self.update_gui();
                //self.udpate_world();
                self.update_drawing_buffer();
            }
            GameState::Playing => {
                self.check_events();
                self.update_gui();
                self.udpate_world();
                self.update_drawing_buffer();
            }
        }
        drop(_t); // hack to update the benchmark time here
        
        self.smooth_wait();
        
        self.game.benchmark.save();
        self.game.benchmark.clear();
        
        self.game.step.actual_count = if self.game.step.actual_count < 1_000 { self.game.step.actual_count + 1 } else { 0 };
    }

    fn check_events(&mut self) {
        EventsSolver::new(self.game, self.events, self.camera.clone()).solve();
    }

    fn update_gui(&mut self) {
        self.game.gui.update(self.game);
    }

    fn udpate_world(&mut self) {
        EntitySolver::new(self.game).solve();
        ApplyCacheSolver::new(self.game).solve();
        MapSolver::new(self.game).solve();
        self.special();
    }

    fn update_drawing_buffer(&mut self) {
        DrawingBufferSolver::new(self.game).solve_2(); // TODO: store the buffer during all the process ?
    }
    
    fn smooth_wait(&mut self) {
        SmoothWaitSolver::new(self.game, self.instant.clone()).solve();
    }

    fn special(&mut self) {
        if !self.game.settings.special { return }
        for _ in 0..1_000 {
            if self.game.entities.len() > 200_000 { continue }
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let position = euclid::default::Point2D::new(
                rng.gen_range(0..self.game.map.size.width * crate::game::map::RATIO_POSITION),
                rng.gen_range(0..self.game.map.size.height * crate::game::map::RATIO_POSITION),
            );
            let angle = (position.to_vector() - euclid::default::Vector2D::new(
                (self.game.map.size.width * crate::game::map::RATIO_POSITION) / 2,
                (self.game.map.size.height * crate::game::map::RATIO_POSITION) / 2
            )).to_f32().normalize().angle_to(
                euclid::default::Vector2D::new(0.0, -1.0)
            );
            let h = angle.to_degrees();
            let length = (position - euclid::default::Point2D::new(
                (self.game.map.size.width * crate::game::map::RATIO_POSITION) / 2,
                (self.game.map.size.height * crate::game::map::RATIO_POSITION) / 2))
                .to_f32().length();
            if length > ((self.game.map.size.width * crate::game::map::RATIO_POSITION) / 2) as f32 { continue }
            let s = 255;
            let v = ((length / ((self.game.map.size.width * crate::game::map::RATIO_POSITION) / 2) as f32) * 255.0) as i32 as u8;
            
            let color = crate::game::settings::colors::from_hsv_to_rgb((h, s, v));
            self.game.new_entity(EntityInfo {
                player: 0,
                position: position,
                speed: euclid::default::Vector2D::new(0.0, 0.0),
                mass: crate::game::entity::RATIO_MASS * 20,
                timer: EntityTimer::default(),
                color: crate::game::entity::EntityColor {
                    center: [color.0, color.1, color.2, 255],
                    edge: [color.0, color.1, color.2, 255]
                },
                texture: 0,
                characteristics: crate::game::entity::EntityCharacteristics::default(),
            });
        }
    }
}