use crate::{environment::Environment, evaluation_errors::LisrEvaluationError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum Expression {
    String {
        value: String,
    },
    Number {
        value: f64,
    },

    Quotation {
        text_of_quotation: String,
    },

    Identifier(Identifier),

    Assignment {
        variable: Identifier,
        value: Box<Expression>,
    },
    Definition {
        variable: Identifier,
        value: Box<Expression>,
    },

    True,
    False,

    If {
        predicate: Box<Expression>,
        consequent: Box<Expression>,
        alternative: Box<Expression>,
    },
    And {
        operands: Vec<Expression>,
    },
    Or {
        operands: Vec<Expression>,
    },

    Lambda {
        parameters: Vec<Parameter>,
        body: Box<Expression>,
    },

    Begin {
        sequence: Vec<Expression>,
    },

    Application {
        procedure: Box<Expression>,
        arguments: Vec<Expression>,
    },

    // TODO: cons, list and '(1 2 3) would create this.
    List {
        elements: Vec<Expression>,
    },

    LisrInternalObject {
        name: String,
    },

    // Implemented as a flat-closure (a.k.a. one block closure) for simplicity - each
    // function has a copy of it's enclosing environment. This means a procedure
    // cannot modify it's enclosing environment, but can only read it.
    CompoundProcedure {
        parameters: Vec<Parameter>,
        body: Box<Expression>,
        environment: Box<Environment>,
    },

    PrimitiveProcedure {
        procedure: fn(Vec<Expression>) -> Result<Expression, LisrEvaluationError>,
    },
}
