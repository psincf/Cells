use std::sync::Arc;
use std::sync::atomic::AtomicUsize;


use slab::Slab;
#[derive(Clone, Default)]
pub struct IndexMatrixPhysics {
    pub inside: IndexMatrixPhysicsInside, 
    pub bigger: Vec<IndexMatrixPhysicsBigger>,
}

#[derive(Clone, Default)]
pub struct IndexMatrixPhysicsInside {
    pub index: i32,
    pub locations: Slab<(usize, usize, usize)>,
}

impl IndexMatrixPhysicsInside {
    pub fn flat(&self) -> Vec<(i32, (usize, usize, usize))> {
        let mut vec = Vec::new();
        for location in self.locations.iter() {
            vec.push((self.index, *location));
        }
        vec
    }
}

#[derive(Clone, Default)]
pub struct IndexMatrixPhysicsBigger {
    pub index: i32,
    pub locations: Vec<(usize, usize)>,
}

#[derive(Clone, Default)]
pub struct MatrixIndex { // TODO: Find a better name ?
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl MatrixIndex {
    #[inline]
    pub fn xyz(&self) -> (usize, usize, usize) {
        (self.x, self.y, self.z)
    }
}

pub struct EntityIndexMain(usize);
macro_trait_impl::deref!(EntityIndexMain, 0, usize); //TODO: ?

pub struct EntityIndex {
    pub main: usize,
    pub main_ptr: Arc<AtomicUsize>,
    pub player: usize,
    //pub matrix: MatrixIndex,
    pub matrix_physics: IndexMatrixPhysics, //TODO: make it optionnal
    pub texture: usize,
    pub unique_id: usize,
    pub drawing_buffer: usize,
}