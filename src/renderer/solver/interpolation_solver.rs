use crate::new_timer_monothread_renderer;

use super::RendererSolver;

pub struct InterpolationSolver<'a> {
    solver: *mut RendererSolver<'a>,
}

impl<'a> InterpolationSolver<'a> {
    pub fn new(solver: *mut RendererSolver ) -> InterpolationSolver {
        InterpolationSolver {
            solver,
        }
    }

    pub fn solve(&mut self) {
        new_timer_monothread_renderer!(_t, "interpolation");
        self.update_drawable_smooth();
        //self.update_drawable_smooth_opt();
        //self.update_drawable_smooth_2();
        //self.no_update();
    }

    #[allow(dead_code)]
    fn no_update(&mut self) {
        let solver = unsafe { &mut *(self.solver as *mut RendererSolver) };
        let drawable_game = &solver.drawable_game;
        if drawable_game.entities.len() == 0 { return }
        solver.renderer.drawable_game_cache.instant = drawable_game.instant;
        solver.renderer.drawable_game_cache.local_player = drawable_game.local_player.clone();
        solver.renderer.drawable_game_cache.state = drawable_game.state.clone();
        solver.renderer.drawable_game_cache.update_count = drawable_game.update_count;
        solver.renderer.drawable_game_cache.entities.clear();
        solver.renderer.drawable_game_cache.entities.reserve(drawable_game.entities.len());
        unsafe { solver.renderer.drawable_game_cache.entities.set_len(drawable_game.entities.len()) };

        for i in 0..drawable_game.entities.len() {
            solver.renderer.drawable_game_cache.entities[i] = drawable_game.entities[i].clone();
            solver.renderer.drawable_game_cache.entities[i].position = drawable_game.position[i];
            solver.renderer.drawable_game_cache.entities[i].mass = drawable_game.mass[i];
        }
    }

    fn update_drawable_smooth(&mut self) {
        let solver = unsafe { &mut *(self.solver as *mut RendererSolver) };
        let pre_drawable_game = &solver.pre_drawable_game;
        let drawable_game = &solver.drawable_game;
        if drawable_game.entities.len() == 0 { /* return */ }
        solver.renderer.drawable_game_cache.instant = drawable_game.instant;
        solver.renderer.drawable_game_cache.local_player = drawable_game.local_player.clone();
        solver.renderer.drawable_game_cache.state = drawable_game.state.clone();
        solver.renderer.drawable_game_cache.update_count = drawable_game.update_count;
        solver.renderer.drawable_game_cache.entities.clear();
        solver.renderer.drawable_game_cache.entities.reserve(drawable_game.entities.len());
        unsafe { solver.renderer.drawable_game_cache.entities.set_len(drawable_game.entities.len()) };

        let lerp_ratio = solver.instant.duration_since(drawable_game.instant).as_nanos() as f32 / 1_000_000.0 / solver.update_duration.as_millis() as f32;
        let lerp_ratio = lerp_ratio.min(1.0);

        for i in 0..drawable_game.entities.len() {
            let entity = &drawable_game.entities[i];
            solver.renderer.drawable_game_cache.entities[i] = entity.clone();

            if let Some(entity_before) = pre_drawable_game.entities.get(entity.old_buffer_id) {
                let mass = entity_before.mass + ((entity.mass - entity_before.mass) * lerp_ratio);
                let position_difference = (entity.position - entity_before.position).to_f32();
                let position = entity_before.position + (position_difference * lerp_ratio).to_i32();

                solver.renderer.drawable_game_cache.entities[i].position = position;
                solver.renderer.drawable_game_cache.entities[i].mass = mass;
            }
        }
    }

