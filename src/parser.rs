#![allow(dead_code)]

use crate::{
    ast::{BlockStatement, Expression, Identifier, Program, Statement},
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
            Token::LParen => Self::Call,
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
    pub fn new(mut lexer: Lexer<'a>) -> Self {
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

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program::new();

        while self.current_token != Token::Eof {
            match self.parse_statement() {
                Ok(stmt) => program.push(stmt),
                Err(e) => {
                    println!("{:?}", e);
                    continue;
                }
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
            //miette::bail!("Expected Assign");
            return Err(miette::miette!(
                severity = miette::Severity::Error,
                code = "expected::rparen",
                help = "always close your parens",
                labels = vec![miette::LabeledSpan::at(0..5, "here")],
                //url = "https://example.com",
                help = "Use `=` after the identifier",
                "Expected Assign!!!"
            ).with_source_code(self.lexer.source_code().to_string()));
        }
        self.next_token();
        self.next_token();

        let value = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token == Token::Semicolon {
            self.next_token();
        }

        Ok(Statement::Let {
            token: current_token,
            name,
            value,
        })
    }

    fn parse_return_statement(&mut self) -> Result<Statement> {
        let current_token = self.current_token.clone();
        self.next_token();

        let return_value = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token == Token::Semicolon {
            self.next_token();
        }

        Ok(Statement::Return {
            token: current_token,
            value: return_value,
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
            // Prefix operators
            Token::Ident(ident) => Expression::Ident(Identifier::new(ident.clone())),
            Token::Int(i) => {
                Expression::IntegerLiteral(i.parse().expect("Failed parsing Token::Int(i)"))
            }
            Token::True => Expression::Boolean(true),
            Token::False => Expression::Boolean(false),
            Token::LParen => self.parse_grouped_expression()?,
            Token::If => self.parse_if_expression()?,
            Token::Function => self.parse_function_literal()?,
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
                Token::LParen => {
                    if let Ok(expr) = self.parse_call_expression(left_exp.clone()) {
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

    fn parse_if_expression(&mut self) -> Result<Expression> {
        //let token = self.current_token.clone();
        if self.peek_token != Token::LParen {
            miette::bail!("Expected Left Parenthesis before condition");
        }
        self.next_token(); // jump over LParen
        self.next_token();

        let condition = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token != Token::RParen {
            miette::bail!("Expected Right Parenthesis after condition");
        }
        self.next_token(); // jump over RParen

        if self.peek_token != Token::LBrace {
            miette::bail!("Expected Left Brace at beginning of block");
        }
        self.next_token(); // jump over LBrace

        let consequence = self.parse_block_statement()?;

        let alternative = if self.peek_token == Token::Else {
            self.next_token(); // jump over the else
            if self.peek_token != Token::LBrace {
                miette::bail!("Expected Left Brace after `else`")
            }
            self.next_token(); // jump over LBrace
            self.parse_block_statement().ok()
        } else {
            None
        };

        Ok(Expression::If {
            condition: Box::new(condition),
            consequence,
            alternative,
        })
    }

    fn parse_block_statement(&mut self) -> Result<BlockStatement> {
        let mut block_statement = BlockStatement::new();
        self.next_token();

        while self.current_token != Token::RBrace && self.current_token != Token::Eof {
            if let Ok(stmt) = self.parse_statement() {
                block_statement.push(stmt);
            };
            self.next_token();
        }

        Ok(block_statement)
    }

    fn parse_function_literal(&mut self) -> Result<Expression> {
        if self.peek_token != Token::LParen {
            miette::bail!("Expeced LParen after `fn`");
        }
        self.next_token();

        let parameters = self.parse_function_parameters()?;

        if self.peek_token != Token::LBrace {
            miette::bail!("Expeced LBrace after parameter list");
        }
        self.next_token();

        let body = self.parse_block_statement()?;

        Ok(Expression::FunctionLiteral { parameters, body })
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<Identifier>> {
        let mut identifiers = Vec::new();

        if self.peek_token == Token::RParen {
            self.next_token();
            return Ok(identifiers);
        }
        self.next_token();

        let identifier = Identifier::new(self.current_token.to_string());
        identifiers.push(identifier);

        while self.peek_token == Token::Comma {
            self.next_token();
            self.next_token();
            identifiers.push(Identifier::new(self.current_token.to_string()));
        }

        if self.peek_token != Token::RParen {
            miette::bail!("Expected RParen")
        }
        self.next_token();

        Ok(identifiers)
    }

    fn parse_call_expression(&mut self, function: Expression) -> Result<Expression> {
        let arguments = self.parse_call_arguments()?;
        Ok(Expression::Call {
            function: Box::new(function),
            arguments,
        })
    }

    fn parse_call_arguments(&mut self) -> Result<Vec<Expression>> {
        let mut args = Vec::new();
        if self.peek_token == Token::RParen {
            self.next_token();
            return Ok(args);
        }
        self.next_token();

        if let Ok(expr) = self.parse_expression(Precedence::Lowest) {
            args.push(expr)
        }

        while self.peek_token == Token::Comma {
            self.next_token();
            self.next_token();
            if let Ok(expr) = self.parse_expression(Precedence::Lowest) {
                args.push(expr)
            }
        }

        if self.peek_token != Token::RParen {
            miette::bail!("Expected RParen");
        }
        self.next_token();

        Ok(args)
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
let y = true;
let foobar = y;
";
        let program = program_from_input(input);

        assert_eq!(program.len(), 3);
        assert_eq!(
            program[0],
            Statement::Let {
                token: Token::Let,
                name: "x".into(),
                value: Expression::IntegerLiteral(5),
            }
        );
        assert_eq!(
            program[1],
            Statement::Let {
                token: Token::Let,
                name: "y".into(),
                value: Expression::Boolean(true),
            }
        );
        assert_eq!(
            program[2],
            Statement::Let {
                token: Token::Let,
                name: "foobar".into(),
                value: Expression::Ident(Identifier::new("y".to_string()))
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
                value: Expression::IntegerLiteral(5),
            }
        );
        assert_eq!(
            program[1],
            Statement::Return {
                token: Token::Return,
                value: Expression::IntegerLiteral(10),
            }
        );
        assert_eq!(
            program[2],
            Statement::Return {
                token: Token::Return,
                value: Expression::IntegerLiteral(993322),
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
        assert_eq!(
            program_from_input("3 > 5 == false").to_string(),
            "((3 > 5) == false)"
        );
        assert_eq!(
            program_from_input("3 < 5 == true").to_string(),
            "((3 < 5) == true)"
        );

        assert_eq!(
            program_from_input("1 + (2 + 3) + 4").to_string(),
            "((1 + (2 + 3)) + 4)"
        );
        assert_eq!(
            program_from_input("(5 + 5) * 2").to_string(),
            "((5 + 5) * 2)"
        );
        assert_eq!(
            program_from_input("2 / (5 + 5)").to_string(),
            "(2 / (5 + 5))"
        );
        assert_eq!(program_from_input("-(5 + 5)").to_string(), "(-(5 + 5))");
        assert_eq!(
            program_from_input("!(true == true)").to_string(),
            "(!(true == true))"
        );

        assert_eq!(
            program_from_input("a + add(b * c) + d").to_string(),
            "((a + add((b * c))) + d)"
        );
        assert_eq!(
            program_from_input("add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))").to_string(),
            "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))"
        );
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

    #[test]
    fn test_if_expression() {
        let input = "if (x < y) { x }";
        let program = program_from_input(input);
        let mut consequence = BlockStatement::new();
        consequence.push(Statement::Expr(Expression::Ident(Identifier::new(
            "x".into(),
        ))));
        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::If {
                condition: Box::new(Expression::Infix {
                    token: Token::LessThan,
                    operator: "<".into(),
                    left: Box::new(Expression::Ident(Identifier::new("x".into()))),
                    right: Box::new(Expression::Ident(Identifier::new("y".into()))),
                }),
                consequence,
                alternative: None,
            })
        );
    }

    #[test]
    fn test_if_else_expression() {
        let input = "if (x < y) { x } else { y }";
        let program = program_from_input(input);
        let mut consequence = BlockStatement::new();
        consequence.push(Statement::Expr(Expression::Ident(Identifier::new(
            "x".into(),
        ))));
        let mut alternative = BlockStatement::new();
        alternative.push(Statement::Expr(Expression::Ident(Identifier::new(
            "y".into(),
        ))));
        let alternative = Some(alternative);
        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::If {
                condition: Box::new(Expression::Infix {
                    token: Token::LessThan,
                    operator: "<".into(),
                    left: Box::new(Expression::Ident(Identifier::new("x".into()))),
                    right: Box::new(Expression::Ident(Identifier::new("y".into()))),
                }),
                consequence,
                alternative,
            })
        );
    }

    #[test]
    fn test_function_literal() {
        let input = "fn(x, y) { x + y; }";
        let program = program_from_input(input);
        let mut body = BlockStatement::new();
        body.push(Statement::Expr(Expression::Infix {
            token: Token::Plus,
            operator: "+".into(),
            left: Box::new(Expression::Ident(Identifier::new("x".into()))),
            right: Box::new(Expression::Ident(Identifier::new("y".into()))),
        }));

        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::FunctionLiteral {
                parameters: vec![Identifier::new("x".into()), Identifier::new("y".into())],
                body,
            })
        )
    }

    #[test]
    fn test_function_parameter_parsing() {
        let program = program_from_input("fn() {};");
        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::FunctionLiteral {
                parameters: vec![],
                body: BlockStatement::new(),
            })
        );

        let program = program_from_input("fn(x) {};");
        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::FunctionLiteral {
                parameters: vec![Identifier::new("x".into())],
                body: BlockStatement::new(),
            })
        );

        let program = program_from_input("fn(x, y, z) {};");
        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::FunctionLiteral {
                parameters: vec![
                    Identifier::new("x".into()),
                    Identifier::new("y".into()),
                    Identifier::new("z".into())
                ],
                body: BlockStatement::new(),
            })
        );
    }

    #[test]
    fn test_call_expression_parsing() {
        let program = program_from_input("add(1, 2 * 3, 4 + 5)");
        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::Call {
                function: Box::new(Expression::Ident(Identifier::new("add".to_string()))),
                arguments: vec![
                    Expression::IntegerLiteral(1),
                    Expression::Infix {
                        token: Token::Asterisk,
                        operator: "*".to_string(),
                        left: Box::new(Expression::IntegerLiteral(2)),
                        right: Box::new(Expression::IntegerLiteral(3)),
                    },
                    Expression::Infix {
                        token: Token::Plus,
                        operator: "+".to_string(),
                        left: Box::new(Expression::IntegerLiteral(4)),
                        right: Box::new(Expression::IntegerLiteral(5)),
                    },
                ]
            })
        );
    }
}
