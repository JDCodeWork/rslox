use crate::{
    errors::{Locate, LoxError, RuntimeError},
    lox::{ast::LiteralExpr, token::Token},
};
use std::collections::HashMap;
use std::fmt;

/**
 * Here we have a problem, in Crafting Interpreters, the Environment was implemented using a values map and a pointer to the enclosing environment, that is fine in Java
 *
 * because it has garbage collection, but in Rust we have to deal with ownership and borrowing rules
 *
 * To solve this, a lot of languages would use techniques like a property called scopes, which is a stack of maps.
 *
 */

/**
 * So over a few time working with the scopes approach, I was trying implemented the function's closure by capturing the current scopes stack when defining the function, and then when calling the function, but it was a mess to manage the scopes stack correctly.
 *
 * Due to that, I decided to use the Arena Pattern to manage the Environments, so each Environment will have a reference to its enclosing Environment using an Arena to manage the memory.
 */

pub type EnvBindings = HashMap<String, LiteralExpr>;
pub type EnvId = usize;

#[derive(Clone, Debug)]
pub struct EnvNode {
    pub values: EnvBindings,
    pub parent: Option<EnvId>,
}

#[derive(Clone, Debug)]
pub struct Environment {
    pub nodes: Vec<EnvNode>,
    pub curr_node: EnvId,
}

impl EnvNode {
    pub fn new() -> Self {
        Self {
            parent: None,
            values: HashMap::new(),
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        let globals = EnvNode::new();

        Self {
            nodes: vec![globals],
            curr_node: 0,
        }
    }
}

impl Environment {
    pub fn define(&mut self, name: String, value: LiteralExpr) {
        let scope = &mut self.nodes[self.curr_node];

        scope.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<LiteralExpr, LoxError> {
        let env = &self.nodes[0];

        if let Some(value) = env.values.get(&name.lexeme.clone()) {
            return Ok(value.to_owned());
        }

        Err(RuntimeError::UndefinedVariable(name.lexeme.clone()).at(name.line))
    }

    pub fn get_at(&self, at: usize, name: &Token) -> Result<LiteralExpr, LoxError> {
        let Some(lit) = self.nodes[self.ancestor(at)]
            .values
            .get(&name.lexeme.clone())
        else {
            return Err(RuntimeError::UndefinedVariable(name.lexeme.clone()).at(name.line));
        };

        Ok(lit.clone())
    }

    pub fn ancestor(&self, distance: usize) -> usize {
        let mut curr = Some(self.curr_node);

        for _ in 0..distance {
            let Some(curr_id) = curr else {
                break;
            };

            curr = self.nodes[curr_id].parent;
        }

        curr.unwrap()
    }

    pub fn assign(&mut self, name: Token, value: LiteralExpr) -> Result<(), LoxError> {
        let env = &mut self.nodes[0];

        if env.values.contains_key(&name.lexeme.clone()) {
            env.values.insert(name.lexeme.clone(), value);

            return Ok(());
        }

        Err(RuntimeError::UndefinedVariable(name.lexeme.clone()).at(name.line))
    }

    pub fn assign_at(
        &mut self,
        at: usize,
        name: Token,
        value: LiteralExpr,
    ) -> Result<(), LoxError> {
        let env_id = self.ancestor(at);
        let target_env = &mut self.nodes[env_id];
        target_env.values.insert(name.lexeme.clone(), value);

        Ok(())
    }

    pub fn push_closure(&mut self, bindings: EnvBindings, parent: EnvId) {
        let mut new_scope = EnvNode::new();

        new_scope.parent = Some(parent);
        new_scope.values = bindings;

        self.nodes.push(new_scope);
        self.curr_node = self.nodes.len() - 1;
    }

    pub fn push_node(&mut self) {
        let mut new_scope = EnvNode::new();
        new_scope.parent = Some(self.curr_node);

        self.nodes.push(new_scope);
        self.curr_node = self.nodes.len() - 1;
    }

    pub fn pop_node(&mut self) {
        let curr_env = &self.nodes[self.curr_node];

        if let Some(parent_id) = curr_env.parent {
            self.curr_node = parent_id;
        } else {
            self.curr_node = 0
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Environment (Current Node: {})", self.curr_node)?;
        for (i, node) in self.nodes.iter().enumerate() {
            if let Some(parent) = node.parent {
                writeln!(f, "  [{}] Parent: {}", i, parent)?;
            } else {
                writeln!(f, "  [{}] Root", i)?;
            }
            for (key, val) in &node.values {
                write!(f, "    {}: ", key)?;
                val.fmt_indented(f, 2)?;
            }
        }
        Ok(())
    }
}
