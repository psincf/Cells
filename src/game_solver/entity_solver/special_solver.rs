use crate::prelude::*;
use anyhow::Result;
use std::collections::HashMap;

thread_local! {
    pub static WASM: WASMStruct = {
        let mut wasm = WASMStruct::new();
        wasm.set_linker();
        wasm.set_instance();
        return wasm
    };
}

thread_local! {
    pub static ITERATOR_LIST: std::cell::RefCell<IteratorList> = {
        std::cell::RefCell::new(IteratorList::new())
    }
}

#[allow(dead_code)]
pub struct WASMStruct {
    engine: wasmtime::Engine,
    store: wasmtime::Store,
    linker: wasmtime::Linker,
    module: wasmtime::Module,
    instance: Option<wasmtime::Instance>,
    instance_fn_start: Option<wasmtime::TypedFunc<u64, ()>>,
}

impl WASMStruct {
    pub fn new() -> WASMStruct {
        let engine = wasmtime::Engine::new(&wasmtime::Config::new()).unwrap();
        let store = wasmtime::Store::new(&engine);
        
        let mut file = std::fs::File::open("wasm_scripts/example.wasm").unwrap();
        let mut bytes = Vec::new();
        {
            use std::io::Read;
            file.read_to_end(&mut bytes).unwrap();
        }
        let module = wasmtime::Module::new(store.engine(), bytes).unwrap();
        let linker = wasmtime::Linker::new(&store);

        WASMStruct {
            engine,
            store,
            linker,
            module,
            instance: None,
            instance_fn_start: None,
        }
    }

    pub fn set_instance(&mut self) {
        self.instance = Some(self.linker.instantiate(&self.module).unwrap());
        self.instance_fn_start = Some(
            self.instance.as_mut().unwrap().get_func("the_test").unwrap().typed::<u64, ()>().unwrap().clone()
        );
    }

    pub fn set_linker(&mut self) {
        let functions = add_function_to_wasm(&self.store);
        for f in functions {
            self.linker.define(
                "env",
                f.0,
                f.1,
            ).unwrap();
        }
    }

    #[allow(dead_code)]
    pub fn set_module(&mut self) {
        let mut file = std::fs::File::open("wasm_scripts/example.wasm").unwrap();
        let mut bytes = Vec::new();
        {
            use std::io::Read;
            file.read_to_end(&mut bytes).unwrap();
        }
        self.module = wasmtime::Module::new(self.store.engine(), bytes).unwrap();
    }
}

#[allow(dead_code)]
pub struct SpecialSolver<'a> {
    game: &'a Game,
    entity: usize
}

impl<'a> SpecialSolver<'a> {
    pub fn new(entity: usize, game: &'a Game) -> SpecialSolver<'a> {
        SpecialSolver {
            game,
            entity
        }
    }

    pub fn solve(&mut self) -> Result<()> {
        WASM.with( |wasm| {
            wasm.instance_fn_start.as_ref().unwrap().call(self.entity as u64).unwrap();
        });
        ITERATOR_LIST.with( |iterator_list| {
            iterator_list.borrow_mut().reset();
        });

        Ok(())
    }
}

fn add_function_to_wasm(store: &wasmtime::Store) -> HashMap<&'static str, wasmtime::Func> {
    let mut funcs = HashMap::new();
    
    funcs.insert("num_entities", wasmtime::Func::wrap(store, move || {
        crate::APP.get().game.entities.len() as u64
    }));

    funcs.insert("get_position_x", wasmtime::Func::wrap(store, |index: u64| {
        let position = crate::APP.get().game.entities.position[index as usize];
        return position.x;
    }));
    funcs.insert("get_position_y", wasmtime::Func::wrap(store, |index: u64| {
        let position = crate::APP.get().game.entities.position[index as usize];
        return position.y;
    }));
    funcs.insert("get_speed_x", wasmtime::Func::wrap(store, |index: u64| {
        let speed = crate::APP.get().game.entities.speed[index as usize];
        return speed.x;
    }));
    funcs.insert("get_speed_y", wasmtime::Func::wrap(store, |index: u64| {
        let speed = crate::APP.get().game.entities.speed[index as usize];
        return speed.y;
    }));
    funcs.insert("get_mass", wasmtime::Func::wrap(store, |index: u64| {
        let mass = crate::APP.get().game.entities.mass[index as usize];
        return mass;
    }));
    funcs.insert("get_color", wasmtime::Func::wrap(store, |index: u64| {
        let color = crate::APP.get().game.entities.core[index as usize].color;
        return unsafe { std::mem::transmute::<crate::game::entity::EntityColor, u64>(color) };
    }));


    funcs.insert("add_position", wasmtime::Func::wrap(store, move |index: u64, position_x: i32, position_y: i32| {
        crate::APP.get().game.entities.send_buffer(index as usize, EntityAction::AddPosition(position_x, position_y));
    }));
    funcs.insert("add_speed", wasmtime::Func::wrap(store, move |index: u64, speed_x: f32, speed_y: f32| {
        crate::APP.get().game.entities.send_buffer(index as usize, EntityAction::AddSpeed(speed_x, speed_y));
    }));
    funcs.insert("set_color", wasmtime::Func::wrap(store, move |index: u64, color_center: u32, color_edge: u32| {
        crate::APP.get_mut().game.entities.send_buffer(index as usize, EntityAction::SetColor( unsafe { std::mem::transmute([color_center, color_edge]) } ));
    }));

    
    funcs.insert("iter_entities", wasmtime::Func::wrap(store, || {
        let entities_iterator = 0..crate::APP.get().game.entities.len() as u64;
        let mut iterator_id = 0;
        ITERATOR_LIST.with(|list| {
            iterator_id = list.borrow_mut().add_iterator_entities(entities_iterator);
        });
        return iterator_id;
    }));

