use buffer::BufferMulti;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::sync::Weak;
use std::sync::atomic::AtomicUsize;

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone, Default)]
pub struct PlayerInfo {
    pub kind: PlayerKind, // TODO: change to PlayerKind?
    pub entities: Vec<usize>,
    pub cell_default_color: crate::game::entity::EntityColor,
    pub cell_default_texture: usize,
}


impl PlayerInfo {
    pub fn from_player(player: &Player) -> PlayerInfo {
        PlayerInfo {
            kind: player.kind.clone(),
            entities: player.entities.clone(),
            cell_default_color: player.cell_default_color,
            cell_default_texture: player.cell_default_texture,
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone, PartialEq)]
pub enum PlayerKind {
    Player,
    Neutral,
}

impl Default for PlayerKind {
    fn default() -> PlayerKind {
        PlayerKind::Neutral
    }
}

pub enum PlayerAction {
    AddEntity(usize),
    KillEntity(usize), //TODO: DeleteEntity instead ?
    Move(Weak<AtomicUsize>),
}

#[repr(C)]
pub struct Player {
    pub kind: PlayerKind,
    pub entities: Vec<usize>,
    pub cell_default_color: crate::game::entity::EntityColor, // TODO: Change this
    pub cell_default_texture: usize, // TODO: Change this
    pub buffer: BufferMulti<PlayerAction>,
}

impl Player {
    pub fn new(info: PlayerInfo) -> Player {
        Player {
            kind: info.kind,
            entities: Vec::new(),
            cell_default_color: info.cell_default_color,
            cell_default_texture: info.cell_default_texture,
            buffer: BufferMulti::with_capacity(1, 8),
        }
    }
}