use crate::{
    environment::Environment,
    expression::{Expression, Identifier, Parameter},
};

#[derive(Debug, PartialEq)]
pub enum LisrEvaluationError {
    RuntimeError { reason: &'static str },
    TypeError,
    UndefinedIdentifier,
}

pub fn evaluate<I>(expressions: I) -> Result<Expression, LisrEvaluationError>
where
    I: IntoIterator<Item = Expression>,
{
    let mut environment = Environment::new();
    setup_primitive_procedures(&mut environment);

    // Keeps evaluating even if an error happens.
    let outcome = expressions
        .into_iter()
        .map(|expression| evaluate_expression(expression, &mut environment))
        .last();

    match outcome {
        Some(Ok(outcome)) => Ok(outcome),
        Some(Err(outcome)) => Err(outcome),
        // On empty input, just return an empty list.
        None => Ok(Expression::EmptyList),
    }
}

fn evaluate_expression(
    expression: Expression,
    environment: &mut Environment,
) -> Result<Expression, LisrEvaluationError> {
    match expression {
        Expression::EmptyList
        | Expression::String { .. }
        | Expression::Number { .. }
        | Expression::Quotation { .. }
        | Expression::True
        | Expression::False
        | Expression::LisrInternalObject { .. }
        | Expression::PrimitiveProcedure { .. }
        | Expression::CompoundProcedure { .. } => Ok(expression),
        Expression::Identifier(identifier) => {
            let value = environment.lookup_value(&identifier)?;
            Ok(value)
        }
        Expression::Assignment { variable, value } => {
            // Check if the value has already been defined.
            environment.lookup_value(&variable)?;

            let evaluated_value = evaluate_expression(*value, environment)?;
            environment.define_variable(&variable, &evaluated_value);
            Ok(evaluated_value)
        }
        Expression::Definition { variable, value } => {
            let evaluated_value = evaluate_expression(*value, environment)?;
            environment.define_variable(&variable, &evaluated_value);
            Ok(Expression::Identifier(variable))
        }
        Expression::Lambda { parameters, body } => Ok(Expression::CompoundProcedure {
            parameters,
            body,
            environment: Box::new(environment.clone()),
        }),
        Expression::If {
            predicate,
            consequent,
            alternative,
        } => {
            let evaluated_predicate = evaluate_expression(*predicate, environment)?;
            match evaluated_predicate {
                Expression::True => evaluate_expression(*consequent, environment),
                Expression::False => evaluate_expression(*alternative, environment),
                _ => Err(LisrEvaluationError::RuntimeError {
                    reason: "Predicate of an if expression did not evaluate to a boolean value",
                }),
            }
        }
        Expression::And { operands } => {
            for operand in operands.into_iter() {
                let evaluated_operand = evaluate_expression(operand, environment)?;
                match evaluated_operand {
                    Expression::True => {
                        continue;
                    }
                    Expression::False => return Ok(Expression::False),
                    _ => {
                        return Err(LisrEvaluationError::RuntimeError {
                            reason:
                                "Operand of an and expression did not evaluate to a boolean value",
                        })
                    }
                }
            }
            Ok(Expression::True)
        }
        Expression::Or { operands } => {
            for operand in operands.into_iter() {
                let evaluated_operand = evaluate_expression(operand, environment)?;
                match evaluated_operand {
                    Expression::False => {
                        continue;
                    }
                    Expression::True => return Ok(Expression::True),
                    _ => {
                        return Err(LisrEvaluationError::RuntimeError {
                            reason:
                                "Operand of an or expression did not evaluate to a boolean value",
                        })
                    }
                }
            }
            Ok(Expression::False)
        }
        Expression::Begin { sequence } => {
            let result = sequence
                .into_iter()
                .map(|expression| evaluate_expression(expression, environment))
                .last();
            let Some(result ) = result else {
                return Err(LisrEvaluationError::RuntimeError { reason: "A sequence of expressions in a begin statement cannot be empty" })
            };
            result
        }
        Expression::Cons { first, rest } => Ok(Expression::Cons {
            first: Box::new(evaluate_expression(*first, environment)?),
            rest: Box::new(evaluate_expression(*rest, environment)?),
        }),
        Expression::Application {
            procedure,
            arguments,
        } => {
            let procedure = evaluate_expression(*procedure, environment)?;
            let arguments = arguments
                .into_iter()
                .map(|argument| evaluate_expression(argument, environment))
                .collect::<Result<Vec<Expression>, LisrEvaluationError>>()?;
            apply(procedure, arguments, environment)
        }
    }
}

fn apply(
    procedure: Expression,
    arguments: Vec<Expression>,
    environment: &Environment,
) -> Result<Expression, LisrEvaluationError> {
    match procedure {
        Expression::PrimitiveProcedure { procedure } => procedure(arguments),
        Expression::CompoundProcedure {
            parameters,
            body,
            environment: closed_environment,
        } => {
            let mut combined_environment = environment.clone();
            for (identifier, expression) in closed_environment.into_iter() {
                combined_environment.define_variable(&identifier, &expression);
            }
            apply_compound_procedure(arguments, parameters, *body, combined_environment)
        }
        _ => Err(LisrEvaluationError::RuntimeError {
            reason: "Object cannot be invoked",
        }),
    }
}

fn apply_compound_procedure(
    arguments: Vec<Expression>,
    parameters: Vec<Parameter>,
    body: Expression,
    mut environment: Environment,
) -> Result<Expression, LisrEvaluationError> {
    if arguments.len() != parameters.len() {
        return Err(LisrEvaluationError::RuntimeError {
            reason: "Function called with a wrong number of arguments",
        });
    }

    for (argument, parameter) in arguments.into_iter().zip(parameters.into_iter()) {
        environment.define_variable(
            &Identifier {
                name: parameter.name,
            },
            &argument,
        );
    }

    evaluate_expression(body, &mut environment)
}

fn setup_primitive_procedures(environment: &mut Environment) {
    environment.define_variable(
        &Identifier {
            name: "+".to_string(),
        },
        &Expression::PrimitiveProcedure {
            procedure: primitive_addition,
        },
    );
    environment.define_variable(
        &Identifier {
            name: "-".to_string(),
        },
        &Expression::PrimitiveProcedure {
            procedure: primitive_subtraction,
        },
    );
    environment.define_variable(
        &Identifier {
            name: "*".to_string(),
        },
        &Expression::PrimitiveProcedure {
            procedure: primitive_multiplication,
        },
    );
    environment.define_variable(
        &Identifier {
            name: "/".to_string(),
        },
        &Expression::PrimitiveProcedure {
            procedure: primitive_division,
        },
    );
    environment.define_variable(
        &Identifier {
            name: "remainder".to_string(),
        },
        &Expression::PrimitiveProcedure {
            procedure: primitive_remainder,
        },
    );
    environment.define_variable(
        &Identifier {
            name: "=".to_string(),
        },
        &Expression::PrimitiveProcedure {
            procedure: primitive_equals,
        },
    );
    environment.define_variable(
        &Identifier {
            name: "<".to_string(),
        },
        &Expression::PrimitiveProcedure {
            procedure: primitive_less_than,
        },
    );
    environment.define_variable(
        &Identifier {
            name: "car".to_string(),
        },
        &Expression::PrimitiveProcedure {
            procedure: primitive_car,
        },
    );
    environment.define_variable(
        &Identifier {
            name: "cdr".to_string(),
        },
        &Expression::PrimitiveProcedure {
            procedure: primitive_cdr,
        },
    );
    environment.define_variable(
        &Identifier {
            name: "empty-list?".to_string(),
        },
        &Expression::PrimitiveProcedure {
            procedure: primitive_is_empty_list,
        },
    );
}

fn primitive_remainder(mut arguments: Vec<Expression>) -> Result<Expression, LisrEvaluationError> {
    let divisor = arguments.pop();
    let dividend = arguments.pop();

    if !arguments.is_empty() {
        return Err(LisrEvaluationError::RuntimeError {
            reason: "Remainder requires two arguments - a dividend and a divisor",
        });
    }

    match (dividend, divisor) {
        (
            Some(Expression::Number { value: dividend }),
            Some(Expression::Number { value: divisor }),
        ) => Ok(Expression::Number {
            value: dividend % divisor,
        }),
        _ => Err(LisrEvaluationError::TypeError),
    }
}

fn primitive_equals(arguments: Vec<Expression>) -> Result<Expression, LisrEvaluationError> {
    let mut arguments = arguments.into_iter();
    let Some(mut current) = arguments.next() else {
        return Err(LisrEvaluationError::RuntimeError { reason: "Equals cannot be invoked without any arguments" });
    };

    for next in arguments {
        match (current, &next) {
            (Expression::Number { value: a }, Expression::Number { value: b }) => {
                if a != *b {
                    return Ok(Expression::False);
                }
            }
            (Expression::String { value: a }, Expression::String { value: b }) => {
                if a != *b {
                    return Ok(Expression::False);
                }
            }
            _ => {
                return Err(LisrEvaluationError::RuntimeError {
                    reason: "Equals is only implemented for pairs of strings or numbers",
                })
            }
        }
        current = next;
    }

    Ok(Expression::True)
}

fn primitive_less_than(mut arguments: Vec<Expression>) -> Result<Expression, LisrEvaluationError> {
    let right = arguments.pop();
    let left = arguments.pop();

    if !arguments.is_empty() {
        return Err(LisrEvaluationError::RuntimeError {
            reason: "'Less than' function requires two arguments",
        });
    }

    match (left, right) {
        (Some(Expression::Number { value: left }), Some(Expression::Number { value: right })) => {
            if left < right {
                Ok(Expression::True)
            } else {
                Ok(Expression::False)
            }
        }
        (Some(Expression::String { value: left }), Some(Expression::String { value: right })) => {
            if left < right {
                Ok(Expression::True)
            } else {
                Ok(Expression::False)
            }
        }
        _ => Err(LisrEvaluationError::RuntimeError {
            reason: "Equals is only implemented for pairs of strings or numbers",
        }),
    }
}

fn primitive_car(mut arguments: Vec<Expression>) -> Result<Expression, LisrEvaluationError> {
    let pair = arguments.pop();

    if !arguments.is_empty() {
        return Err(LisrEvaluationError::RuntimeError {
            reason: "'car' function requires one 'cons' argument",
        });
    }

    match pair {
        Some(Expression::Cons { first, rest: _ }) => Ok(*first),
        _ => Err(LisrEvaluationError::RuntimeError {
            reason: "'car' requires one 'cons' argument",
        }),
    }
}

fn primitive_cdr(mut arguments: Vec<Expression>) -> Result<Expression, LisrEvaluationError> {
    let pair = arguments.pop();

    if !arguments.is_empty() {
        return Err(LisrEvaluationError::RuntimeError {
            reason: "'cdr' function requires one 'cons' argument",
        });
    }

    match pair {
        Some(Expression::Cons { first: _, rest }) => Ok(*rest),
        _ => Err(LisrEvaluationError::RuntimeError {
            reason: "'cdr' requires one 'cons' argument",
        }),
    }
}

fn primitive_is_empty_list(
    mut arguments: Vec<Expression>,
) -> Result<Expression, LisrEvaluationError> {
    let object = arguments.pop();

    if !arguments.is_empty() {
        return Err(LisrEvaluationError::RuntimeError {
            reason: "'empty-list?' function requires exactly one argument",
        });
    }

    match object {
        Some(Expression::EmptyList) => Ok(Expression::True),
        _ => Ok(Expression::False),
    }
}

// Creates a function that can apply a primitive reducer to a sequence of
// Expressions accumulating the result.
fn create_primitive_procedure(
    reducer: fn(Expression, Expression) -> Result<Expression, LisrEvaluationError>,
) -> impl Fn(Vec<Expression>) -> Result<Expression, LisrEvaluationError> {
    move |arguments: Vec<Expression>| -> Result<Expression, LisrEvaluationError> {
        let Some(result) = arguments.into_iter().try_reduce(reducer)? else {
            return Err(LisrEvaluationError::RuntimeError { reason: "An arithmetic function cannot be invoked without any arguments" })
        };
        Ok(result)
    }
}

fn primitive_add(a: Expression, b: Expression) -> Result<Expression, LisrEvaluationError> {
    match (a, b) {
        (Expression::Number { value: augend }, Expression::Number { value: addend }) => {
            Ok(Expression::Number {
                value: augend + addend,
            })
        }
        (Expression::String { value: a }, Expression::String { value: b }) => {
            Ok(Expression::String { value: a + &b })
        }
        _ => Err(LisrEvaluationError::TypeError),
    }
}

fn primitive_subtract(a: Expression, b: Expression) -> Result<Expression, LisrEvaluationError> {
    match (a, b) {
        (Expression::Number { value: minuend }, Expression::Number { value: subtrahend }) => {
            Ok(Expression::Number {
                value: minuend - subtrahend,
            })
        }
        _ => Err(LisrEvaluationError::TypeError),
    }
}

fn primitive_multiply(a: Expression, b: Expression) -> Result<Expression, LisrEvaluationError> {
    match (a, b) {
        (
            Expression::Number { value: multiplier },
            Expression::Number {
                value: multiplicand,
            },
        ) => Ok(Expression::Number {
            value: multiplier * multiplicand,
        }),
        _ => Err(LisrEvaluationError::TypeError),
    }
}

fn primitive_divide(a: Expression, b: Expression) -> Result<Expression, LisrEvaluationError> {
    match (a, b) {
        (Expression::Number { value: a }, Expression::Number { value: b }) => {
            Ok(Expression::Number { value: a / b })
        }
        _ => Err(LisrEvaluationError::TypeError),
    }
}

// An ugly workaround because putting the result of create_primitive_procedure
// into a PrimitiveProcedure (which is an enum variant) is a bit problematic for me.
// This approach is at least simple.
fn primitive_addition(arguments: Vec<Expression>) -> Result<Expression, LisrEvaluationError> {
    create_primitive_procedure(primitive_add)(arguments)
}

fn primitive_subtraction(arguments: Vec<Expression>) -> Result<Expression, LisrEvaluationError> {
    create_primitive_procedure(primitive_subtract)(arguments)
}

fn primitive_multiplication(arguments: Vec<Expression>) -> Result<Expression, LisrEvaluationError> {
    create_primitive_procedure(primitive_multiply)(arguments)
}

fn primitive_division(arguments: Vec<Expression>) -> Result<Expression, LisrEvaluationError> {
    create_primitive_procedure(primitive_divide)(arguments)
}
