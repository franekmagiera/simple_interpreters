#[derive(Debug)]
pub enum LisrEvaluationError {
    RuntimeError { reason: &'static str },
    TypeError,
    UndefinedIdentifier,
}
