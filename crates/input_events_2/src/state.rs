use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;

struct ButtonStateGeneric<T: Clone + Copy + Eq + Hash> {
    inner: Mutex<HashMap<T, ButtonState>>,
}

impl<T: Clone + Copy + Eq + Hash> ButtonStateGeneric<T> {
    pub fn is_pressed(&self, keycode: T) -> bool {
        let button_state = self.inner.lock().unwrap();
        if let Some(button) = button_state.get(&keycode) {
            return button.is_pressed()
        } else {
            return false;
        }
    }

    pub fn take(&self, keycode: T) -> bool {
        let mut button_state = self.inner.lock().unwrap();
        if let Some(button) = button_state.get_mut(&keycode) {
            return button.take()
        } else {
            return false;
        }
    }

    fn press(&self, keycode: T) {
        let mut button_state = self.inner.lock().unwrap();
        if let Some(button) = button_state.get_mut(&keycode) {
            return button.press()
        } else {
            button_state.insert(keycode, ButtonState::default());
            button_state.get_mut(&keycode).unwrap().press();
        }
    }

    fn release(&self, keycode: T) {
        let mut button_state = self.inner.lock().unwrap();
        if let Some(button) = button_state.get_mut(&keycode) {
            return button.release()
        } else {
            button_state.insert(keycode, ButtonState::default());
            button_state.get_mut(&keycode).unwrap().release();
        }
    }
}

impl<T: Clone + Copy + Eq + Hash> Default for ButtonStateGeneric<T> {
    fn default() -> ButtonStateGeneric<T> {
        ButtonStateGeneric {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Default)]
pub struct ButtonState {
    pressed: bool,
    released: bool,
    used: bool,
}

impl ButtonState {
    fn press(&mut self) {
        if self.pressed == false {
            self.pressed = true;
            self.released = false;
            self.used = false;
        }
    }

    fn release(&mut self) {
        self.pressed = false;
        self.released = true;    
        self.used = false;    
    }

    fn is_pressed(&self) -> bool {
        self.pressed
    }

    fn take(&mut self) -> bool {
        if self.pressed == true && self.used == false {
            self.used = true;
            return true
        }
        return false
    }
}

#[derive(Default)]
pub struct MouseState {
    position: Mutex<(i32, i32)>,
    button: ButtonStateGeneric<winit::event::MouseButton>,
}

impl MouseState {
    pub fn get_position(&self) -> (i32, i32) {
        *self.position.lock().unwrap()
    }

    fn set_position(&self, position: (i32, i32)) {
        *self.position.lock().unwrap() = position;
    }

    pub fn is_pressed(&self, keycode: winit::event::MouseButton) -> bool {
        self.button.is_pressed(keycode)
    }

    pub fn take(&self, keycode: winit::event::MouseButton) -> bool {
        self.button.take(keycode)
    }

    fn press(&self, keycode: winit::event::MouseButton) {
        self.button.press(keycode)
    }

    fn release(&self, keycode: winit::event::MouseButton) {
        self.button.release(keycode)

    }
}

#[derive(Default)]
pub struct KeyboardState {
    button: ButtonStateGeneric<winit::event::VirtualKeyCode>,
}

impl KeyboardState {
    pub fn is_pressed(&self, keycode: winit::event::VirtualKeyCode) -> bool {
        self.button.is_pressed(keycode)
    }

    pub fn take(&self, keycode: winit::event::VirtualKeyCode) -> bool {
        self.button.take(keycode)
    }

    fn press(&self, keycode: winit::event::VirtualKeyCode) {
        self.button.press(keycode)
    }

    fn release(&self, keycode: winit::event::VirtualKeyCode) {
        self.button.release(keycode)

    }
}

#[derive(Default)]
pub struct State {
    pub mouse: MouseState,
    pub keyboard: KeyboardState,
}

impl State {
    pub fn update(&mut self, event: &winit::event::Event<()>) {
        match event {
            winit::event::Event::WindowEvent{event, ..} => {
                use winit::event::WindowEvent;
                match event {
                    WindowEvent::MouseInput{state, button, ..} => {
                        use winit::event::ElementState;
                        match state {
                            ElementState::Pressed => {
                                self.mouse.press(*button);
                            }
                            ElementState::Released => {
                                self.mouse.release(*button);
                            }
                        }
                    },

                    WindowEvent::CursorMoved{position, ..} => {
                        let position = (position.x as i32, position.y as i32);                            
                        self.mouse.set_position(position);
                    },

                    WindowEvent::KeyboardInput{input, ..} => {
                        use winit::event::ElementState;
                        match input.state {
                            ElementState::Pressed => {
                                if let Some(keycode) = input.virtual_keycode {
                                    self.keyboard.press(keycode);
                                }
                            }

                            ElementState::Released => {                                    
                                if let Some(keycode) = input.virtual_keycode {
                                    self.keyboard.release(keycode);
                                }
                            }
                        }
                    },
                    _ => {}

                }
            },
            _ => { }
        }
    }
}