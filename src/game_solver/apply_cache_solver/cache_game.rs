

use crate::prelude::*;

use crate::new_timer_monothread;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Weak;
pub struct CacheGameSolver<'a> {
    game: &'a mut Game,
}

impl<'a> CacheGameSolver<'a> {
    pub fn new(game: &mut Game) -> CacheGameSolver {
        CacheGameSolver {
            game,
        }
    }
    
    pub fn solve(&mut self) {
        let game_bis = unsafe { &*(self.game as *const Game) };
        new_timer_monothread!(_t, "apply_cache_game");
        {
            new_timer_monothread!(_bench_add, "apply_cache_game_add");
            let add_entity_infos:Vec<Box<EntityInfo>> = self.game.buffer_add_entity.receive().collect();
            for info in add_entity_infos {
                self.game.new_entity(*info);
            }
        }
        {
            new_timer_monothread!(_bench_delete, "apply_cache_game_delete");
            let kill_entity_infos:Vec<Weak<AtomicUsize>> = self.game.buffer_kill_entity.receive().collect();
            for entity_index in kill_entity_infos {
                let index = entity_index.upgrade().expect("Bug Kill entity twice").load(Ordering::Relaxed);
                let entity = unsafe { &mut *(&mut self.game.entities.core[index] as *mut EntityCore) };
                let player = unsafe { &mut *(&mut self.game.players[entity.player] as *mut Player) };
                self.game.id_generator.remove(entity.index.unique_id);
                self.game.map.delete_entity(&self.game.entities, entity);
                player.entities.swap_remove(entity.index.player);
                if player.entities.len() != entity.index.player {
                    self.game.entities.core[player.entities[entity.index.player]].index.player = entity.index.player;
                }
                self.game.entities.swap_remove(index);
                if self.game.entities.len() != index {
                    let entity_moved = &mut self.game.entities.core[index];
                    entity_moved.index.main = index;
                    entity_moved.index.main_ptr.store(index, Ordering::Relaxed);
                    self.game.players[entity_moved.player].entities[entity_moved.index.player] = index;
                    self.game.map.update_entity_index(&game_bis.entities, entity_moved);
                }
            }
        }
    }
    /*
    pub fn solve_multithread(&mut self) {
        new_timer_monothread!(_t, "apply_cache_game");
        new_timer_monothread!(bench_delete, "apply_cache_game_delete");
        self.delete_entities_multithread3();
        drop(bench_delete);
        new_timer_monothread!(bench_add, "apply_cache_game_add");
        self.add_entities_multithread2();
        drop(bench_add);
    }

    pub fn delete_entities_multithread_basic(&mut self) {
        let kill_entity_infos:Vec<Weak<AtomicUsize>> = self.game.buffer_kill_entity.receive().collect();
        for entity_index in kill_entity_infos {
            let index = entity_index.upgrade().expect("Bug Kill entity twice").load(Ordering::Relaxed);
            self.delete_entity_basic(index);
        }
    }

    pub fn delete_entity_basic(&mut self, entity_index: usize) {
        let entity = &mut self.game.entities.core[entity_index];
        let player = &mut self.game.players[entity.player];
        self.game.map.buffer.send(MapAction::KillEntity(Box::new((entity.index.matrix.clone(), entity.index.matrix_physics.inside.clone()))));
        player.buffer.send(PlayerAction::KillEntity(entity.index.player));
        self.game.entities.swap_remove(entity_index);
        if self.game.entities.len() != entity_index {
            let entity_moved = &mut self.game.entities.core[entity_index];
            entity_moved.index.main = entity_index;
            entity_moved.index.main_ptr.store(entity_index, Ordering::Relaxed);
            let player_moved = &self.game.players[entity_moved.player];
            self.game.map.buffer.send(MapAction::Move(Arc::downgrade(&entity_moved.index.main_ptr)));
            player_moved.buffer.send(PlayerAction::Move(Arc::downgrade(&entity_moved.index.main_ptr)));
        }
    }

    pub fn delete_entities_multithread2(&mut self) {
        let kill_entity_infos:Vec<Weak<AtomicUsize>> = self.game.buffer_kill_entity.receive().collect();
        let mut kill_entity_infos_arc:Vec<usize> = Vec::new();

        for weak_ptr in kill_entity_infos.iter() {
            let index = weak_ptr.upgrade().expect("Bug Kill entity twice").load(Ordering::Relaxed);
            kill_entity_infos_arc.push(index);
        }
        
        let amount_deleted = kill_entity_infos_arc.len();
        let new_max_len = self.game.entities.len() - amount_deleted;
        let mut entities_to_replace = Vec::with_capacity(amount_deleted);
        for entity_index in kill_entity_infos_arc.iter() {
            let entity = &mut self.game.entities.core[*entity_index];
            let player = &mut self.game.players[entity.player];
            self.game.map.buffer.send(MapAction::KillEntity(Box::new((entity.index.matrix.clone(), entity.index.matrix_physics.inside.clone()))));
            player.buffer.send(PlayerAction::KillEntity(entity.index.player));
            entity.alive = false;
            if *entity_index < new_max_len { entities_to_replace.push(*entity_index); }
        }

        let amout_to_replace = entities_to_replace.len();
        let mut entities_to_move = Vec::with_capacity(amout_to_replace);
        let atomic_len_entities = AtomicUsize::new(self.game.entities.len());

        {
            let max = atomic_len_entities.fetch_sub(amount_deleted, Ordering::Relaxed);
            let min = max - amount_deleted;
            let range = min..max;
            for i in range {
                let entity = &mut self.game.entities.core[i];
                if entity.alive { entities_to_move.push(i); }
            }
        }
        assert!(entities_to_move.len() == entities_to_replace.len());

        for i in 0..amout_to_replace {
            let entity_to_replace_index = entities_to_replace[i];
            let entity_to_replace = &mut self.game.entities.core[entity_to_replace_index] as *mut _;
            let entity_to_move_index = entities_to_move[i];
            let entity_to_move = &mut self.game.entities.core[entity_to_move_index] as *mut _;
            unsafe { std::ptr::swap(entity_to_move, entity_to_replace) };

            let entity_to_move =  &mut self.game.entities.core[entity_to_replace_index];
            entity_to_move.index.main = entity_to_replace_index;
            entity_to_move.index.main_ptr.store(entity_to_replace_index, Ordering::Relaxed);
            let player_moved = &self.game.players[entity_to_move.player];
            self.game.map.buffer.send(MapAction::Move(Arc::downgrade(&entity_to_move.index.main_ptr)));
            player_moved.buffer.send(PlayerAction::Move(Arc::downgrade(&entity_to_move.index.main_ptr)));
        }

        for i in new_max_len..self.game.entities.len() {
            unsafe { std::ptr::drop_in_place(&mut self.game.entities.core[i]); }
        }

        assert!(new_max_len <= self.game.entities.len());
        unsafe { self.game.entities.set_len(new_max_len); }
        //self.game.entities.truncate(new_max_len); //TODO: multithread this
    }

    pub fn delete_entities_multithread3(&mut self) {
        let kill_entity_infos: Vec<Weak<AtomicUsize>> = self.game.buffer_kill_entity.receive().collect();
        let mut kill_entity_infos_arc: Vec<usize> = Vec::new();

        for weak_ptr in kill_entity_infos.iter() {
            let index = weak_ptr.upgrade().expect("Bug Kill entity twice").load(Ordering::Relaxed);
            kill_entity_infos_arc.push(index);
        }
        

        let amount_deleted = kill_entity_infos_arc.len();
        let new_max_len = self.game.entities.len() - amount_deleted;
        let entities_to_replace = Arc::new(Mutex::new(Vec::with_capacity(amount_deleted)));
        {
            new_timer_monothread!(_t, "to_replace");
            let atomic_count = Arc::new(AtomicUsize::new(0));
            let step = 128;
            for _ in 0..self.game.threadpool.num_threads().max(1) {
                let mut local_entities_to_replace = Vec::with_capacity(amount_deleted); //TODO: change capacity... and allocate on a pre-allocated buffer ? Directly in the thread ?
                unsafe {
                    let game = unsafe_ptr::UnsafePtr::new(self.game);
                    let kill_entity_infos_arc = unsafe_ptr::UnsafePtr::new(&mut kill_entity_infos_arc);
                    let atomic_count = atomic_count.clone();
                    let entities_to_replace = entities_to_replace.clone();
                    self.game.threadpool.compute_unsafe( move || {
                        loop {
                            let game = unsafe { game.ref_mut() };
                            let kill_entity_infos_arc = unsafe { kill_entity_infos_arc.ref_const() };
                            threadpool::range_break!(range, atomic_count, amount_deleted, step);
                            for i in range.start..range.end {
                                let entity_index = kill_entity_infos_arc[i];
                                let entity = &mut game.entities.core[entity_index];
                                let player = &mut game.players[entity.player];

                                game.map.delete_entity_multithread(entity);
                                //game.map.buffer.send(MapAction::KillEntity(Box::new((entity.index.matrix.clone(), entity.index.matrix_physics.inside.clone())))); //TODO: No allocation / TODO: So what to do with this?
                                player.buffer.send(PlayerAction::KillEntity(entity.index.player));

                                entity.alive = false;
                                if entity_index < new_max_len { local_entities_to_replace.push(entity_index); }
                            }
                        }
                        entities_to_replace.lock().append(&mut local_entities_to_replace);
                    });
                }
            }
            self.game.threadpool.sync_spin();
        }

        let mut entities_to_replace = entities_to_replace.lock();
        let amout_to_replace = entities_to_replace.len();
        let mut entities_to_move = Vec::with_capacity(amout_to_replace);
        {
            new_timer_monothread!(_t, "to_move");
            let atomic_len_entities = AtomicUsize::new(self.game.entities.len());
            {
                let max = atomic_len_entities.fetch_sub(amount_deleted, Ordering::Relaxed);
                let min = max - amount_deleted;
                let range = min..max;
                for i in range {
                    let entity = &mut self.game.entities.core[i];
                    if entity.alive { entities_to_move.push(i); }
                }
            }
            assert!(entities_to_move.len() == entities_to_replace.len());
        }
        {
            new_timer_monothread!(_t, "move");

            let atomic_count = Arc::new(AtomicUsize::new(0));
            let step = 128;
            for _ in 0..self.game.threadpool.num_threads().max(1) {
                let game = unsafe_ptr::UnsafePtr::new(&mut self.game);
                let entities_to_replace = unsafe_ptr::UnsafePtr::new(&mut entities_to_replace);
                let entities_to_move = unsafe_ptr::UnsafePtr::new(&mut entities_to_move);
                let atomic_count = atomic_count.clone();
                unsafe {
                    self.game.threadpool.compute_unsafe(  move || {
                        loop {
                            let game = unsafe { game.ref_mut() };
                            let entities_to_replace = unsafe { entities_to_replace.ref_mut() };
                            let entities_to_move = unsafe { entities_to_move.ref_mut() };
                            threadpool::range_break!(range, atomic_count, entities_to_replace.len(), step);
                            for i in range.start..range.end {
                                let entity_to_replace_index = entities_to_replace[i];
                                let entity_to_replace = &mut game.entities.core[entity_to_replace_index] as *mut _;
                                let entity_to_move_index = entities_to_move[i];
                                let entity_to_move = &mut game.entities.core[entity_to_move_index] as *mut _;
                                unsafe { std::ptr::swap(entity_to_move, entity_to_replace) };
                
                                let entity_to_move =  &mut game.entities.core[entity_to_replace_index];
                                entity_to_move.index.main = entity_to_replace_index;
                                entity_to_move.index.main_ptr.store(entity_to_replace_index, Ordering::Relaxed);
                                let player_moved = &mut game.players[entity_to_move.player];
                                game.map.buffer.send(MapAction::Move(Arc::downgrade(&entity_to_move.index.main_ptr)));
                                player_moved.entities[entity_to_move.index.player] = entity_to_move.index.main;
                                //player_moved.buffer.send(PlayerAction::Move(Arc::downgrade(&entity_to_move.index.main_ptr))); // TODO: So what to do with this?
                            }
                        }

                    });
                }
            }
            self.game.threadpool.sync_spin()
        }
        {
            new_timer_monothread!(_t, "drop");
            for i in new_max_len..self.game.entities.len() {
                unsafe { std::ptr::drop_in_place(&mut self.game.entities.core[i]); } // TODO: No dealloc
                /*
                unsafe { std::ptr::drop_in_place(&mut self.game.entities.core[i].buffer); }
                unsafe { std::ptr::drop_in_place(&mut self.game.entities.core[i].buffer2); }
                unsafe { std::ptr::drop_in_place(&mut self.game.entities.core[i].characteristics.throw_entity); }
                unsafe { std::ptr::drop_in_place(&mut self.game.entities.core[i].index.main_ptr); }
                */
                
            }
        }

        assert!(new_max_len <= self.game.entities.len());
        unsafe { self.game.entities.set_len(new_max_len); }
        //self.game.entities.truncate(new_max_len); //TODO: multithread this
    }

    pub fn add_entities_multithread_basic(&mut self) {
        let add_entity_infos: Vec<Box<EntityInfo>> = self.game.buffer_add_entity.receive().collect();
        let add_entity_len = add_entity_infos.len();
        let mut actual_len = self.game.entities.len();

        self.game.entities.reserve(add_entity_len);
        unsafe { self.game.entities.set_len(actual_len + add_entity_len); }

        for info in add_entity_infos.into_iter() {
            actual_len += 1; //TODO: Not multithread
            self.add_entity_basic(info, actual_len - 1);
        }
    }

    pub fn add_entity_basic(&mut self, info: Box<EntityInfo>, index: usize) {
        let mut entity = Entity::new(&info, &self.game.map);
        entity.index.main = index;        
        entity.index.main_ptr.store(index, Ordering::Relaxed);

        unsafe { std::ptr::write(&mut self.game.entities.core[index], entity) } // No drop because len set before initialization
        
        self.game.map.buffer.send(MapAction::AddEntity(index));
        self.game.players[info.player].buffer.send(PlayerAction::AddEntity(index));
    }

    pub fn add_entities_multithread2(&mut self) {
        let add_entity_infos: Vec<Box<EntityInfo>> = self.game.buffer_add_entity.receive().collect();
        let add_entity_len = add_entity_infos.len();
        let actual_len = Arc::new(AtomicUsize::new(self.game.entities.len()));

        self.game.entities.reserve(add_entity_len);
        unsafe { self.game.entities.set_len(actual_len.load(Ordering::Relaxed) + add_entity_len); }
        {
            let game = unsafe_ptr::UnsafePtr::new(&mut self.game);
            self.game.threadpool.compute_iter_each_thread_join(&add_entity_infos, 512,  move |info| {
                let game = unsafe { game.ref_mut() };
                let index = actual_len.fetch_add(1, Ordering::Relaxed); //TODO: Optimize
                let mut entity = Entity::new(&info, &game.map);
                entity.index.main = index;        
                entity.index.main_ptr.store(index, Ordering::Relaxed);
        
                unsafe { std::ptr::write(&mut game.entities.core[index], entity) } // No drop because len set before initialization
                
                game.map.add_entity_multithread(&mut game.entities.core[index]);
                //game.map.buffer.send(MapAction::AddEntity(index)); //TODO: So what to do with this?
                game.players[info.player].buffer.send(PlayerAction::AddEntity(index));
            });
        }
    }
    */
}
