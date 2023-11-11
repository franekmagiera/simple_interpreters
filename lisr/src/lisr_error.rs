use crate::{evaluate::LisrEvaluationError, parse::LisrParseError, scan::LisrScanError};

#[derive(Debug)]
pub enum LisrError<'a> {
    LisrScanError(LisrScanError<'a>),
    LisrParseError(LisrParseError),
    LisrEvaluationError(LisrEvaluationError),
}

impl<'a> From<LisrScanError<'a>> for LisrError<'a> {
    fn from(error: LisrScanError<'a>) -> Self {
        LisrError::LisrScanError(error)
    }
}

impl From<LisrParseError> for LisrError<'_> {
    fn from(error: LisrParseError) -> Self {
        LisrError::LisrParseError(error)
    }
}

impl From<LisrEvaluationError> for LisrError<'_> {
    fn from(error: LisrEvaluationError) -> Self {
        LisrError::LisrEvaluationError(error)
    }
}
