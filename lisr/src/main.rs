#![feature(iterator_try_reduce)]

use crate::interpret::interpret;

mod environment;
mod evaluate;
mod expression;
mod interpret;
mod lisr_error;
mod node;
mod parse;
mod scan;
mod token;
mod translate;

fn main() {
    let input = "
        (define (reverse xs)
            (begin
                (define (iter ys accumulator)
                    (if (empty-list? ys)
                         accumulator
                         (iter (cdr ys) (cons (car ys) accumulator))
                    )
                )
                (iter xs ())
            )
        )

        (define one-two-three (cons 1 (cons 2 (cons 3 ()))))

        (reverse one-two-three)
    ";

    let result = interpret(input);
    println!("{:#?}", result);
}
