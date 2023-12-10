use std::{cell::RefCell, rc::Rc};

use beryllium::{Event, KeyInfo, Keycode, SDL};

pub trait Slot<'a> {
    fn on_signal(&mut self, signal: SignalType);
}

pub struct SignalHandler<'a> {
    sdl: Box<&'a SDL>,
    slots: Vec<Rc<RefCell<dyn Slot<'a>>>>,
}

impl<'a> SignalHandler<'a> {
    pub fn new(sdl: &'a SDL) -> Self {
        Self {
            sdl: Box::new(sdl),
            slots: vec![],
        }
    }
    pub fn connect(&mut self, slot: Rc<RefCell<dyn Slot<'a>>>) {
        self.slots.push(slot);
    }
    fn emit(&mut self, signal_value: SignalType) {
        for slot in &mut self.slots {
            (*slot.borrow_mut()).on_signal(signal_value);
        }
    }
    pub fn wait_event(&mut self) {
        let frame_start = self.sdl.get_ticks();
        loop {
            if let Some(event) = self.sdl.poll_events().and_then(Result::ok) {
                match event {
                    Event::Quit(_) => {
                        self.emit(SignalType::Quit);
                    }
                    Event::Keyboard(key_event) => {
                        if key_event.is_pressed {
                            self.emit(SignalType::KeyPressed(key_event.key.keycode));
                        } else {
                            self.emit(SignalType::KeyReleased(key_event.key.keycode));
                        }
                    }
                    Event::MouseMotion(motion_event) => {
                        self.emit(SignalType::MouseMoved(
                            motion_event.y_delta,
                            motion_event.x_delta,
                        ));
                    }
                    Event::MouseWheel(wheel_event) => {
                        self.emit(SignalType::MouseScrolled(wheel_event.y_delta));
                    }
                    _ => (),
                };
            } else {
                break;
            };
        }
        let frame_delta = self.sdl.get_ticks() - frame_start;
        self.sdl.delay_ms(16 - frame_delta.min(16));
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SignalType {
    KeyPressed(Keycode),
    KeyReleased(Keycode),
    MouseMoved(i32, i32),
    MouseScrolled(i32),
    Quit,
}

pub trait Controller<'a, O, T> {
    fn update_control_parameters(&self, update: &'a mut dyn FnMut(&mut T));
    fn process_signals(&'a self, obj: &mut O);
}
