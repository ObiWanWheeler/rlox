use std::{cell::RefCell, rc::Rc};

use crate::{
    common::{LoxCallable, LoxType},
    interpreter::RuntimeException,
};

pub struct Clock;

impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _: &mut crate::interpreter::Interpreter,
        _: Vec<Rc<RefCell<LoxType>>>,
    ) -> Result<Rc<RefCell<LoxType>>, RuntimeException> {
        Ok(Rc::new(RefCell::new(LoxType::Number(
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as f32,
        ))))
    }
}
