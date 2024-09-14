use std::ops;

use crate::token::Token;

#[derive(Debug)]
pub struct Program(Vec<Statement>);

impl Program {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, stmt: Statement) {
        self.0.push(stmt)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl ops::Index<usize> for Program {
    type Output = Statement;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Let { token: Token, name: String },
    Return,
    Expr(Expression),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expression {
    Ident(String),
}
