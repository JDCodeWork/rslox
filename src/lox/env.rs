use std::collections::HashMap;

use crate::{
    errors::{Err, RuntimeErr},
    lox::{ast::LiteralExpr, token::Token},
};

/**
 * Here we have a problem, in Crafting Interpreters, the Environment was implemented using a values map and a pointer to the enclosing environment, that is fine in Java
 * because it has garbage collection, but in Rust we have to deal with ownership and borrowing rules
 *
 * In my first iteration I tried to implement it similarly to the book using BTreeMap for the values and Option Box<Environment> for the enclosing environment but
 * it didn't work well due to Rust's ownership model.
 *
 *
 * To solve this, a lot of languages would use other techniques like using a property called scopes, which is a stack of maps, also I found that in this context for the
 * variables lookup a HashMap would be more efficient than a BTreeMap, that is because in most cases the number of variables per scope is small and the lookup time for
 * a HashMap is O(1) on average, while for a BTreeMap is O(log n).
 *
 */

#[derive(Clone, Debug)]
pub struct Environment {
    scopes: Vec<HashMap<String, LiteralExpr>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }
}

impl Environment {
    pub fn define(&mut self, name: String, value: LiteralExpr) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    pub fn get(&self, name: Token) -> Result<LiteralExpr, Err> {
        for scope in self.scopes.iter() {
            if let Some(val) = scope.get(&name.get_lexeme()) {
                return Ok(val.clone());
            }
        }

        Err(RuntimeErr::UndefinedVariable(name.get_lexeme(), name.get_line()).to_err())
    }

    pub fn assign(&mut self, name: Token, value: LiteralExpr) -> Result<(), Err> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(&name.get_lexeme()) {
                scope.insert(name.get_lexeme(), value);
                return Ok(());
            }
        }

        Err(RuntimeErr::UndefinedVariable(name.get_lexeme(), name.get_line()).to_err())
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }
}
