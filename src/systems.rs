use std::{cell::RefCell, rc::Rc};

use beryllium::Keycode;

use crate::controls::{Controller, SignalHandler, SignalType, Slot};

pub struct Program {
    pub loop_active: bool,
    // pub timer: &'a dyn Fn() -> u32,
}

pub struct ProgramController {
    quit: bool,
}

impl<'a> ProgramController {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self { quit: false }))
    }
    pub fn on_key_pressed(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::ESCAPE => self.quit = true,
            _ => (),
        }
    }
}

impl<'a> Slot for ProgramController {
    fn on_signal(&mut self, signal: SignalType) {
        match signal {
            SignalType::KeyPressed(key) => self.on_key_pressed(key),
            _ => (),
        }
    }
}

impl<'a> Controller<'a, Program, ProgramController> for Rc<RefCell<ProgramController>> {
    fn update_control_parameters(&self, update: &'a mut (dyn FnMut(&mut ProgramController))) {
        update(&mut (**self).borrow_mut());
    }
    fn process_signals(&'a self, obj: &mut Program) {
        let self_obj = (**self).borrow_mut();
        obj.loop_active = !self_obj.quit;
    }
}
