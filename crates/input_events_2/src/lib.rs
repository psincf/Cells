mod state;
use state::{State};

pub mod event;
use event::Event;

#[derive(Default)]
pub struct Events {
    pub state: State,
}

impl Events {
    pub fn update(&mut self, event: &winit::event::Event<()>) {
        self.state.update(event);
    }

    pub fn update_and_return_event(&mut self, event: &winit::event::Event<()>) -> Option<Event> {
        self.state.update(event);
        if let Some(event) = Event::from_winit(&self.state, event) {
            return Some(event);
        }
        return None
    }
}
