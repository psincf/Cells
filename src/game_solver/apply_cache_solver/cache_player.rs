use crate::game::Game;
use crate::game::player::PlayerAction;
use crate::new_timer_monothread;
use std::sync::atomic::Ordering;

pub struct CachePlayerSolver<'a> {
    game: &'a mut Game,
}

impl<'a> CachePlayerSolver<'a> {
    pub fn new(game: &'a mut Game) -> CachePlayerSolver {
        CachePlayerSolver {
            game,
        }
    }

    pub fn solve(&mut self) {
        new_timer_monothread!(_t, "apply_cache_player");
        for index_player in 0..self.game.players.len() {
            let mut entity_killed = Vec::new();
            let player = &mut self.game.players[index_player];
            for action in player.buffer.receive() {
                match action {
                    PlayerAction::AddEntity(index_main) => {
                        let entity = &mut self.game.entities.core[index_main];
                        entity.index.player = player.entities.len();
                        player.entities.push(index_main);
                    }
                    
                    PlayerAction::KillEntity(index) => {
                        entity_killed.push(index);
                    }
                    
                    PlayerAction::Move(weak_ptr) => { //TODO: can be optimized without checking if unit is alive with optimization in CacheGameSolver
                        if let Some(entity_index) = weak_ptr.upgrade() {
                            let entity = &mut self.game.entities.core[entity_index.load(Ordering::Relaxed)];
                            player.entities[entity.index.player] = entity.index.main;
                        }
                    }
                }
            }
            entity_killed.sort_unstable_by( |a, b| if a < b { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Less } ); // Invert sort: Necessary in order to be sure that a potential moved index with swap_remove is a valid index.
            for entity_index in entity_killed {
                player.entities.swap_remove(entity_index);
                if player.entities.len() != entity_index {
                    self.game.entities.core[player.entities[entity_index]].index.player = entity_index;
                }
            }
        }
    }
}