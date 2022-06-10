use std::{collections::HashMap, cell::RefCell, rc::Rc};

use crate::{
    common::{LoxType, Token},
    interpreter::RuntimeException,
};

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, LoxType>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new(parent: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            values: HashMap::new(),
            parent,
        }
    }

    pub fn define(&mut self, name: String, value: LoxType) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<LoxType, RuntimeException> {
        if let Some(val) = self.values.get(&name.raw) {
            Ok(val.clone())
        } else if let Some(ref parent) = self.parent {
            parent.borrow().get(name)
        } else {
            Err(RuntimeException::report(
                name.clone(),
                &format!("Attempted to access undefined variable {}.", name.raw),
            ))
        }
    }

    pub fn assign(&mut self, name: &Token, value: LoxType) -> Result<(), RuntimeException> {
        if self.values.contains_key(&name.raw) {
            self.values.insert(name.raw.clone(), value);
            return Ok(());
        } else if let Some(ref mut parent) = self.parent {
            parent.borrow_mut().assign(name, value)?;
            return Ok(());
        } else {
            Err(RuntimeException::report(
                name.clone(),
                &format!("Attempted to assign to undefined variable {}", name.raw),
            ))
        }
    }
}
