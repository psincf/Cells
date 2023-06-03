use crate::prelude::*;

use crate::game::entity::EntityFlags;
use crate::utils::BufferChoice; //TODO: bad

use euclid::default::{Point2D, Vector2D};
use std::sync::Arc;

pub struct CacheEntitySolver;

impl CacheEntitySolver {
    pub fn solve(game_ptr: *mut Game, entity_index: usize, buffer: BufferChoice) {
        let game = unsafe { &mut *game_ptr };
        let entity = unsafe { &mut (*game_ptr).entities.core[entity_index] };
        let entity_position = unsafe { &mut (*game_ptr).entities.position[entity_index] };
        let entity_mass = unsafe { &mut (*game_ptr).entities.mass[entity_index] };
        let entity_speed = unsafe { &mut (*game_ptr).entities.speed[entity_index] };
        let entity_timer = unsafe { &mut (*game_ptr).entities.timer[entity_index] };
        let entity_flags = unsafe { &mut (*game_ptr).entities.flags[entity_index] };
        let drawable_entity = unsafe { &mut (*game_ptr).entities.drawable_entities[entity_index] };
        match buffer {
            BufferChoice::First => {
                for action in game.entities.receive_buffer(entity_index) {
                    use EntityAction::*;
                    match action {
                        AddCollisionRatioTime(time) => {
                            let old_time = entity_timer.collision_ratio.unwrap_or(0);
                            let new_time = old_time + time;
                            if new_time > 0 {
                                entity_timer.collision_ratio = Some(new_time);
                            } else {
                                entity_timer.collision_ratio = None;
                            }
                        }
                        AddCollisionTime(time) => {
                            let old_time = entity_timer.collision.unwrap_or(0);
                            let new_time = old_time + time;
                            if new_time > 0 {
                                entity_timer.collision = Some(new_time);
                            } else {
                                entity_timer.collision = None;
                            }
                        }
                        AddPosition(mut x, mut y) => {
                            if x == 0 || y == 0 { continue }
                            x = x.min(1_000_000_000).max(-1_000_000_000);
                            y = y.min(1_000_000_000).max(-1_000_000_000);
                            entity_position.x += x;
                            entity_position.y += y;

                            if entity_flags.contains(EntityFlags::BOUNCE) {
                                crate::game_solver::entity_solver::position_solver::bounce(entity_position, entity_speed);
                            } else {
                                crate::game_solver::entity_solver::position_solver::not_bounce(entity_position, entity_speed);
                            }
                            
                            entity_flags.insert(EntityFlags::MOVED);
                            drawable_entity.position = *entity_position;
                        }
                        AddInertiaTime(time) => {
                            let old_time = entity_timer.inertia.unwrap_or(0);
                            let new_time = old_time + time;
                            if new_time > 0 {
                                entity_timer.inertia = Some(new_time);
                            } else {
                                entity_timer.inertia = None;
                            }
                        }
                        AddLifetimeLeftTime(time) => {
                            let old_time = entity_timer.lifetime_left.unwrap();
                            let new_time = old_time + time;
                            entity_timer.lifetime_left = Some(new_time);
                        }
                        AddMergeableTime(time) => {
                            let old_time = entity_timer.mergeable.unwrap_or(0);
                            let new_time = old_time + time;
                            if new_time > 0 {
                                entity_timer.mergeable = Some(new_time);
                            } else {
                                entity_timer.mergeable = None;
                            }
                        }
                        AddMass(mass) => {
                            *entity_mass += mass;
                            *entity_mass = (*entity_mass).max(entity.characteristics.mass_min);
                            *entity_mass = (*entity_mass).min(entity.characteristics.mass_max);
                            if *entity_mass < 0 { *entity_mass = 1; }
                            entity_flags.insert(EntityFlags::MASS_CHANGED);
                            
                            drawable_entity.mass = *entity_mass as f32;
                        },
                        AddSpeed(mut x, mut y) => {
                            x = x.min(1_000_000_000.0).max(-1_000_000_000.0);
                            y = y.min(1_000_000_000.0).max(-1_000_000_000.0);                                
                            entity_speed.x += x;
                            entity_speed.y += y;

                            entity_speed.x = entity_speed.x.min(1_000_000_000.0).max(-1_000_000_000.0);
                            entity_speed.y = entity_speed.y.min(1_000_000_000.0).max(-1_000_000_000.0);
                            
                            if entity_speed.x.abs() < 1.0 { entity_speed.x = 0.0; } 
                            if entity_speed.y.abs() < 1.0 { entity_speed.y = 0.0; } 
                            /*
                            let (x, y) = (x as f32, y as f32);
                            let temp_x = (entity_speed.x as f32).powf(2.0) * (entity_speed.x as f32).signum() + x.powf(2.0) * x.signum();
                            let temp_y = (entity_speed.y as f32).powf(2.0) * (entity_speed.y as f32).signum() + y.powf(2.0) * y.signum();

                            entity_speed.x = ((temp_x.abs()).sqrt() as i32) * (temp_x as i32).signum();
                            entity_speed.y = ((temp_y.abs()).sqrt() as i32) * (temp_y as i32).signum();
                            */
                        },
                        Killed(killer_index) => {
                            if entity.killed == None {
                                //TODO: decide who kill with mass ?
                                entity.killed = Some(killer_index);
                                //game.buffer.send(GameAction::KillEntity(Arc::downgrade(&entity.index.main_ptr)));
                                game.buffer_kill_entity.send(Arc::downgrade(&entity.index.main_ptr));
                            }
                        }
                        MulSpeed(_x, _y) => {
                            panic!()
                        }
                        KilledConfirmed(_killer_index) => {
                            panic!()
                        }
                        SetColor(color) => {
                            entity.color = color;
                            drawable_entity.color = unsafe { std::mem::transmute(color.center) };
                            drawable_entity.color_2 = unsafe { std::mem::transmute(color.edge) };
                        }
                        Split(count) => {
                            let mut rng = rand::thread_rng();
                            use rand::Rng;
                            for i in 0..count {
                                if game.players[entity.player].entities.len() + i >= game.settings.max_split { break; } //TODO: good ? Not good: Not atomic!! What if multiple Split at the same time
                                let max_mass_taken = *entity_mass / 2;
                                if max_mass_taken > entity.characteristics.mass_min {
                                    let mass_taken = rng.gen_range(entity.characteristics.mass_min..max_mass_taken);
                                    *entity_mass -= mass_taken;

                                    let direction = rng.gen_range(0.0..360.0);
                                    let length = game.entities.get_radius(entity_index) / 100.0;
                                    let distance = Vector2D::from_angle_and_length(euclid::Angle::degrees(direction), length);
                                    let distance_i32 = distance.to_i32();

                                    let new_entity_info = EntityInfo {
                                        player: entity.player,
                                        position: Point2D::new(entity_position.x + distance_i32.x, entity_position.y + distance_i32.y),
                                        speed: Vector2D::new(distance.x, distance.y),
                                        mass: mass_taken,
                                        characteristics: entity.characteristics.clone(),
                                        timer: EntityTimer {
                                            mergeable: Some(1_000), // TODO: change
                                            inertia: Some(20), // TODO: change
                                            collision: Some(1),
                                            collision_ratio: Some(50),
                                            ..Default::default()
                                        },
                                        color: entity.color,
                                        texture: entity.index.texture,
                                    };
                                    //game.buffer.send(GameAction::AddEntity(Box::new(new_entity_info)));
                                    game.buffer_add_entity.send(Box::new(new_entity_info));
                                }
                            }
                        }
                    }
                }
                if let Some(killer_index) = entity.killed {
                    game.entities.send_buffer2(entity_index, EntityAction::KilledConfirmed(killer_index));
                }
            }
    
            BufferChoice::Second => {
                for action in game.entities.receive_buffer2(entity_index) {
                    use EntityAction::*;
                    match action {
                        MulSpeed(x, y) => {
                            entity_speed.x *= x;
                            entity_speed.y *= y;
                            
                            if entity_speed.x.abs() < 0.1 { entity_speed.x = 0.0; } 
                            if entity_speed.y.abs() < 0.1 { entity_speed.y = 0.0; } 
                        }
                        KilledConfirmed(mut killer_index) => { //TODO: Reminder: 2 Cells can't kill each other -> infinite loop
                            if killer_index == entity.index.main { continue } //TODO: Killed by itself
                            let mut killer = &game.entities.core[killer_index];
                            loop {
                                if let Some(new_killer) = killer.killed {
                                    if new_killer == killer_index { game.entities.send_buffer(killer.index.main, EntityAction::AddMass(*entity_mass)); break } //TODO: Killed by itself. Badly done !!
                                    killer_index = new_killer;
                                    killer = &game.entities.core[killer_index];
                                    continue
                                }
                                game.entities.send_buffer(killer.index.main, EntityAction::AddMass(*entity_mass));
                                break
                            }
                            if let Some(on_death) = entity.characteristics.on_death.clone() {
                                match on_death {
                                    OnDeathEffect::Split(count) => {
                                        if !killer.characteristics.can_split_on_kill { return }
                                        game.entities.send_buffer(killer.index.main, EntityAction::Split(count));
                                    }
                                }
                            }
                        }
                        _ => {
                            panic!()
                        }
                    }
                }
            }
        }
    }
}