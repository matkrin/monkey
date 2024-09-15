#![allow(dead_code)]

use crate::{
    ast::{Expression, Program, Statement},
    lexer::Lexer,
    token::Token,
};
use miette::Result;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Lowest,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    Call,
}

impl From<&Token> for Precedence {
    fn from(value: &Token) -> Self {
        match value {
            Token::Equal => Self::Equals,
            Token::NotEqual => Self::Equals,
            Token::LessThan => Self::LessGreater,
            Token::GreaterThan => Self::LessGreater,
            Token::Plus => Self::Sum,
            Token::Minus => Self::Sum,
            Token::Slash => Self::Product,
            Token::Asterisk => Self::Product,
            _ => Self::Lowest,
        }
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    peek_token: Token,
}

impl<'a> Parser<'a> {
    fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();

        Self {
            lexer,
            current_token,
            peek_token,
        }
    }

    fn next_token(&mut self) {
        self.current_token = self.lexer.next_token();
        std::mem::swap(&mut self.current_token, &mut self.peek_token);
    }

    fn current_precedence(&self) -> Precedence {
        Precedence::from(&self.current_token)
    }

    fn peek_precedence(&self) -> Precedence {
        Precedence::from(&self.peek_token)
    }

    //fn expect_peek(&mut self, token: Token) -> bool {
    //    if self.peek_token == token {
    //        self.next_token();
    //        true
    //    } else {
    //        false
    //    }
    //}

