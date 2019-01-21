use glutin::{self, dpi, VirtualKeyCode};

pub enum BoidControlEvent {
    Stop,
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

    pub fn process(&self, event: glutin::Event) -> Option<BoidControlEvent> {
        match event {
            glutin::Event::WindowEvent { event: e, .. } => self.process_window_event(e),
            _ => None,
        }
    }

    fn process_window_event(&self, event: glutin::WindowEvent) -> Option<BoidControlEvent> {
        use glutin::{ElementState, KeyboardInput, WindowEvent};
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
                _ => Some(BoidControlEvent::Key(key)),
            },

            WindowEvent::CursorMoved { position: pos, .. } => {
                let dpi::PhysicalPosition { x, y } = pos.to_physical(self.hidpi_factor);
                Some(BoidControlEvent::MouseMove(x as f32, y as f32))
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
