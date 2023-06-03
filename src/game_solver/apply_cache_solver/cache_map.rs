/*
use crate::game::entity::index::MatrixIndex;
use crate::game::Game;
use crate::game::map::MapAction;
use crate::new_timer_monothread;
use std::sync::atomic::Ordering;

pub struct CacheMapSolver<'a> {
    game: &'a mut Game
}

impl<'a> CacheMapSolver<'a> {
    pub fn new(game: &'a mut Game) -> CacheMapSolver {
        CacheMapSolver {
            game,
        }
    }
    pub fn solve(&mut self) { //TODO: Still monothread / also maybe delete this ?
        new_timer_monothread!(_t, "apply_cache_map");
        let game_bis = unsafe { &*(self.game as *const Game) };
        let mut entity_killed_matrix = Vec::new();
        let mut entity_killed_matrix_physics = Vec::new();
        for action in self.game.map.buffer.receive() {
            match action {
                MapAction::AddEntity(index_main) => {
                    let entity = &mut self.game.entities.core[index_main];
                    self.game.map.add_entity(&game_bis.entities, entity);
                }
                
                MapAction::KillEntity(matrix_index_ptr) => {
                    entity_killed_matrix.push((matrix_index_ptr).0);
                    if matrix_index_ptr.1.index != 0 {
                        entity_killed_matrix_physics.append(&mut matrix_index_ptr.1.flat());
                    }
                }
                
                MapAction::Move(weak_ptr) => { //TODO: can be optimized without checking if unit is alive with optimization in CacheGameSolver
                    if let Some(entity_index) = weak_ptr.upgrade() {
                        let entity_index = entity_index.load(Ordering::Relaxed);
                        let entity = &mut self.game.entities.core[entity_index];
                        let (x_field, y_field, z) = entity.index.matrix.xyz();
                        self.game.map.matrix_simple[x_field][y_field][z] = entity.index.main;

                        assert!(!(entity.index.matrix_physics.inside.index == 0 && entity.characteristics.collide));
                        //if entity.index.matrix_physics.inside.index != 0 {
                        if entity.characteristics.collide {
                            let matrix_physics = self.game.map.matrix_physics.matrix_with_index_mut(entity.index.matrix_physics.inside.index as usize);
                            for matrix_index in entity.index.matrix_physics.inside.locations.iter() {
                                let (x, y, z) = *matrix_index;
                                matrix_physics[x][y][z].entity = entity_index;
                            }
                        }
                    }
                }
            }
        }
        new_timer_monothread!(_t, "apply_cache_map_kill");
        self.kill_entity(entity_killed_matrix, entity_killed_matrix_physics);
    }

    pub fn kill_entity(&mut self, mut entity_killed_matrix: Vec<MatrixIndex>, mut entity_killed_matrix_physics: Vec<(i32, (usize, usize, usize))>) {
        entity_killed_matrix.sort_unstable_by( |a, b|
            if a.x < b.x { std::cmp::Ordering::Greater } else if a.x > b.x { std::cmp::Ordering::Less } else {
                if a.y < b.y { std::cmp::Ordering::Greater } else if a.y > b.y { std::cmp::Ordering::Less } else {
                    if a.z < b.z { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Less }
                }
            }
        ); // Invert sort: Necessary in order to be sure that a potential moved index with swap_remove is a valid index.
        
        
        entity_killed_matrix_physics.sort_unstable_by( |a, b|
            if a.0 < b.0 { std::cmp::Ordering::Greater } else if a.0 > b.0 { std::cmp::Ordering::Less } else {
                if a.1 < b.1 { std::cmp::Ordering::Greater } else  { std::cmp::Ordering::Less }
            }
        ); // Invert sort: Necessary in order to be sure that a potential moved index with swap_remove is a valid index.
        

        for matrix_index in entity_killed_matrix {
            let (x_field, y_field, z) = matrix_index.xyz();
            self.game.map.matrix_simple[x_field][y_field].remove(z);

            // Check if another entity have its matrix_index swapped
            if z != self.game.map.matrix_simple[x_field][y_field].len() {
                let entity = &mut self.game.entities.core[self.game.map.matrix_simple[x_field][y_field][z]];
                entity.index.matrix.z = z;
            }
        }

        for matrix_index in entity_killed_matrix_physics {
            let matrix = self.game.map.matrix_physics.matrix_with_index_mut(matrix_index.0 as usize);
            let (x_field, y_field, z) = matrix_index.1;
            matrix[x_field][y_field].remove(z);

            // Check if another entity have its matrix_index swapped
            if z != matrix[x_field][y_field].len() {
                let matrix_cell = &matrix[x_field][y_field][z];
                let entity = &mut self.game.entities.core[matrix_cell.entity];
                entity.index.matrix_physics.inside.locations[matrix_cell.cell_index].2 = z;
            }
        }
    }
}
*/