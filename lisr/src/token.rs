#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    LeftParen,
    RightParen,

    String { value: String },
    Number { value: f64 },

    Quotation { text_of_quotation: String },

    Identifier { name: String },

    Set,
    Define,

    True,
    False,
    If,
    And,
    Or,

    Lambda,
    Begin,
}
