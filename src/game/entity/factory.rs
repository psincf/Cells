use super::entities::Entities;
use super::index::EntityIndex;
use super::index::MatrixIndex;

use crate::prelude::*;

use std::ops::Range;

use buffer::Buffer;
use euclid::default::{Point2D, Rect, Size2D, Vector2D};
use parking_lot::{Mutex, MutexGuard};
use rustc_hash::FxHashSet;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub struct EntityInfo {
    pub player: usize,
    pub position: Point2D<i32>,
    pub speed: Vector2D<f32>,
    pub mass: i64,
    pub color: super::EntityColor,
    pub timer: EntityTimer,
    pub characteristics: EntityCharacteristics,
}

impl EntityInfo {
    pub fn from_entity(game: &Game, entity_index: usize) -> EntityInfo {
        let entity_core = &game.entities.core[entity_index];
        let entity_position = game.entities.position[entity_index];
        let entity_speed = game.entities.speed[entity_index];
        let entity_mass = game.entities.mass[entity_index];
        let entity_timer = &game.entities.timer[entity_index];
        EntityInfo {
            player: entity_core.player,
            position: entity_position,
            speed: entity_speed,
            mass: entity_mass,
            color: entity_core.color.clone(),
            timer: EntityTimer {
                collision: entity_timer.collision.clone(),
                mergeable: entity_timer.mergeable.clone(),
                inertia: entity_timer.inertia.clone(),
                lifetime_left: entity_timer.lifetime_left.clone(),
            },
            characteristics: EntityCharacteristics::from_other(&entity_core.characteristics)
        }
    }
}

bitflags::bitflags! {
    pub struct EntityFlags: u32 {
        const GRAVITY = 0b0000_0000_0000_0000_0000_0000_0000_0001; //TODO: update when changed
        const THROW = 0b0000_0000_0000_0000_0000_0000_0000_0010; //TODO: update when changed
        const EATER = 0b0000_0000_0000_0000_0000_0000_0000_0100; //TODO: update when changed
        const COLLIDE = 0b0000_0000_0000_0000_0000_0000_0000_1000; //TODO: update when changed
        const MOVABLE = 0b0000_0000_0000_0000_0000_0000_0001_0000; //TODO: update when changed
        const MOVED = 0b0000_0000_0000_0000_0000_0000_0010_0000; //TODO: update when changed
        const MASS_CHANGED = 0b0000_0000_0000_0000_0000_0000_0100_0000; //TODO: update when changed
        const BOUNCE = 0b0000_0000_0000_0000_0000_0000_1000_0000; //TODO: update when changed

        const MATRIX_SIMPLE_TO_CHANGE = 0b0000_0000_0000_0000_0000_0001_0000_0000; //TODO: update when changed
    }
}


#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub struct ThrowEntityInfo {
    pub mass_minimum_to_throw: i64,
    pub mass_self_added: i64,
    pub mass_entity_thrown: i64,
    pub throw_ratio: f32,
    pub direction: Range<f32>,
    pub power: Range<i32>,
    pub color: super::ThrownEntityColor,
    pub timer_entity_thrown: EntityTimer,
    pub characteristics_entity_thrown: ThrownEntityCharacteristics,
}

