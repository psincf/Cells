use crate::prelude::*;
use crate::game::entity::index::IndexMatrixPhysicsBigger;
use crate::game::entity::entities::Entities;
use euclid::default::{Point2D, Rect, Size2D};

use arrayvec::ArrayVec;

use slab::Slab;

pub struct MatrixPhysicsCellContent {
    pub entity: usize,
    pub position: Point2D<i32>,
    pub mass: f32,
    pub cell_index: usize,
}

#[derive(Default)]
pub struct MatrixPhysicsCell {
    inner: Slab<MatrixPhysicsCellContent>,
    lock: spin::Mutex<()>, //TODO: do something with this ?
    bigger_cell_cache: Vec<(usize, usize, usize)>, //TODO: ?
}

impl MatrixPhysicsCell {
    pub fn lock(&mut self) -> (spin::MutexGuard<()>, &mut Slab<MatrixPhysicsCellContent>) {
        let lock = self.lock.lock();
        return (lock, &mut self.inner);
    }

    pub fn get_bigger_cells(&self) -> &Vec<(usize, usize, usize)> {
        &self.bigger_cell_cache
    }
}

macro_trait_impl::deref!(MatrixPhysicsCell, inner, Slab<MatrixPhysicsCellContent>);
macro_trait_impl::index!(MatrixPhysicsCell, inner, MatrixPhysicsCellContent);


pub struct MatrixPhysics {
    index: i32,
    size_field: i32,
    inner: Vec<Vec<MatrixPhysicsCell>>,
}

impl MatrixPhysics {
    fn new(index: i32, size_total: Size2D<i32>, size_field: i32) -> MatrixPhysics {
        let size = Size2D::new(
            size_total.width / size_field + (size_total.width % size_field).min(1),
            size_total.height / size_field + (size_total.height % size_field).min(1),
        );

        let mut matrix = Vec::with_capacity(size.width as usize);

        for x in 0..size.width {
            matrix.push(Vec::with_capacity(size.height as usize));
            for _y in 0..size.height {
                matrix[x as usize].push(Default::default());
            }
        }
        
        MatrixPhysics {
            index: index,
            size_field: size_field,
            inner: matrix,
        }
    }

    fn intersect_with(&self, entities: &Entities, entity: usize) -> Vec<(usize, usize)> {
        let entity_position = entities.position[entity];
        let entity_radius = entities.get_radius(entity);

        let mut list_index = Vec::new();
        let size_field = self.size_field;

        let x_field = (entity_position.x / size_field) as usize;
        let y_field = (entity_position.y / size_field) as usize;

        let scope_field = (entity_radius / size_field as f32) as i32 + 1;

        let mut x_min = x_field as i32 - scope_field; if x_min < 0 { x_min = 0; }
        let mut x_max = x_field as i32 + scope_field; if x_max >= self.inner.len() as i32 { x_max = self.inner.len() as i32 - 1; }
        let mut y_min = y_field as i32 - scope_field; if y_min < 0 { y_min = 0; }
        let mut y_max = y_field as i32 + scope_field; if y_max >= self.inner[0].len() as i32 { y_max = self.inner[0].len() as i32 - 1 };

        let rect_cell = entities.get_rect(entity);
        for x in x_min..=x_max {
            for y in y_min..=y_max {
                let rect_field = Rect::new(Point2D::new(x * size_field, y * size_field), Size2D::new(size_field, size_field));
                if rect_cell.intersects(&rect_field) {
                    list_index.push((x as usize, y as usize));
                }
            }
        }    
        return list_index;
    }

    #[inline]
    fn bigger_cells_index(index_matrix: usize, x_field: usize, y_field: usize, matrix_count: usize, size_field_ratio: usize) -> Vec<(usize, usize, usize)> {
        let mut bigger_cells = Vec::new();
        for matrix_bigger_index in (index_matrix as usize + 1)..(matrix_count) {
            let new_x = x_field / size_field_ratio.pow(matrix_bigger_index as u32 - index_matrix as u32) as usize;
            let new_y = y_field / size_field_ratio.pow(matrix_bigger_index as u32 - index_matrix as u32) as usize;
            bigger_cells.push((matrix_bigger_index, new_x, new_y));
        }
        bigger_cells
    }

    #[inline]
    pub fn get_size_matrix(&self) -> Size2D<i32> {
        Size2D::new(self.inner.len() as i32, self.inner[0].len() as i32)
    }

    #[inline]
    pub fn get_size_cell(&self) -> i32 {
        self.size_field
    }
}

macro_trait_impl::index!(MatrixPhysics, inner, Vec<MatrixPhysicsCell>);

pub struct MapPhysics {
    pub matrix_list: ArrayVec<[MatrixPhysics; 16]>,
    size_field_initial: i32,
    size_field_ratio: i32,
    count: i32
}

