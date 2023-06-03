use crate::prelude::*;

use crate::game::entity::EntityFlags;

use euclid::default::Point2D;
use euclid::default::Vector2D;

pub struct PositionSolver<'a> {
    entity_index: usize,
    game: &'a mut Game,
}

impl<'a> PositionSolver<'a> {
    #[inline]
    pub fn new(entity_index: usize, game: &'a mut Game) -> PositionSolver<'a> {
        PositionSolver {
            entity_index: entity_index,
            game,
        }
    }

    #[inline]
    pub fn solve(&mut self) {
        let entity_position = &mut self.game.entities.position[self.entity_index];
        let entity_speed = &mut self.game.entities.speed[self.entity_index];
        let entity_bounce = self.game.entities.flags[self.entity_index].contains(EntityFlags::BOUNCE);

        *entity_speed = entity_speed.min(Vector2D::new(1_000_000_000.0, 1_000_000_000.0)).max(Vector2D::new(-1_000_000_000.0, -1_000_000_000.0));
        *entity_position += entity_speed.to_i32();
        
        if entity_bounce {
            bounce(entity_position, entity_speed);
        } else {
            not_bounce(entity_position, entity_speed);
        }

        self.game.entities.flags[self.entity_index].insert(EntityFlags::MOVED);
        self.game.entities.drawable_entities[self.entity_index].position = *entity_position;
    }
}

#[inline]
pub fn bounce(entity_position: &mut Point2D<i32>, entity_speed: &mut Vector2D<f32>) {
    if entity_position.x <= 0{
        entity_position.x *= -1;
        entity_speed.x = entity_speed.x.abs();
    } else if entity_position.x >= crate::APP.get().game.map.max().width {
        entity_position.x = crate::APP.get().game.map.max().width - (entity_position.x - crate::APP.get().game.map.max().width);
        entity_speed.x = -entity_speed.x.abs();
    }
    if entity_position.y <= 0 {
        entity_position.y *= -1;
        entity_speed.y = entity_speed.y.abs();
    } else if entity_position.y >= crate::APP.get().game.map.max().height {
        entity_position.y = crate::APP.get().game.map.max().height - (entity_position.y - crate::APP.get().game.map.max().height);
        entity_speed.y = -entity_speed.y.abs();
    }

    entity_position.x = entity_position.x.min(crate::APP.get().game.map.max().width);
    entity_position.x = entity_position.x.max(0);
    entity_position.y = entity_position.y.min(crate::APP.get().game.map.max().height);
    entity_position.y = entity_position.y.max(0);
}

#[inline]
#[allow(dead_code)]
pub fn bounce_basic(entity_position: &mut Point2D<i32>, entity_speed: &mut Vector2D<f32>) {
    if entity_position.x <= 0 { 
        entity_position.x = 0;
        entity_speed.x = entity_speed.x.abs();
    } else if entity_position.x >= crate::APP.get().game.map.max().width {
        entity_position.x = crate::APP.get().game.map.max().width;
        entity_speed.x = -entity_speed.x.abs();
    }
    if entity_position.y <= 0 {
        entity_position.y = 0;
        entity_speed.y = entity_speed.y.abs();
    } else if entity_position.y >= crate::APP.get().game.map.max().height {
        entity_position.y = crate::APP.get().game.map.max().height;
        entity_speed.y = -entity_speed.y.abs();
    }
}

#[inline]
pub fn not_bounce(entity_position: &mut Point2D<i32>, entity_speed: &mut Vector2D<f32>) {
    if entity_position.x <= 0 { 
        entity_position.x = 0;
        entity_speed.x = 0.0;
    } else if entity_position.x >= crate::APP.get().game.map.max().width {
        entity_position.x = crate::APP.get().game.map.max().width;
        entity_speed.x = 0.0;
    }
    if entity_position.y <= 0 {
        entity_position.y = 0;
        entity_speed.y = 0.0;
    } else if entity_position.y >= crate::APP.get().game.map.max().height {
        entity_position.y = crate::APP.get().game.map.max().height;
        entity_speed.y = 0.0;
    }
}