use crate::{
    evaluate::evaluate, expression::Expression, parse::parse, scan::scan, translate::translate,
};

pub fn interpret(input: &str) -> Expression {
    let tokens = scan(input).unwrap();
    let nodes = parse(tokens).unwrap();
    let expressions = translate(nodes).unwrap();
    let result = evaluate(expressions).unwrap();
    return result;
}
