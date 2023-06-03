use crate::prelude::*;
use crate::game::entity::ThrownEntityColor;

use euclid::Angle;
use euclid::default::{Point2D, Rotation2D, Vector2D};

pub struct ThrowingEntitySolver<'a> {
    entity: &'a EntityCore,
    game: &'a Game,
}

impl<'a> ThrowingEntitySolver<'a> {
    #[inline]
    pub fn new(entity: &'a EntityCore, game: &'a Game) -> ThrowingEntitySolver<'a> {
        ThrowingEntitySolver {
            entity,
            game,
        }
    }

    pub fn solve(&self) {
        let entity = self.entity;
        let entity_position = self.game.entities.position[entity.index.main];
        let entity_mass = self.game.entities.mass[entity.index.main];
        let entity_radius = self.game.entities.get_radius(entity.index.main);
        let game = self.game;

        let info = self.entity.characteristics.throw_entity.as_ref().unwrap();
        
        if entity_mass > info.mass_minimum_to_throw {
            use rand::Rng;
            let mut rng = rand::thread_rng();

            let quantity = ((entity_mass as f32).sqrt() * 0.001 * info.throw_ratio) as i32 + 1;
            /* //TODO
            let quantity = (entity_mass as f32 / RATIO_MASS as f32).sqrt() * info.throw_ratio;
            let rest_quantity = quantity.fract();
            let rest_quantity_i32 = if rest_quantity > rng.gen() { 1 } else { 0 };
            let quantity = quantity.trunc() as i32 + rest_quantity_i32;
            */
            for _ in 0..quantity {
                let angle = Angle::degrees(rng.gen_range(info.direction.start..info.direction.end));
                let direction = Rotation2D::new(angle).transform_vector(Vector2D::new(1.0, 0.0));
                let power = rng.gen_range(info.power.start..info.power.end) as f32;
                let ratio_position = entity_radius + 100.0;
                let speed = direction.to_f32() * power;
                let position = (direction.to_f32() * ratio_position).to_i32() + entity_position.to_vector();
                let characteristics = match &info.characteristics_entity_thrown {
                    ThrownEntityCharacteristics::Same => { entity.characteristics.clone() } 
                    ThrownEntityCharacteristics::Custom(new_info) => { *new_info.clone() }
                    ThrownEntityCharacteristics::CustomIndex(index) => { game.entities_characteristics[*index].clone() }
                };

                let color = match &info.color {
                    ThrownEntityColor::Same => {
                        entity.color
                    }
                    ThrownEntityColor::Custom(color) => {
                        *color
                    }
                    ThrownEntityColor::Random(vec) => {
                        let index = rng.gen_range(0..vec.len());
                        vec[index]
                    }
                };

                let texture = match &info.texture {
                    ThrownEntityTexture::Same => {
                        entity.index.texture
                    }
                    ThrownEntityTexture::CustomIndex(index) => {
                        *index
                    }
                    ThrownEntityTexture::Random(vec) => {
                        let index = rng.gen_range(0..vec.len());
                        vec[index]
                    }
                };
                let new_entity_info = EntityInfo { // TODO: First in entity buffer, and only if not dead ( Throw entity only if the thrower is not dead, to avoid having infinite creation if it is eated )
                    player: 0,
                    position: Point2D::new(position.x, position.y),
                    speed: Vector2D::new(speed.x, speed.y),
                    mass: info.mass_entity_thrown,
                    characteristics: characteristics,
                    timer: info.timer_entity_thrown.clone(),
                    color: color,
                    texture: texture,
                };
                //game.buffer.send(GameAction::AddEntity(Box::new(new_entity_info)));
                game.buffer_add_entity.send(Box::new(new_entity_info));
                self.game.entities.send_buffer(entity.index.main, EntityAction::AddMass(info.mass_self_added));
            }
        }
    }
}