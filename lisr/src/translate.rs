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
    use super::*;

    #[test]
    fn should_create_application() {
        let application = Node::List {
            elements: VecDeque::from([
                Node::Leaf {
                    token: Token::Identifier {
                        name: String::from("square"),
                    },
                },
                Node::Leaf {
                    token: Token::Number { value: 10.0 },
                },
            ]),
        };

        let result = translate(vec![application]).unwrap();

        assert_eq!(
            result.get(0),
            Some(&Expression::Application {
                procedure: Box::new(Expression::Identifier(Identifier {
                    name: String::from("square")
                })),
                arguments: vec![Expression::Number { value: 10.0 }]
            })
        );
    }

    #[test]
    fn should_create_application_if_a_list_is_first_in_a_list() {
        let application = Node::List {
            elements: VecDeque::from([
                Node::List {
                    elements: VecDeque::from([
                        Node::Leaf {
                            token: Token::Identifier {
                                name: String::from("make-adder"),
                            },
                        },
                        Node::Leaf {
                            token: Token::Number { value: 5.0 },
                        },
                    ]),
                },
                Node::Leaf {
                    token: Token::Number { value: 42.0 },
                },
            ]),
        };

        let result = translate(vec![application]).unwrap();

        assert_eq!(
            result.get(0),
            Some(&Expression::Application {
                procedure: Box::new(Expression::Application {
                    procedure: Box::new(Expression::Identifier(Identifier {
                        name: String::from("make-adder")
                    })),
                    arguments: vec![Expression::Number { value: 5.0 }]
                }),
                arguments: vec![Expression::Number { value: 42.0 }]
            })
        );
    }

    #[test]
    fn should_create_assignment() {
        let assignment = Node::List {
            elements: VecDeque::from([
                Node::Leaf { token: Token::Set },
                Node::Leaf {
                    token: Token::Identifier {
                        name: String::from("x"),
                    },
                },
                Node::Leaf {
                    token: Token::Number { value: 42.0 },
                },
            ]),
        };

        let result = translate(vec![assignment]).unwrap();

        assert_eq!(
            result.get(0),
            Some(&Expression::Assignment {
                variable: Identifier {
                    name: String::from("x")
                },
                value: Box::new(Expression::Number { value: 42.0 })
            })
        );
    }

    #[test]
    fn should_create_definition() {
        let definition = Node::List {
            elements: VecDeque::from([
                Node::Leaf {
                    token: Token::Define,
                },
                Node::Leaf {
                    token: Token::Identifier {
                        name: String::from("x"),
                    },
                },
                Node::Leaf {
                    token: Token::Number { value: 42.0 },
                },
            ]),
        };

        let result = translate(vec![definition]).unwrap();

        assert_eq!(
            result.get(0),
            Some(&Expression::Definition {
                variable: Identifier {
                    name: String::from("x")
                },
                value: Box::new(Expression::Number { value: 42.0 })
            })
        );
    }

    #[test]
    fn should_create_if() {
        let if_statement = Node::List {
            elements: VecDeque::from([
                Node::Leaf { token: Token::If },
                Node::Leaf { token: Token::True },
                Node::Leaf {
                    token: Token::Number { value: 1.0 },
                },
                Node::Leaf {
                    token: Token::Number { value: 0.0 },
                },
            ]),
        };

        let result = translate(vec![if_statement]).unwrap();

        assert_eq!(
            result.get(0),
            Some(&Expression::If {
                predicate: Box::new(Expression::True),
                consequent: Box::new(Expression::Number { value: 1.0 }),
                alternative: Box::new(Expression::Number { value: 0.0 })
            })
        );
    }

    #[test]
    fn should_create_and() {
        let and = Node::List {
            elements: VecDeque::from([
                Node::Leaf { token: Token::And },
                Node::Leaf { token: Token::True },
                Node::Leaf {
                    token: Token::False,
                },
            ]),
        };

        let result = translate(vec![and]).unwrap();

        assert_eq!(
            result.get(0),
            Some(&Expression::And {
                operands: vec![Expression::True, Expression::False]
            })
        );
    }

    #[test]
    fn should_create_or() {
        let or = Node::List {
            elements: VecDeque::from([
                Node::Leaf { token: Token::Or },
                Node::Leaf {
                    token: Token::False,
                },
                Node::Leaf { token: Token::True },
            ]),
        };

        let result = translate(vec![or]).unwrap();

        assert_eq!(
            result.get(0),
            Some(&Expression::Or {
                operands: vec![Expression::False, Expression::True]
            })
        );
    }

    #[test]
    fn should_create_lambda() {
        let lambda = Node::List {
            elements: VecDeque::from([
                Node::Leaf {
                    token: Token::Lambda,
                },
                Node::List {
                    elements: VecDeque::from([
                        Node::Leaf {
                            token: Token::Identifier {
                                name: String::from("x"),
                            },
                        },
                        Node::Leaf {
                            token: Token::Identifier {
                                name: String::from("y"),
                            },
                        },
                    ]),
                },
                Node::List {
                    elements: VecDeque::from([
                        Node::Leaf {
                            token: Token::Identifier {
                                name: String::from("+"),
                            },
                        },
                        Node::Leaf {
                            token: Token::Identifier {
                                name: String::from("x"),
                            },
                        },
                        Node::Leaf {
                            token: Token::Identifier {
                                name: String::from("y"),
                            },
                        },
                    ]),
                },
            ]),
        };

        let result = translate(vec![lambda]).unwrap();

        assert_eq!(
            result.get(0),
            Some(&Expression::Lambda {
                parameters: vec![
                    Parameter {
                        name: String::from("x")
                    },
                    Parameter {
                        name: String::from("y")
                    }
                ],
                body: Box::new(Expression::Application {
                    procedure: Box::new(Expression::Identifier(Identifier {
                        name: String::from("+")
                    })),
                    arguments: vec![
                        Expression::Identifier(Identifier {
                            name: String::from("x")
                        }),
                        Expression::Identifier(Identifier {
                            name: String::from("y")
                        })
                    ]
                })
            })
        );
    }

    #[test]
    fn should_create_begin() {
        let begin = Node::List {
            elements: VecDeque::from([
                Node::Leaf {
                    token: Token::Begin,
                },
                Node::Leaf {
                    token: Token::Number { value: 1.0 },
                },
            ]),
        };

        let result = translate(vec![begin]).unwrap();

        assert_eq!(
            result.get(0),
            Some(&Expression::Begin {
                sequence: vec![Expression::Number { value: 1.0 }]
            })
        );
    }

    #[test]
    fn should_create_cons() {
        let cons = Node::List {
            elements: VecDeque::from([
                Node::Leaf { token: Token::Cons },
                Node::Leaf {
                    token: Token::Number { value: 1.0 },
                },
                Node::List {
                    elements: VecDeque::new(),
                },
            ]),
        };

        let result = translate(vec![cons]).unwrap();

        assert_eq!(
            result.get(0),
            Some(&Expression::Cons {
                first: Box::new(Expression::Number { value: 1.0 }),
                rest: Box::new(Expression::EmptyList)
            })
        );
    }

    #[test]
    fn should_return_an_error_for_non_invokable_objects() {
        // TODO: Parametrize the test for other non-invokable objects.
        let ast = Node::List {
            elements: VecDeque::from([
                Node::Leaf {
                    token: Token::String {
                        value: String::from("add"),
                    },
                },
                Node::Leaf {
                    token: Token::Number { value: 1.0 },
                },
                Node::Leaf {
                    token: Token::Number { value: 2.0 },
                },
            ]),
        };

        let error = translate(vec![ast]).unwrap_err();

        assert_eq!(error, LisrParseError::ObjectNotInvokable);
    }

    #[test]
    fn lambda_should_require_parameters_and_body() {
        let ast = Node::List {
            elements: VecDeque::from([Node::Leaf {
                token: Token::Lambda,
            }]),
        };

        let error = translate(vec![ast]).unwrap_err();

        assert_eq!(error, LisrParseError::LambdaRequiresParameterListAndBody);
    }

    #[test]
    fn assignment_should_require_a_variable_and_a_value() {
        let ast = Node::List {
            elements: VecDeque::from([
                Node::Leaf { token: Token::Set },
                Node::Leaf {
                    token: Token::Identifier {
                        name: String::from("variable"),
                    },
                },
            ]),
        };

        let error = translate(vec![ast]).unwrap_err();

        assert_eq!(
            error,
            LisrParseError::AssignmentRequiresOneVariableAndOneValue
        );
    }

    #[test]
    fn should_require_a_variable_in_definition() {
        let ast = Node::List {
            elements: VecDeque::from([
                Node::Leaf {
                    token: Token::Define,
                },
                Node::Leaf { token: Token::True },
                Node::Leaf {
                    token: Token::False,
                },
            ]),
        };

        let error = translate(vec![ast]).unwrap_err();

        assert_eq!(error, LisrParseError::VariableRequiredInThisContext);
    }

    #[test]
    fn should_require_a_variable_in_function_definition() {
        let ast = Node::List {
            elements: VecDeque::from([
                Node::Leaf {
                    token: Token::Define,
                },
                Node::List {
                    elements: VecDeque::from([
                        Node::Leaf {
                            token: Token::Number { value: 1.0 },
                        },
                        Node::Leaf {
                            token: Token::Number { value: 2.0 },
                        },
                    ]),
                },
                Node::Leaf {
                    token: Token::Number { value: 3.0 },
                },
            ]),
        };

        let error = translate(vec![ast]).unwrap_err();

        assert_eq!(error, LisrParseError::VariableRequiredInThisContext);
    }

    #[test]
    fn should_require_two_arguments_for_cons() {
        let ast = Node::List {
            elements: VecDeque::from([
                Node::Leaf { token: Token::Cons },
                Node::Leaf {
                    token: Token::Number { value: 1.0 },
                },
            ]),
        };

        let error = translate(vec![ast]).unwrap_err();

        assert_eq!(error, LisrParseError::ConsRequiresTwoArguments);
    }

    #[test]
    fn should_convert_function_definition_to_a_lambda() {
        let ast = Node::List {
            elements: VecDeque::from([
                Node::Leaf {
                    token: Token::Define,
                },
                Node::List {
                    elements: VecDeque::from([
                        Node::Leaf {
                            token: Token::Identifier {
                                name: String::from("square"),
                            },
                        },
                        Node::Leaf {
                            token: Token::Identifier {
                                name: String::from("x"),
                            },
                        },
                    ]),
                },
                Node::List {
                    elements: VecDeque::from([
                        Node::Leaf {
                            token: Token::Identifier {
                                name: String::from("*"),
                            },
                        },
                        Node::Leaf {
                            token: Token::Identifier {
                                name: String::from("x"),
                            },
                        },
                        Node::Leaf {
                            token: Token::Identifier {
                                name: String::from("x"),
                            },
                        },
                    ]),
                },
            ]),
        };

        let result = translate(vec![ast]).unwrap();

        assert_eq!(
            result.get(0),
            Some(&Expression::Definition {
                variable: Identifier {
                    name: String::from("square")
                },
                value: Box::new(Expression::Lambda {
                    parameters: vec![Parameter {
                        name: String::from("x")
                    }],
                    body: Box::new(Expression::Application {
                        procedure: Box::new(Expression::Identifier(Identifier {
                            name: String::from("*")
                        })),
                        arguments: vec![
                            Expression::Identifier(Identifier {
                                name: String::from("x")
                            }),
                            Expression::Identifier(Identifier {
                                name: String::from("x")
                            })
                        ]
                    })
                })
            })
        );
    }
}
