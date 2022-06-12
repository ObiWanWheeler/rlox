use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    common::{LoxType, Token},
    interpreter::RuntimeException,
};

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Rc<RefCell<LoxType>>>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new(parent: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            values: HashMap::new(),
            parent,
        }
    }

    pub fn define(&mut self, name: String, value: Rc<RefCell<LoxType>>) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Rc<RefCell<LoxType>>, RuntimeException> {
        if let Some(val) = self.values.get(&name.raw) {
            Ok(Rc::clone(val))
        } else if let Some(ref parent) = self.parent {
            RefCell::borrow(&parent).get(name)
        } else {
            Err(RuntimeException::report(
                name.clone(),
                &format!("Attempted to access undefined variable {}.", name.raw),
            ))
        }
    }

    pub fn get_at(
        &self,
        distance: usize,
        name: &Token,
    ) -> Result<Rc<RefCell<LoxType>>, RuntimeException> {
        if distance == 0 {
            match self.values.get(&name.raw) {
                Some(v) => Ok(Rc::clone(v)),
                None => Err(RuntimeException::report(
                    name.clone(),
                    &format!(
                        "No variable with name {} at depth {}",
                        name.raw.clone(),
                        distance
                    ),
                )),
            }
        } else {
            match RefCell::borrow(&self.ancestor(distance))
                .values
                .get(&name.raw)
            {
                Some(v) => Ok(Rc::clone(v)),
                None => Err(RuntimeException::report(
                    name.clone(),
                    &format!(
                        "No variable with name {} at depth {}",
                        name.raw.clone(),
                        distance
                    ),
                )),
            }
        }
    }

    fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment>> {
        let mut env = self.parent().expect("No parent scope at this distance");
        for _ in 1..distance {
            let outer = RefCell::borrow(&env)
                .parent()
                .expect("Global scope has no parent scope");
            env = outer;
        }

        env
    }

    pub fn parent(&self) -> Option<Rc<RefCell<Environment>>> {
        Some(Rc::clone(self.parent.as_ref()?))
    }

    pub fn assign(
        &mut self,
        name: &Token,
        value: Rc<RefCell<LoxType>>,
    ) -> Result<(), RuntimeException> {
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

    pub fn assign_at(
        &mut self,
        distance: usize,
        name: &Token,
        value: Rc<RefCell<LoxType>>,
    ) -> Result<(), RuntimeException> {
        if distance == 0 {
            match self.values.insert(name.raw.to_string(), value) {
                Some(_) => Ok(()),
                None => Err(RuntimeException::report(
                    name.clone(),
                    &format!("Unable to assign to undefined variable {}", name.raw),
                )),
            }
        } else {
            match self
                .ancestor(distance)
                .borrow_mut()
                .values
                .insert(name.raw.to_string(), value)
            {
                Some(_) => Ok(()),
                None => Err(RuntimeException::report(
                    name.clone(),
                    &format!("Unable to assign to undefined variable {}", name.raw),
                )),
            }
        }
    }
}
