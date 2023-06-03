mod cache_entity;
mod cache_game;
mod cache_map;
mod cache_player;

use cache_entity::CacheEntitySolver;
use cache_game::CacheGameSolver;
//use cache_map::CacheMapSolver;
use cache_player::CachePlayerSolver;


use crate::prelude::*;
use crate::utils::BufferChoice; //TODO: bad
use threadpool::utils::ParallelIterator;

use crate::new_timer_monothread;

pub struct ApplyCacheSolver<'a> {    
    game: &'a mut Game,
}

impl<'a> ApplyCacheSolver<'a> {
    pub fn new(game: &'a mut Game) -> ApplyCacheSolver<'a> {
        ApplyCacheSolver {
            game,
        }
    }

    pub fn solve(&mut self) {
        self.apply_cache_entities();
        self.apply_cache_game();
        self.apply_cache_player();
        self.apply_cache_map();
    }

    fn apply_cache_entities(&mut self) { //TODO: change this
        let game = unsafe_ptr::UnsafePtr::new(self.game);
        new_timer_monothread!(_t, "apply_cache_entity");
        
        self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), 512, | index: usize | {
            if self.game.entities.buffer_is_some(index) {
                CacheEntitySolver::solve(game.raw(), index, BufferChoice::First);
            }
        });

        self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), 512, | index: usize | {
            if self.game.entities.buffer2_is_some(index) {
                CacheEntitySolver::solve(game.raw(), index, BufferChoice::Second);
            }
        });

        self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), 512, | index: usize | {
            if self.game.entities.buffer_is_some(index) {
                CacheEntitySolver::solve(game.raw(), index, BufferChoice::First);
            }
        });
    }

    fn apply_cache_game(&mut self) { // TODO: multithread
        CacheGameSolver::new(self.game).solve();
        //CacheGameSolver::new(self.game).solve_multithread();
    }

    fn apply_cache_player(&mut self) {
        CachePlayerSolver::new(self.game).solve();
    }

    fn apply_cache_map(&mut self) {
        //CacheMapSolver::new(self.game).solve();
    }
}