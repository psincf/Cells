use super::*;

use crate::game::Map;
use euclid::default::Point2D;
use euclid::default::Vector2D;

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

macro_rules! entity_ref_macro_impl {
    ($struct: ident) => {
        impl<'a> $struct<'a> {
            #[inline]
            pub fn core(&self) -> &EntityCore {
                &self.entities.core[self.index]
            }
            #[inline]
            pub fn position(&self) -> &Point2D<i32> {
                &self.entities.position[self.index]
            }
            #[inline]
            pub fn speed(&self) -> &Vector2D<f32> {
                &self.entities.speed[self.index]
            }
            #[inline]
            pub fn mass(&self) -> i64 {
                self.entities.mass[self.index]
            }
            #[inline]
            pub fn direction(&self) -> &Option<Point2D<i32>> {
                &self.entities.direction[self.index]
            }
            #[inline]
            pub fn timer(&self) -> &EntityTimer {
                &self.entities.timer[self.index]
            }
            #[inline]
            pub fn flags(&self) -> &EntityFlags {
                &self.entities.flags[self.index]
            }
            #[inline]
            pub fn get_radius(&self) -> f32 {
                (self.entities.mass[self.index] as f32 / std::f32::consts::PI).sqrt()
            }
        }
        
    };
}

pub struct EntityRef<'a> {
    index: usize,
    entities: &'a Entities,
}
entity_ref_macro_impl!(EntityRef);

pub struct EntityRefMut<'a> {
    index: usize,
    entities: &'a mut Entities,
}
entity_ref_macro_impl!(EntityRefMut);

impl<'a> EntityRefMut<'a> {
    #[inline]
    pub fn core_mut(&mut self) -> &mut EntityCore {
        &mut self.entities.core[self.index]
    }
    #[inline]
    pub fn position_mut(&mut self) -> &mut Point2D<i32> {
        &mut self.entities.position[self.index]
    }
    #[inline]
    pub fn speed_mut(&mut self) -> &mut Vector2D<f32> {
        &mut self.entities.speed[self.index]
    }
    #[inline]
    pub fn mass_mut(&mut self) -> &mut i64 {
        &mut self.entities.mass[self.index]
    }
    #[inline]
    pub fn direction_mut(&mut self) -> &mut Option<Point2D<i32>> {
        &mut self.entities.direction[self.index]
    }
    #[inline]
    pub fn timer_mut(&mut self) -> &mut EntityTimer {
        &mut self.entities.timer[self.index]
    }
    #[inline]
    pub fn flags_mut(&mut self) -> &mut EntityFlags {
        &mut self.entities.flags[self.index]
    }
}

pub struct Entities {
    pub core: Vec<EntityCore>,
    pub position: Vec<Point2D<i32>>,
    pub speed: Vec<Vector2D<f32>>,
    pub mass: Vec<i64>,
    pub direction: Vec<Option<Point2D<i32>>>,
    pub timer: Vec<EntityTimer>,
    pub flags: Vec<EntityFlags>,
    pub mass_evolution: Vec<Option<f32>>, //TODO: Bad ? Duplicate with EntityCharacteristics in EntityCore. Make an EntityCharacteristicsFactory N
    pub lifetime: Vec<i32>,
    pub index_matrix_simple: Vec<MatrixIndex>,
    pub special: Vec<Vec<EntitySpecial>>,
    buffer: Vec<Buffer<EntityAction>>, // TODO: make it better
    buffer_is_some: Vec<AtomicBool>, // TODO: make it better
    buffer2: Vec<Buffer<EntityAction>>, // TODO: make it better
    buffer2_is_some: Vec<AtomicBool>, // TODO: make it better
    pub drawable_entities: Vec<DrawableEntity>,
}

impl Entities {
    pub fn new() -> Entities {
        Entities {
            core: Vec::new(),
            position: Vec::new(),
            speed: Vec::new(),
            mass: Vec::new(),
            direction: Vec::new(),
            timer: Vec::new(),
            flags: Vec::new(),
            mass_evolution: Vec::new(),
            lifetime: Vec::new(),
            index_matrix_simple: Vec::new(),
            special: Vec::new(),
            buffer: Vec::new(),
            buffer_is_some: Vec::new(),
            buffer2: Vec::new(),
            buffer2_is_some: Vec::new(),
            drawable_entities: Vec::new(),
        }
    }

