use fmt::Write;
use std::{fmt, ops};

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

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        for stmt in &self.0 {
            out.push_str(&stmt.to_string())
        }
        write!(f, "{}", out)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Let {
        token: Token,
        name: String,
        value: Expression,
    },
    Return {
        token: Token,
        value: Expression,
    },
    Expr(Expression),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Let { token, name, value } => write!(f, "{} {} = {};", token, name, value),
            Self::Return { token, value } => write!(f, "{} {};", token, value),
            Self::Expr(expr) => write!(f, "{}", expr),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Ident(String),
    IntegerLiteral(isize),
    Prefix {
        token: Token,
        operator: String,
        right: Box<Expression>,
    },
    Infix {
        token: Token,
        operator: String,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Boolean(bool),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Ident(value) => write!(f, "{}", value),
            Expression::IntegerLiteral(value) => write!(f, "{}", value),
            Expression::Prefix {
                token: _,
                operator,
                right,
            } => write!(f, "({}{})", operator, right),
            Expression::Infix {
                token: _,
                operator,
                left,
                right,
            } => write!(f, "({} {} {})", left, operator, right),
            Expression::Boolean(value) => write!(f, "{}", value),
        }
    }
}