    #[allow(dead_code)]
    fn update_drawable_smooth_opt(&mut self) {
        let solver = unsafe { &mut *(self.solver as *mut RendererSolver) };
        let pre_drawable_game = &solver.pre_drawable_game;
        let drawable_game = &solver.drawable_game;
        if drawable_game.entities.len() == 0 { return }
        solver.renderer.drawable_game_cache.instant = drawable_game.instant;
        solver.renderer.drawable_game_cache.local_player = drawable_game.local_player.clone();
        solver.renderer.drawable_game_cache.state = drawable_game.state.clone();
        solver.renderer.drawable_game_cache.update_count = drawable_game.update_count;
        solver.renderer.drawable_game_cache.entities.clear();
        solver.renderer.drawable_game_cache.entities.reserve(drawable_game.entities.len());

        let lerp_ratio = solver.instant.duration_since(drawable_game.instant).as_nanos() as f32 / 1_000_000.0 / solver.update_duration.as_millis() as f32;
        let lerp_ratio = lerp_ratio.min(1.0);

        let min_x = solver.renderer.camera_cache.as_ref().unwrap().0.x - solver.renderer.camera_cache.as_ref().unwrap().0.size as i32 - (crate::APP.get().window.window.inner_size().width as i32 / 2) * solver.renderer.camera_cache.as_ref().unwrap().0.size as i32;
        let max_x = solver.renderer.camera_cache.as_ref().unwrap().0.x + solver.renderer.camera_cache.as_ref().unwrap().0.size as i32 + (crate::APP.get().window.window.inner_size().width as i32 / 2) * solver.renderer.camera_cache.as_ref().unwrap().0.size as i32;
        let min_y = solver.renderer.camera_cache.as_ref().unwrap().0.y - solver.renderer.camera_cache.as_ref().unwrap().0.size as i32 - (crate::APP.get().window.window.inner_size().height as i32 / 2)  * solver.renderer.camera_cache.as_ref().unwrap().0.size as i32;
        let max_y = solver.renderer.camera_cache.as_ref().unwrap().0.y + solver.renderer.camera_cache.as_ref().unwrap().0.size as i32 + (crate::APP.get().window.window.inner_size().height as i32 / 2)  * solver.renderer.camera_cache.as_ref().unwrap().0.size as i32;

        solver.renderer.drawable_game_cache.local_player.entities.clear();
        let mut local_player = drawable_game.local_player.entities.clone();
        local_player.sort_unstable_by(|a, b| if a < b { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Less } );

        for i in 0..drawable_game.entities.len() {
            let entity = &drawable_game.entities[i];
            let radius = entity.get_radius() as i32 + 1;
            if *local_player.last().unwrap_or(&usize::MAX) == i {
                local_player.pop();
                solver.renderer.drawable_game_cache.local_player.entities.push(solver.renderer.drawable_game_cache.entities.len());
            } else if entity.position.x + radius < min_x || entity.position.x - radius > max_x || entity.position.y + radius < min_y || entity.position.y - radius > max_y {
                continue
            }
            solver.renderer.drawable_game_cache.entities.push(entity.clone());

            if let Some(entity_before) = pre_drawable_game.entities.get(entity.old_buffer_id) {
                let mass = entity_before.mass + ((entity.mass - entity_before.mass) * lerp_ratio);
                let position_difference = (entity.position - entity_before.position).to_f32();
                let position = entity_before.position + (position_difference * lerp_ratio).to_i32();

                solver.renderer.drawable_game_cache.entities.last_mut().unwrap().position = position;
                solver.renderer.drawable_game_cache.entities.last_mut().unwrap().mass = mass;
            }
        }
    }

    /*
    fn update_drawable_smooth_2(&mut self) {
        let solver = unsafe { &mut *(self.solver as *mut RendererSolver) };
        let pre_drawable_game = &solver.pre_drawable_game;
        let drawable_game = &solver.drawable_game;
        if drawable_game.entities.len() == 0 { return }
        solver.renderer.drawable_game_cache.instant = drawable_game.instant;
        solver.renderer.drawable_game_cache.local_player = drawable_game.local_player.clone();
        solver.renderer.drawable_game_cache.state = drawable_game.state.clone();
        solver.renderer.drawable_game_cache.update_count = drawable_game.update_count;
        solver.renderer.drawable_game_cache.entities.clear();
        solver.renderer.drawable_game_cache.entities.reserve(drawable_game.entities.len());
        unsafe { solver.renderer.drawable_game_cache.entities.set_len(drawable_game.entities.len()) };

        let lerp_ratio = solver.instant.duration_since(drawable_game.instant).as_nanos() as f32 / 1_000_000.0 / solver.update_duration.as_millis() as f32;
        let lerp_ratio = lerp_ratio.min(1.0);

        for i in 0..drawable_game.entities.len() {
            let entity = &drawable_game.entities[i];
            solver.renderer.drawable_game_cache.entities[i] = entity.clone();

            if let Some(entity_before) = pre_drawable_game.entities.get(entity.old_buffer_id) {
                let position_before = pre_drawable_game.position[entity.old_buffer_id];
                let position_after = drawable_game.position[i];
                let mass_before = pre_drawable_game.mass[entity.old_buffer_id];
                let mass_after = drawable_game.mass[i];

                let mass = mass_before + ((mass_after - mass_before) * lerp_ratio);
                let position_difference = (position_after - position_before).to_f32();
                let position = position_before + (position_difference * lerp_ratio).to_i32();

                solver.renderer.drawable_game_cache.entities[i].position = position;
                solver.renderer.drawable_game_cache.entities[i].mass = mass;
            } else {
                solver.renderer.drawable_game_cache.entities[i].position = drawable_game.position[i];
                solver.renderer.drawable_game_cache.entities[i].mass = drawable_game.mass[i];
            }
        }
    }
    */
}