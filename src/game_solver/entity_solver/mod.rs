mod collisions_solver;
mod eating_solver;
mod gravity_solver;
mod no_interactions_solver;
mod throwing_entity_solver;
mod velocity_solver;
pub mod position_solver;
pub mod special_solver;

use collisions_solver::CollisionsSolver;
use eating_solver::EatingSolver;
use gravity_solver::GravitySolver;
use no_interactions_solver::NoInteractionsSolver;
use throwing_entity_solver::ThrowingEntitySolver;
use velocity_solver::VelocitySolver;
use position_solver::PositionSolver;
use special_solver::SpecialSolver;

use crate::prelude::*;
use crate::game::entity::EntityFlags;
use euclid::default::Vector2D;
//use parking_lot::Mutex;
use threadpool::utils::ParallelIterator;

use crate::new_timer_monothread;


pub struct EntitySolver<'a> {
    game: &'a mut Game,
    step: usize,
    //gravity_entities: Mutex<Vec<usize>>,
}

impl<'a> EntitySolver<'a> {
    pub fn new(game: &'a mut Game) -> EntitySolver<'a> {
        EntitySolver {
            game,
            step: 512,
            //gravity_entities: Mutex::new(Vec::new()),
        }
    }

    pub fn solve(&mut self) {
        new_timer_monothread!(_t, "update_entities");
        //self.solve_before_collisions();
        self.solve_before_collisions_2();
        self.solve_collisions();
        self.solve_position();
    }

    #[allow(dead_code)]
    fn solve_before_collisions(&mut self) {
        new_timer_monothread!(_t, "update_entities_before_collisions");
        let game = unsafe_ptr::UnsafePtr::new(self.game);
        self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), self.step, |index| {
            let game = unsafe { game.ref_mut() };
            NoInteractionsSolver::new(index, self.game).solve(); //TODO: make it independant without buffer by updating directly? Better performance!! Too much problem: Need to update map, because it can't search for eating or collision in a proper way without it!
            
            if !(game.entities.speed[index] == Vector2D::zero() && game.entities.direction[index] == None) {
                VelocitySolver::new(index, game).solve_2();
            }
            /*
            if entity_core.characteristics.gravity.is_some() {
                GravitySolver::new(entity_core, self.game).solve();
            }
            if entity_core.characteristics.throw_entity.is_some() {
                ThrowingEntitySolver::new(entity_core, self.game).solve();
            }
            if entity_core.characteristics.killer == true {
                EatingSolver::new(entity_core, self.game).solve();
            }
            */
            if self.game.entities.flags[index].contains(EntityFlags::GRAVITY) {
                GravitySolver::new(index, self.game).solve_3();
            }
            if self.game.entities.flags[index].contains(EntityFlags::THROW) {
                let entity_core = &self.game.entities.core[index];
                ThrowingEntitySolver::new(entity_core, self.game).solve();
            }
            if self.game.entities.flags[index].contains(EntityFlags::EATER) {
                let entity_core = &self.game.entities.core[index];
                EatingSolver::new(entity_core, self.game).solve();
            }
        });
    }
    
    fn solve_collisions(&mut self) {
        new_timer_monothread!(_t, "update_entities_collisions");
        CollisionsSolver::new(self.game, self.step).solve();
    }

    fn solve_position(&mut self) {
        new_timer_monothread!(_t, "update_entities_solve_position");
        let game = unsafe_ptr::UnsafePtr::new(self.game);
        self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), self.step, |index| {
            let game = unsafe { game.ref_mut() };
            if game.entities.speed[index] == Vector2D::zero() { return }
            PositionSolver::new(index, game).solve();
        });
    }

    #[allow(dead_code)]
    fn solve_before_collisions_2(&mut self) {
        new_timer_monothread!(_t, "update_entities_before_collisions");
        let game = unsafe_ptr::UnsafePtr::new(self.game);
        {
            new_timer_monothread!(_t, "update_entities_no_interaction");
            self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), self.step, |index| {
                NoInteractionsSolver::new(index, self.game).solve();
            });
        }
        /*
        {
            new_timer_monothread!(_t, "update_entities_gravity_prepare");
            self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), self.step, |index| {
                if self.game.entities.flags[index].contains(EntityFlags::GRAVITY) {
                    self.gravity_entities.lock().push(index);
                }
            });
        }
        {
            new_timer_monothread!(_t, "update_entities_gravity_execute");
            let entities_with_gravity = self.gravity_entities.lock().clone();
            if !entities_with_gravity.is_empty() {
                self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), self.step, |index| {
                    let entity_core = &self.game.entities.core[index];
                    GravitySolver::new(index, self.game).solve_per_entity(&entities_with_gravity);
                });
            }
        }
        */
        {
            new_timer_monothread!(_t, "update_entities_gravity");
            self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), self.step, |index| {
                if self.game.entities.flags[index].contains(EntityFlags::GRAVITY) {
                    GravitySolver::new(index, self.game).solve_3();
                }
            });
        }
        {
            new_timer_monothread!(_t, "update_entities_velocity");
            self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), self.step, |index| {
                let game = unsafe { game.ref_mut() };
                if game.entities.speed[index] == Vector2D::zero() && game.entities.direction[index] == None { return }
                VelocitySolver::new(index, game).solve_2();
            });
        }
        {
            new_timer_monothread!(_t, "update_entities_throw");
            self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), self.step, |index| {
                let entity_core = &self.game.entities.core[index];
                //if entity_core.characteristics.throw_entity.is_some() {
                if self.game.entities.flags[index].contains(EntityFlags::THROW) {
                    ThrowingEntitySolver::new(entity_core, self.game).solve();
                }
            });
        }
        {
            new_timer_monothread!(_t, "update_entities_eat");
            self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), self.step, |index| {
                let entity_core = &self.game.entities.core[index];
                //if entity_core.characteristics.killer == true {
                if self.game.entities.flags[index].contains(EntityFlags::EATER) {
                    EatingSolver::new(entity_core, self.game).solve();
                }
            });
        }
        {
            new_timer_monothread!(_t, "update_entities_special");
            self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), self.step, |index| {
                let entity_special = &self.game.entities.special[index];
                if !entity_special.is_empty() {
                    for _ in entity_special.iter() {
                        match SpecialSolver::new(index, self.game).solve() {
                            Ok(()) => {}
                            Err(err) => {dbg!(err);}
                        }
                    }
                }
            });
        }
    }
}