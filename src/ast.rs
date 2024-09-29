use fmt::Write;
use std::{collections::HashMap, fmt, ops};

use crate::token::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Program(Program),
    Statement(Statement),
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn statements(&self) -> &[Statement] {
        &self.0
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
            Self::Let { token, name, value } => write!(f, "{} {} = {};", token.kind, name, value),
            Self::Return { token, value } => write!(f, "{} {};", token.kind, value),
            Self::Expr(expr) => write!(f, "{}", expr),
        }
    }
}

pub type BlockStatement = Program;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier(String);
impl Identifier {
    pub fn new(identifier: String) -> Self {
        Self(identifier)
    }
    pub fn value(&self) -> &str {
        &self.0
    }
}
impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Ident(Identifier),
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
    If {
        condition: Box<Expression>,
        consequence: BlockStatement,
        alternative: Option<BlockStatement>,
    },
    FunctionLiteral {
        parameters: Vec<Identifier>,
        body: BlockStatement,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    StringLiteral(String),
    ArrayLiteral(Vec<Expression>),
    IndexExpr {
        left: Box<Expression>,
        index: Box<Expression>,
    },
    HashLiteral(Vec<(Expression, Expression)>),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Ident(Identifier(value)) => write!(f, "{}", value),
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
            Expression::If {
                condition,
                consequence,
                alternative,
            } => {
                let alternative = match alternative {
                    Some(alt) => format!("else {}", alt),
                    None => "".into(),
                };
                write!(f, "if{} {} {}", condition, consequence, alternative)
            }
            Expression::FunctionLiteral { parameters, body } => {
                let params: Vec<_> = parameters.iter().map(|param| param.to_string()).collect();
                write!(f, "({}){}", params.join(", "), body)
            }
            Expression::Call {
                function,
                arguments,
            } => {
                let args: Vec<_> = arguments.iter().map(|arg| arg.to_string()).collect();
                write!(f, "{}({})", function, args.join(", "))
            }
            Expression::StringLiteral(s) => write!(f, "{}", s),
            Expression::ArrayLiteral(v) => {
                let elements: Vec<_> = v.iter().map(|it| it.to_string()).collect();
                write!(f, "[{}]", elements.join(", "))
            }
            Expression::IndexExpr { left, index } => write!(f, "({}[{}])", left, index),
            Expression::HashLiteral(v) => {
                let pairs: Vec<_> = v .iter() .map(|(key, val)| format!("{}:{}", key, val)) .collect();
                write!(f, "{{{}}}", pairs.join(", "))
            }
        }
    }
}
