use crate::{evaluate::LisrEvaluationError, parse::LisrParseError, scan::LisrScanError};

#[derive(Debug)]
pub enum LisrError<'a> {
    Scan(LisrScanError<'a>),
    Parse(LisrParseError),
    Evaluation(LisrEvaluationError),
}

impl<'a> From<LisrScanError<'a>> for LisrError<'a> {
    fn from(error: LisrScanError<'a>) -> Self {
        LisrError::Scan(error)
    }
}

impl From<LisrParseError> for LisrError<'_> {
    fn from(error: LisrParseError) -> Self {
        LisrError::Parse(error)
    }
}

impl From<LisrEvaluationError> for LisrError<'_> {
    fn from(error: LisrEvaluationError) -> Self {
        LisrError::Evaluation(error)
    }
}
