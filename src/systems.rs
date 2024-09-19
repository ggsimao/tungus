use std::{cell::RefCell, rc::Rc};

use beryllium::Keycode;

use crate::controls::{Controller, SignalHandler, SignalType, Slot};

pub struct Program<'a> {
    pub loop_active: bool,
    pub timer: &'a dyn Fn() -> u32,
}

pub struct ProgramController {
    signal_list: Vec<SignalType>,
    quit: bool,
}

impl<'a> ProgramController {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            signal_list: vec![],
            quit: false,
        }))
    }
    pub fn on_key_pressed(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::ESCAPE => self.quit = true,
            _ => (),
        }
    }
}

impl<'a> Slot<'a> for ProgramController {
    fn on_signal(&mut self, signal: SignalType) {
        self.signal_list.push(signal);
    }
}

impl<'a> Controller<'a, Program<'a>, ProgramController> for Rc<RefCell<ProgramController>> {
    fn update_control_parameters(&self, update: &'a mut (dyn FnMut(&mut ProgramController))) {
        update(&mut (**self).borrow_mut());
    }
    fn process_signals(&'a self, obj: &mut Program) {
        let mut self_obj = (**self).borrow_mut();
        for signal in self_obj.signal_list.clone() {
            match signal {
                SignalType::KeyPressed(key) => self_obj.on_key_pressed(key),
                _ => (),
            }
        }
        obj.loop_active = !self_obj.quit;
        self_obj.signal_list.clear();
    }
}
