use crate::{
    environment::Environment,
    evaluation_errors::LisrEvaluationError,
    expression::{Expression, Identifier, Parameter},
};

pub fn evaluate<I>(expressions: I) -> Result<Expression, LisrEvaluationError>
where
    I: IntoIterator<Item = Expression>,
{
    let mut environment = Environment::new();
    setup_primitive_procedures(&mut environment);

    let outcome = expressions
        .into_iter()
        .map(|expression| evaluate_expression(expression, &mut environment))
        .last();

    match outcome {
        Some(Ok(outcome)) => Ok(outcome),
        Some(Err(outcome)) => Err(outcome),
        // On empty input, just return an empty list.
        None => Ok(Expression::List {
            elements: Vec::new(),
        }),
    }
}

fn evaluate_expression(
    expression: Expression,
    environment: &mut Environment,
) -> Result<Expression, LisrEvaluationError> {
    match expression {
        Expression::String { .. }
        | Expression::Number { .. }
        | Expression::Quotation { .. }
        | Expression::True
        | Expression::False => Ok(expression),
        Expression::Identifier(identifier) => {
            let value = environment.lookup_value(&identifier)?;
            Ok(value)
        }
        Expression::Definition { variable, value } => {
            let evaluated_value = evaluate_expression(*value, environment)?;
            environment.define_variable(&variable, &evaluated_value);
            Ok(evaluated_value)
        }
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
        Expression::Lambda { parameters, body } => Ok(Expression::CompoundProcedure {
            parameters,
            body,
            environment: Box::new(environment.clone()),
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
            apply(procedure, arguments)
        }
        _ => todo!(),
    }
}

fn apply(
    procedure: Expression,
    arguments: Vec<Expression>,
) -> Result<Expression, LisrEvaluationError> {
    match procedure {
        Expression::PrimitiveProcedure { procedure } => procedure(arguments),
        Expression::CompoundProcedure {
            parameters,
            body,
            environment,
        } => apply_compound_procedure(arguments, parameters, *body, *environment),
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
            Ok(Expression::Number { value: augend + addend })
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
            Ok(Expression::Number { value: minuend - subtrahend })
        }
        _ => Err(LisrEvaluationError::TypeError),
    }
}

fn primitive_multiply(a: Expression, b: Expression) -> Result<Expression, LisrEvaluationError> {
    match (a, b) {
        (Expression::Number { value: multiplier }, Expression::Number { value: multiplicand }) => {
            Ok(Expression::Number { value: multiplier * multiplicand })
        }
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
