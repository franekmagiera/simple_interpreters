use std::collections::VecDeque;

use crate::{
    expression::{Expression, Identifier, Parameter},
    node::Node,
    parse::LisrParseError,
    token::Token,
};

pub fn translate<I>(nodes: I) -> Result<Vec<Expression>, LisrParseError>
where
    I: IntoIterator<Item = Node>,
{
    nodes.into_iter().map(translate_node).collect()
}

fn translate_node(node: Node) -> Result<Expression, LisrParseError> {
    match node {
        Node::Leaf { token } => translate_leaf(token),
        Node::List { elements } => translate_list(elements),
    }
}

fn translate_leaf(token: Token) -> Result<Expression, LisrParseError> {
    match token {
        Token::String { value } => Ok(Expression::String { value }),
        Token::Number { value } => Ok(Expression::Number { value }),
        Token::Quotation { text_of_quotation } => translate_quotation(text_of_quotation),
        Token::Identifier { name } => Ok(Expression::Identifier(Identifier { name })),
        Token::Set => Ok(Expression::LisrInternalObject {
            name: String::from("set!"),
        }),
        Token::Define => Ok(Expression::LisrInternalObject {
            name: String::from("define"),
        }),
        Token::True => Ok(Expression::True),
        Token::False => Ok(Expression::False),
        Token::If => Ok(Expression::LisrInternalObject {
            name: String::from("if"),
        }),
        Token::And => Ok(Expression::LisrInternalObject {
            name: String::from("and"),
        }),
        Token::Or => Ok(Expression::LisrInternalObject {
            name: String::from("or"),
        }),
        Token::Lambda => Ok(Expression::LisrInternalObject {
            name: String::from("lambda"),
        }),
        Token::Begin => Ok(Expression::LisrInternalObject {
            name: String::from("begin"),
        }),
        Token::Cons => Ok(Expression::LisrInternalObject {
            name: String::from("cons"),
        }),
        Token::LeftParen | Token::RightParen => {
            panic!("Cannot translate parentheses to an expression")
        }
    }
}

fn translate_quotation(text_of_quotation: String) -> Result<Expression, LisrParseError> {
    // TODO: add translation to lists.
    Ok(Expression::Quotation { text_of_quotation })
}

fn translate_list(mut elements: VecDeque<Node>) -> Result<Expression, LisrParseError> {
    if let Some(first) = elements.pop_front() {
        let mut rest = elements;
        match first {
            Node::Leaf { ref token } => match token {
                Token::String { .. }
                | Token::Number { .. }
                | Token::Quotation { .. }
                | Token::True
                | Token::False => {
                    return Err(LisrParseError::ObjectNotInvokable);
                }
                Token::LeftParen | Token::RightParen => {
                    panic!("Cannot translate parentheses to expression")
                }
                Token::Identifier { .. } => {
                    return create_application(first, rest);
                }
                Token::Set => {
                    return create_assignment(rest);
                }
                Token::Define => {
                    return create_definition(rest);
                }
                Token::If => {
                    return create_if(rest);
                }
                Token::And => {
                    return create_and(rest);
                }
                Token::Or => {
                    return create_or(rest);
                }
                // TODO: Why not validate stuff in create_lambda like I do for other tokens?
                Token::Lambda => {
                    let parameters = rest.pop_front();
                    let body = rest.pop_front();
                    match (parameters, body) {
                        (Some(Node::List { elements }), Some(body)) => {
                            return create_lambda(elements, body);
                        }
                        _ => {
                            return Err(LisrParseError::LambdaRequiresParameterListAndBody);
                        }
                    }
                }
                Token::Begin => {
                    return create_begin(rest);
                }
                Token::Cons => {
                    return create_cons(rest);
                }
            },
            Node::List { .. } => {
                // This must be an application if the first element is a list.
                let procedure = translate_node(first)?;
                let arguments = translate(rest)?;
                return Ok(Expression::Application {
                    procedure: Box::new(procedure),
                    arguments,
                });
            }
        }
    }
    // Got an empty list.
    Ok(Expression::EmptyList)
}

fn create_application(
    procedure: Node,
    arguments: VecDeque<Node>,
) -> Result<Expression, LisrParseError> {
    let procedure = translate_node(procedure)?;
    let arguments = translate(arguments)?;
    Ok(Expression::Application {
        procedure: Box::new(procedure),
        arguments,
    })
}

fn create_assignment(mut arguments: VecDeque<Node>) -> Result<Expression, LisrParseError> {
    let variable = arguments.pop_front();
    let value = arguments.pop_front();

    if !arguments.is_empty() {
        // We should have consumed everything an assignment requires.
        return Err(LisrParseError::AssignmentRequiresOneVariableAndOneValue);
    }

    match (variable, value) {
        (
            Some(Node::Leaf {
                token: Token::Identifier { name },
            }),
            Some(value),
        ) => {
            let value = translate_node(value)?;
            Ok(Expression::Assignment {
                variable: Identifier { name },
                value: Box::new(value),
            })
        }
        _ => Err(LisrParseError::AssignmentRequiresOneVariableAndOneValue),
    }
}

