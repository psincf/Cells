use crate::game::Game;

use std::time::Duration;

pub struct SmoothWaitSolver<'a> {
    game: &'a mut Game,
    instant: std::time::Instant,
}

impl<'a> SmoothWaitSolver<'a> {
    pub fn new(game: &'a mut Game, instant: std::time::Instant) -> SmoothWaitSolver<'a> {
        SmoothWaitSolver {
            game: game,
            instant: instant,
        }
    }

    pub fn solve(&mut self) {
        self.smooth_wait();
        self.update_drawing_buffer();

    }
    
    fn smooth_wait(&mut self) {
        if self.game.step.full_speed { return }
        if self.game.step.changed_map { self.game.step.changed_map = false; return }

        self.game.threadpool.lock_barrier();

        let elapsed = self.instant.elapsed();
        let old_time = self.game.step.last_duration;
        {
            let mut done = false;
            let mut new_time = self.game.step.last_duration;
            for &step in self.game.step.duration_vec.iter() {
                if elapsed < step {
                    if self.game.step.last_duration == step { done = true; break }
                    if (step.as_millis() as f32) * 0.8 > elapsed.as_millis() as f32 { new_time = step; done = true; break }
                }
            }
            if !done { new_time = *self.game.step.duration_vec.last().unwrap() };
            
            self.wait(old_time, new_time);

            self.game.step.last_duration = new_time;
        }
        
        self.game.threadpool.unlock_barrier();

    }

    fn wait(&mut self, old_time: Duration, new_time: Duration) {
        /*
        while self.instant.elapsed() < old_time {  }
        while self.instant.elapsed() < new_time {  }
        */
        let channel = std::sync::mpsc::channel();
        *self.game.step.waiting.lock() = Some((self.instant, old_time, new_time, channel.0));
        channel.1.recv().unwrap();
        *self.game.step.waiting.lock() = None;
    }

    fn update_drawing_buffer(&mut self) {
        unsafe { self.game.drawable.set_with_ptr_same().instant = std::time::Instant::now(); } //TODO: Bug when editor mode. Also bad: Not neat
        unsafe { self.game.drawable.set_with_ptr_same().update_duration = self.game.step.last_duration; }
        unsafe { self.game.drawable.change_ptr_last_set() };
    }
}