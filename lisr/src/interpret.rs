use crate::{
    evaluate::evaluate, expression::Expression, lisr_error::LisrError, parse::parse, scan::scan,
    translate::translate,
};

pub fn interpret(input: &str) -> Result<Expression, LisrError> {
    let tokens = scan(input)?;
    let nodes = parse(tokens)?;
    let expressions = translate(nodes)?;
    let result = evaluate(expressions)?;
    return Ok(result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_arithmetic_operations() {
        let input = "
            (+ (- 10 5)
               (* 2 2.5)
               (/ 50 10)
               (remainder 15 10)
            )
        ";

        let result = interpret(input).unwrap();

        assert_eq!(result, Expression::Number { value: 20.0 });
    }

    #[test]
    fn test_logical_operations() {
        let input = "
            (and (= 1 1) (< 3 4) (or false true))
        ";

        let result = interpret(input).unwrap();

        assert_eq!(result, Expression::True);
    }

    #[test]
    fn test_string_concatenation() {
        let input = "(+ \"a\" \" long\" \" string\")";

        let result = interpret(input).unwrap();

        assert_eq!(
            result,
            Expression::String {
                value: String::from("a long string")
            }
        );
    }

    #[test]
    fn test_definition() {
        let input = "
            (define x 5)
            x
        ";

        let result = interpret(input).unwrap();

        assert_eq!(result, Expression::Number { value: 5.0 });
    }

    #[test]
    fn test_function_definition() {
        let input = "
            (define (square x) (* x x))
            (square 4)
        ";

        let result = interpret(input).unwrap();

        assert_eq!(result, Expression::Number { value: 16.0 });
    }

    #[test]
    fn test_assignment() {
        let input = "
            (define x 5)
            (set! x 6)
            x
        ";

        let result = interpret(input).unwrap();

        assert_eq!(result, Expression::Number { value: 6.0 });
    }

    #[test]
    fn test_closures() {
        let input = "
            (define (make-adder n) (lambda (x) (+ x n)))
            (define two-plus (make-adder 2))
            (two-plus 3)
        ";

        let result = interpret(input).unwrap();

        assert_eq!(result, Expression::Number { value: 5.0 });
    }

    #[test]
    fn test_if() {
        let input = "(if false 1 (if true 2 3))";

        let result = interpret(input).unwrap();

        assert_eq!(result, Expression::Number { value: 2.0 });
    }

    #[test]
    fn test_begin_block() {
        let input = "
            (define x 0)
            (define y 0)
            (if true
                (begin (set! x 1)
                       (set! y 1)
                )
                ()
            )
            (+ x y)
        ";

        let result = interpret(input).unwrap();

        assert_eq!(result, Expression::Number { value: 2.0 });
    }

    #[test]
    fn test_cons() {
        let input = "
            (define xs (cons 1 2))
            (+ (car xs) (cdr xs))
        ";

        let result = interpret(input).unwrap();

        assert_eq!(result, Expression::Number { value: 3.0 });
    }

    #[test]
    fn test_empty_list() {
        let input = "
            (define one-two-three (cons 1 (cons 2 (cons 3 ()))))
            (empty-list? one-two-three)
        ";

        let result = interpret(input).unwrap();

        assert_eq!(result, Expression::False);
    }

    #[test]
    fn test_recursive_procedure() {
        let input = "
            (define one-two-three (cons 1 (cons 2 (cons 3 ()))))
    
            (define (last-element xs)
                (if (empty-list? xs)
                    ()
                    (if (empty-list? (cdr xs))
                        (car xs)
                        (last-element (cdr xs))
                    )
                )
            )

            (last-element one-two-three)
        ";

        let result = interpret(input).unwrap();

        assert_eq!(result, Expression::Number { value: 3.0 });
    }

    // And a couple of fun programs:

    #[test]
    fn test_gcd() {
        let input = "
            (define (gcd a b)
                (if (= b 0)
                    a
                    (gcd b (remainder a b))
                )
            )

            (gcd 16 28)
        ";

        let result = interpret(input).unwrap();

        assert_eq!(result, Expression::Number { value: 4.0 });
    }

    #[test]
    fn test_fast_expt() {
        let input = "
        (define (even? n)
            (= (remainder n 2) 0)
        )

        (define (fast-expt-inner b n a)
            (if (= n 0)
                a
                (if (even? n)
                    (fast-expt-inner (* b b) (/ n 2) a)
                    (fast-expt-inner b (- n 1) (* a b))
                )
            )
        )

        (define (fast-expt b n) (fast-expt-inner b n 1))

        (fast-expt 2 11)
        ";

        let result = interpret(input).unwrap();

        assert_eq!(result, Expression::Number { value: 2048.0 });
    }

    #[test]
    fn test_square_root() {
        let input = "
            (define (square x) (* x x))
            (define (average x y) (/ (+ x y) 2))
            (define (abs x)
                (if (< x 0)
                    (- 0 x)
                    x
                )
            )

            (define (sqrt x)
                (begin
                    (define (good-enough? guess)
                        (<
                            (/
                                (abs (- (square guess) x))
                                x
                            )
                            0.0001
                        )
                    )

                    (define (improve-guess guess)
                        (average guess (/ x guess))
                    )

                    (define (sqrt-iter guess)
                        (if (good-enough? guess)
                            guess
                            (sqrt-iter (improve-guess guess))
                        )
                    )

                    (sqrt-iter 1.0)
                )
            )   

            (sqrt 9)
        ";

        let result = interpret(input).unwrap();
        if let Expression::Number { value } = result {
            assert!(f64::abs(value - 3.0) < 0.0001)
        } else {
            panic!("The result of a square root function should be a number.");
        }
    }

    #[test]
    fn test_append() {
        let input = "
            (define (append xs ys)
                (if (empty-list? xs)
                    ys
                    (cons (car xs) (append (cdr xs) ys))
                )
            )

            (define xs (cons 1 (cons 2 ())))
            (define ys (cons 3 (cons 4 ())))

            (append xs ys)
        ";

        let result = interpret(input).unwrap();

        assert_eq!(
            result,
            Expression::Cons {
                first: Box::new(Expression::Number { value: 1.0 }),
                rest: Box::new(Expression::Cons {
                    first: Box::new(Expression::Number { value: 2.0 }),
                    rest: Box::new(Expression::Cons {
                        first: Box::new(Expression::Number { value: 3.0 }),
                        rest: Box::new(Expression::Cons {
                            first: Box::new(Expression::Number { value: 4.0 }),
                            rest: Box::new(Expression::EmptyList)
                        })
                    })
                })
            }
        );
    }

    #[test]
    fn test_accumulate() {
        let input = "
            (define (accumulate op initial sequence)
                (if (empty-list? sequence)
                    initial
                    (op (car sequence) (accumulate op initial (cdr sequence)))
                )
            )

            (define (sum xs) (accumulate + 0 xs))

            (define one-two-three (cons 1 (cons 2 (cons 3 ()))))

            (sum one-two-three)
        ";

        let result = interpret(input).unwrap();

        assert_eq!(result, Expression::Number { value: 6.0 });
    }
}
