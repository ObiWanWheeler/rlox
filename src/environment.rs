use std::collections::HashMap;

use crate::{
    common::{LiteralType, Token},
    interpreter::RuntimeException,
};

#[derive(Clone, Debug)]
pub struct Environment {
    values: HashMap<String, LiteralType>,
    pub parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new(parent: Option<Box<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            parent,
        }
    }

    pub fn define(&mut self, name: String, value: LiteralType) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<LiteralType, RuntimeException> {
        if let Some(val) = self.values.get(&name.raw) {
            Ok(val.clone())
        } else if let Some(ref parent) = self.parent {
            parent.get(name)
        } else {
            Err(RuntimeException::report(
                name.clone(),
                &format!("Attempted to access undefined variable {}.", name.raw),
            ))
        }
    }

    pub fn assign(&mut self, name: &Token, value: LiteralType) -> Result<(), RuntimeException> {
        if self.values.contains_key(&name.raw) {
            self.values.insert(name.raw.clone(), value);
            return Ok(());
        } else if let Some(ref mut parent) = self.parent {
            parent.assign(name, value)?;
            return Ok(());
        } else {
            Err(RuntimeException::report(
                name.clone(),
                &format!("Attempted to assign to undefined variable {}", name.raw),
            ))
        }
    }
}
