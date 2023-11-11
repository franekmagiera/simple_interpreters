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

#[derive(Debug, PartialEq)]
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
                                reason: "Identifier cannot start with an operator.",
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
        // This should never happen, because it is called only in the scan function,
        // which checks that there is something in peek().
        // The caller of that function should validate that.
        panic!("Failed while scanning a literal - nothing to scan.");
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
                        reason: "A number cannot have more than one decimal point.",
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
        "cons" => Token::Cons,
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
    fn should_scan_left_and_right_parentheses() {
        let input = "()";

        let tokens = scan(input).unwrap();

        assert_eq!(tokens, vec![Token::LeftParen, Token::RightParen]);
    }

    #[test]
    fn should_scan_numbers() {
        let input = "(+ 2.5 3.5)";

        let tokens = scan(input).unwrap();

        assert_eq!(tokens.get(2), Some(&Token::Number { value: 2.5 }));
        assert_eq!(tokens.get(3), Some(&Token::Number { value: 3.5 }));
    }

    #[test]
    fn should_scan_unicode_identifiers() {
        let input = "(😲 🍟)";

        let tokens = scan(input).unwrap();

        assert_eq!(
            tokens.get(1),
            Some(&Token::Identifier {
                name: String::from("😲")
            })
        );
        assert_eq!(
            tokens.get(2),
            Some(&Token::Identifier {
                name: String::from("🍟")
            })
        );
    }

    #[test]
    fn should_scan_dash_as_identifier() {
        let input = "(- 2 1)";

        let tokens = scan(input).unwrap();

        assert_eq!(
            tokens.get(1),
            Some(&Token::Identifier {
                name: String::from("-")
            })
        );
    }

    #[test]
    fn should_scan_dash_as_a_part_of_a_negative_number() {
        let input = "(- 5 -2.5)";

        let tokens = scan(input).unwrap();

        assert_eq!(tokens.get(3), Some(&Token::Number { value: -2.5 }));
    }

    #[test]
    fn should_scan_dash_when_it_is_the_last_character() {
        let input = "-";

        let tokens = scan(input).unwrap();

        assert_eq!(
            tokens.get(0),
            Some(&Token::Identifier {
                name: String::from("-")
            })
        );
    }

    #[test]
    fn should_scan_primitive_operators_as_identifiers() {
        let operators = vec!["+", "*", "/", "<", ">"];

        for operator in operators.iter() {
            let input = format!("({} 2 1", operator);

            let tokens = scan(&input).unwrap();

            let actual_token_for_operator = tokens.get(1);
            let expected_token = Token::Identifier {
                name: String::from(*operator),
            };

            assert_eq!(actual_token_for_operator, Some(&expected_token));
        }
    }

    #[test]
    fn should_scan_quotation() {
        let input = "(concat 'one 'two)";

        let tokens = scan(input).unwrap();

        assert_eq!(
            tokens.get(2),
            Some(&Token::Quotation {
                text_of_quotation: String::from("one")
            })
        );
        assert_eq!(
            tokens.get(3),
            Some(&Token::Quotation {
                text_of_quotation: String::from("two")
            })
        );
    }

    #[test]
    fn should_scan_strings() {
        let input = "\"a very\nlong string\"";

        let tokens = scan(input).unwrap();

        assert_eq!(
            tokens.get(0),
            Some(&Token::String {
                value: String::from("a very\nlong string")
            })
        );
    }

    #[test]
    fn should_match_keywords() {
        let keyword_to_expected_token = vec![
            ("set!", Token::Set),
            ("define", Token::Define),
            ("true", Token::True),
            ("false", Token::False),
            ("if", Token::If),
            ("and", Token::And),
            ("or", Token::Or),
            ("lambda", Token::Lambda),
            ("begin", Token::Begin),
            ("cons", Token::Cons),
        ];

        for (keyword, expected_token) in keyword_to_expected_token.iter() {
            let tokens = scan(keyword).unwrap();

            assert_eq!(tokens.get(0), Some(expected_token));
        }
    }

    #[test]
    fn should_reject_identifiers_that_start_with_primitive_operator() {
        let input = "(define <3 'heart)";

        let error = scan(input).unwrap_err();

        assert_eq!(
            error,
            LisrScanError::InvalidIdentifier {
                reason: "Identifier cannot start with an operator."
            }
        );
    }

    #[test]
    fn should_not_allow_empty_quotation() {
        let input = "(concat ' 'abc)";

        let error = scan(input).unwrap_err();

        assert_eq!(error, LisrScanError::EmptyQuotation);
    }

    #[test]
    fn should_return_an_error_when_string_is_not_closed() {
        let input = "\"Oops, an unclosed string";

        let error = scan(input).unwrap_err();

        assert_eq!(error, LisrScanError::UnclosedString);
    }

    #[test]
    fn should_reject_numbers_with_more_than_one_decimal_point() {
        let input = "123.456.78";

        let error = scan(input).unwrap_err();

        assert_eq!(
            error,
            LisrScanError::InvalidNumber {
                reason: "A number cannot have more than one decimal point."
            }
        );
    }

    #[test]
    fn should_reject_numbers_that_do_not_contain_digits_only() {
        let input = "0x123";

        let error = scan(input).unwrap_err();

        assert_eq!(
            error,
            LisrScanError::InvalidNumber {
                reason: "Numbers can contain digits only."
            }
        );
    }

    #[test]
    fn should_reject_identifiers_that_start_with_a_digit() {
        let input = "(define (5plus x) (+ 5 x))";

        let error = scan(input).unwrap_err();

        assert_eq!(
            error,
            LisrScanError::InvalidNumber {
                reason: "Numbers can contain digits only."
            }
        );
    }
}
