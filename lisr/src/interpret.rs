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
