#[derive(Clone, PartialEq)]
pub enum Event {
    Keyboard(KeyboardButtonEvent),
    MouseButton(MouseButtonEvent),
    MouseMoved(MouseMoved),
    MouseWheel(MouseWheel),
}

#[derive(Clone, PartialEq)]
pub enum ButtonEventKind {
    Pressed,
    Released,
}

#[derive(Clone, PartialEq)]
pub enum KeyboardButton {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    Space,

    Up,
    Down,
    Left,
    Right,

    Add,
    Subtract,
    Multiply,
    Divide,

    RCtrl,
    Delete,
}

#[derive(Clone, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Clone, PartialEq)]
pub struct KeyboardButtonEvent {
    pub kind: ButtonEventKind,
    pub button: KeyboardButton,
}

#[derive(Clone, PartialEq)]
pub struct MouseButtonEvent {
    pub kind: ButtonEventKind,
    pub button: MouseButton,
    pub location: (i32, i32),
}

#[derive(Clone, PartialEq)]
pub struct MouseMoved {
    pub location: (i32, i32),
}

#[derive(Clone, PartialEq)]
pub struct MouseWheel {
    pub amount: i32,
    pub location: (i32, i32),
}

impl Event {
    pub fn from_winit(state: &crate::state::State, event: &winit::event::Event<()>) -> Option<Event> {
        match event {
            winit::event::Event::WindowEvent { window_id: _, event} => {
                use winit::event::WindowEvent;
                use winit::event::ElementState;
                match event {
                    WindowEvent::KeyboardInput { device_id: _, input, is_synthetic: _ } => {
                        let button_event_kind = match input.state {
                            ElementState::Pressed => { ButtonEventKind::Pressed }
                            ElementState::Released => { ButtonEventKind::Released }
                        };
                        if let Some(key_code) = input.virtual_keycode {
                            use winit::event::VirtualKeyCode;
                            let keyboard_button = match key_code {
                                VirtualKeyCode::A => { KeyboardButton::A }
                                VirtualKeyCode::B => { KeyboardButton::B }
                                VirtualKeyCode::C => { KeyboardButton::C }
                                VirtualKeyCode::D => { KeyboardButton::D }
                                VirtualKeyCode::E => { KeyboardButton::E }
                                VirtualKeyCode::F => { KeyboardButton::F }
                                VirtualKeyCode::G => { KeyboardButton::G }
                                VirtualKeyCode::H => { KeyboardButton::H }
                                VirtualKeyCode::I => { KeyboardButton::I }
                                VirtualKeyCode::J => { KeyboardButton::J }
                                VirtualKeyCode::K => { KeyboardButton::K }
                                VirtualKeyCode::L => { KeyboardButton::L }
                                VirtualKeyCode::M => { KeyboardButton::M }
                                VirtualKeyCode::N => { KeyboardButton::N }
                                VirtualKeyCode::O => { KeyboardButton::O }
                                VirtualKeyCode::P => { KeyboardButton::P }
                                VirtualKeyCode::Q => { KeyboardButton::Q }
                                VirtualKeyCode::R => { KeyboardButton::R }
                                VirtualKeyCode::S => { KeyboardButton::S }
                                VirtualKeyCode::T => { KeyboardButton::T }
                                VirtualKeyCode::U => { KeyboardButton::U }
                                VirtualKeyCode::V => { KeyboardButton::V }
                                VirtualKeyCode::W => { KeyboardButton::W }
                                VirtualKeyCode::X => { KeyboardButton::X }
                                VirtualKeyCode::Y => { KeyboardButton::Y }
                                VirtualKeyCode::Z => { KeyboardButton::Z }

                                VirtualKeyCode::Space => { KeyboardButton::Space }

                                VirtualKeyCode::Up => { KeyboardButton::Up }
                                VirtualKeyCode::Down => { KeyboardButton::Down }
                                VirtualKeyCode::Left => { KeyboardButton::Left }
                                VirtualKeyCode::Right => { KeyboardButton::Right }

                                VirtualKeyCode::NumpadAdd => { KeyboardButton::Add }
                                VirtualKeyCode::NumpadSubtract => { KeyboardButton::Subtract }
                                VirtualKeyCode::NumpadMultiply => { KeyboardButton::Multiply }
                                VirtualKeyCode::NumpadDivide => { KeyboardButton::Divide }

                                VirtualKeyCode::RControl => { KeyboardButton::RCtrl }
                                VirtualKeyCode::Delete => { KeyboardButton::Delete }
                                _ => { return None; }
                            };
                            return Some(Event::Keyboard(KeyboardButtonEvent {
                                kind: button_event_kind,
                                button: keyboard_button,
                            }))
                        }
                    }
                    WindowEvent::MouseInput { device_id: _, state: button_state, button, ..  } => {
                        let button_event_kind = match button_state {
                            ElementState::Pressed => { ButtonEventKind::Pressed }
                            ElementState::Released => { ButtonEventKind::Released }
                        };
                        let mouse_button = match button {
                            winit::event::MouseButton::Left => { MouseButton::Left }
                            winit::event::MouseButton::Right => { MouseButton::Right }
                            winit::event::MouseButton::Middle => { MouseButton::Middle }
                            winit::event::MouseButton::Other(_) => { return None; }
                        };
                        let position = state.mouse.get_position();

                        return Some(Event::MouseButton(MouseButtonEvent {
                            kind: button_event_kind,
                            button: mouse_button,
                            location: position,
                        }))

                    }
                    WindowEvent::MouseWheel {  device_id: _, delta: _, phase: _, .. } => {

                    }
                    WindowEvent::CursorMoved {  device_id: _, position: _, .. } => {
                        let position = state.mouse.get_position();
                        return Some(Event::MouseMoved(MouseMoved {
                            location: position,
                        }))
                    }
                    _ => {  }
                }
            }
            _ => {  }
        }
        return None;
    }
}