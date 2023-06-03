pub mod premade;
pub mod matrix_physics;
pub mod matrix_simple;

use matrix_physics::MapPhysics;
use matrix_simple::MatrixSimple;

use crate::prelude::*;
use crate::game::entity::index::MatrixIndex;
use crate::game::entity::index::IndexMatrixPhysicsInside;
use crate::game::entity::entities::Entities;

use buffer::BufferMulti;
use euclid::default::Size2D;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::sync::Weak;
use std::sync::atomic::AtomicUsize;

pub const RATIO_POSITION: i32 = 10_000;

pub enum MapAction {
    AddEntity(usize),
    KillEntity(Box<(MatrixIndex, IndexMatrixPhysicsInside)>),
    Move(Weak<AtomicUsize>),
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub struct MapInfo {
    pub size: Size2D<i32>,
}

pub struct Map {
    pub matrix_simple: MatrixSimple,
    pub matrix_physics: MapPhysics,
    pub size: Size2D<i32>,
    pub size_field: i32,
    pub buffer: BufferMulti<MapAction>,
}

impl Map {
    pub fn new(info: MapInfo) -> Map {
        let matrix_simple = MatrixSimple::new(info.size * RATIO_POSITION, RATIO_POSITION);
        let matrix_physics = MapPhysics::new(info.size * RATIO_POSITION, RATIO_POSITION, 10, 3);
        
        let size = info.size;
        let buffer = BufferMulti::with_capacity(1, 8);
        
        Map {
            matrix_simple,
            matrix_physics,
            size,
            size_field: RATIO_POSITION,
            buffer,
        }
    }

    pub fn max(&self) -> Size2D<i32> {
        Size2D::new(
            self.size.width * self.size_field - 1,
            self.size.height * self.size_field - 1
        )
    }
    
    pub fn update_entity_index(&mut self, entities: &Entities, entity: &mut EntityCore) {
        self.matrix_simple.update_entity_index(entities, entity.index.main);
        if entity.characteristics.collide {
            self.matrix_physics.update_entity_index(entity);
        }
    }

    pub fn update_entity(&mut self, entities: &Entities, entity: &mut EntityCore) {
        self.matrix_simple.update_entity(unsafe { &mut *(entities as *const Entities as *mut Entities) }, entity.index.main);
        if entity.characteristics.collide {
            self.matrix_physics.update_entity(entities, entity);
        }
    }

    pub fn delete_entity(&mut self, entities: &Entities, entity: &mut EntityCore) {
        self.matrix_simple.delete_entity(entities, entity.index.main);
        if entity.characteristics.collide { self.matrix_physics.delete_entity(entities, entity) }; //TODO: don't forget to call this function when the collide field change from true to false! If not, field of matrix_field will stay forever if entity is killed and will create bug! TODO2: don't call it here?
    }

    pub fn add_entity(&mut self, entities: &Entities, entity: &mut EntityCore) {
        self.matrix_simple.add_entity(unsafe { &mut *(entities as *const Entities as *mut Entities) }, entity.index.main);
        if entity.characteristics.collide { self.matrix_physics.add_entity(entities, entity) };
    }

    pub fn delete_entity_multithread(&mut self, entities: &Entities, entity: &mut EntityCore) {
        self.matrix_simple.delete_entity_multithread(entities, entity.index.main);
        if entity.characteristics.collide { self.matrix_physics.delete_entity_multithread(entities, entity) };
    }

    pub fn add_entity_multithread(&mut self, entities: &Entities, entity: &mut EntityCore) {
        self.matrix_simple.add_entity_multithread(unsafe { &mut *(entities as *const Entities as *mut Entities) }, entity.index.main);
        if entity.characteristics.collide { self.matrix_physics.add_entity_multithread(entities, entity) };
    }
}