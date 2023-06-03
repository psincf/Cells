use crate::prelude::*;

use euclid::default::{Point2D, Vector2D};

pub struct CollisionsReactionSolver<'a> {
    entity: &'a EntityCore,
    game: &'a Game,
}

impl<'a> CollisionsReactionSolver<'a> {
    pub fn new(entity: &'a EntityCore, game: &'a Game) -> CollisionsReactionSolver<'a> {
        CollisionsReactionSolver {
            entity: entity,
            game: game,
        }
    }

    pub fn solve(&self) { //TODO: Better physics engine
        self.solve_pression_average();
        //self.solve_basic();
    }

    fn solve_pression_average(&self) {
        let entity = self.entity;
        let entity_mass = self.game.entities.mass[entity.index.main];
        let entity_radius = self.game.entities.get_radius(entity.index.main);
        let entity_collision_ratio = self.game.entities.timer[entity.index.main].collision_ratio.unwrap_or(1);
        let mut total_count = 0.0;
        let mut new_pression = 1.0;
        let mut total_moving = Vector2D::zero();
        let mut total_speed = Vector2D::zero();
        for other_index in entity.colliding_info.entities_colliding().iter() {
            let other = &self.game.entities.core[*other_index];
            let other_mass = self.game.entities.mass[*other_index];
            let other_radius = self.game.entities.get_radius(*other_index);
            let other_collision_ratio = self.game.entities.timer[*other_index].collision_ratio.unwrap_or(1);
            let ratio_collision_finale = entity_collision_ratio.max(other_collision_ratio);
            let pression = other.colliding_info.colliding_pression.get().min(10E20);
            let distance_to_collide = entity_radius + other_radius;

            // Compute distance
            //let distance = (entity.position - other.position).to_f32();
            let distance = (entity.colliding_info.colliding_position.get() - other.colliding_info.colliding_position.get()).to_f32();
            let distance_difference = distance_to_collide - distance.length(); if distance_difference < 0.0 { continue };
            new_pression += (distance_difference / distance_to_collide) * 2.0 * pression.sqrt();

            let ratio_mass = (other_mass as f32 * pression / (entity_mass as f32 + other_mass as f32 * pression) as f32).min(0.99);
            let ratio_count = 1.0 / (1.0 - ratio_mass);
            total_count += ratio_count;
            let max_speed = distance_difference * ratio_mass * pression.sqrt().sqrt().min(4.0) / ratio_collision_finale as f32;
            let ratio_speed = max_speed / distance.length();

            // Collision speed
            let mut moving = Vector2D::new(distance.x, distance.y) * ratio_speed;
            moving *= self.game.settings.collision_speed;
            
            // Speed with mass ratio
            moving *= ratio_count;

            use rand::Rng;
            let mut rng = rand::thread_rng();
            
            if distance.x.abs() < 5.0 {
                moving.x = rng.gen_range(-5.0..5.0) * ratio_count;
            }
            if distance.y.abs() < 5.0 {
                moving.y = rng.gen_range(-5.0..5.0) * ratio_count;
            }
            /*
            if distance.length() < 20.0 {
                moving = Vector2D::new(
                    rng.gen_range(-50.0..50.0) * ratio_mass,
                    rng.gen_range(-50.0..50.0) * ratio_mass
                );
            }
            */
            total_moving += moving;
            total_speed += moving;
        }

        if total_count > 0.0 {
            total_moving /= total_count;
            total_speed /= total_count;
            /*
            entity.buffer.send(EntityAction::AddPosition(total_moving.x as i32, total_moving.y as i32));
            entity.buffer.send(EntityAction::AddSpeed(total_speed.x as i32, total_speed.y as i32));
            */
            let total_moving = round_vector(total_moving);
            let total_speed = total_moving / 2.0;
            entity.colliding_info.colliding_position_new.set(entity.colliding_info.colliding_position.get() + total_moving.to_i32());
            entity.colliding_info.colliding_position_new.set(
                Point2D::new(
                entity.colliding_info.colliding_position_new.get().x.max(0).min(self.game.map.max().width),
                entity.colliding_info.colliding_position_new.get().y.max(0).min(self.game.map.max().height)
                )
            );
            // TODO: speed badly calculated (and not used by the way)
            entity.colliding_info.colliding_speed_new.set(entity.colliding_info.colliding_speed.get() + total_speed);
        }
        entity.colliding_info.colliding_pression_new.set(new_pression);
    }

    #[allow(dead_code)]
    fn solve_basic(&self) {
        let entity = self.entity;
        let entity_mass = self.game.entities.mass[entity.index.main];
        let entity_radius = self.game.entities.get_radius(entity.index.main);
        let mut total_count = 0;
        let mut total_moving = Vector2D::zero();
        let mut total_speed = Vector2D::zero();
        for other_index in entity.colliding_info.entities_colliding().iter() {
            total_count += 1;
            let other = &self.game.entities.core[*other_index];
            let other_mass = self.game.entities.mass[*other_index];
            let other_radius = self.game.entities.get_radius(*other_index);
            let distance_to_collide = entity_radius + other_radius;
            // Compute distance
            //let distance = (entity.position - other.position).to_f32();
            let distance = (entity.colliding_info.colliding_position.get() - other.colliding_info.colliding_position.get()).to_f32();
            let ratio_mass = (other_mass as f32 / (entity_mass as f32 + other_mass as f32) as f32).min(0.99);
            let distance_difference = distance_to_collide - distance.length(); if distance_difference < 0.0 { continue };
            let max_speed = distance_difference;
            let ratio_speed = max_speed / distance.length();

            // Collision speed
            let mut moving = Vector2D::new(distance.x, distance.y) * ratio_speed;
            moving *= self.game.settings.collision_speed;
            
            // Speed with mass ratio
            moving *= ratio_mass;

            use rand::Rng;
            let mut rng = rand::thread_rng();
            if distance.x.abs() < 5.0 {
                moving.x = rng.gen_range(-5.0..5.0) * ratio_mass;
            }
            if distance.y.abs() < 5.0 {
                moving.y = rng.gen_range(-5.0..5.0) * ratio_mass;
            }
            
            /*
            if distance.length() < 20.0 {
                moving = Vector2D::new(
                    rng.gen_range(-50.0..50.0) * ratio_mass,
                    rng.gen_range(-50.0..50.0) * ratio_mass
                );
            }
            */
            total_moving += moving;
            total_speed += moving / 10.0;
        }

        if total_count > 0 {
            /*
            entity.buffer.send(EntityAction::AddPosition(total_moving.x as i32, total_moving.y as i32));
            entity.buffer.send(EntityAction::AddSpeed(total_speed.x as i32, total_speed.y as i32));
            */
            entity.colliding_info.colliding_position_new.set(entity.colliding_info.colliding_position.get() + total_moving.to_i32());
            entity.colliding_info.colliding_speed_new.set(entity.colliding_info.colliding_speed.get() + total_speed);
        }
    }
    /*
    #[allow(dead_code)]
    fn solve_basic_energy(&self) {
        let entity = self.entity;
        let entity_speed = self.game.entities.speed[entity.index.main];
        let entity_direction = entity_speed.normalize();
        let original_energy = entity_speed.length().powi(2) * entity.mass as f32;
        let mut actual_energy = to_energy(entity.mass, entity_speed);
        for &other_index in entity.colliding_info.entities_colliding().iter() {
            let other = &self.game.entities.core[other_index];
            let other_speed = self.game.entities.speed[other_index];
            let other_energy = to_energy(other.mass, other_speed);

            let distance_to_collide = entity.get_radius() + other.get_radius();
            // Compute distance
            //let distance = (entity.position - other.position).to_f32();
            let distance = (entity.colliding_info.colliding_position.get() - other.colliding_info.colliding_position.get()).to_f32();
            let ratio_mass = (other.mass as f32 / (entity.mass as f32 + other.mass as f32) as f32).min(0.99);
            let distance_difference = distance_to_collide - distance.length(); if distance_difference < 0.0 { continue };
            let max_speed = distance_difference;
            let ratio_speed = max_speed / distance.length();

            // Collision speed
            let mut moving = Vector2D::new(distance.x, distance.y) * ratio_speed;
            moving *= self.game.settings.collision_speed;
            
            // Speed with mass ratio
            moving *= ratio_mass;

            use rand::Rng;
            let mut rng = rand::thread_rng();

            if distance.x.abs() < 5.0 {
                moving.x = rng.gen_range(-100.0, 100.0) * ratio_mass;
            }
            if distance.y.abs() < 5.0 {
                moving.y = rng.gen_range(-100.0, 100.0) * ratio_mass;
            }

            let angle_collision = distance.angle_to(entity_direction).to_degrees();
            let energy_left = (ratio_mass * 2.0) * actual_energy * (1.0 - angle_collision.abs() / 90.0).max(0.0);
            let other_new_speed = self.game.entities.speed[other_index];

            //other.colliding_info.
        }
    }
    */
}

#[inline]
fn round_vector(vector: Vector2D<f32>) -> Vector2D<f32> {
    let vector_sign = Vector2D::new(vector.x.signum(), vector.y.signum());
    let vector = Vector2D::new(
        vector_sign.x * vector.x.abs().ceil(),
        vector_sign.y * vector.y.abs().ceil()
    );
    return vector;
}

#[inline]
#[allow(dead_code)]
fn to_energy(mass: i64, velocity: Vector2D<f32>) -> f32 {
    let energy = velocity.length().powi(2) * mass as f32;
    return energy;
}

/*
#[inline]
fn to_velocity(energy: f32, direction: Vector2D<f32>, distance: Vector2D<f32>) -> Vector2D<f32> {

}
*/