    fn parse_program(&mut self) -> Program {
        let mut program = Program::new();

        while self.current_token != Token::Eof {
            match self.parse_statement() {
                Ok(stmt) => program.push(stmt),
                Err(e) => {}
            }
            self.next_token();
        }

        program
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match &self.current_token {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement> {
        let current_token = self.current_token.clone();
        self.next_token();
        let name = match &self.current_token {
            Token::Ident(ident) => ident.clone(),
            t => miette::bail!("Expected Ident, got: {}", t),
        };

        if self.peek_token != Token::Assign {
            miette::bail!("Expected ")
        }

        // skip until semicolon for now
        while self.current_token != Token::Semicolon {
            self.next_token()
        }

        Ok(Statement::Let {
            token: current_token,
            name,
            value: todo!(),
        })
    }

    fn parse_return_statement(&mut self) -> Result<Statement> {
        let current_token = self.current_token.clone();
        self.next_token();

        // skip until semicolon for now
        while self.current_token != Token::Semicolon {
            self.next_token()
        }

        Ok(Statement::Return {
            token: current_token,
            value: todo!(),
        })
    }

    fn parse_expression_statement(&mut self) -> Result<Statement> {
        let expression = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token == Token::Semicolon {
            self.next_token()
        }
        Ok(Statement::Expr(expression))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression> {
        let mut left_exp = match &self.current_token {
            Token::Ident(ident) => Expression::Ident(ident.clone()),
            Token::Int(i) => {
                Expression::IntegerLiteral(i.parse().expect("Failed parsing Token::Int(i)"))
            }
            Token::True => Expression::Boolean (true),
            Token::False => Expression::Boolean(false),
            Token::LParen => self.parse_grouped_expression()?,
            // Prefix operators
            Token::Minus | Token::Bang => self.parse_prefix_expression()?,
            _ => miette::bail!("Cannot parse expression yet"),
        };

        while self.peek_token != Token::Semicolon && precedence < self.peek_precedence() {
            self.next_token();
            match &self.current_token {
                // Infix operators
                Token::Plus
                | Token::Minus
                | Token::Slash
                | Token::Asterisk
                | Token::Equal
                | Token::NotEqual
                | Token::LessThan
                | Token::GreaterThan => {
                    if let Ok(expr) = self.parse_infix_expression(left_exp.clone()) {
                        left_exp = expr;
                    }
                }
                _ => return Ok(left_exp),
            };
        }
        Ok(left_exp)
    }

    fn parse_prefix_expression(&mut self) -> Result<Expression> {
        let current_token = self.current_token.clone();
        let operator = current_token.to_string();

        self.next_token();

        let right = self.parse_expression(Precedence::Prefix)?;

        Ok(Expression::Prefix {
            token: current_token,
            operator,
            right: Box::new(right),
        })
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Result<Expression> {
        let current_token = self.current_token.clone();
        let operator = current_token.to_string();
        let precedence = self.current_precedence();

        self.next_token();

        let right = self.parse_expression(precedence)?;

        Ok(Expression::Infix {
            token: current_token,
            operator,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn parse_grouped_expression(&mut self) -> Result<Expression> {
        self.next_token();

        let expression = self.parse_expression(Precedence::Lowest);

        if self.peek_token != Token::RParen {
            miette::bail!("Expected Token::RParen");
        }

        self.next_token();

        expression
    }
}

mod tests {
    use super::*;

    fn program_from_input(input: &str) -> Program {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        parser.parse_program()
    }

    #[test]
    fn test_let_statement() {
        let input = "let x = 5;
let y = 10;
let foobar = 838383;
";
        let program = program_from_input(input);

        assert_eq!(program.len(), 3);
        assert_eq!(
            program[0],
            Statement::Let {
                token: Token::Let,
                name: "x".into(),
                value: todo!()
            }
        );
        assert_eq!(
            program[1],
            Statement::Let {
                token: Token::Let,
                name: "y".into(),
                value: todo!()
            }
        );
        assert_eq!(
            program[2],
            Statement::Let {
                token: Token::Let,
                name: "foobar".into(),
                value: todo!()
            }
        );
    }

    #[test]
    fn test_return_statement() {
        let input = "return 5;
return 10;
return 993322;
";
        let program = program_from_input(input);

        assert_eq!(program.len(), 3);
        assert_eq!(
            program[0],
            Statement::Return {
                token: Token::Return,
                value: todo!()
            }
        );
        assert_eq!(
            program[0],
            Statement::Return {
                token: Token::Return,
                value: todo!()
            }
        );
        assert_eq!(
            program[0],
            Statement::Return {
                token: Token::Return,
                value: todo!()
            }
        );
    }

    #[test]
    fn test_integer_literal_expression() {
        let input = "5;";
        let program = program_from_input(input);

        assert_eq!(program.len(), 1);
        assert_eq!(program[0], Statement::Expr(Expression::IntegerLiteral(5)));
    }

    #[test]
    fn test_parsing_prefix_expression() {
        let input = "!5";
        let program = program_from_input(input);

        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::Prefix {
                token: Token::Bang,
                operator: "!".into(),
                right: Box::new(Expression::IntegerLiteral(5)),
            })
        );

        let input = "-5";
        let program = program_from_input(input);

        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::Prefix {
                token: Token::Minus,
                operator: "-".into(),
                right: Box::new(Expression::IntegerLiteral(5)),
            })
        );

        let program = program_from_input("!true;");
        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::Prefix {
                token: Token::Bang,
                operator: "!".into(),
                right: Box::new(Expression::Boolean(true)),
            })
        );
    }

    #[test]
    fn test_parsing_infix_expression() {
        let input = "5 + 5;";
        let program = program_from_input(input);
        let five = Box::new(Expression::IntegerLiteral(5));

        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::Infix {
                token: Token::Plus,
                operator: "+".into(),
                left: five.clone(),
                right: five.clone(),
            })
        );

