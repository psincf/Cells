use crate::prelude::*;
use crate::new_timer_monothread;
use threadpool::utils::ParallelIterator;

pub struct DrawingBufferSolver<'a> {
    game: &'a mut Game,
    step: usize,
}

impl<'a> DrawingBufferSolver<'a> {
    pub fn new(game: &'a mut Game) -> DrawingBufferSolver<'a> {
        DrawingBufferSolver {
            game,
            step: 1024,
        }
    }
    #[allow(dead_code)]
    pub fn solve_multithread(&mut self) { // TODO: Refactor it
        new_timer_monothread!(_t, "synchronize_drawing_buffer");
        //if self.game.update_count % 5 != 0 { return }
        let data = unsafe { self.game.drawable.set_with_ptr() };
        data.entities.clear();
        data.entities.reserve(self.game.entities.len());
        data.position.clear();
        data.position.reserve(self.game.entities.len());
        data.mass.clear();
        data.mass.reserve(self.game.entities.len());
        data.lifetime.clear();
        data.lifetime.reserve(self.game.entities.len());
        unsafe { data.entities.set_len(self.game.entities.len()) };
        unsafe { data.position.set_len(self.game.entities.len()) };
        unsafe { data.mass.set_len(self.game.entities.len()) };
        unsafe { data.lifetime.set_len(self.game.entities.len()) };

        {
            let data = unsafe_ptr::UnsafePtr::new(data);
            
            self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), self.step, |index: usize| {
                let data = unsafe { data.ref_mut() };
                let entity_core = &self.game.entities.core[index];
                
                data.entities[index] = DrawableEntity {
                    unique_id: entity_core.index.unique_id,
                    lifetime: 0,
                    old_buffer_id: entity_core.index.drawing_buffer,
                    position: Default::default(),
                    mass: 0.0,
                    color: entity_core.color.center,
                    color_2: entity_core.color.edge,
                };
                unsafe { *(&entity_core.index.drawing_buffer as *const _ as *mut _) = entity_core.index.main };
            });
            self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), self.step, |index: usize| {
                let data = unsafe { data.ref_mut() };
                let entity_mass = self.game.entities.mass[index];
                data.mass[index] = entity_mass as f32;
            });
            unsafe { std::ptr::copy_nonoverlapping(self.game.entities.position.as_ptr(), data.ref_mut().position.as_mut_ptr(), self.game.entities.position.len()) }
            unsafe { std::ptr::copy_nonoverlapping(self.game.entities.lifetime.as_ptr(), data.ref_mut().lifetime.as_mut_ptr(), self.game.entities.lifetime.len()) }
        }

        let update_count = self.game.step.actual_count;
        let state = self.game.state.clone();
        let local_player = PlayerInfo::from_player(&self.game.players[self.game.settings.local_player]);
        
        data.instant = std::time::Instant::now();
        data.update_count = update_count;
        data.state = state;
        data.local_player = local_player;
        data.background_color = self.game.settings.background_color;
        //unsafe { self.game.drawable.change_ptr_last_set() };
    }

    pub fn solve_2(&mut self) {
        new_timer_monothread!(_t, "synchronize_drawing_buffer");
        let game = unsafe_ptr::UnsafePtr::new(self.game);

        let data = unsafe { self.game.drawable.set_with_ptr() };
        data.entities.clear();
        data.entities.reserve(self.game.entities.len());
        unsafe { data.entities.set_len(self.game.entities.len()) };
        
        unsafe { std::ptr::copy_nonoverlapping::<DrawableEntity>(self.game.entities.drawable_entities.as_ptr(), data.entities.as_mut_ptr(), self.game.entities.drawable_entities.len()) }
        self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), self.step, |index: usize| {
            unsafe { game.ref_mut().entities.drawable_entities[index].old_buffer_id = index };
        });
        
        #[cfg(not(feature="shipping"))]
        if crate::DEBUG_SETTINGS.get().draw_color_pression { self.draw_color_pression(); }
        
        let update_count = self.game.step.actual_count;
        let state = self.game.state.clone();
        let local_player = PlayerInfo::from_player(&self.game.players[self.game.settings.local_player]);
        
        data.instant = std::time::Instant::now();
        data.update_count = update_count;
        data.editor_state = self.game.editor_state.clone();
        data.state = state;
        data.local_player = local_player;
        data.background_color = self.game.settings.background_color;
    }

    fn draw_color_pression(&self) {
        let data = unsafe { self.game.drawable.set_with_ptr_same() };
        for entity in self.game.entities.core.iter() {
            data.entities[entity.index.main].color = crate::game::settings::DEFAULT_COLOR_UNIFORM[14].center;
            data.entities[entity.index.main].color_2 = crate::game::settings::DEFAULT_COLOR_UNIFORM[0].center;
            //data.entities[entity.index.main].color[0] = (entity.colliding_info.colliding_pression.get() * 2.0).min(255.0) as u8;
            data.entities[entity.index.main].color[1] = 255 - (entity.colliding_info.colliding_pression.get() * 2.0).min(255.0) as u8;
            data.entities[entity.index.main].color[2] = 255 - (entity.colliding_info.colliding_pression.get() * 2.0).min(255.0) as u8;
        }
    }
}