    pub fn new_entity(&mut self, map: &Map, info: &EntityInfo) -> usize {
        let mut entity_core = EntityCore::new(&info, &map);
        let index = self.core.len();
        entity_core.index.main = index;
        entity_core.index.main_ptr.store(index, Ordering::Relaxed);

        let position = {
            let mut position = info.position;
            if position.x <= 0 { position.x = 0 }
            if position.x > map.max().width { position.x = map.max().width } // TODO: Not here
            if position.y <= 0 { position.y = 0 }
            if position.y > map.max().height { position.y = map.max().height } // TODO: Not here
            position
        };

        let mass = info.mass.min(info.characteristics.mass_max).max(info.characteristics.mass_min);

        let mut flags = EntityFlags::empty();
        if info.characteristics.gravity.is_some() { flags.insert(EntityFlags::GRAVITY) }
        if info.characteristics.throw_entity.is_some() { flags.insert(EntityFlags::THROW) }
        if info.characteristics.killer { flags.insert(EntityFlags::EATER) }
        if info.characteristics.collide { flags.insert(EntityFlags::COLLIDE) }
        if info.characteristics.affected_by_gravity { flags.insert(EntityFlags::MOVABLE) }
        if info.characteristics.bounce { flags.insert(EntityFlags::BOUNCE) }

        self.core.push(entity_core);
        self.position.push(position);
        self.speed.push(info.speed);
        self.mass.push(mass);
        self.direction.push(None);
        self.timer.push(info.timer.clone());
        self.flags.push(flags);
        self.mass_evolution.push(info.characteristics.mass_evolution);
        self.lifetime.push(0);
        self.index_matrix_simple.push(MatrixIndex::default());
        self.special.push(info.characteristics.special.clone());
        self.buffer.push(Buffer::new());
        self.buffer_is_some.push(AtomicBool::new(false));
        self.buffer2.push(Buffer::new());
        self.buffer2_is_some.push(AtomicBool::new(false));
        self.drawable_entities.push(DrawableEntity {
            old_buffer_id: usize::MAX,
            unique_id: 0, //TODO!!!
            lifetime: 0, //TODO!!! or Not ?
            position: position,
            mass: mass as f32,
            color: info.color.center,
            color_2: info.color.edge,
        });

        return index;
    }

    pub fn buffer_is_some(&self, entity_index: usize) -> bool {
        self.buffer_is_some[entity_index].load(Ordering::Relaxed)
    }
    
    pub fn buffer2_is_some(&self, entity_index: usize) -> bool {
        self.buffer2_is_some[entity_index].load(Ordering::Relaxed)
    }

    pub fn receive_buffer(&self, entity_index: usize) -> buffer::BufferIterator<EntityAction> {
        self.buffer_is_some[entity_index].store(false, Ordering::Relaxed);
        self.buffer[entity_index].receive()
    }
    
    pub fn receive_buffer2(&self, entity_index: usize) -> buffer::BufferIterator<EntityAction> {
        self.buffer2_is_some[entity_index].store(false, Ordering::Relaxed);
        self.buffer2[entity_index].receive()
    }

    pub fn send_buffer(&self, entity_index: usize, action: EntityAction) {
        self.buffer[entity_index].send(action);
        self.buffer_is_some[entity_index].store(true, Ordering::Relaxed);
    }
    
    pub fn send_buffer2(&self, entity_index: usize, action: EntityAction) {
        self.buffer2[entity_index].send(action);
        self.buffer2_is_some[entity_index].store(true, Ordering::Relaxed);
    }

    pub fn init_colliding_info(&self, entity_index: usize) {
        let entity_core = &self.core[entity_index];
        let entity_position = self.position[entity_index];
        let entity_speed = self.speed[entity_index];
        entity_core.colliding_info.colliding_position.set(entity_position);
        entity_core.colliding_info.colliding_position_new.set(entity_position);
        entity_core.colliding_info.colliding_speed.set(entity_speed);
        entity_core.colliding_info.colliding_speed_new.set(entity_speed);
    }