    funcs.insert("iter_entities_near", wasmtime::Func::wrap(store, |entity_index: u64, distance: f32| {
        let entities_iterator_near = IteratorEntitiesNear::new(&crate::APP.get().game, entity_index as usize, distance);
        let mut iterator_id = 0;
        ITERATOR_LIST.with(|list| {
            iterator_id = list.borrow_mut().add_iterator_entities_near(entities_iterator_near);
        });
        return iterator_id;
    }));
    funcs.insert("iter_next_1", wasmtime::Func::wrap(store, |iterator_id: u64| {
        let mut next = None;
        ITERATOR_LIST.with(|list| {
            next = list.borrow_mut().next_1(iterator_id);
        });
        let result = if next.is_none() {
            u64::MAX
        } else {
            next.unwrap()
        };
        return result;
    }));
    
    funcs
}



pub struct IteratorList {
    list_entities: Vec<std::ops::Range<u64>>,
    list_entities_near: Vec<IteratorEntitiesNear>,
}

impl IteratorList {
    pub fn new() -> IteratorList {
        IteratorList {
            list_entities: Vec::new(),
            list_entities_near: Vec::new(),
        }
    }

    pub fn next_1(&mut self, id: u64) -> Option<u64> {
        let id_array:[u32; 2] = unsafe { std::mem::transmute(id) };
        let result;
        match id_array[0] {
            0 => {
                result = self.list_entities[id_array[1] as usize].next();
            },
            1 => {
                result = self.list_entities_near[id_array[1] as usize].next(&crate::APP.get().game);
            }
            _ => {panic!()}
        }
        return result;
    }

    pub fn add_iterator_entities(&mut self, iterator: std::ops::Range<u64>) -> u64 {
        let id = [0, self.list_entities.len() as u32];
        self.list_entities.push(iterator);
        return unsafe { std::mem::transmute(id) }
    }

    pub fn add_iterator_entities_near(&mut self, iterator: IteratorEntitiesNear) -> u64 {
        let id = [1, self.list_entities_near.len() as u32];
        self.list_entities_near.push(iterator);
        return unsafe { std::mem::transmute(id) }
    }

    pub fn reset(&mut self) {
        self.list_entities.clear();
        self.list_entities_near.clear();
    }
}

pub struct IteratorEntitiesNear {
    _x_min: usize,
    x_max: usize,
    y_min: usize,
    y_max: usize,
    x: usize,
    y: usize,
    iterator: std::slice::Iter<'static, crate::game::map::matrix_simple::MatrixSimpleCellContent> ,
    origin: euclid::default::Point2D<i32>,
    distance: f32,
}

impl IteratorEntitiesNear {
    pub fn new(game: &Game, entity_index: usize, distance: f32) -> IteratorEntitiesNear {
        //let (x_field, y_field, _z) = game.entities.core[entity_index].index.matrix.xyz();
        let (x_field, y_field, _z) = game.entities.index_matrix_simple[entity_index].xyz();
        let scope_field = (distance / game.map.matrix_simple.size_field as f32) as i32 + 1;
        
        let x_min = (x_field as i32 - scope_field).max(0) as usize;
        let x_max = (x_field as i32 + scope_field).min(game.map.matrix_simple.size.width - 1) as usize;
        let y_min = (y_field as i32 - scope_field).max(0) as usize;
        let y_max = (y_field as i32 + scope_field).min(game.map.matrix_simple.size.height - 1) as usize;

        IteratorEntitiesNear {
            _x_min: x_min,
            x_max,
            y_min,
            y_max,
            x: x_min,
            y: y_min,
            iterator: unsafe { std::mem::transmute(game.map.matrix_simple[x_min][y_min].iter()) },
            origin: game.entities.position[entity_index],
            distance,
        }
    }

    pub fn next(&mut self, game: &Game) -> Option<u64> {
        loop {
            if let Some(cell) = self.iterator.next() {
                //if (game.entities.position[index.entity] - self.origin).to_f32().length() < self.distance {
                if (cell.position - self.origin).to_f32().length() < self.distance {
                    return Some(cell.entity as u64)
                } else {
                    continue
                }
            } else {
                self.y += 1;
                if self.y <= self.y_max {
                    self.iterator = unsafe { std::mem::transmute(game.map.matrix_simple[self.x][self.y].iter()) };
                    continue
                } else {
                    self.x += 1;
                    if self.x <= self.x_max {
                        self.y = self.y_min;
                        self.iterator = unsafe { std::mem::transmute(game.map.matrix_simple[self.x][self.y].iter()) };
                        continue
                    } else {
                        return None;
                    }
                }
            }
        }
    }
}
