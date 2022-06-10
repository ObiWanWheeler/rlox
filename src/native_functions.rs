use crate::{common::{LoxCallable, LoxType}};

pub struct Clock;

impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0 
    }

    fn call(&self, _: &mut crate::interpreter::Interpreter, _: Vec<crate::common::LoxType>) -> Result<crate::common::LoxType, crate::interpreter::RuntimeException> {
        Ok(LoxType::Number(std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_millis() as f32))
    }
}
