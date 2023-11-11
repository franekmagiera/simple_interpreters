// Lisr's grammar:
// list <- "(" { list } ")" | leaf
// leaf <- identifier | string | quotation | number;

use std::collections::VecDeque;
use std::iter::Peekable;

use crate::node::Node;
use crate::token::Token;

#[derive(Debug, PartialEq)]
pub enum LisrParseError {
    UnexpectedRightParentheses,
    UnclosedList,
    ObjectNotInvokable,
    VariableRequiredInThisContext,
    AssignmentRequiresOneVariableAndOneValue,
    IfRequiresPredicateConsequentAndAlternative,
    LambdaRequiresParameterListAndBody,
    UnexpectedExpressionForLambdaParameter,
    DefinitionRequiresVariableAndBody,
    ConsRequiresTwoArguments,
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Node>, LisrParseError> {
    let mut nodes: Vec<Node> = Vec::new();
    let mut tokens = tokens.iter().peekable();

    while tokens.peek().is_some() {
        let node = parse_list(&mut tokens)?;
        nodes.push(node);
    }

    Ok(nodes)
}

fn parse_list<'a>(
    tokens: &mut Peekable<impl Iterator<Item = &'a Token>>,
) -> Result<Node, LisrParseError> {
    if let Some(&token) = tokens.peek() {
        match token {
            Token::LeftParen => {
                tokens.next();
                return Ok(Node::List {
                    elements: parse_list_elements(tokens)?,
                });
            }
            Token::RightParen => {
                return Err(LisrParseError::UnexpectedRightParentheses);
            }
            _ => {
                tokens.next();
                return Ok(Node::Leaf {
                    token: token.clone(),
                });
            }
        }
    }

    // The caller of parse_list should make sure there is something to parse.
    panic!("Internal error: cannot parse a list - empty input.");
}

fn parse_list_elements<'a>(
    tokens: &mut Peekable<impl Iterator<Item = &'a Token>>,
) -> Result<VecDeque<Node>, LisrParseError> {
    let mut elements: VecDeque<Node> = VecDeque::new();

    while let Some(&token) = tokens.peek() {
        match token {
            Token::LeftParen => {
                elements.push_back(parse_list(tokens)?);
            }
            Token::RightParen => {
                tokens.next();
                return Ok(elements);
            }
            _ => {
                elements.push_back(Node::Leaf {
                    token: token.clone(),
                });
                tokens.next();
            }
        }
    }

    Err(LisrParseError::UnclosedList)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn should_parse_a_leaf() {
        let tokens = vec![Token::Number { value: 42.0 }];

        let nodes = parse(tokens).unwrap();

        assert_eq!(
            nodes.get(0),
            Some(&Node::Leaf {
                token: Token::Number { value: 42.0 }
            })
        );
    }

    #[test]
    fn should_parse_a_list() {
        let tokens = vec![
            Token::LeftParen,
            Token::Identifier {
                name: String::from("+"),
            },
            Token::Number { value: 1.0 },
            Token::Number { value: 2.0 },
            Token::RightParen,
        ];

        let nodes = parse(tokens).unwrap();

        assert_eq!(
            nodes.get(0),
            Some(&Node::List {
                elements: VecDeque::from([
                    Node::Leaf {
                        token: Token::Identifier {
                            name: String::from("+")
                        }
                    },
                    Node::Leaf {
                        token: Token::Number { value: 1.0 }
                    },
                    Node::Leaf {
                        token: Token::Number { value: 2.0 }
                    }
                ])
            })
        );
    }

    #[test]
    fn should_parse_an_empty_list() {
        let tokens = vec![Token::LeftParen, Token::RightParen];

        let nodes = parse(tokens).unwrap();

        assert_eq!(
            nodes.get(0),
            Some(&Node::List {
                elements: VecDeque::new()
            })
        );
    }

    #[test]
    fn should_return_error_on_unexpected_right_parentheses() {
        let tokens = vec![Token::RightParen];

        let error = parse(tokens).unwrap_err();

        assert_eq!(error, LisrParseError::UnexpectedRightParentheses);
    }

    #[test]
    fn should_not_allow_unclosed_lists() {
        let tokens = vec![
            Token::LeftParen,
            Token::Identifier {
                name: String::from("+"),
            },
            Token::Number { value: 2.0 },
            Token::Number { value: 2.0 },
        ];

        let error = parse(tokens).unwrap_err();

        assert_eq!(error, LisrParseError::UnclosedList);
    }
}
