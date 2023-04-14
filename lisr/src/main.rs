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
    ",
    )
    .unwrap();

    let nodes = parse::parse(tokens).unwrap();
    let expressions = translate::translate(nodes).unwrap();

    println!("===> {:#?}", evaluate::evaluate(expressions));
}
