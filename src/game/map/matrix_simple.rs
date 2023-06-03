use crate::game::Game;
use crate::game::entity::index::MatrixIndex;
use crate::game::entity::entities::Entities;
use euclid::default::{Point2D, Size2D};

//use spin::Mutex;
//use parking_lot::Mutex;

use slab::Slab;

pub struct MatrixSimpleCellContent {
    pub entity: usize,
    pub position: Point2D<i32>,
}
#[derive(Default)]
pub struct MatrixSimpleCell {
    inner: Slab<MatrixSimpleCellContent>,
    lock: spin::mutex::SpinMutex<()>, //TODO: do something with this ?
}

impl MatrixSimpleCell {
    pub fn lock(&mut self) -> (spin::mutex::SpinMutexGuard<()>, &mut Slab<MatrixSimpleCellContent>) {
        let lock = self.lock.lock();
        return (lock, &mut self.inner);
    }
}

macro_trait_impl::deref!(MatrixSimpleCell, inner, Slab<MatrixSimpleCellContent>);
macro_trait_impl::index!(MatrixSimpleCell, inner, MatrixSimpleCellContent);

pub struct MatrixSimple {
    inner: Vec<Vec<MatrixSimpleCell>>,
    pub size: Size2D<i32>,
    pub size_field: i32,
}

macro_trait_impl::index!(MatrixSimple, inner, Vec<MatrixSimpleCell>);

impl MatrixSimple {
    pub fn new(size_total: Size2D<i32>, size_field: i32) -> MatrixSimple {
        let mut matrix = Vec::new();

        let size = Size2D::new(
            size_total.width / size_field + (size_total.width % size_field).min(1),
            size_total.height / size_field + (size_total.height % size_field).min(1),
        );

        for x in 0..size.width as usize {
            matrix.push(Vec::new());
            for _ in 0..size.height {
                matrix[x].push(Default::default());
            }
        }
        let size_field = size_field;
        
        MatrixSimple {
            inner: matrix,
            size,
            size_field,
        }
    }

    pub fn rebuild(&mut self, game: &mut Game, size_total: Size2D<i32>, size_field: i32) {
        if self.size_field == size_field { return }
        let mut new = Self::new(size_total, size_field);
        let entities = unsafe { &mut *(&mut game.entities as *mut Entities) };
        for entity in 0..game.entities.len() {
            self.delete_entity(entities, entity);
            new.add_entity(entities, entity);
        }
        *self = new;
    }

    #[inline]
    fn get_field_entity(&self, entities: &Entities, entity: usize) -> (usize, usize) {
        let entity_position = entities.position[entity];
        let x = (entity_position.x / self.size_field) as usize;
        let y = (entity_position.y / self.size_field) as usize;

        return (x, y)
    }

    #[inline]
    pub fn entity_has_moved(&self, entities: &Entities, entity: usize) -> bool {
        let (x, y) = self.get_field_entity(entities, entity);
        let index = &entities.index_matrix_simple[entity];
        return !(index.x == x && index.y == y)
    }

    pub fn intersect_with(&self, entities: &Entities, entity: usize) -> Vec<(usize, usize)> {
        let (x_field, y_field, _z) = entities.index_matrix_simple[entity].xyz();
        let radius = entities.get_radius(entity);
        let scope_field = (radius / self.size_field as f32) as i32 + 1;
        let mut cells = Vec::new();
        
        let x_min = (x_field as i32 - scope_field).max(0);
        let x_max = (x_field as i32 + scope_field).min(self.size.width - 1);
        let y_min = (y_field as i32 - scope_field).max(0);
        let y_max = (y_field as i32 + scope_field).min(self.size.height - 1);
        let rect = entities.get_rect(entity);
        for x in x_min..=x_max {
            for y in y_min..=y_max {
                let rect_cell = euclid::default::Rect::new(euclid::default::Point2D::new(x * self.size_field, y * self.size_field), Size2D::new(self.size_field, self.size_field));
                if rect.intersects(&rect_cell) {
                    cells.push((x as usize, y as usize));
                }
            }
        }

        return cells
    }
    
    pub fn update_entity_index(&mut self, entities: &Entities, entity: usize) {
        let (x, y, z) = entities.index_matrix_simple[entity].xyz();
        let (_lock, cell) = self.inner[x][y].lock();

        cell[z].entity = entity;
    }

    pub fn update_entity_position_no_lock(&mut self, entities: &Entities, entity: usize) {
        let position = entities.position[entity];
        let (x, y, z) = entities.index_matrix_simple[entity].xyz();
        let cell = &mut self.inner[x][y];

        cell[z].position = position;
    }

    pub fn update_entity_position(&mut self, entities: &Entities, entity: usize) {
        let position = entities.position[entity];
        let (x, y, z) = entities.index_matrix_simple[entity].xyz();
        let (_lock, cell) = self.inner[x][y].lock();

        cell[z].position = position;
    }

    pub fn update_entity(&mut self, entities: &mut Entities, entity: usize) {
        self.delete_entity(entities, entity);
        self.add_entity(entities, entity);
    }

    pub fn update_entity_2(&mut self, entities: &mut Entities, entity: usize) {
        if !self.entity_has_moved(entities, entity) {
            self.update_entity_position(entities, entity);
        } else {
            self.update_entity(entities, entity);
        }
    }

    pub fn delete_entity(&mut self, entities: &Entities, entity: usize) {
        let (x, y, z) = entities.index_matrix_simple[entity].xyz();
        self[x][y].remove(z);
    }

    pub fn add_entity(&mut self, entities: &mut Entities, entity: usize) {
        let (x, y) = self.get_field_entity(entities, entity);
        let z = self[x][y].insert(MatrixSimpleCellContent {
                entity: entity,
                position: entities.position[entity]
        });
        entities.index_matrix_simple[entity] = MatrixIndex { x, y, z };
    }

    pub fn update_entity_multithread(&mut self, entities: &mut Entities, entity: usize) {
        self.delete_entity_multithread(entities, entity);
        self.add_entity_multithread(entities, entity);
    }

    pub fn update_entity_multithread_2(&mut self, entities: &mut Entities, entity: usize) {
        if !self.entity_has_moved(entities, entity) {
            self.update_entity_position(entities, entity);
        } else {
            self.update_entity_multithread(entities, entity);
        }
    }

    pub fn delete_entity_multithread(&mut self, entities: &Entities, entity: usize) {
        let (x, y, z) = entities.index_matrix_simple[entity].xyz();
        let (_lock, cell) = self.inner[x][y].lock();
        
        cell.remove(z);
    }

    pub fn add_entity_multithread(&mut self, entities: &mut Entities, entity: usize) {
        let (x, y) = self.get_field_entity(entities, entity);
        let (_lock, cell) = self.inner[x][y].lock();

        let z = cell.insert(MatrixSimpleCellContent {
            entity: entity,
            position: entities.position[entity]
        });
        entities.index_matrix_simple[entity] = MatrixIndex { x, y, z };
    }
}