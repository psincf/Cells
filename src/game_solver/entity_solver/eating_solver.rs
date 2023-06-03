use crate::prelude::*;

use euclid::default::Point2D;

pub struct EatingSolver<'a> {
    entity: &'a EntityCore,
    game: &'a Game,
}

impl<'a> EatingSolver<'a> {
    #[inline]
    pub fn new(entity: &'a EntityCore, game: &'a Game) -> EatingSolver<'a> {
        EatingSolver {
            entity,
            game,
        }
    }
    pub fn solve(&self) {
        if self.game.entities.mass[self.entity.index.main] > crate::game::entity::RATIO_MASS * 10_000 {
            self.solve_v1();
        } else {
            self.solve_v2();
        }
    }

    pub fn solve_v1(&self) {
        let entity = self.entity;
        let game = self.game;

        let (x_field, y_field, _z) = game.entities.index_matrix_simple[entity.index.main].xyz();
        let radius = self.game.entities.get_radius(entity.index.main);
        let scope_field = (radius / game.map.matrix_simple.size_field as f32) as i32 + 1;
        
        let x_min = (x_field as i32 - scope_field).max(0);
        let x_max = (x_field as i32 + scope_field).min(game.map.matrix_simple.size.width - 1);
        let y_min = (y_field as i32 - scope_field).max(0);
        let y_max = (y_field as i32 + scope_field).min(game.map.matrix_simple.size.height - 1);

        let entity_position = game.entities.position[entity.index.main];

        for x in x_min..=x_max { // Don't change ..= to .. !! We need to take x_max !
            for y in y_min..=y_max {
                for cell in game.map.matrix_simple[x as usize][y as usize].iter() { //TODO: optimize by storing info of entity directly in Matrix ?? More cache efficient
                    if cell.entity == entity.index.main { continue }
                    let other = &game.entities.core[cell.entity];
                    if !check_position(entity_position, radius, cell.position) { continue }
                    manage_kill(game, entity, other);
                }
            }
        }
    }

    pub fn solve_v2(&self) {
        let entity = self.entity;
        let game = self.game;
        let cells = game.map.matrix_simple.intersect_with(&game.entities, entity.index.main);

        let entity_position = game.entities.position[entity.index.main];
        let radius = self.game.entities.get_radius(entity.index.main);

        for (x, y) in cells {
            for cell in game.map.matrix_simple[x as usize][y as usize].iter() { //TODO: optimize by storing info of entity directly in Matrix ?? More cache efficient
                if cell.entity == entity.index.main { continue }
                let other = &game.entities.core[cell.entity];
                if !check_position(entity_position, radius, cell.position) { continue }
                manage_kill(game, entity, other);
            }
        }
    }
}

#[inline]
fn check_position(position_1: Point2D<i32>, radius_1: f32, position_2: Point2D<i32>) -> bool {
    return (position_1 - position_2).to_f32().length() < radius_1
}
    
fn manage_kill(game: &Game, entity: &EntityCore, other: &EntityCore) { //TODO: bad name and bad design ?
    let mut killed = false;

    let entity_mass = game.entities.mass[entity.index.main];
    let entity_timer = &game.entities.timer[entity.index.main];

    let other_mass = game.entities.mass[other.index.main];
    let other_timer = &game.entities.timer[other.index.main];

    if entity.player == other.player {
        if !other.characteristics.mergeable { return }
        if entity_timer.mergeable == None && other_timer.mergeable == None {
            if entity_mass > other_mass {
                killed = true;
            } else if entity_mass == other_mass {
                if entity.index.main > other.index.main {
                    killed = true;
                }
            }
        }
    } else {
        if other.characteristics.invincible == true { return }
        if entity_mass as f32 > other_mass as f32 * 1.33 {
            killed = true;
        }
    }
    
    if killed {
        let entity_position = game.entities.position[entity.index.main];
        let other_position = game.entities.position[other.index.main];
        let radius_entity = game.entities.get_radius(entity.index.main);
        let radius_other = game.entities.get_radius(other.index.main);
        let scope = radius_entity - 0.5 * radius_other;
        if (entity_position - other_position).to_f32().length() < scope {
            game.entities.send_buffer(other.index.main, EntityAction::Killed(entity.index.main));
        }
    }
}