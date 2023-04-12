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
    (two-plus 40)
    ",
    )
    .unwrap();

    let nodes = parse::parse(tokens).unwrap();
    let expressions = translate::translate(nodes).unwrap();

    println!("===> {:#?}", evaluate::evaluate(expressions));
}
