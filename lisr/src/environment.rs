use crate::{
    evaluate::LisrEvaluationError,
    expression::{Expression, Identifier},
};
use std::collections::{hash_map::IntoIter, HashMap};

type Frame = HashMap<Identifier, Expression>;

// Environment consists of one frame only, because functions are implemented as
// one-block closures. Every compound procedure has it's own copy of the
// environment. Not the most elegant solution, but at least it's simple.
#[derive(Debug, Clone, PartialEq)]
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

    pub fn into_iter(self) -> IntoIter<Identifier, Expression> {
        self.frame.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::{Expression, Identifier};

    use super::*;

    #[test]
    fn should_store_a_variable() {
        let mut env = Environment::new();

        let variable = Identifier {
            name: String::from("num"),
        };
        let value = Expression::Number { value: 42.0 };

        env.define_variable(&variable, &value);

        let result = env.lookup_value(&variable);

        assert_eq!(result, Ok(Expression::Number { value: 42.0 }));
    }

    #[test]
    fn should_return_an_error_for_undefined_variable() {
        let env = Environment::new();

        let variable = Identifier {
            name: String::from("undefined"),
        };

        let error = env.lookup_value(&variable).unwrap_err();

        assert_eq!(error, LisrEvaluationError::UndefinedIdentifier);
    }
}
