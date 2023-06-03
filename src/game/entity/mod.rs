pub mod index;
pub mod entities;
//pub mod factory;
pub use entities::Entities;
use index::EntityIndex;
use index::MatrixIndex;

use crate::prelude::*;

use std::ops::Range;

use buffer::Buffer;
use euclid::default::{Point2D, Rect, Size2D, Vector2D};
use parking_lot::{Mutex, MutexGuard};
use rustc_hash::FxHashSet;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

pub const RATIO_MASS: i64 = 1_000_000;

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone, Copy)]
pub struct EntityColor {
    pub center: [u8;4],
    pub edge: [u8;4]
}

impl Default for EntityColor {
    fn default() -> EntityColor {
        EntityColor {
            center: [128, 128, 128, 255],
            edge: [0, 0, 0, 255],
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone, Default)]
pub struct EntityInfo {
    pub player: usize,
    pub position: Point2D<i32>,
    pub speed: Vector2D<f32>,
    pub mass: i64,
    pub color: EntityColor,
    pub texture: usize,
    pub timer: EntityTimer,
    pub characteristics: EntityCharacteristics,
}

impl EntityInfo {
    pub fn validate(&mut self, game: &Game) {
        if self.player >= game.players.len() { self.player = game.players.len() - 1; }

        self.position.x = self.position.x.max(0).min(game.map.max().width);
        self.position.y = self.position.y.max(0).min(game.map.max().height);

        self.characteristics.validate();
        self.mass = self.mass.max(self.characteristics.mass_min).min(self.characteristics.mass_max);
    }
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
            texture: entity_core.index.texture,
            timer: entity_timer.clone(),
            characteristics: entity_core.characteristics.clone(),
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
    pub color: ThrownEntityColor,
    pub texture: ThrownEntityTexture,
    pub timer_entity_thrown: EntityTimer,
    pub characteristics_entity_thrown: ThrownEntityCharacteristics,
}

impl ThrowEntityInfo {
    pub fn validate(&mut self) {
        self.throw_ratio = self.throw_ratio.max(-360.0);

        self.direction.end = self.direction.end.min(360.0).max(self.direction.start + 0.1);
        self.power.start = self.power.start.max(0);
        self.power.end = self.power.end.max(self.power.start + 1);

        self.color.validate();
        self.characteristics_entity_thrown.validate();
    }
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
            color: ThrownEntityColor::Same,
            texture: ThrownEntityTexture::Same,
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

impl ThrownEntityCharacteristics {
    pub fn validate(&mut self) {
        if let ThrownEntityCharacteristics::Custom(e) = self {
            e.validate();
        }
    }
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
pub enum ThrownEntityColor {
    Same,
    Custom(EntityColor),
    Random(Vec<EntityColor>),
}

impl ThrownEntityColor {
    pub fn validate(&mut self) {
        if let ThrownEntityColor::Random(vec) = self {
            if vec.len() == 0 { vec.push(EntityColor::default()); }
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub enum OnDeathEffect {
    Split(usize),
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub enum DistanceRatio {
    Linear,
    Squared,
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub struct EntityGravityInfo {
    pub power: f32,
    pub distance_ratio: DistanceRatio,
    pub distance_limit: Range<f32>,
    pub distance_clamp: Range<f32>,
    pub speed_limit: Range<f32>,
    pub speed_clamp: Range<f32>,
}

impl EntityGravityInfo {
    pub fn validate(&mut self) {
        self.distance_limit.start = self.distance_limit.start.max(0.0);
        self.distance_clamp.start = self.distance_clamp.start.max(0.0);
        self.speed_limit.start = self.speed_limit.start.max(0.0);
        self.speed_clamp.start = self.speed_clamp.start.max(0.0);

        self.distance_limit.end = self.distance_limit.end.max(self.distance_limit.start);
        self.distance_clamp.end = self.distance_clamp.end.max(self.distance_clamp.start);
        self.speed_limit.end = self.speed_limit.end.max(self.speed_limit.start);
        self.speed_clamp.end = self.speed_clamp.end.max(self.speed_clamp.start);
    }
}

impl Default for EntityGravityInfo {
    fn default() -> EntityGravityInfo {
        EntityGravityInfo {
            power: 1.0,
            distance_ratio: DistanceRatio::Squared,
            distance_limit: 0.0..f32::MAX,
            distance_clamp: 0.0..f32::MAX,
            speed_limit: 0.0..f32::MAX,
            speed_clamp: 0.0..f32::MAX,
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub struct EntityCharacteristics {
    pub killer: bool,
    pub collide: bool,
    pub collide_when_mergeable: bool,
    pub mergeable: bool,
    pub affected_by_gravity: bool, //TODO: implement it better
    pub bounce: bool,
    pub can_split_on_kill: bool,
    pub invincible: bool,
    pub inertia: i32, //TODO: Better
    pub mass_min: i64,
    pub mass_max: i64,
    pub mass_evolution: Option<f32>,
    pub on_death: Option<OnDeathEffect>,
    pub gravity: Option<EntityGravityInfo>, //TODO: Better
    pub throw_entity: Option<ThrowEntityInfo>,
    pub special: Vec<EntitySpecial>,
}

impl EntityCharacteristics {
    pub fn validate(&mut self) {
        if !self.collide { self.collide_when_mergeable = false; }
        self.inertia = self.inertia.max(0);
        self.mass_min = self.mass_min.max(1);
        self.mass_max = self.mass_max.max(self.mass_min);
        if let Some(mass_evolution) = self.mass_evolution.as_mut() {
            *mass_evolution = mass_evolution.max(0.0);
        }
        if let Some(gravity) = self.gravity.as_mut() {
            gravity.validate();
        }
        if let Some(info) = self.throw_entity.as_mut() {
            info.validate();
        }
    }
}

impl Default for EntityCharacteristics {
    fn default() -> EntityCharacteristics {
        EntityCharacteristics {
            killer: false,
            collide: false,
            collide_when_mergeable: false,
            mergeable: true,
            gravity: None,
            affected_by_gravity: true,
            bounce: true,
            invincible: false,
            mass_evolution: None,
            mass_min: 1_000_000,
            mass_max: 1_000_000_000_000,
            inertia: 10,
            can_split_on_kill: false,
            on_death: None,
            throw_entity: None,
            special: Vec::new(),
            //special: vec![EntitySpecial::WASM(String::new())],
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone, Default)]
pub struct EntityTimer {
    pub collision: Option<i32>,
    pub collision_ratio: Option<i32>,
    pub mergeable: Option<i32>,
    pub inertia: Option<i32>, // TODO: change the way inertia decrease each update
    pub lifetime_left: Option<i32>,
}

pub struct EntityCollidingInfo { // TODO: Only for colliding entities? Maybe with SoA ? Or through a Box<> for less size of Entity ?
    buffer_colliding_solver: Mutex<FxHashSet<usize>>, // TODO: change this? Warning: Collision of non-similar index!!
    pub colliding_position: std::cell::Cell<Point2D<i32>>, // TODO: change this?
    pub colliding_position_new: std::cell::Cell<Point2D<i32>>, // TODO: change this?
    pub colliding_speed: std::cell::Cell<Vector2D<f32>>, // TODO: change this?
    pub colliding_speed_new: std::cell::Cell<Vector2D<f32>>, // TODO: change this?
    pub colliding_pression: std::cell::Cell<f32>, // TODO: change this?
    pub colliding_pression_new: std::cell::Cell<f32>, // TODO: change this?
    pub colliding_energy_to_add: Mutex<Vec<Vector2D<f32>>>, // TODO: change this?
}

unsafe impl Send for EntityCollidingInfo{}
unsafe impl Sync for EntityCollidingInfo{}

impl EntityCollidingInfo {
    pub fn clear_buffer_collider(&self) {
        self.buffer_colliding_solver.lock().clear();
    }

    pub fn shrink_to_fit_buffer_collider(&self) {
        self.buffer_colliding_solver.lock().shrink_to_fit();
    }

    pub fn insert_collider(&self, index_entity_colliding: usize) {
        self.buffer_colliding_solver.lock().insert(index_entity_colliding);
    }

    pub fn entities_colliding(&self) -> MutexGuard<FxHashSet<usize>> {
        self.buffer_colliding_solver.lock()
    }
}

impl Default for EntityCollidingInfo {
    fn default() -> EntityCollidingInfo {
        EntityCollidingInfo {
            buffer_colliding_solver: Mutex::new(FxHashSet::with_hasher(Default::default())),
            colliding_position: std::cell::Cell::default(),
            colliding_position_new: std::cell::Cell::default(),
            colliding_speed: std::cell::Cell::default(),
            colliding_speed_new: std::cell::Cell::default(),
            colliding_pression: std::cell::Cell::new(1.0),
            colliding_pression_new: std::cell::Cell::new(1.0),
            colliding_energy_to_add: Mutex::new(Vec::new()),
        }
    }
}
pub struct EntityCore {
    pub alive: bool, // TODO: Good idea ? Still not use!
    pub player: usize,
    pub color: EntityColor,
    pub characteristics: EntityCharacteristics,
    pub colliding_info: EntityCollidingInfo,
    pub index: EntityIndex,
    pub killed: Option<usize>,
    pub origin: Option<std::sync::Weak<usize>>,
}

impl EntityCore {
    pub fn new(info: &EntityInfo, _map: &Map) -> EntityCore {
        assert!(info.characteristics.mass_min <= info.characteristics.mass_max);
        let player = info.player;
        let color = info.color;
        let characteristics = info.characteristics.clone();
        let colliding_info = EntityCollidingInfo::default();
        let index = EntityIndex {
            main: 0,
            main_ptr: Default::default(),
            player: 0,
            //matrix: Default::default(),
            matrix_physics: Default::default(),
            texture: info.texture,
            unique_id: 0,
            drawing_buffer: std::usize::MAX,
        };
        let killed = None;

        EntityCore {
            alive: true,
            player,
            color,
            characteristics,
            colliding_info,
            index,
            killed,
            origin: None,
        }
    }
}
//#[derive(Debug)]
pub enum EntityAction {
    AddCollisionRatioTime(i32),
    AddCollisionTime(i32),
    AddPosition(i32, i32),
    AddMass(i64),
    AddMergeableTime(i32),
    AddInertiaTime(i32),
    AddLifetimeLeftTime(i32),
    AddSpeed(f32, f32),
    Killed(usize),
    KilledConfirmed(usize),
    MulSpeed(f32, f32),
    SetColor(EntityColor),
    Split(usize),
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub enum EntitySpecial {
    WASM(String)
}

#[derive(Clone)]
#[repr(C)]
pub struct DrawableEntity {
    pub old_buffer_id: usize,
    pub unique_id: usize,
    pub lifetime: i32,
    pub position: Point2D<i32>,
    pub mass: f32,
    pub color: [u8;4],
    pub color_2: [u8;4],
}

impl DrawableEntity {
    #[inline]
    pub fn get_radius(&self) -> f32 {
        (self.mass / std::f32::consts::PI).sqrt()
    }
}