    pub fn update(&mut self, entity_index: usize, info: EntityInfo) {
        let entity_core = &mut self.core[entity_index];
        let entity_position = &mut self.position[entity_index];
        let entity_speed = &mut self.speed[entity_index];
        let entity_mass = &mut self.mass[entity_index];
        let entity_timer = &mut self.timer[entity_index];
        let entity_flags = &mut self.flags[entity_index];
        let entity_mass_evolution = &mut self.mass_evolution[entity_index];
        let entity_drawable = &mut self.drawable_entities[entity_index];

        if entity_core.player != info.player {
            let game = &mut crate::APP.get_mut().game;
            game.players[entity_core.player].entities.swap_remove(entity_core.index.player);
            if game.players[entity_core.player].entities.len() != entity_core.index.player {
                game.entities.core[game.players[entity_core.player].entities[entity_core.index.player]].index.player = entity_core.index.player;
            }
            entity_core.player = info.player;
            entity_core.index.player = game.players[entity_core.player].entities.len();
            game.players[entity_core.player].entities.push(entity_core.index.main);
        }

        let mut flags = EntityFlags::empty();
        if info.characteristics.gravity.is_some() { flags.insert(EntityFlags::GRAVITY) }
        if info.characteristics.throw_entity.is_some() { flags.insert(EntityFlags::THROW) }
        if info.characteristics.killer { flags.insert(EntityFlags::EATER) }
        if info.characteristics.collide { flags.insert(EntityFlags::COLLIDE) } if !info.characteristics.collide && entity_flags.contains(EntityFlags::COLLIDE) { crate::APP.get_mut().game.map.matrix_physics.delete_entity_multithread(&crate::APP.get().game.entities, entity_core); }
        if info.characteristics.affected_by_gravity { flags.insert(EntityFlags::MOVABLE) }
        if info.characteristics.bounce { flags.insert(EntityFlags::BOUNCE) }

        *entity_position = info.position;
        *entity_speed = info.speed;
        *entity_timer = info.timer;
        *entity_flags = flags;
        *entity_mass = info.mass;
        *entity_mass_evolution = info.characteristics.mass_evolution;
        entity_core.characteristics = info.characteristics.clone();
        entity_core.color = info.color;
        entity_core.index.texture = info.texture;

        entity_drawable.position = info.position;
        entity_drawable.mass = info.mass as f32;
        entity_drawable.color = info.color.center;
        entity_drawable.color_2 = info.color.edge;

        self.special[entity_index] = info.characteristics.special.clone();
    }

