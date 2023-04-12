use std::collections::VecDeque;

use crate::token::Token;

#[derive(Debug)]
pub enum Node {
    Leaf { token: Token },
    List { elements: VecDeque<Node> },
}
