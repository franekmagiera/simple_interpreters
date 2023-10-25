// Lisr's grammar:
// list <- "(" { list } ")" | leaf
// leaf <- identifier | string | quotation | number;

use std::collections::VecDeque;
use std::iter::Peekable;

use crate::node::Node;
use crate::token::Token;

#[derive(Debug)]
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
    use crate::scan;

    use super::*;

    #[test]
    fn parse_test() -> Result<(), LisrParseError> {
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

        match scan::scan(input) {
            Ok(tokens) => {
                for node in parse(tokens)? {
                    println!("{:#?}", node);
                }
            }
            Err(_) => {
                panic!("Scanning should have gone OK...");
            }
        }

        Ok(())
    }
}