impl Default for ThrowEntityInfo {
    fn default() -> ThrowEntityInfo {
        ThrowEntityInfo {
            mass_minimum_to_throw: 200_000_000,
            mass_self_added: -1_000_000,
            mass_entity_thrown: 2_000_000,
            throw_ratio: 1.0,
            direction: 0.0..360.0,
            power: 500..1_000,
            color: super::ThrownEntityColor::Same,
            timer_entity_thrown: EntityTimer::default(),
            characteristics_entity_thrown: ThrownEntityCharacteristics::Same,
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub enum ThrownEntityCharacteristics {
    Same,
    Custom(Box<EntityCharacteristics>),
    CustomIndex(usize),
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub enum ThrownEntityTexture {
    Same,
    CustomIndex(usize),
    Random(Vec<usize>),
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub enum OnDeathEffect {
    Split(usize),
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub struct EntityGravityInfo {
    pub power: f32,
    //pub distance_clamp_min: f32, //TODO: bad name
    pub distance_accepted_min: f32, //TODO: bad name
    pub distance_accepted_max: f32, //TODO: bad name
    //pub speed_accepted_min: f32, //TODO: terrible name
    pub speed_clamp_min: f32,
    pub speed_clamp_max: f32,
}

impl Default for EntityGravityInfo {
    fn default() -> EntityGravityInfo {
        EntityGravityInfo {
            power: 1.0,
            //distance_clamp_min: crate::game::map::RATIO_POSITION as f32,
            distance_accepted_min: 0.0,
            distance_accepted_max: std::f32::MAX,
            //speed_accepted_min: 0.0,
            speed_clamp_min: 0.0,
            speed_clamp_max: std::f32::MAX,
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub struct EntityCharacteristics {
    pub killer: bool,
    pub collide: bool,
    pub bounce: bool,
    pub invincible: bool,
    pub can_split: bool,
    pub inertia: i32, //TODO: Better
    pub mass_evolution: Option<f32>,
    pub mass_min: i64,
    pub mass_max: i64,
    pub on_death: Option<OnDeathEffect>,
    pub gravity: Option<EntityGravityInfo>, //TODO: Better
    pub throw_entity: Option<ThrowEntityInfo>,
    //pub special: Vec<EntitySpecial>,
}

impl EntityCharacteristics {
    fn from_other(characteristics: &super::EntityCharacteristics) -> EntityCharacteristics {
        EntityCharacteristics {
            killer: characteristics.killer,
            collide: characteristics.collide,
            bounce: characteristics.bounce,
            invincible: characteristics.invincible,
            can_split: characteristics.can_split,
            inertia: characteristics.inertia,
            mass_evolution: characteristics.mass_evolution,
            mass_min: characteristics.mass_min,
            mass_max: characteristics.mass_max,
            on_death: match characteristics.on_death.clone() {
                Some(death) => {
                    match death {
                        super::OnDeathEffect::Split(count) => {
                            Some(OnDeathEffect::Split(count))
                        }
                    }
                }
                None => { None }
            },
            gravity: match characteristics.gravity.clone() {
                Some(gravity) => {
                    Some(EntityGravityInfo {
                        power: gravity.power,
                        distance_accepted_min: gravity.distance_accepted_min,
                        distance_accepted_max: gravity.distance_accepted_max,
                        speed_clamp_min: gravity.speed_clamp_min,
                        speed_clamp_max: gravity.speed_clamp_max,
                    })
                }
                None => { None }
            },
            throw_entity: match characteristics.throw_entity.clone() {
                Some(throw_info) => {
                    Some(ThrowEntityInfo {
                        mass_minimum_to_throw: throw_info.mass_minimum_to_throw,
                        mass_self_added: throw_info.mass_self_added,
                        mass_entity_thrown: throw_info.mass_entity_thrown,
                        throw_ratio: throw_info.throw_ratio,
                        direction: throw_info.direction,
                        power: throw_info.power,
                        color: throw_info.color,
                        timer_entity_thrown: EntityTimer {
                            collision: throw_info.timer_entity_thrown.collision.clone(),
                            mergeable: throw_info.timer_entity_thrown.mergeable.clone(),
                            inertia: throw_info.timer_entity_thrown.inertia.clone(),
                            lifetime_left: throw_info.timer_entity_thrown.lifetime_left.clone(),
                        },
                        characteristics_entity_thrown: match throw_info.characteristics_entity_thrown {
                            super::ThrownEntityCharacteristics::Same => ThrownEntityCharacteristics::Same,
                            super::ThrownEntityCharacteristics::CustomIndex(index) => ThrownEntityCharacteristics::CustomIndex(index),
                            super::ThrownEntityCharacteristics::Custom(c) => ThrownEntityCharacteristics::Custom(Box::new(EntityCharacteristics::from_other(&c))),
                        }
                    })
                }
                None => { None }
            }
        }
    }
}

impl Default for EntityCharacteristics {
    fn default() -> EntityCharacteristics {
        EntityCharacteristics {
            killer: false,
            collide: false,
            gravity: None,
            //movable: true,
            bounce: true,
            invincible: false,
            mass_evolution: None,
            mass_min: 1_000_000,
            mass_max: 1_000_000_000_000,
            inertia: 10,
            can_split: false,
            on_death: None,
            throw_entity: None,
            //special: Vec::new(),
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone, Default)]
pub struct EntityTimer {
    pub collision: Option<i32>,
    //pub collision_ratio: Option<i32>,
    pub mergeable: Option<i32>,
    pub inertia: Option<i32>, // TODO: change the way inertia decrease each update
    pub lifetime_left: Option<i32>,
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub enum EntitySpecial {
    WASM(String)
}
