use crate::game::Game;
use crate::game::entity::EntityAction;
use crate::game::entity::DistanceRatio;

use euclid::default::Point2D;
use euclid::default::Vector2D;

pub struct GravitySolver<'a> {
    entity_index: usize,
    game: &'a Game,
}

impl<'a> GravitySolver<'a> {
    #[inline]
    pub fn new(entity_index: usize, game: &'a Game) -> GravitySolver<'a> {
        GravitySolver {
            entity_index,
            game,
        }
    }

    pub fn solve_3(&self) {
        let info = self.game.entities.core[self.entity_index].characteristics.gravity.as_ref().unwrap();
        let entity_position = self.game.entities.position[self.entity_index];
        let entity_mass = self.game.entities.mass[self.entity_index];

        if info.distance_limit.end == std::f32::MAX {
            for other in 0..self.game.entities.len() { //TODO: better
                apply_gravity(self.game, &info, self.entity_index, entity_position, entity_mass, other);
            }
        } else {
            let distance = info.distance_limit.end;
            let (x_field, y_field, _z) = self.game.entities.index_matrix_simple[self.entity_index].xyz();
            let scope_field = (distance / self.game.map.matrix_simple.size_field as f32) as i32 + 1;
            
            let x_min = (x_field as i32 - scope_field).max(0) as usize;
            let x_max = (x_field as i32 + scope_field).min(self.game.map.matrix_simple.size.width - 1) as usize;
            let y_min = (y_field as i32 - scope_field).max(0) as usize;
            let y_max = (y_field as i32 + scope_field).min(self.game.map.matrix_simple.size.height - 1) as usize;

            
            for x in x_min..=x_max { // Don't change ..= to .. !! We need to take x_max !
                for y in y_min..=y_max {
                    for cell in self.game.map.matrix_simple[x][y].iter() { //TODO: optimize by storing info of entity directly in Matrix ?? More cache efficient
                        apply_gravity(self.game, &info, self.entity_index, entity_position, entity_mass, cell.entity);
                    }
                }
            }
        }

        fn apply_gravity(game: &Game, info: &crate::game::entity::EntityGravityInfo, entity_index: usize, entity_position: Point2D<i32>, entity_mass: i64, other_index: usize) {
            let other_position = game.entities.position[other_index];
            if other_index == entity_index { return }
            if !game.entities.flags[other_index].contains(crate::game::entity::EntityFlags::MOVABLE) { return }
            let distance = entity_position - other_position; if distance == euclid::default::Vector2D::zero() { return }
            let distance_f32 = distance.to_f32();
            let distance_length = distance_f32.length();

            if distance_length < info.distance_limit.start { return }
            if distance_length > info.distance_limit.end { return }
            
            let direction = distance_f32.normalize();

            let (distance_ratio, power_ratio) = match info.distance_ratio {
                DistanceRatio::Linear => (1, 0.05),
                DistanceRatio::Squared => (2, 500.0),
            };
            let velocity_before_clamp = distance_length.max(info.distance_clamp.start).min(info.distance_clamp.end).recip().powi(distance_ratio) * entity_mass as f32 * info.power * power_ratio;
            let velocity_after_clamp = velocity_before_clamp.signum() * velocity_before_clamp.abs().min(info.speed_clamp.end).max(info.speed_clamp.start);

            let velocity = direction * velocity_after_clamp;

            //if velocity.length() < info.speed_accepted_min { return; }

            game.entities.send_buffer(other_index, EntityAction::AddSpeed(velocity.x, velocity.y));
            //unsafe { (*(&game.entities.speed[other_index] as *const _ as *mut Vector2D<f32>)) += velocity; }
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn solve_per_entity(&self, others: &[usize]) {
        if !self.game.entities.flags[self.entity_index].contains(crate::game::entity::EntityFlags::MOVABLE) { return }

        let entity_position = self.game.entities.position[self.entity_index];

        for i in 0..others.len() { //TODO: better
            let other_index = others[i];
            if other_index == self.entity_index { continue }

            let other_position = self.game.entities.position[other_index];
            let other_mass = self.game.entities.mass[other_index];
            let other_gravity_info = self.game.entities.core[other_index].characteristics.gravity.as_ref().unwrap();

            let distance = other_position - entity_position; if distance == euclid::default::Vector2D::zero() { continue }
            let distance_f32 = distance.to_f32();
            let distance_length = distance_f32.length();

            if distance_length < other_gravity_info.distance_limit.start { continue }
            if distance_length > other_gravity_info.distance_limit.end { continue }
            
            let direction = distance_f32.normalize();

            let (distance_ratio, power_ratio) = match other_gravity_info.distance_ratio {
                DistanceRatio::Linear => (1, 0.05),
                DistanceRatio::Squared => (2, 500.0),
            };

            let velocity_before_clamp = distance_length.max(other_gravity_info.distance_clamp.start).min(other_gravity_info.distance_clamp.end).recip().powi(distance_ratio) * other_mass as f32 * other_gravity_info.power * power_ratio;
            let velocity_after_clamp = velocity_before_clamp.signum() * velocity_before_clamp.abs().min(other_gravity_info.speed_clamp.end).max(other_gravity_info.speed_clamp.start);

            let velocity = direction * velocity_after_clamp;

            //if velocity.length() < other_gravity_info.speed_accepted_min { continue; }

            //self.game.entities.send_buffer(other, EntityAction::AddSpeed(velocity.x, velocity.y));
            unsafe { (*(&self.game.entities.speed[self.entity_index] as *const _ as *mut Vector2D<f32>)) += velocity; }
        }
    }
}