impl MapPhysics {
    pub fn new(size_total: Size2D<i32>, size_field_initial: i32, size_field_ratio: i32, count: i32) -> MapPhysics {
        let mut matrix_list: ArrayVec<[MatrixPhysics; 16]> = ArrayVec::new();
        for i in 0..count {
            let size_field = size_field_initial * size_field_ratio.pow(i as u32);
            matrix_list.push(MatrixPhysics::new(i, size_total, size_field));
        }

        for (index_matrix, matrix) in matrix_list.iter_mut().enumerate() {
            for (x, row) in matrix.inner.iter_mut().enumerate() {
                for (y, cell) in row.iter_mut().enumerate() {
                    cell.bigger_cell_cache = MatrixPhysics::bigger_cells_index(index_matrix, x, y, count as usize, size_field_ratio as usize);
                }
            }
        }

        MapPhysics {
            matrix_list,
            size_field_initial,
            size_field_ratio,
            count
        }
    }
    
    pub fn size_field_initial(&self) -> i32 {
        self.size_field_initial
    }

    pub fn size_field_ratio(&self) -> i32 {
        self.size_field_ratio
    }

    pub fn count(&self) -> i32 {
        self.count
    }

    pub fn rebuild(&mut self, game: &mut Game, size_total: Size2D<i32>, size_field_initial: i32, size_field_ratio: i32, count: i32) {
        if self.size_field_initial == size_field_initial && self.size_field_ratio == size_field_ratio && self.count == count { return }
        let mut new = Self::new(size_total, size_field_initial, size_field_ratio, count);
        let entities = unsafe { &*(&game.entities as *const Entities) };
        for entity in game.entities.core.iter_mut() {
            if entity.characteristics.collide {
                self.delete_entity(entities, entity);
                new.add_entity(entities, entity);
            }
        }
        *self = new;
    }
    
    pub fn get_matrix_inner_and_matrix_bigger(&mut self, entities: &Entities, entity: usize) -> (&mut MatrixPhysics, Vec<&mut MatrixPhysics>) {
        let entity_radius = entities.get_radius(entity);
        for i in 0..self.count as usize {
            let matrix_list = unsafe { &mut *(&mut self.matrix_list as * mut ArrayVec<[MatrixPhysics;16]>)};
            let matrix_inner = &mut matrix_list[i as usize];
            let size_field = matrix_inner.size_field;
            if entity_radius * 2.0 < size_field as f32 {
                let matrix_len = self.matrix_list.len();
                let mut matrix_bigger = Vec::new();
                if i + 1 < matrix_len { matrix_bigger = self.matrix_list.get_mut((i + 1)..matrix_len).unwrap().iter_mut().collect(); }
                return (matrix_inner, matrix_bigger)
            }
        }
        return (self.matrix_list.last_mut().unwrap(), vec![]);
    }

    #[inline]
    pub fn entity_has_moved(&mut self, entities: &Entities, entity: &EntityCore) -> bool {
        let (matrix_to_go, _matrix_bigger_to_go) = self.get_matrix_inner_and_matrix_bigger(entities, entity.index.main);

        if matrix_to_go.index != entity.index.matrix_physics.inside.index { return true }
        else {
            let list_cell_to_go = matrix_to_go.intersect_with(entities, entity.index.main);
            let list_cell_actual = &entity.index.matrix_physics.inside.locations;
            let has_to_delete_cells = list_cell_actual
                .iter()
                .any(|cell_actual| { !list_cell_to_go.iter().any(|cell_to_go| { cell_to_go.0 == cell_actual.0 && cell_to_go.1 == cell_actual.1 }) });

            return has_to_delete_cells || list_cell_actual.len() != list_cell_to_go.len();
        }
    }

    pub fn matrix_with_index_mut(&mut self, index: usize) -> &mut MatrixPhysics {
        &mut self.matrix_list[index]
    }

    pub fn matrix_with_index(&self, index: usize) -> &MatrixPhysics {
        &self.matrix_list[index]
    }

    pub fn update_entity_index(&mut self, entity: &mut EntityCore) {
        let matrix_entity = &entity.index.matrix_physics.inside;
        let matrix_id = matrix_entity.index;
        let matrix = self.matrix_with_index_mut(matrix_id as usize);
        
        for (x_field, y_field, z) in matrix_entity.locations.iter() {
            let (x_field, y_field, z) = (*x_field, *y_field, *z);
            let (_lock, cell) = matrix[x_field][y_field].lock();

            cell[z].entity = entity.index.main;
        }
    }

    pub fn update_entity(&mut self, entities: &Entities, entity: &mut EntityCore) { // TODO: Check if cell moved OR mass changed for optimization. Or not ?
        self.delete_entity(entities, entity);
        self.add_entity(entities, entity);
    }

    pub fn delete_entity(&mut self, _entities: &Entities, entity: &mut EntityCore) {
        let matrix_entity = &entity.index.matrix_physics.inside;
        let matrix_id = matrix_entity.index;
        let matrix = self.matrix_with_index_mut(matrix_id as usize);
        for &(x_field, y_field, z) in matrix_entity.locations.iter() {
            matrix[x_field][y_field].remove(z);
        }
        entity.index.matrix_physics.inside.locations.clear();
        entity.index.matrix_physics.bigger.clear();
    }