        let input = "5 - 5;";
        let program = program_from_input(input);

        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::Infix {
                token: Token::Minus,
                operator: "-".into(),
                left: five.clone(),
                right: five.clone(),
            })
        );

        let input = "5 * 5;";
        let program = program_from_input(input);

        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::Infix {
                token: Token::Asterisk,
                operator: "*".into(),
                left: five.clone(),
                right: five.clone(),
            })
        );

        let program = program_from_input("5 / 5;");
        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::Infix {
                token: Token::Slash,
                operator: "/".into(),
                left: five.clone(),
                right: five.clone(),
            })
        );

        let program = program_from_input("5 > 5;");
        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::Infix {
                token: Token::GreaterThan,
                operator: ">".into(),
                left: five.clone(),
                right: five.clone(),
            })
        );

        let program = program_from_input("5 < 5;");
        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::Infix {
                token: Token::LessThan,
                operator: "<".into(),
                left: five.clone(),
                right: five.clone(),
            })
        );

        let program = program_from_input("5 == 5;");
        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::Infix {
                token: Token::Equal,
                operator: "==".into(),
                left: five.clone(),
                right: five.clone(),
            })
        );

        let program = program_from_input("5 != 5;");
        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::Infix {
                token: Token::NotEqual,
                operator: "!=".into(),
                left: five.clone(),
                right: five.clone(),
            })
        );

        let program = program_from_input("true == true");
        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::Infix {
                token: Token::Equal,
                operator: "==".into(),
                left: Box::new(Expression::Boolean(true)),
                right: Box::new(Expression::Boolean(true)),
            })
        );

        let program = program_from_input("true != false");
        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::Infix {
                token: Token::NotEqual,
                operator: "!=".into(),
                left: Box::new(Expression::Boolean(true)),
                right: Box::new(Expression::Boolean(false)),
            })
        );
    }

    #[test]
    fn test_operator_precedence_parsing() {
        let program = program_from_input("-a * b");
        assert_eq!(program.to_string(), "((-a) * b)");

        let program = program_from_input("!-a");
        assert_eq!(program.to_string(), "(!(-a))");

        let program = program_from_input("a + b + c");
        assert_eq!(program.to_string(), "((a + b) + c)");

        assert_eq!(program_from_input("a + b - c").to_string(), "((a + b) - c)");
        assert_eq!(program_from_input("a * b * c").to_string(), "((a * b) * c)");
        assert_eq!(program_from_input("a * b / c").to_string(), "((a * b) / c)");
        assert_eq!(
            program_from_input("a + b * c + d / e - f").to_string(),
            "(((a + (b * c)) + (d / e)) - f)"
        );
        assert_eq!(
            program_from_input("3 + 4; -5 * 5").to_string(),
            "(3 + 4)((-5) * 5)"
        );
        assert_eq!(
            program_from_input("5 > 4 == 3 < 4").to_string(),
            "((5 > 4) == (3 < 4))"
        );
        assert_eq!(
            program_from_input("5 < 4 != 3 > 4").to_string(),
            "((5 < 4) != (3 > 4))"
        );
        assert_eq!(
            program_from_input("3 + 4 * 5 == 3 * 1 + 4 * 5").to_string(),
            "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))"
        );

        assert_eq!(program_from_input("true").to_string(), "true");
        assert_eq!(program_from_input("false").to_string(), "false");
        assert_eq!(program_from_input("3 > 5 == false").to_string(), "((3 > 5) == false)");
        assert_eq!(program_from_input("3 < 5 == true").to_string(), "((3 < 5) == true)");

        assert_eq!(program_from_input("1 + (2 + 3) + 4").to_string(), "((1 + (2 + 3)) + 4)");
        assert_eq!(program_from_input("(5 + 5) * 2").to_string(), "((5 + 5) * 2)");
        assert_eq!(program_from_input("2 / (5 + 5)").to_string(), "(2 / (5 + 5))");
        assert_eq!(program_from_input("-(5 + 5)").to_string(), "(-(5 + 5))");
        assert_eq!(program_from_input("!(true == true)").to_string(), "(!(true == true))");
    }

    #[test]
    fn test_parsing_boolean() {
        let program = program_from_input("false;");
        assert_eq!(program.len(), 1);
        assert_eq!(program[0], Statement::Expr(Expression::Boolean(false)));

        let program = program_from_input("true;");
        assert_eq!(program.len(), 1);
        assert_eq!(program[0], Statement::Expr(Expression::Boolean(true)));
    }
}
