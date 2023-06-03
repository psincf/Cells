use crate::prelude::*;
use euclid::default::Point2D;

pub struct CollisionsDetectionSolverInner<'a> {
    pub entity: &'a EntityCore,
    pub game: &'a Game,
}

impl<'a> CollisionsDetectionSolverInner<'a> {
    pub fn new(entity: &'a EntityCore, game: &'a Game) -> CollisionsDetectionSolverInner<'a> {
        CollisionsDetectionSolverInner {            
            entity,
            game,
        }
    }
    
    pub fn solve_multi_matrix(&mut self) { // TODO: Refactor this
        let entity = self.entity;
        let entity_position = self.game.entities.position[self.entity.index.main];
        let entity_mass = self.game.entities.mass[self.entity.index.main] as f32;
        let game = self.game;
        let mut entities = smallvec::SmallVec::<[usize; 20]>::new();
        let matrix = game.map.matrix_physics.matrix_with_index(entity.index.matrix_physics.inside.index as usize);
        for matrix_physics_index in entity.index.matrix_physics.inside.locations.iter() {
            let (x, y, _z) = *matrix_physics_index;

            let matrix_physics_xy = &matrix[x][y];
            for cell in matrix_physics_xy.iter() {
                if cell.entity <= entity.index.main { continue }
                if !test_collide((entity_position, entity_mass), (cell.position, cell.mass)) { continue }
                if !check_entity(game, self.entity.index.main, cell.entity) { continue }
                entities.push(cell.entity);
            }
        }

        for bigger_matrix in entity.index.matrix_physics.bigger.iter() {
            let matrix = game.map.matrix_physics.matrix_with_index(bigger_matrix.index as usize);
            for matrix_physics_index in bigger_matrix.locations.iter() {
                let (x, y) = *matrix_physics_index;
    
                let matrix_physics_xy = &matrix[x][y];
                for cell in matrix_physics_xy.iter() {
                    if !test_collide((entity_position, entity_mass), (cell.position, cell.mass)) { continue }
                    if !check_entity(game, self.entity.index.main, cell.entity) { continue }
                    entities.push(cell.entity);
                }
            }
        }

        entities.sort();
        entities.dedup();

        entity.colliding_info.colliding_pression_new.set(1.0);
        for other in entities {
            prepare_collide(game, self.entity.index.main, other);
        }
        entity.colliding_info.colliding_pression_new.set(entity.colliding_info.colliding_pression_new.get());
    }
}

#[inline]
fn test_collide((entity_position, entity_mass): (Point2D<i32>, f32), (other_position, other_mass): (Point2D<i32>, f32)) -> bool {
    let entity_radius = (entity_mass / std::f32::consts::PI).sqrt();
    let other_radius = (other_mass / std::f32::consts::PI).sqrt();
    let distance_to_collide = entity_radius + other_radius;
    let distance = (entity_position - other_position).to_f32().length();

    if distance < distance_to_collide { return true } else { return false }
}

#[inline]
fn check_entity(game: &Game, entity_index: usize, other_index: usize) -> bool {
    let entity = &game.entities.core[entity_index];
    //let entity_player = &game.players[entity.player];
    let entity_timer = &game.entities.timer[entity_index];
    let other = &game.entities.core[other_index];
    let other_timer = &game.entities.timer[other_index];

    if other.player != entity.player { return false }
    //if entity_player.kind == PlayerKind::Player && entity_timer.mergeable == None && other_timer.mergeable == None { return false } //TODO: Check if correct
    if entity_timer.mergeable == None && other_timer.mergeable == None && !entity.characteristics.collide_when_mergeable && !other.characteristics.collide_when_mergeable { return false } //TODO: Check if correct
    if entity_timer.collision.is_some() || other_timer.collision.is_some() { return false }
    return true;
}

fn prepare_collide(game: &Game, entity_index: usize, other_index: usize) {
    let entity = &game.entities.core[entity_index];
    let other = &game.entities.core[other_index];

    entity.colliding_info.insert_collider(other_index);
    other.colliding_info.insert_collider(entity_index);
    
    let entity_position = game.entities.position[entity_index];
    let other_position = game.entities.position[other.index.main];
    let distance_to_collide = game.entities.get_radius(entity.index.main) + game.entities.get_radius(other_index);
    let distance = (entity_position - other_position).to_f32().length();

    if distance < distance_to_collide {
        //entity.colliding_info.colliding_pression_new.set(entity.colliding_info.colliding_pression_new.get() + (distance_to_collide - distance) / distance_to_collide);
        entity.colliding_info.colliding_pression_new.set(entity.colliding_info.colliding_pression_new.get() + ((distance_to_collide - distance) / distance_to_collide) * 2.0 * other.colliding_info.colliding_pression.get().sqrt());
    } else {
        unreachable!();
    }
}