fn create_definition(mut arguments: VecDeque<Node>) -> Result<Expression, LisrParseError> {
    let defined_entity = arguments.pop_front();
    let value = arguments.pop_front();

    match (defined_entity, value) {
        (Some(defined_entity), Some(value)) => {
            match defined_entity {
                // Simple variable definition.
                Node::Leaf {
                    token: Token::Identifier { name },
                } => {
                    let value = translate_node(value)?;
                    Ok(Expression::Definition {
                        variable: Identifier { name },
                        value: Box::new(value),
                    })
                }
                // Function definition.
                Node::List { mut elements } => {
                    let function_name = elements.pop_front();
                    match function_name {
                        Some(Node::Leaf {
                            token:
                                Token::Identifier {
                                    name: function_name,
                                },
                        }) => {
                            let body = value;
                            let lambda = create_lambda(elements, body)?;
                            Ok(Expression::Definition {
                                variable: Identifier {
                                    name: function_name,
                                },
                                value: Box::new(lambda),
                            })
                        }
                        _ => Err(LisrParseError::VariableRequiredInThisContext),
                    }
                }
                _ => Err(LisrParseError::VariableRequiredInThisContext),
            }
        }
        _ => Err(LisrParseError::DefinitionRequiresVariableAndBody),
    }
}

fn create_if(mut arguments: VecDeque<Node>) -> Result<Expression, LisrParseError> {
    let predicate = arguments.pop_front();
    let consequent = arguments.pop_front();
    let alternative = arguments.pop_front();

    if !arguments.is_empty() {
        // We should have consumed everything that an if statement requires.
        return Err(LisrParseError::IfRequiresPredicateConsequentAndAlternative);
    }

    match (predicate, consequent, alternative) {
        (Some(predicate), Some(consequent), Some(alternative)) => {
            let predicate = translate_node(predicate)?;
            let consequent = translate_node(consequent)?;
            let alternative = translate_node(alternative)?;
            Ok(Expression::If {
                predicate: Box::new(predicate),
                consequent: Box::new(consequent),
                alternative: Box::new(alternative),
            })
        }
        _ => Err(LisrParseError::IfRequiresPredicateConsequentAndAlternative),
    }
}

fn create_and(arguments: VecDeque<Node>) -> Result<Expression, LisrParseError> {
    let operands = translate(arguments)?;
    Ok(Expression::And { operands })
}

fn create_or(arguments: VecDeque<Node>) -> Result<Expression, LisrParseError> {
    let operands = translate(arguments)?;
    Ok(Expression::Or { operands })
}

fn create_lambda(parameters: VecDeque<Node>, body: Node) -> Result<Expression, LisrParseError> {
    let parameters = translate_parameters(parameters)?;
    let body = translate_node(body)?;
    Ok(Expression::Lambda {
        parameters,
        body: Box::new(body),
    })
}

fn translate_parameters(parameters: VecDeque<Node>) -> Result<Vec<Parameter>, LisrParseError> {
    parameters
        .into_iter()
        .map(|element| match element {
            Node::Leaf {
                token: Token::Identifier { name },
            } => Ok(Parameter { name }),
            _ => Err(LisrParseError::UnexpectedExpressionForLambdaParameter),
        })
        .collect()
}

fn create_begin(arguments: VecDeque<Node>) -> Result<Expression, LisrParseError> {
    let sequence = translate(arguments)?;
    Ok(Expression::Begin { sequence })
}

fn create_cons(mut arguments: VecDeque<Node>) -> Result<Expression, LisrParseError> {
    let left = arguments.pop_front();
    let right = arguments.pop_front();
    match (left, right) {
        (Some(left), Some(right)) => Ok(Expression::Cons {
            first: Box::new(translate_node(left)?),
            rest: Box::new(translate_node(right)?),
        }),
        _ => Err(LisrParseError::ConsRequiresTwoArguments),
    }
}

#[cfg(test)]
mod tests {
    use crate::parse;
    use crate::scan;

    use super::*;

    #[test]
    fn parse_test() {
        let input = "
        (+ 1 2 (* 1.2 -.5) (/ 6 2))
        (😲 🍟)
        123
        456
        'some-quotation
        (define (square x) (* x x))
        (and true true)
        (or true false)
        (if true 1 2)
        (define x 5)
        (set! x 6)
        \"a long string 123 -2 , (1 2 3)\"
        (define (adder x) (lambda (y) (+ y x)))
        ((adder 5) 5)
        ";

        let tokens = scan::scan(input).unwrap();
        let nodes = parse::parse(tokens).unwrap();
        let expressions = translate(nodes).unwrap();

        for expression in expressions.iter() {
            println!("{:#?}", expression);
        }
    }
}
