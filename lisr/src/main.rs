#![feature(iterator_try_reduce)]

mod environment;
mod evaluate;
mod evaluation_errors;
mod expression;
mod node;
mod parse;
mod scan;
mod token;
mod translate;

fn main() {
    let tokens = scan::scan(
        "
    (define (make-adder n) (lambda (x) (+ x n)))
    (define two-plus (make-adder 2))
    (+ 8 (two-plus 40) -10 12.5)
    (+ \"abc\" \"def\")
    (* (+ (- (/ 8 2) 2) 12) 0.5)
    (remainder 5.2 3.0)
    (= 2 2 2 2 3)
    (= \"a string\" \"a string\")
    (and (< \"abc\" \"def\") (< 3 4))
    (or (< 2 1) (< 3 2))
    (define x 12)
    x
    (or (= 2 3) (set! x 14))
    x
    (begin (+ 2 2) (- 2 3) (set! x 15) (* 2 3))
    (car (cons 1 2))
    (car (car (cons (cons -2 3) 4)))
    (car (cdr (cons 5 (cons 6 7))))
    (define one-two-three (cons 1 (cons 2 (cons 3 ()))))
    
    (empty-list? one-two-three)
    (empty-list? ())

    (car (cdr one-two-three))

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
    (two-plus (last-element one-two-three))
    ",
    )
    .unwrap();

    let nodes = parse::parse(tokens).unwrap();
    let expressions = translate::translate(nodes).unwrap();

    println!("===> {:#?}", evaluate::evaluate(expressions));
}
