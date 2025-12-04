use std::collections::BTreeMap;

use crate::{
    errors::{Err, RuntimeErr},
    lox::{ast::Literal, token::Token},
};

#[derive(Clone, Debug)]
pub struct Environment {
    values: BTreeMap<String, Literal>,
    enclosing: Option<Box<Environment>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            values: BTreeMap::new(),
            enclosing: None,
        }
    }
}

impl Environment {
    pub fn new(enclosing: Option<Environment>) -> Self {
        Self {
            values: BTreeMap::new(),
            enclosing: enclosing.map(Box::new),
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: Token) -> Result<Literal, Err> {
        if let Some(val) = self.values.get(&name.get_lexeme()) {
            return Ok(val.to_owned());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.get(name);
        }

        Err(RuntimeErr::UndefinedVariable(name.get_lexeme(), name.get_line()).to_err())
    }

    pub fn assign(&mut self, name: Token, value: Literal) -> Result<(), Err> {
        if self.values.contains_key(&name.get_lexeme()) {
            self.values.insert(name.get_lexeme(), value);
            return Ok(());
        }

        if let Some(ref mut enclosing) = self.enclosing {
            enclosing.assign(name, value)?;
            return Ok(());
        }

        Err(RuntimeErr::UndefinedVariable(name.get_lexeme(), name.get_line()).to_err())
    }
}
