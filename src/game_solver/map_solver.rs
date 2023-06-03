use crate::prelude::*;
use crate::game::entity::entities::Entities;
use crate::game::entity::EntityFlags;
use crate::game::settings::AutoSpawnEntityColor;

use crate::new_timer_monothread;

use euclid::default::{Point2D, Vector2D};
use threadpool::utils::ParallelIterator;

pub struct MapSolver<'a> {
    game: &'a mut Game,    
}

impl<'a> MapSolver<'a> {
    pub fn new(game: &'a mut Game) -> MapSolver {
        MapSolver {
            game,
        }
    }

    pub fn solve(&mut self) { // TODO: multithread
        new_timer_monothread!(_t, "update_map");
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for _ in 0..self.game.settings.auto_spawn.amount { //TODO: do it before cache_game ? as a special Entity ?
            if self.game.entities.len() > self.game.settings.max_cells_spawn as usize { continue }
            use crate::game::settings::{AutoSpawnEntityTexture, AutoSpawnMass, SpawnPosition};
            let position = match self.game.settings.auto_spawn.position.clone() {
                SpawnPosition::Exact(x, y) => { Point2D::new(x, y) }
                SpawnPosition::Random => { Point2D::new(rng.gen_range(0..self.game.map.max().width), rng.gen_range(0..self.game.map.max().height)) }
                SpawnPosition::RandomRange(range_x, range_y) => { Point2D::new(rng.gen_range(range_x.start..range_x.end), rng.gen_range(range_y.start..range_y.end)) }
            };

            let mass = match self.game.settings.auto_spawn.mass.clone() {
                AutoSpawnMass::Exact(mass) => { mass }
                AutoSpawnMass::Random(vec) => {
                    let index = rng.gen_range(0..vec.len());
                    vec[index]
                }
                AutoSpawnMass::RandomRange(range) => {
                    rng.gen_range(range.start..=range.end)
                }
            };
            let mass = mass.max(self.game.settings.auto_spawn.characteristics.mass_min).min(self.game.settings.auto_spawn.characteristics.mass_max);

            let color = match self.game.settings.auto_spawn.color.clone() {
                AutoSpawnEntityColor::Custom(color) => { color }
                AutoSpawnEntityColor::Random(vec) => {
                    let index = rng.gen_range(0..vec.len());
                    vec[index]
                }
            };

            let texture = match self.game.settings.auto_spawn.texture.clone() {
                AutoSpawnEntityTexture::CustomIndex(index) => { index }
                AutoSpawnEntityTexture::Random(vec) => {
                    let index = rng.gen_range(0..vec.len());
                    vec[index]
                }
            };

            self.game.new_entity(EntityInfo { //TODO: with GameAction instead ?
                player: 0,
                position,
                speed: Vector2D::new(0.0, 0.0),
                mass,
                characteristics: self.game.settings.auto_spawn.characteristics.clone(),
                timer: self.game.settings.auto_spawn.timer.clone(),
                color,
                texture,
            });
        }
        //self.update_map();
        self.update_map_multithread_2();
        self.shrink_map();
        
    }

    #[allow(dead_code)]
    fn update_map(&mut self) {
        let entities = unsafe { &mut *(&self.game.entities as *const Entities as *mut Entities) };
        for entity in self.game.entities.core.iter_mut() {
            let moved = entities.flags[entity.index.main].contains(EntityFlags::MOVED);
            let mass_changed = entities.flags[entity.index.main].contains(EntityFlags::MASS_CHANGED);
            let collide = entities.flags[entity.index.main].contains(EntityFlags::COLLIDE);
            
            if moved == false && mass_changed == false { continue } // TODO: Refactor this, and detect if collide has gone from true to false!!
            self.game.map.matrix_simple.update_entity_2(entities, entity.index.main);
            if collide {
                self.game.map.matrix_physics.update_entity_2(entities, entity);
            }

            self.game.entities.flags[entity.index.main].remove(EntityFlags::MOVED);
            self.game.entities.flags[entity.index.main].remove(EntityFlags::MASS_CHANGED);
        }
    }

    #[allow(dead_code)]
    fn update_map_multithread(&mut self) {
        let game = unsafe_ptr::UnsafePtr::new(self.game);
        new_timer_monothread!(_t, "1");

        self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), 512, |index: usize| {
            let game = unsafe { game.ref_mut() };
            let moved = game.entities.flags[index].contains(EntityFlags::MOVED);
            let mass_changed = game.entities.flags[index].contains(EntityFlags::MASS_CHANGED);
            let collide = game.entities.flags[index].contains(EntityFlags::COLLIDE);

            if moved == false && mass_changed == false { return } // TODO: Refactor this, and detect if collide has gone from true to false!! And optimization if only mass_changed!!
            game.map.matrix_simple.update_entity_multithread_2(&mut game.entities, index);
            if collide {
                let entity_core = &mut game.entities.core[index];
                game.map.matrix_physics.update_entity_multithread_2(&self.game.entities, entity_core);
            }

            game.entities.flags[index].remove(EntityFlags::MOVED);
            game.entities.flags[index].remove(EntityFlags::MASS_CHANGED);
        });
    }

    fn update_map_multithread_2(&mut self) {
        let game = unsafe_ptr::UnsafePtr::new(self.game);
        {
            new_timer_monothread!(_t, "1");
            self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), 512, |index: usize| {
                let game = unsafe { game.ref_mut() };
                let moved = game.entities.flags[index].contains(EntityFlags::MOVED);
                let mass_changed = game.entities.flags[index].contains(EntityFlags::MASS_CHANGED);
                let collide = game.entities.flags[index].contains(EntityFlags::COLLIDE);

                if moved == false && mass_changed == false { return } // TODO: Refactor this, and detect if collide has gone from true to false!! And optimization if only mass_changed!!
                if game.map.matrix_simple.entity_has_moved(&self.game.entities, index) {
                    game.entities.flags[index].insert(EntityFlags::MATRIX_SIMPLE_TO_CHANGE);
                } else {
                    game.map.matrix_simple.update_entity_position_no_lock(&mut game.entities, index);
                }
                if collide {
                    let entity_core = &mut game.entities.core[index];
                    game.map.matrix_physics.update_entity_multithread_2(&self.game.entities, entity_core);
                }

                game.entities.flags[index].remove(EntityFlags::MOVED);
                game.entities.flags[index].remove(EntityFlags::MASS_CHANGED);
            });
        }

        {
            new_timer_monothread!(_t, "2");
            self.game.threadpool.compute_range_each_thread_join(0..self.game.entities.len(), 512, |index: usize| {
                let game = unsafe { game.ref_mut() };
                if game.entities.flags[index].contains(EntityFlags::MATRIX_SIMPLE_TO_CHANGE) {
                    game.entities.flags[index].remove(EntityFlags::MATRIX_SIMPLE_TO_CHANGE);
                    game.map.matrix_simple.update_entity_multithread(&mut game.entities, index);
                }
            });
        }
    }

    fn shrink_map(&mut self) {
        new_timer_monothread!(_t, "shrink_map");
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let i = rng.gen_range(0..self.game.map.matrix_simple.size.width);
            let j = rng.gen_range(0..self.game.map.matrix_simple.size.height);
            self.game.map.matrix_simple[i as usize][j as usize].shrink_to_fit();
        }


    }
}