    pub fn add_entity(&mut self, entities: &Entities, entity: &mut EntityCore) {
        let (matrix_inner, matrix_bigger_list) = self.get_matrix_inner_and_matrix_bigger(entities, entity.index.main);

        let list = matrix_inner.intersect_with(entities, entity.index.main);
        for (x, y) in list {
            let z = matrix_inner[x][y].insert(MatrixPhysicsCellContent {
                entity: entity.index.main,
                position: entities.position[entity.index.main],
                mass: entities.mass[entity.index.main] as f32,
                cell_index: entity.index.matrix_physics.inside.locations.next_key_id(),
            });
            let _cell_index = entity.index.matrix_physics.inside.locations.insert((x as usize, y as usize, z));
        }
        entity.index.matrix_physics.inside.index = matrix_inner.index as i32;

        for matrix in matrix_bigger_list {
            let list = matrix.intersect_with(entities, entity.index.main);
            entity.index.matrix_physics.bigger.push(IndexMatrixPhysicsBigger { index: matrix.index, locations: list } );
        }

    }

    pub fn update_entity_2(&mut self, entities: &Entities, entity: &mut EntityCore) {
        if self.entity_has_moved(entities, entity) {
            self.delete_entity(entities, entity);
            self.add_entity(entities, entity);
        } else {
            self.update_cell_content(entities, entity);
        }
    }

    pub fn delete_entity_multithread(&mut self, _entities: &Entities, entity: &mut EntityCore) { // TODO: Directly in MapSolver
        let matrix_entity = &entity.index.matrix_physics.inside;
        let matrix_id = matrix_entity.index;
        let matrix = self.matrix_with_index_mut(matrix_id as usize);
        for &(x, y, z) in matrix_entity.locations.iter() {
            let (_lock, cell) = matrix[x][y].lock();
            cell.remove(z);
        }

        entity.index.matrix_physics.inside.locations.clear();
        entity.index.matrix_physics.bigger.clear();
    }

    pub fn add_entity_multithread(&mut self, entities: &Entities, entity: &mut EntityCore) { //TODO: Directly in MapSolver
        let (matrix_inner, matrix_bigger_list) = self.get_matrix_inner_and_matrix_bigger(entities, entity.index.main);

        let list = matrix_inner.intersect_with(entities, entity.index.main);
        for (x, y) in list {
            let (_lock, cell) = matrix_inner[x][y].lock();
            let z = cell.insert(MatrixPhysicsCellContent {
                entity: entity.index.main,
                position: entities.position[entity.index.main],
                mass: entities.mass[entity.index.main] as f32,
                cell_index: entity.index.matrix_physics.inside.locations.len(),
            });
            entity.index.matrix_physics.inside.locations.insert((x as usize, y as usize, z));
        }
        entity.index.matrix_physics.inside.index = matrix_inner.index as i32;

        for matrix in matrix_bigger_list {
            let list = matrix.intersect_with(entities, entity.index.main);
            entity.index.matrix_physics.bigger.push(IndexMatrixPhysicsBigger { index: matrix.index, locations: list } );
        }

    }
    
    pub fn update_entity_multithread(&mut self, entities: &Entities, entity: &mut EntityCore) {
        self.delete_entity_multithread(entities, entity);
        self.add_entity_multithread(entities, entity);
    }

    pub fn update_entity_multithread_2(&mut self, entities: &Entities, entity: &mut EntityCore) {
        if self.entity_has_moved(entities, entity) {
            self.delete_entity_multithread(entities, entity);
            self.add_entity_multithread(entities, entity);
        } else {
            self.update_cell_content_multithread(entities, entity);
        }
    }
    
    pub fn update_cell_content(&mut self, entities: &Entities, entity: &mut EntityCore) {
        let matrix_actual = self.matrix_with_index_mut(entity.index.matrix_physics.inside.index as usize);
        let list_cell_actual = &entity.index.matrix_physics.inside.locations;

        for &(x, y, z) in list_cell_actual.iter() {
            matrix_actual[x][y][z].position = entities.position[entity.index.main];
            matrix_actual[x][y][z].mass = entities.mass[entity.index.main] as f32;
        }
    }

    pub fn update_cell_content_multithread(&mut self, entities: &Entities, entity: &mut EntityCore) {
        let matrix_actual = self.matrix_with_index_mut(entity.index.matrix_physics.inside.index as usize);
        let list_cell_actual = &entity.index.matrix_physics.inside.locations;

        for &(x, y, z) in list_cell_actual.iter() {
            let (_lock, cell) = matrix_actual[x][y].lock();
            cell[z].position = entities.position[entity.index.main];
            cell[z].mass = entities.mass[entity.index.main] as f32;
        }
    }
}