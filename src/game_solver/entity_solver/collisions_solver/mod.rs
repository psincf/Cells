pub mod detection;
pub mod reaction;

use detection::CollisionsDetectionSolver;
use reaction::CollisionsReactionSolver;

use crate::new_timer_monothread;
use crate::prelude::*;
use threadpool::utils::ParallelIterator;

pub struct CollisionsSolver<'a> {
    game: &'a Game,
    step: usize,
}

impl<'a> CollisionsSolver<'a> {
    #[inline]
    pub fn new(game: &'a Game, step: usize) -> CollisionsSolver<'a> {
        CollisionsSolver {
            game,
            step,
        }
    }

    pub fn solve(&self) {
        new_timer_monothread!(_t, "detection");
        //let entities_colliding = &self.game.entities;
        self.init_cache_solver();

        let entities_colliding = self.game.solver_cache.collision.read().unwrap();
        let entities_colliding: &Vec<&EntityCore> = & *entities_colliding;

        self.game.threadpool.compute_iter_each_thread_join(entities_colliding, self.step, |entity| {
            CollisionsDetectionSolver::new(entity, self.game).solve();
        });

        self.game.threadpool.compute_iter_each_thread_join(entities_colliding, self.step, |entity| {
            // TODO: For the moment: Iter entity with colliding activated -> Next: Iter only those who collide ?
            self.game.entities.init_colliding_info(entity.index.main);
        });

        drop(_t); new_timer_monothread!(_t, "compute");
        for _ in 0..5 { //TODO: not sure about this
            self.game.threadpool.compute_iter_each_thread_join(entities_colliding, self.step, |entity| {
                CollisionsReactionSolver::new(entity, self.game).solve();
            });

            self.game.threadpool.compute_iter_each_thread_join(entities_colliding, self.step, |entity| {
                entity.colliding_info.colliding_position.set(entity.colliding_info.colliding_position_new.get());
                entity.colliding_info.colliding_speed.set(entity.colliding_info.colliding_speed_new.get());
                entity.colliding_info.colliding_pression.set(entity.colliding_info.colliding_pression_new.get());
            });
        }

        self.game.threadpool.compute_iter_each_thread_join(entities_colliding, self.step, |entity| {
            let old_position = self.game.entities.position[entity.index.main];
            let old_speed = self.game.entities.speed[entity.index.main];
            let new_position = entity.colliding_info.colliding_position.get();
            //let new_speed = entity.colliding_info.colliding_speed.get();
            let new_speed = (new_position - old_position).to_f32() / 2.0 + old_speed;
            entity.colliding_info.clear_buffer_collider();
            entity.colliding_info.shrink_to_fit_buffer_collider();
            self.game.entities.send_buffer(entity.index.main, EntityAction::AddPosition(new_position.x - old_position.x, new_position.y - old_position.y));
            self.game.entities.send_buffer(entity.index.main, EntityAction::AddSpeed(new_speed.x - old_speed.x, new_speed.y - old_speed.y));
        });
    }

    fn init_cache_solver(&self) { //TODO: make it multithread ?
        self.game.solver_cache.collision.write().unwrap().clear();
        {
            let mut local_entities = Vec::new();
            for entity_index in 0..self.game.entities.len() {
                //if entity.characteristics.collide == false { continue }
                if !self.game.entities.flags[entity_index].contains(crate::game::entity::EntityFlags::COLLIDE) { continue }
                let entity = &self.game.entities.core[entity_index];
                
                let entity = unsafe { &*(entity as *const EntityCore) };
                local_entities.push(entity);
            }
            self.game.solver_cache.collision.write().unwrap().append(&mut local_entities);
        }
        /*
        self.game.threadpool.compute_iter_each_thread_and_wait(&self.game.entities, self.step, |entity| {
            let mut local_entities = Vec::new();
            if entity.characteristics.collide == false { return }
            let entity = unsafe { &*(entity as *const Entity) };
            local_entities.push(entity);
            self.game.solver_cache.collision.write().unwrap().append(&mut local_entities);
        });
        */
    }
}