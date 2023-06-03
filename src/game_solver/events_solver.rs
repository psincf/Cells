use std::{borrow::Borrow};

use crate::prelude::*;

use crate::game::entity::entities::Entities;
use crate::window::Events;
use crate::new_timer_monothread;
use crate::renderer::Camera;

use euclid::default::{Point2D, Vector2D};
use winit::event::VirtualKeyCode;

pub struct EventsSolver<'a> {
    game: &'a mut Game,
    events: &'a mut Events,
    camera: Camera,
}

impl<'a> EventsSolver<'a> {
    pub fn new(game: &'a mut Game, events: &'a mut Events, camera: Camera) -> EventsSolver<'a> {
        EventsSolver {
            game,
            events,
            camera
        }
    }

    pub fn solve(&mut self) {
        match self.game.state {
            GameState::MainMenu => {
                
            }
            GameState::Editor => {
                self.check_events_editor();
            }
            GameState::Playing => {
                self.check_events_playing();
                
                #[cfg(not(feature = "shipping"))]
                if *crate::DEBUG.get() == true {
                    self.check_events_playing_debug();
                }
            }
        }
        self.events.buffer_events.clear();
    }

    fn check_events_playing(&mut self) {
        new_timer_monothread!(_t, "check_events");
        let local_player = unsafe { & *(&self.game.players[self.game.settings.local_player] as *const Player) };
        let entities = unsafe { &*(&self.game.entities as *const Entities) };

        match self.game.gui.state.try_borrow().unwrap().clone() {
            crate::game::gui::GUIState::Open(_) => {
                for entity_index in local_player.entities.iter() {
                    self.game.entities.direction[*entity_index] = None;
                }
                return
            }
            _ => {}
        }
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // If mouse is not pointing GUI, update the position of mouse in the world. NECESSARY!!
        if !self.events.mouse_events.mouse_on_gui {
            let size = *self.events.resize_events.size.lock().unwrap();
            self.events.mouse_events.update_mouse_position_world(size, &self.camera);
            for entity_index in local_player.entities.iter() {
                self.game.entities.direction[*entity_index] = Some(self.events.mouse_events.mouse_position_world);
            }
        }

        for (event, _camera_and_position_world) in self.events.buffer_events.iter() {
            use input_events::event::ButtonEventKind;
            match event {
                input_events::event::Event::Keyboard(event) => {
                    let (_camera, position_world) = _camera_and_position_world.as_ref().unwrap();
                    use input_events::event::KeyboardButton;
                    if event.kind == ButtonEventKind::Pressed {
                        match event.button {
                            KeyboardButton::R => {
                                if local_player.entities.is_empty() {
                                    self.game.new_entity(EntityInfo {
                                        player: 1,
                                        position: Point2D::new(rng.gen_range(0..self.game.map.max().width), rng.gen_range(0..self.game.map.max().height)),
                                        speed: Vector2D::new(0.0, 0.0),
                                        mass: 1_000_000,
                                        characteristics: self.game.settings.local_player_characteristics.clone(),
                                        timer: EntityTimer::default(),
                                        color: self.game.players[self.game.settings.local_player].cell_default_color,
                                        texture: self.game.players[self.game.settings.local_player].cell_default_texture,
                                    });
                                }
                            }
                            KeyboardButton::Space => {
                                for i in 0..local_player.entities.len() {
                                    if local_player.entities.len() < self.game.settings.max_split {
                                        let entity_index = local_player.entities[i];
                                        let mut entity: EntityRefMut = unsafe { std::mem::transmute(self.game.entities.get_mut(entity_index)) };
                                        let entity_core = &mut self.game.entities.core[entity_index];
                                        let entity_position = self.game.entities.position[entity_index];
                                        let entity_speed = self.game.entities.speed[entity_index];
                                        let entity_mass = self.game.entities.mass[entity_index];
                                        let entity_radius = entity.get_radius();
                                        let entity_timer = &mut self.game.entities.timer[entity_index];
                                        let mut entity_info = None;
                                        if entity_mass > 20_000_000 {
                                            let radius = entity_radius;
                                            let direction = Vector2D::new(position_world.x, position_world.y);
                                            let mut speed = (direction - entity_position.to_vector()).to_f32();
                                            if speed.length() == 0.0 { speed.x = 1.0; }
                                            let ratio_speed = entity_radius / speed.length();
                                            speed *= ratio_speed * self.game.settings.unit_speed_split;
                                            let ratio_position = (radius / 2.0) / speed.length();
                                            entity_info = Some(EntityInfo {
                                                player: 1,
                                                position: Point2D::new(entity_position.x + (speed.x * ratio_position) as i32, entity_position.y + (speed.y * ratio_position) as i32),
                                                speed: Vector2D::new(speed.x + entity_speed.x, speed.y + entity_speed.y),
                                                //speed: Vector2D::new(speed.x + entity.speed.x, speed.y + entity.speed.y),
                                                mass: entity_mass / 2,
                                                characteristics: entity_core.characteristics.clone(),
                                                timer: EntityTimer {
                                                    collision: Some(1),
                                                    collision_ratio: Some(10),
                                                    mergeable: Some(1_000),
                                                    inertia: Some(20),
                                                    ..Default::default()
                                                },
                                                color: entity_core.color,
                                                texture: entity_core.index.texture,
                                            });
                                            entity_timer.mergeable = Some(1_000); // TODO: in buffer instead
                                        }
                                        if let Some(entity_info) = entity_info {
                                            /*
                                            entity.buffer.send(EntityAction::AddMass(-entity.mass / 2));
                                            self.game.buffer_add_entity.send(Box::new(entity_info));
                                            */
                                            *entity.mass_mut() /= 2;
                                            self.game.entities.drawable_entities[entity_index].mass = entity.mass() as f32;
                                            self.game.map.update_entity(entities, entity_core); //TODO: Not good. But needed if multiple spaces in one update. Find another solution
                                            self.game.new_entity(entity_info);
                                        }
                                    }
                                }
                            }
                            _ => {  }
                        }
                    }
                }
                input_events::event::Event::MouseButton(_event) => {
                    
                }
                input_events::event::Event::MouseMoved(_event) => {
                    
                }
                input_events::event::Event::MouseWheel(_event) => {
                    
                }
            }
        }

        if self.events.input_events.state.keyboard.is_pressed(VirtualKeyCode::W) {
            let throw_food_info = self.game.settings.local_player_food_settings.clone();
            for i in 0..local_player.entities.len() {
                for _ in 0..throw_food_info.throw_ratio {
                    let entity_index = local_player.entities[i];
                    let entity: EntityRefMut = unsafe { std::mem::transmute(self.game.entities.get_mut(entity_index)) };
                    let entity_core: &EntityCore = unsafe { std::mem::transmute(&self.game.entities.core[entity_index]) };
                    let entity_position = self.game.entities.position[entity_index];
                    let mut entity_info = None;
                    if entity.mass() > throw_food_info.mass_minimum_to_throw {
                        self.game.entities.send_buffer(entity_index, EntityAction::AddMass(throw_food_info.mass_self_added));
                        let radius = entity.get_radius();
                        let direction_point = Point2D::new(self.events.mouse_events.mouse_position_world.x, self.events.mouse_events.mouse_position_world.y);
                        let mut direction = (direction_point - entity_position).to_f32();
                        let direction_angle = direction.angle_from_x_axis();
                        let direction_speed_angle = euclid::Angle::degrees(direction_angle.to_degrees() + rng.gen_range(throw_food_info.angle.clone()));
                        let direction_speed = Vector2D::from_angle_and_length(direction_speed_angle, 1.0);

                        if direction.length() == 0.0 { direction.x = 1.0; }
                        direction = direction.normalize();

                        let ratio_speed = rng.gen_range(throw_food_info.power.clone()) as f32 * ((entity.mass() as f32).log10());
                        let speed = direction_speed * ratio_speed;

                        let color = match &throw_food_info.color {
                            crate::game::entity::ThrownEntityColor::Same => {
                                entity_core.color
                            }
                            crate::game::entity::ThrownEntityColor::Custom(color) => {
                                *color
                            }
                            crate::game::entity::ThrownEntityColor::Random(vec) => {
                                let index = rng.gen_range(0..vec.len());
                                vec[index]
                            }
                        };

                        let ratio_position = radius + 10.0;
                        entity_info = Some(EntityInfo {
                            player: 0,
                            position: Point2D::new(entity_position.x + (direction.x * ratio_position) as i32, entity_position.y + (direction.y * ratio_position) as i32),
                            speed: Vector2D::new(speed.x, speed.y),
                            mass: throw_food_info.mass_entity_thrown,
                            characteristics: throw_food_info.characteristics_entity_thrown.clone(),
                            timer: throw_food_info.timer.clone(),
                            color: color,
                            texture: entity_core.index.texture,
                        });
                    }
                    if let Some(entity_info) = entity_info {
                        self.game.new_entity(entity_info);
                    }
                }
            }
        }
    }

    #[cfg(not(feature = "shipping"))]
    fn check_events_playing_debug(&mut self) {
        let local_player = unsafe { & *(&self.game.players[self.game.settings.local_player] as *const Player) };
        let entities = unsafe { &*(&self.game.entities as *const Entities) };

        for (event, camera_and_position_world) in self.events.buffer_events.iter() {
            use input_events::event::ButtonEventKind;
            match event {
                input_events::event::Event::Keyboard(event) => {
                    use input_events::event::KeyboardButton;
                    if event.kind == ButtonEventKind::Pressed {
                        match event.button {
                            KeyboardButton::E => { 
                                for entity_index in self.game.players[self.game.settings.local_player].entities.iter() {
                                    self.game.entities.timer[*entity_index].mergeable = None;
                                }
                            }
                            KeyboardButton::R => {
                                for entity_index in local_player.entities.iter() {
                                    let mut entity = self.game.entities.get_mut(*entity_index);
                                    *entity.mass_mut() = 1_000_000;
                                    self.game.map.update_entity(entities, entity.core_mut());
                                    self.game.entities.drawable_entities[*entity_index].mass = entity.mass() as f32;
                                }
                            }
                            
                            KeyboardButton::K => {
                                
                            }

                            KeyboardButton::Up => { self.game.threadpool.set_threads(self.game.threadpool.num_threads() + 1); }
                            KeyboardButton::Down => { self.game.threadpool.set_threads(self.game.threadpool.num_threads().max(1) - 1); }

                            KeyboardButton::Add => { self.game.settings.unit_speed += 100.0; }
                            KeyboardButton::Subtract => { self.game.settings.unit_speed -= 100.0; }
                            KeyboardButton::Multiply => { self.game.settings.unit_speed *= 2.0; }
                            KeyboardButton::Divide => { self.game.settings.unit_speed /= 2.0; }
                            _ => {  }
                        }
                    }
                }
                input_events::event::Event::MouseButton(event) => {
                    match event.button {
                        input_events::event::MouseButton::Right => {
                            if event.kind == input_events::event::ButtonEventKind::Pressed {
                                for i in local_player.entities.iter() {
                                    let entity = &mut self.game.entities.core[*i];
                                    let entity_position = &mut self.game.entities.position[*i];
                                    let position = camera_and_position_world.as_ref().unwrap().1.clone();
                                    //entity.buffer.send(EntityAction::AddPosition(position.x.max(0).min(self.game.map.max().x) - entity.position.x, position.y.max(0).min(self.game.map.max().y) - entity.position.y));
                                    entity_position.x = position.x.max(0).min(self.game.map.max().width);
                                    entity_position.y = position.y.max(0).min(self.game.map.max().height);
                                    self.game.map.update_entity(entities, entity);
                                    self.game.entities.update_drawable(*i);
                                }
                            }
                        }
                        _ => {  }
                    }
                }
                input_events::event::Event::MouseMoved(_event) => {
                    
                }
                input_events::event::Event::MouseWheel(_event) => {
                    
                }
            }
        }
        if self.events.input_events.state.keyboard.is_pressed(VirtualKeyCode::Z) {
            let local_player = &self.game.players[self.game.settings.local_player];
            for i in local_player.entities.iter() {
                let entity = &mut self.game.entities.get_mut(*i);
                *entity.mass_mut() += 100_000_000;
                self.game.entities.drawable_entities[*i].mass = entity.mass() as f32;
            }
        }
        
        #[cfg(feature = "serialize")]
        (|| {
            if self.events.input_events.state.keyboard.take(VirtualKeyCode::S) { self.game.to_binary_file(); }
            if self.events.input_events.state.keyboard.take(VirtualKeyCode::Q) { self.game.from_binary_file(); }
            if self.events.input_events.state.keyboard.take(VirtualKeyCode::O) { self.game.to_ron_file(); }
            if self.events.input_events.state.keyboard.take(VirtualKeyCode::P) { self.game.from_ron_file(); }
        })();

        if self.events.input_events.state.keyboard.take(VirtualKeyCode::D) { self.game.state = GameState::Editor }
    }

    fn check_events_editor(&mut self) {
        new_timer_monothread!(_t, "check_events");
        /*
        if self.events.input_events.state.keyboard.is_pressed(VirtualKeyCode::Up) { APP.get_mut().renderer.camera_future.y -= (10.0 * self.camera.size) as i32; }
        if self.events.input_events.state.keyboard.is_pressed(VirtualKeyCode::Down) { APP.get_mut().renderer.camera_future.y += (10.0 * self.camera.size) as i32; }
        if self.events.input_events.state.keyboard.is_pressed(VirtualKeyCode::Left) { APP.get_mut().renderer.camera_future.x -= (10.0 * self.camera.size) as i32; }
        if self.events.input_events.state.keyboard.is_pressed(VirtualKeyCode::Right) { APP.get_mut().renderer.camera_future.x += (10.0 * self.camera.size) as i32; }
        */
        if self.events.input_events.state.keyboard.take(VirtualKeyCode::D) { self.game.state = GameState::Playing }

        if self.game.editor_state.selection.is_some() && self.events.input_events.state.mouse.is_pressed(winit::event::MouseButton::Left) {
            if self.game.editor_state.selection.is_some() {
                self.events.mouse_events.update_mouse_position_world(self.events.resize_events.size.lock().unwrap().clone(), &self.camera);
                let mut selected = Vec::new();
                let mut selected_atomic = Vec::new();
                let selection = euclid::default::Rect::from_points([self.game.editor_state.selection.unwrap(), self.events.mouse_events.mouse_position_world].iter());

                let min_selection = selection.min().max(Point2D::zero()) / self.game.map.matrix_simple.size_field;
                let min_x = min_selection.x;
                let min_y = min_selection.y;
                let max_selection = selection.max() / self.game.map.matrix_simple.size_field + Vector2D::new(1, 1);
                let max_x = max_selection.x.min(self.game.map.matrix_simple.size.width - 1);
                let max_y = max_selection.y.min(self.game.map.matrix_simple.size.height - 1);

                for x in min_x..=max_x {
                    for y in min_y..=max_y {
                        for entity in self.game.map.matrix_simple[x as usize][y as usize].iter() {
                            if selection.contains(entity.position) {
                                selected.push(entity.entity);
                                selected_atomic.push(std::sync::Arc::downgrade(&self.game.entities.core[entity.entity].index.main_ptr));
                            }
                        }
                    }
                }
                self.game.editor_state.selected = Some(selected);
                self.game.editor_state.selected_atomic = Some(selected_atomic);
            }
        }

        for event in self.events.buffer_events.iter() {
            if let input_events::event::Event::Keyboard(keyboard_event) = &event.0 {
                if keyboard_event.button == input_events::event::KeyboardButton::Delete {
                    if self.game.editor_state.selected.is_some() {
                        if keyboard_event.kind == input_events::event::ButtonEventKind::Pressed {
                            let entities_to_delete = self.game.editor_state.selected_atomic.clone().unwrap();
                            for e in entities_to_delete.iter() {
                                self.game.delete_entity(e.upgrade().unwrap().load(std::sync::atomic::Ordering::Relaxed));
                            }
                            self.game.editor_state.selected_atomic.take();
                            self.game.editor_state.selected.take();
                        }
                    }
                }
            }

            if let input_events::event::Event::MouseMoved(_moved_event) = &event.0 {
            }

            if let input_events::event::Event::MouseButton(button_event) = &event.0 {
                if button_event.button == input_events::event::MouseButton::Left {
                    if button_event.kind == input_events::event::ButtonEventKind::Pressed {
                        let size = self.game.gui.state_2.borrow().size_width_editor as i32;
                        if button_event.location.0 > size && button_event.location.0 < APP.get().window.window.inner_size().width as i32 - size {
                            if self.game.editor_state.tab == 1 || self.game.editor_state.tab == 2 {
                                self.game.editor_state.selected = None;
                                self.events.mouse_events.update_mouse_position_world(self.events.resize_events.size.lock().unwrap().clone(), &self.camera);
                                self.game.editor_state.selection = Some(self.events.mouse_events.mouse_position_world);
                            }
                            if self.game.editor_state.tab == 0 {
                                if !self.game.editor_state.hovered && self.game.editor_state.new_entity_on_click {
                                    self.events.mouse_events.update_mouse_position_world(self.events.resize_events.size.lock().unwrap().clone(), &self.camera);
                                    let mut info = self.game.editor_state.new_entity.clone();
                                    info.position = self.events.mouse_events.mouse_position_world;
                                    self.game.new_entity(info);
                                }
                            }
                        }
                    }
                    if button_event.kind == input_events::event::ButtonEventKind::Released {
                        if self.game.editor_state.selection.is_some() {
                            self.game.editor_state.selection = None;
                        }
                    }
                }
            }
        }
        
        if self.game.editor_state.tab == 0 {
            self.events.mouse_events.update_mouse_position_world(self.events.resize_events.size.lock().unwrap().clone(), &self.camera);
            let game_bis = unsafe { &*(self.game as *mut Game) };
            let mut info = &mut self.game.editor_state.new_entity;
            info.position = self.events.mouse_events.mouse_position_world;
            info.validate(game_bis);
        }

        self.game.editor_state.hovered = false;
    }
}