use std::collections::VecDeque;

use crate::token::Token;

#[derive(Debug, PartialEq)]
pub enum Node {
    Leaf { token: Token },
    List { elements: VecDeque<Node> },
}
