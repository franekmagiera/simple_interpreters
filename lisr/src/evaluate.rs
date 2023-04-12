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
    environment.define_variable(
        &Identifier {
            name: "+".to_string(),
        },
        &Expression::PrimitiveProcedure {
            procedure: primitive_sum,
        },
    );

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

fn primitive_sum(arguments: Vec<Expression>) -> Result<Expression, LisrEvaluationError> {
    if arguments.is_empty() {
        return Err(LisrEvaluationError::RuntimeError {
            reason: "+ function cannot be invoked without any arguments",
        });
    }

    let mut sum = 0.0;

    for argument in arguments.into_iter() {
        match argument {
            Expression::Number { value } => sum += value,
            _ => {
                return Err(LisrEvaluationError::TypeError);
            }
        }
    }

    Ok(Expression::Number { value: sum })
}
