use std::{iter::Peekable, str::Chars};

use crate::token::Token;

const LEFT_PAREN: char = '(';
const RIGHT_PAREN: char = ')';
const SINGLE_QUOTE: char = '\'';
const DOUBLE_QUOTE: char = '"';
const PLUS: char = '+';
const ASTERISK: char = '*';
const SLASH: char = '/';
const DASH: char = '-';
const DOT: char = '.';
const RADIX: u32 = 10;
const LESS_THAN: char = '<';
const GREATER_THAN: char = '>';

#[derive(Debug)]
pub enum LisrScanError<'a> {
    EmptyQuotation, // Empty quotations are not allowed.
    UnclosedString,
    InvalidNumber { reason: &'a str },
    InvalidIdentifier { reason: &'a str },
}

pub fn scan(input: &str) -> Result<Vec<Token>, LisrScanError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut input = input.chars().peekable();

    while let Some(&char) = input.peek() {
        match char {
            LEFT_PAREN => {
                tokens.push(Token::LeftParen);
                input.next();
            }
            RIGHT_PAREN => {
                tokens.push(Token::RightParen);
                input.next();
            }
            DASH => {
                input.next();

                match input.peek() {
                    Some(&next_char) => {
                        if should_finish_scanning(next_char) {
                            tokens.push(Token::Identifier {
                                name: DASH.to_string(),
                            });
                            input.next();
                        } else {
                            tokens.push(scan_number(&mut input, true)?);
                        }
                    }
                    None => {
                        tokens.push(Token::Identifier {
                            name: DASH.to_string(),
                        });
                    }
                }
            }
            // Numbers with a preceeding plus sign are not allowed (for example: `+42`).
            PLUS | ASTERISK | SLASH | LESS_THAN | GREATER_THAN => {
                input.next();
                match input.peek() {
                    Some(&next_char) => {
                        if should_finish_scanning(next_char) {
                            input.next();
                            tokens.push(Token::Identifier {
                                name: char.to_string(),
                            });
                        } else {
                            return Err(LisrScanError::InvalidIdentifier {
                                reason: "Identifier cannot start with an arithmetic operator.",
                            });
                        }
                    }
                    None => {
                        tokens.push(Token::Identifier {
                            name: char.to_string(),
                        });
                    }
                }
            }
            char if char.is_whitespace() => {
                // Skip whitespaces.
                input.next();
            }
            _ => {
                let literal = scan_literal(&mut input)?;
                tokens.push(literal);
            }
        }
    }

    Ok(tokens)
}

fn scan_literal(input: &mut Peekable<Chars>) -> Result<Token, LisrScanError<'static>> {
    if let Some(&char) = input.peek() {
        match char {
            SINGLE_QUOTE => {
                input.next();
                scan_quotation(input)
            }
            DOUBLE_QUOTE => {
                input.next();
                scan_string(input)
            }
            char if char.is_digit(RADIX) || char == DOT => scan_number(input, false),
            _ => scan_identifier(input),
        }
    } else {
        panic!("Failed while scanning a literal.");
    }
}

fn scan_quotation(input: &mut Peekable<Chars>) -> Result<Token, LisrScanError<'static>> {
    let mut lexeme = String::new();

    while let Some(&char) = input.peek() {
        if should_finish_scanning(char) {
            break;
        }
        lexeme.push(char);
        input.next();
    }

    if lexeme.is_empty() {
        Err(LisrScanError::EmptyQuotation)
    } else {
        Ok(Token::Quotation {
            text_of_quotation: lexeme,
        })
    }
}

fn scan_string(input: &mut Peekable<Chars>) -> Result<Token, LisrScanError<'static>> {
    let mut lexeme = String::new();
    while let Some(&char) = input.peek() {
        match char {
            DOUBLE_QUOTE => {
                input.next();
                return Ok(Token::String { value: lexeme });
            }
            _ => {
                lexeme.push(char);
                input.next();
            }
        }
    }
    Err(LisrScanError::UnclosedString)
}

fn scan_number(
    input: &mut Peekable<Chars>,
    negative: bool,
) -> Result<Token, LisrScanError<'static>> {
    let mut lexeme = String::new();
    let mut is_decimal = false;

    while let Some(&char) = input.peek() {
        match char {
            DOT => {
                if is_decimal {
                    return Err(LisrScanError::InvalidNumber {
                        reason: "Number cannot have more than one decimal point.",
                    });
                }
                is_decimal = true;
                lexeme.push(char);
                input.next();
            }
            char if should_finish_scanning(char) => {
                break;
            }
            char if !char.is_digit(RADIX) => {
                return Err(LisrScanError::InvalidNumber {
                    reason: "Numbers can contain digits only.",
                });
            }
            _ => {
                lexeme.push(char);
                input.next();
            }
        }
    }

    match lexeme.parse::<f64>() {
        Ok(number) => {
            let number = if negative { -1.0 * number } else { number };
            Ok(Token::Number { value: number })
        },
        Err(_) => {
            Err(LisrScanError::InvalidNumber{ reason: "Could not parse the number. Only 64-bit floats represented in decimal system are allowed." })
        }
    }
}

fn scan_identifier(input: &mut Peekable<Chars>) -> Result<Token, LisrScanError<'static>> {
    let mut lexeme = String::new();
    if let Some(&char) = input.peek() {
        if char.is_digit(RADIX) {
            return Err(LisrScanError::InvalidIdentifier {
                reason: "Identifier cannot start with a digit.",
            });
        }
        lexeme.push(char);
        input.next();
    }

    while let Some(&char) = input.peek() {
        if should_finish_scanning(char) {
            break;
        }
        lexeme.push(char);
        input.next();
    }

    if lexeme.is_empty() {
        panic!("Failed while scanning an identifier.");
    }

    Ok(match_keyword_or_identifier(lexeme))
}

fn match_keyword_or_identifier(lexeme: String) -> Token {
    match lexeme.as_str() {
        "set!" => Token::Set,
        "define" => Token::Define,
        "true" => Token::True,
        "false" => Token::False,
        "if" => Token::If,
        "and" => Token::And,
        "or" => Token::Or,
        "lambda" => Token::Lambda,
        "begin" => Token::Begin,
        _ => Token::Identifier { name: lexeme },
    }
}

fn should_finish_scanning(char: char) -> bool {
    char.is_whitespace() || char == LEFT_PAREN || char == RIGHT_PAREN
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_test() -> Result<(), LisrScanError<'static>> {
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
        ";

        let tokens = scan(input)?;
        for token in tokens {
            println!("{:#?}", token);
        }
        Ok(())
    }
}
