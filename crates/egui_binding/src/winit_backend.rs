use winit::event::VirtualKeyCode;

pub struct WinitBackend {
    instant: std::time::Instant,
    //state: InnerState,
    raw_input: egui::RawInput,
    mouse_pos: egui::Pos2,
    clipboard: String,
}

impl WinitBackend {
    pub fn new() -> WinitBackend {
        WinitBackend {
            instant: std::time::Instant::now(),
            //state: InnerState::default(),
            raw_input: egui::RawInput::default(),
            mouse_pos: egui::Pos2::ZERO,
            clipboard: String::new(),
        }
    }

    pub fn update(&mut self, winit_window: &winit::window::Window, winit_event: &winit::event::Event<()>) {
        let mut egui_event = None;
        let mut scroll_delta = egui::Vec2::new(0.0, 0.0);
        
        match winit_event {
            winit::event::Event::WindowEvent {window_id: _, event } => {
                use winit::event::WindowEvent;
                match event {
                    WindowEvent::ReceivedCharacter(ch) => {
                        if *ch != std::char::from_u32(8).unwrap() 
                        && *ch != std::char::from_u32(127).unwrap()
                        && *ch != std::char::from_u32(3).unwrap()
                        && !ch.is_control()
                        && *ch != std::char::from_u32(16).unwrap() {
                            egui_event = Some(egui::Event::Text(ch.to_string()));
                        }
                    }
                    WindowEvent::KeyboardInput{ device_id: _, input, is_synthetic: _ } => {
                        let pressed = input.state == winit::event::ElementState::Pressed;
                        let mut key = None;
                        if let Some(virtual_keycode) = input.virtual_keycode {
                            match virtual_keycode {
                                VirtualKeyCode::Copy => { egui_event = Some(egui::Event::Copy); }
                                VirtualKeyCode::Cut => { egui_event = Some(egui::Event::Cut); }
    
                                VirtualKeyCode::Down => { key = Some(egui::Key::ArrowDown); }
                                VirtualKeyCode::Left => { key = Some(egui::Key::ArrowLeft); }
                                VirtualKeyCode::Right => { key = Some(egui::Key::ArrowRight); }
                                VirtualKeyCode::Up => { key = Some(egui::Key::ArrowUp); }
                                VirtualKeyCode::Back => { key = Some(egui::Key::Backspace); }
                                VirtualKeyCode::Delete => { key = Some(egui::Key::Delete); }
                                VirtualKeyCode::End => { key = Some(egui::Key::End); }
                                VirtualKeyCode::Return => { key = Some(egui::Key::Enter); }
                                VirtualKeyCode::Space => { key = Some(egui::Key::Space); }
                                VirtualKeyCode::Escape => { key = Some(egui::Key::Escape); }
                                VirtualKeyCode::Home => { key = Some(egui::Key::Home); }
                                VirtualKeyCode::Insert => { key = Some(egui::Key::Insert); }
                                VirtualKeyCode::PageDown => { key = Some(egui::Key::PageDown); }
                                VirtualKeyCode::PageUp => { key = Some(egui::Key::PageUp); }
                                VirtualKeyCode::Tab => { key = Some(egui::Key::Tab); }
                                VirtualKeyCode::A => { key = Some(egui::Key::A); }
                                VirtualKeyCode::B => { key = Some(egui::Key::B); }
                                VirtualKeyCode::C => { key = Some(egui::Key::C); 
                                    if self.raw_input.modifiers.command && pressed {  
                                        egui_event = Some(egui::Event::Copy);
                                        key = None;
                                    }
                                }
                                VirtualKeyCode::D => { key = Some(egui::Key::D); }
                                VirtualKeyCode::E => { key = Some(egui::Key::E); }
                                VirtualKeyCode::F => { key = Some(egui::Key::F); }
                                VirtualKeyCode::G => { key = Some(egui::Key::G); }
                                VirtualKeyCode::H => { key = Some(egui::Key::H); }
                                VirtualKeyCode::I => { key = Some(egui::Key::I); }
                                VirtualKeyCode::J => { key = Some(egui::Key::J); }
                                VirtualKeyCode::K => { key = Some(egui::Key::K); }
                                VirtualKeyCode::L => { key = Some(egui::Key::L); }
                                VirtualKeyCode::M => { key = Some(egui::Key::M); }
                                VirtualKeyCode::N => { key = Some(egui::Key::N); }
                                VirtualKeyCode::O => { key = Some(egui::Key::O); }
                                VirtualKeyCode::P => { key = Some(egui::Key::P); }
                                VirtualKeyCode::Q => { key = Some(egui::Key::Q); }
                                VirtualKeyCode::R => { key = Some(egui::Key::R); }
                                VirtualKeyCode::S => { key = Some(egui::Key::S); }
                                VirtualKeyCode::T => { key = Some(egui::Key::T); }
                                VirtualKeyCode::U => { key = Some(egui::Key::U); }
                                VirtualKeyCode::V => { key = Some(egui::Key::V);
                                    if self.raw_input.modifiers.command && pressed {  
                                        egui_event = Some(egui::Event::Text(self.clipboard.clone()));
                                        key = None;
                                    }
                                }
                                VirtualKeyCode::W => { key = Some(egui::Key::W); }
                                VirtualKeyCode::X => { key = Some(egui::Key::X);
                                    if self.raw_input.modifiers.command && pressed {  
                                        egui_event = Some(egui::Event::Cut);
                                        key = None;
                                    }
                                }
                                VirtualKeyCode::Y => { key = Some(egui::Key::Y); }
                                VirtualKeyCode::Z => { key = Some(egui::Key::Z); }
                                
                                VirtualKeyCode::LAlt => { self.raw_input.modifiers.alt = pressed; }
                                VirtualKeyCode::RAlt => { self.raw_input.modifiers.alt = pressed; }
                                VirtualKeyCode::LControl => { self.raw_input.modifiers.ctrl = pressed; self.raw_input.modifiers.command = pressed; }
                                VirtualKeyCode::RControl => { self.raw_input.modifiers.ctrl = pressed; self.raw_input.modifiers.command = pressed; }
                                VirtualKeyCode::LShift => { self.raw_input.modifiers.shift = pressed; }
                                VirtualKeyCode::RShift => { self.raw_input.modifiers.shift = pressed; }
                                _ => {  }
                            }
                        }
    
                        if let Some(key) = key {
                            egui_event = Some(egui::Event::Key {
                                key,
                                pressed,
                                modifiers: self.raw_input.modifiers
                            });
                        }
                    }
                    WindowEvent::MouseInput{ device_id: _, state, button ,..} => {
                        let pressed = *state == winit::event::ElementState::Pressed;
                        use winit::event::MouseButton;
                        match button {
                            MouseButton::Left => {
                                egui_event = Some(egui::Event::PointerButton {
                                    pos: self.mouse_pos,
                                    button: egui::PointerButton::Primary,
                                    pressed: pressed,
                                    modifiers: self.raw_input.modifiers,
                                });
                            }
                            MouseButton::Right => {
                                egui_event = Some(egui::Event::PointerButton {
                                    pos: self.mouse_pos,
                                    button: egui::PointerButton::Secondary,
                                    pressed: pressed,
                                    modifiers: self.raw_input.modifiers,
                                });
                            }
                            MouseButton::Middle => {
                                egui_event = Some(egui::Event::PointerButton {
                                    pos: self.mouse_pos,
                                    button: egui::PointerButton::Middle,
                                    pressed: pressed,
                                    modifiers: self.raw_input.modifiers,
                                });
                            }
                            _ => {}
                        }
                    }
                    WindowEvent::CursorMoved{ device_id: _, position,..} => {
                        self.mouse_pos = egui::Pos2::new(position.x as f32, position.y as f32);
                        egui_event = Some(egui::Event::PointerMoved(self.mouse_pos));
                    }
                    WindowEvent::MouseWheel{ device_id: _, delta, phase: _, ..} => {
                        use winit::event::MouseScrollDelta;
                        match delta {
                            MouseScrollDelta::LineDelta(x, y) => { scroll_delta.x += x * 50.0; scroll_delta.y += y * 50.0; }
                            MouseScrollDelta::PixelDelta(_xy) => {  }
                        }
                    }
                    _ => {  }
                }
            }
           _ => {  }
        }

        let scroll_delta = self.raw_input.scroll_delta + scroll_delta;
        let winit_size: [f32;2] = winit_window.inner_size().into();
        let screen_rect = Some(egui::Rect {
            min: egui::Pos2::new(0.0, 0.0),
            max: egui::Pos2::from(winit_size),
        });
        let pixels_per_point = Some(winit_window.scale_factor() as f32);
        let time = Some(self.instant.elapsed().as_secs_f64());
        let modifiers = self.raw_input.modifiers;
        //let events = if let Some(event) = egui_event { vec![event] } else { vec![] };
        if let Some(event) = egui_event { self.raw_input.events.push(event); }
        let events = self.raw_input.events.clone();
        
        self.raw_input = egui::RawInput {
            scroll_delta,
            screen_rect,
            pixels_per_point,
            time,
            modifiers,
            events,
            ..Default::default()
        }
    }

    pub fn update_end_frame(&mut self, output: &egui::Output) {
        if !output.copied_text.is_empty() {
            self.clipboard = output.copied_text.clone();
        }
    }

    pub fn take(&mut self) -> egui::RawInput {
        let mut to_return = self.raw_input.clone();
        to_return.time = Some(self.instant.elapsed().as_secs_f64());
        self.raw_input.events.clear();
        self.raw_input.scroll_delta = egui::Vec2::ZERO;

        return to_return;
    }
}