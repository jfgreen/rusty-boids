use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

pub enum BoidControlEvent {
    Stop,
    Pause,
    Key(VirtualKeyCode),
    MouseMove(f32, f32),
    MousePress,
    MouseRelease,
}

pub struct EventFilter {
    hidpi_factor: f64,
}

impl EventFilter {
    pub fn new(hidpi_factor: f64) -> Self {
        EventFilter { hidpi_factor }
    }

    pub fn process<T>(&self, event: Event<T>) -> Option<BoidControlEvent> {
        match event {
            Event::WindowEvent { event: e, .. } => self.process_window_event(e),
            _ => None,
        }
    }

    fn process_window_event(&self, event: WindowEvent) -> Option<BoidControlEvent> {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(key),
                        ..
                    },
                ..
            } => match key {
                VirtualKeyCode::Escape | VirtualKeyCode::Q => Some(BoidControlEvent::Stop),
                VirtualKeyCode::Space => Some(BoidControlEvent::Pause),
                _ => Some(BoidControlEvent::Key(key)),
            },

            WindowEvent::CursorMoved { position: pos, .. } => {
                Some(BoidControlEvent::MouseMove(pos.x as f32, pos.y as f32))
            }

            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } => Some(BoidControlEvent::MousePress),

            WindowEvent::MouseInput {
                state: ElementState::Released,
                ..
            } => Some(BoidControlEvent::MouseRelease),

            WindowEvent::CloseRequested => Some(BoidControlEvent::Stop),
            _ => None,
        }
    }
}
