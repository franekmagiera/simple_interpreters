use crate::{
    evaluation_errors::LisrEvaluationError,
    expression::{Expression, Identifier},
};
use std::collections::{hash_map::IntoIter, HashMap};

type Frame = HashMap<Identifier, Expression>;

// Environment consists of one frame only, because functions are implemented as
// one-block closures. Every function has it's own copy of the environment.
// Not the most elegant solution, but at least it's simple.
#[derive(Debug, Clone)]
pub struct Environment {
    frame: Frame,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            frame: HashMap::new(),
        }
    }

    pub fn define_variable(&mut self, variable: &Identifier, value: &Expression) {
        self.frame.insert(variable.clone(), value.clone());
    }

    pub fn lookup_value(&self, variable: &Identifier) -> Result<Expression, LisrEvaluationError> {
        if let Some(definition) = self.frame.get(variable) {
            return Ok(definition.clone());
        }
        Err(LisrEvaluationError::UndefinedIdentifier)
    }

    pub fn into_iter(&self) -> IntoIter<Identifier, Expression> {
        self.frame.clone().into_iter()
    }
}
