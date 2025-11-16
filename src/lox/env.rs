use std::collections::BTreeMap;

use crate::{
    errors::{Err, RuntimeErr},
    lox::ast::Literal,
};

pub struct Enviroment {
    values: BTreeMap<String, Literal>,
    enclosing: Option<Box<Enviroment>>,
}

impl Default for Enviroment {
    fn default() -> Self {
        Self {
            values: BTreeMap::new(),
            enclosing: None,
        }
    }
}

impl Enviroment {
    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<Literal, Err> {
        if let Some(val) = self.values.get(name) {
            return Ok(val.to_owned());
        }

        if let Some(ref enclosing) = self.enclosing {
            return enclosing.get(name);
        }

        Err(RuntimeErr::UndefinedVariable(name.into()).to_err())
    }

    pub fn assign(&mut self, name: &str, value: Literal) -> Result<(), Err> {
        if self.values.contains_key(name) {
            self.values.insert(name.into(), value);
            return Ok(());
        }

        if let Some(ref mut enclosing) = self.enclosing {
            enclosing.assign(name, value)?;
            return Ok(());
        }

        Err(RuntimeErr::UndefinedVariable(name.into()).to_err())
    }
}