    /*
    pub fn update_from_factory(&mut self, entity_index: usize, info: super::factory::EntityInfo) {
        let entity_core = &mut self.core[entity_index];
        let entity_position = &mut self.position[entity_index];
        let entity_speed = &mut self.speed[entity_index];
        let entity_mass = &mut self.mass[entity_index];
        let entity_timer = &mut self.timer[entity_index];
        let entity_flags = &mut self.flags[entity_index];
        let entity_mass_evolution = &mut self.mass_evolution[entity_index];
        let entity_drawable = &mut self.drawable_entities[entity_index];

        if entity_core.player != info.player {
            let game = &mut crate::APP.get_mut().game;
            game.players[entity_core.player].entities.swap_remove(entity_core.index.player);
            if game.players[entity_core.player].entities.len() != entity_core.index.player {
                game.entities.core[game.players[entity_core.player].entities[entity_core.index.player]].index.player = entity_core.index.player;
            }
            entity_core.player = info.player;
            entity_core.index.player = game.players[entity_core.player].entities.len();
            game.players[entity_core.player].entities.push(entity_core.index.main);
        }

        let mut flags = EntityFlags::empty();
        if info.characteristics.gravity.is_some() { flags.insert(EntityFlags::GRAVITY) }
        if info.characteristics.throw_entity.is_some() { flags.insert(EntityFlags::THROW) }
        if info.characteristics.killer { flags.insert(EntityFlags::EATER) }
        if info.characteristics.collide { flags.insert(EntityFlags::COLLIDE) } if !info.characteristics.collide && entity_flags.contains(EntityFlags::COLLIDE) { crate::APP.get_mut().game.map.matrix_physics.delete_entity_multithread(&crate::APP.get().game.entities, entity_core); }
        if info.characteristics.bounce { flags.insert(EntityFlags::BOUNCE) }

        *entity_position = info.position;
        *entity_speed = info.speed;
        {
            entity_timer.collision = info.timer.collision;
            entity_timer.inertia = info.timer.inertia;
            entity_timer.lifetime_left = info.timer.lifetime_left;
            entity_timer.mergeable = info.timer.mergeable;
        }
        *entity_flags = flags;
        *entity_mass = info.mass;
        *entity_mass_evolution = info.characteristics.mass_evolution;

        fn update_characteristics(characteristics: &mut EntityCharacteristics, info: &factory::EntityCharacteristics) {
            characteristics.killer = info.killer;
            characteristics.collide = info.collide;
            characteristics.bounce = info.bounce;
            characteristics.invincible = info.invincible;
            characteristics.can_split = info.can_split;
            characteristics.inertia = info.inertia;
            characteristics.mass_evolution = info.mass_evolution;
            characteristics.mass_min = info.mass_min;
            characteristics.mass_max = info.mass_max;
            characteristics.on_death = match info.on_death.clone() {
                Some(death) => {
                    match death {
                        factory::OnDeathEffect::Split(count) => {
                            Some(OnDeathEffect::Split(count))
                        }
                    }
                }
                None => { None }
            };
            characteristics.gravity = match info.gravity.clone() {
                Some(gravity) => {
                    Some(EntityGravityInfo {
                        power: gravity.power,
                        distance_accepted_min: gravity.distance_accepted_min,
                        distance_accepted_max: gravity.distance_accepted_max,
                        speed_clamp_min: gravity.speed_clamp_min,
                        speed_clamp_max: gravity.speed_clamp_max,
                        distance_clamp_min: characteristics.gravity.clone().unwrap_or_default().distance_clamp_min,
                        speed_accepted_min: characteristics.gravity.clone().unwrap_or_default().speed_accepted_min,
                    })
                }
                None => { None }
            };
            characteristics.throw_entity = match info.throw_entity.clone() {
                Some(throw_info) => {
                    Some(ThrowEntityInfo {
                        mass_minimum_to_throw: throw_info.mass_minimum_to_throw,
                        mass_self_added: throw_info.mass_self_added,
                        mass_entity_thrown: throw_info.mass_entity_thrown,
                        throw_ratio: throw_info.throw_ratio,
                        direction: throw_info.direction,
                        power: throw_info.power,
                        color: throw_info.color,
                        texture: characteristics.throw_entity.clone().unwrap_or_default().texture,
                        timer_entity_thrown: EntityTimer {
                            collision: throw_info.timer_entity_thrown.collision.clone(),
                            collision_ratio: characteristics.throw_entity.clone().unwrap_or_default().timer_entity_thrown.collision_ratio,
                            mergeable: throw_info.timer_entity_thrown.mergeable.clone(),
                            inertia: throw_info.timer_entity_thrown.inertia.clone(),
                            lifetime_left: throw_info.timer_entity_thrown.lifetime_left.clone(),
                        },
                        characteristics_entity_thrown: match throw_info.characteristics_entity_thrown {
                            factory::ThrownEntityCharacteristics::Same => ThrownEntityCharacteristics::Same,
                            factory::ThrownEntityCharacteristics::CustomIndex(i) => ThrownEntityCharacteristics::CustomIndex(i),
                            factory::ThrownEntityCharacteristics::Custom(c) => {
                                let mut origin = if let ThrownEntityCharacteristics::Custom(c_origin) = characteristics.clone().throw_entity.unwrap_or_default().characteristics_entity_thrown {
                                    c_origin
                                } else { Default::default() };
                                update_characteristics(&mut origin, &c);
                                ThrownEntityCharacteristics::Custom(origin)
                            }
                        },
                    })
                }
                None => { None }
            }
        };
        {
            update_characteristics(&mut entity_core.characteristics, &info.characteristics);
            /*
            entity_core.characteristics.killer = info.characteristics.killer;
            entity_core.characteristics.collide = info.characteristics.collide;
            entity_core.characteristics.bounce = info.characteristics.bounce;
            entity_core.characteristics.invincible = info.characteristics.invincible;
            entity_core.characteristics.can_split = info.characteristics.can_split;
            entity_core.characteristics.inertia = info.characteristics.inertia;
            entity_core.characteristics.mass_evolution = info.characteristics.mass_evolution;
            entity_core.characteristics.mass_min = info.characteristics.mass_min;
            entity_core.characteristics.mass_max = info.characteristics.mass_max;
            entity_core.characteristics.on_death = match info.characteristics.on_death.clone() {
                Some(death) => {
                    match death {
                        factory::OnDeathEffect::Split(count) => {
                            Some(OnDeathEffect::Split(count))
                        }
                    }
                }
                None => { None }
            };
            entity_core.characteristics.gravity = match info.characteristics.gravity.clone() {
                Some(gravity) => {
                    Some(EntityGravityInfo {
                        power: gravity.power,
                        distance_accepted_min: gravity.distance_accepted_min,
                        distance_accepted_max: gravity.distance_accepted_max,
                        speed_clamp_min: gravity.speed_clamp_min,
                        speed_clamp_max: gravity.speed_clamp_max,
                        distance_clamp_min: entity_core.characteristics.gravity.clone().unwrap_or_default().distance_clamp_min,
                        speed_accepted_min: entity_core.characteristics.gravity.clone().unwrap_or_default().speed_accepted_min,
                    })
                }
                None => { None }
            };
            entity_core.characteristics.throw_entity = match info.characteristics.throw_entity.clone() {
                Some(throw_info) => {
                    Some(ThrowEntityInfo {
                        mass_minimum_to_throw: throw_info.mass_minimum_to_throw,
                        mass_self_added: throw_info.mass_self_added,
                        mass_entity_thrown: throw_info.mass_entity_thrown,
                        throw_ratio: throw_info.throw_ratio,
                        direction: throw_info.direction,
                        power: throw_info.power,
                        color: throw_info.color,
                        texture: entity_core.characteristics.throw_entity.clone().unwrap_or_default().texture,
                        timer_entity_thrown: EntityTimer {
                            collision: throw_info.timer_entity_thrown.collision.clone(),
                            collision_ratio: entity_core.characteristics.throw_entity.clone().unwrap_or_default().timer_entity_thrown.collision_ratio,
                            mergeable: throw_info.timer_entity_thrown.mergeable.clone(),
                            inertia: throw_info.timer_entity_thrown.inertia.clone(),
                            lifetime_left: throw_info.timer_entity_thrown.lifetime_left.clone(),
                        },
                        characteristics_entity_thrown: match throw_info.characteristics_entity_thrown {
                            factory::ThrownEntityCharacteristics::Same => ThrownEntityCharacteristics::Same,
                            factory::ThrownEntityCharacteristics::CustomIndex(i) => ThrownEntityCharacteristics::CustomIndex(i),
                            factory::ThrownEntityCharacteristics::Custom(c) => ThrownEntityCharacteristics::Custom(),
                        },
                    })
                }
                None => { None }
            }
            */
        }
        entity_core.color = info.color;

        entity_drawable.position = info.position;
        entity_drawable.mass = info.mass as f32;
        entity_drawable.color = info.color.center;
        entity_drawable.color_2 = info.color.edge;

        //self.special[entity_index] = info.characteristics.special.clone();
    }
    */

    pub fn update_drawable(&mut self, entity_index: usize) {
        //self.drawable_entities[entity_index].old_buffer_id: usize;
        //self.drawable_entities[entity_index].unique_id: usize;
        self.drawable_entities[entity_index].lifetime = self.lifetime[entity_index];
        self.drawable_entities[entity_index].position = self.position[entity_index];
        self.drawable_entities[entity_index].mass = self.mass[entity_index] as f32;
        self.drawable_entities[entity_index].color = self.core[entity_index].color.center;
        self.drawable_entities[entity_index].color_2 = self.core[entity_index].color.edge;
    }

    #[inline]
    pub fn get_rect(&self, entity_index: usize) -> Rect<i32> {
        let entity_ref = self.get(entity_index);
        let entity_position = entity_ref.position();
        let radius = self.get_radius(entity_index) as i32;
        Rect::new(Point2D::new(entity_position.x - radius, entity_position.y - radius), Size2D::new(2 * radius, 2 * radius))
    }

    #[inline]
    pub fn get_radius(&self, entity_index: usize) -> f32 {
        (self.mass[entity_index] as f32 / std::f32::consts::PI).sqrt()
    }

    #[inline]
    pub fn get(&self, entity_index: usize) -> EntityRef {
        EntityRef {
            index: entity_index,
            entities: self,
        }
    }

    #[inline]
    pub fn get_mut(&mut self, entity_index: usize) -> EntityRefMut {
        EntityRefMut {
            index: entity_index,
            entities: self,
        }
    }

    pub fn len(&self) -> usize {
        self.core.len()
    }

    pub fn swap_remove(&mut self, index: usize) {
        self.core.swap_remove(index);
        self.position.swap_remove(index);
        self.speed.swap_remove(index);
        self.mass.swap_remove(index);
        self.direction.swap_remove(index);
        self.timer.swap_remove(index);
        self.flags.swap_remove(index);
        self.mass_evolution.swap_remove(index);
        self.lifetime.swap_remove(index);
        self.index_matrix_simple.swap_remove(index);
        self.special.swap_remove(index);
        self.buffer.swap_remove(index);
        self.buffer_is_some.swap_remove(index);
        self.buffer2.swap_remove(index);
        self.buffer2_is_some.swap_remove(index);
        self.drawable_entities.swap_remove(index);
    }

    pub fn clear(&mut self) {
        self.core.clear();
        self.position.clear();
        self.speed.clear();
        self.mass.clear();
        self.direction.clear();
        self.timer.clear();
        self.flags.clear();
        self.mass_evolution.clear();
        self.lifetime.clear();
        self.index_matrix_simple.clear();
        self.special.clear();
        self.buffer.clear();
        self.buffer_is_some.clear();
        self.buffer2.clear();
        self.buffer2_is_some.clear();
        self.drawable_entities.clear();
    }
}