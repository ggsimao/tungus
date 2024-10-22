use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::{Rc, Weak},
    time::{Duration, Instant},
};

use beryllium::{Event, KeyInfo, KeyboardEvent, Keycode, SDL};

pub trait Slot {
    fn on_signal(&mut self, signal: SignalType);
}

pub struct SignalHandler<'a> {
    sdl: &'a SDL,
    slots: Vec<Weak<RefCell<dyn Slot>>>,
}

impl<'a> SignalHandler<'a> {
    pub fn new(sdl: &'a SDL) -> Self {
        Self { sdl, slots: vec![] }
    }
    pub fn connect(&mut self, slot: Weak<RefCell<dyn Slot>>) {
        self.slots.push(slot);
    }
    fn emit(&self, signal_value: SignalType) {
        for slot in &self.slots {
            (*slot.upgrade().unwrap())
                .borrow_mut()
                .on_signal(signal_value);
        }
    }
    pub fn wait_event(&self) {
        // let frame_start = self.sdl.get_ticks();
        let mut new_keys_state = HashMap::new();
        while let Some(event) = self.sdl.poll_events().and_then(Result::ok) {
            match event {
                Event::Quit(_) => {
                    self.emit(SignalType::Quit);
                }
                Event::Keyboard(key_event) => {
                    let keycode = key_event.key.keycode;
                    let pressed = key_event.is_pressed;
                    new_keys_state.insert(keycode, pressed);
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
        }
        for (k, p) in new_keys_state {
            if p {
                self.emit(SignalType::KeyPressed(k));
            } else {
                self.emit(SignalType::KeyReleased(k));
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
