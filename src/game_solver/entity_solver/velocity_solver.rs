use crate::prelude::*;

use euclid::default::Vector2D;

pub struct VelocitySolver<'a> {
    entity_index: usize,
    game: &'a mut Game,
}

impl<'a> VelocitySolver<'a> {
    #[inline]
    pub fn new(entity_index: usize, game: &'a mut Game) -> VelocitySolver<'a> {
        VelocitySolver {
            entity_index,
            game,
        }
    }

    #[allow(dead_code)]
    pub fn solve(&self) {
        if let Some(new_speed) = self.compute_speed() {
            let old_entity_speed = self.game.entities.speed[self.entity_index];
            self.game.entities.send_buffer(self.entity_index, EntityAction::AddSpeed(new_speed.x - old_entity_speed.x, new_speed.y - old_entity_speed.y));
        }
    }

    pub fn solve_2(&mut self) {
        if let Some(new_speed) = self.compute_speed() {
            let mut new_speed = new_speed.min(Vector2D::new(1_000_000_000.0, 1_000_000_000.0)).max(Vector2D::new(-1_000_000_000.0, -1_000_000_000.0));
            if new_speed.length() < 1.0 { new_speed = Vector2D::zero(); }
            let entity_speed_mut = &mut self.game.entities.speed[self.entity_index];
            *entity_speed_mut = new_speed;
        }
    }

    #[inline]
    fn compute_speed(&self) -> Option<Vector2D<f32>> {
        let moving = self.compute_moving();
        let entity_speed = self.game.entities.speed[self.entity_index];
        if moving == Vector2D::zero() && entity_speed == Vector2D::zero() { return None }

        let entity_core = &self.game.entities.core[self.entity_index];
        let entity_timer = &self.game.entities.timer[self.entity_index];

        let inertia = entity_core.characteristics.inertia + entity_timer.inertia.unwrap_or(1);
        let inertia_ratio = 1.0 - 1.0 / inertia as f32;
        let new_speed = entity_speed * inertia_ratio + moving * (1.0 - inertia_ratio);

        return Some(new_speed);
    }

    #[inline]
    fn compute_moving(&self) -> Vector2D<f32> {
        let entity_direction = &self.game.entities.direction[self.entity_index];
        if let Some(direction) = entity_direction {
            //let entity_core = &self.game.entities.core[self.entity_index];
            let entity_mass = self.game.entities.mass[self.entity_index];
            let distance = *direction - self.game.entities.position[self.entity_index];
            if distance == Vector2D::zero() { return Vector2D::zero(); }
            let distance_f32 = distance.to_f32();
            let distance_length = distance_f32.length();

            // progressive acceleration
            let distance_before_slow = 2_000.0;
            let delta_speed = distance_length.min(distance_before_slow) / distance_before_slow;

            // Compute speed
            let speed = delta_speed * self.game.settings.unit_speed / ((entity_mass as f32).log10());
            
            // Compute moving
            let angle = distance_f32.normalize();
            let moving = angle * speed;
            return moving;
        } else {
            return Vector2D::zero();
        };
    }
}