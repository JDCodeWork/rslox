use std::collections::BTreeMap;

use crate::{
    errors::{Err, RuntimeErr},
    lox::ast::Literal,
};

#[derive(Default)]
pub struct Enviroment {
    values: BTreeMap<String, Literal>,
}

impl Enviroment {
    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<Literal, Err> {
        let Some(val) = self.values.get(name) else {
            return Err(RuntimeErr::UndefinedVariable(name.to_string()).to_err());
        };

        Ok(val.to_owned())
    }
}
