use crate::{
    ast::{BlockStatement, Expression, Identifier, Program, Statement},
    lexer::Lexer,
    token::{Span, Token, TokenKind},
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
    Index,
}

impl From<&Token> for Precedence {
    fn from(value: &Token) -> Self {
        match value.kind {
            TokenKind::Equal => Self::Equals,
            TokenKind::NotEqual => Self::Equals,
            TokenKind::LessThan => Self::LessGreater,
            TokenKind::GreaterThan => Self::LessGreater,
            TokenKind::Plus => Self::Sum,
            TokenKind::Minus => Self::Sum,
            TokenKind::Slash => Self::Product,
            TokenKind::Asterisk => Self::Product,
            TokenKind::LParen => Self::Call,
            TokenKind::LBracket => Self::Index,
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

    pub fn parse_program(&mut self) -> (Program, Vec<miette::Report>) {
        let mut program = Program::new();
        let mut errors = Vec::new();

        while self.current_token.kind != TokenKind::Eof {
            match self.parse_statement() {
                Ok(stmt) => program.push(stmt),
                Err(e) => {
                    errors.push(e);
                }
            }
            self.next_token();
        }

        (program, errors)
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match &self.current_token.kind {
            TokenKind::Let => self.parse_let_statement(),
            TokenKind::Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement> {
        let current_token = self.current_token.clone();
        self.next_token();
        let name = match &self.current_token.kind {
            TokenKind::Ident(ident) => ident.clone(),
            t => miette::bail!("Expected Ident, got: {}", t),
        };

        if self.peek_token.kind != TokenKind::Assign {
            //miette::bail!("Expected Assign");
            let Span { start, end } = self.peek_token.span;
            return Err(miette::miette!(
                severity = miette::Severity::Error,
                labels = vec![miette::LabeledSpan::at(start..end, "here")],
                //url = "https://example.com",
                help = "Use `=` after the identifier",
                "Expected Assignment"
            )
            .with_source_code(self.lexer.source_code().to_string()));
        }
        self.next_token();
        self.next_token();

        let value = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token.kind == TokenKind::Semicolon {
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

        if self.peek_token.kind == TokenKind::Semicolon {
            self.next_token();
        }

        Ok(Statement::Return {
            token: current_token,
            value: return_value,
        })
    }

    fn parse_expression_statement(&mut self) -> Result<Statement> {
        let expression = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token.kind == TokenKind::Semicolon {
            self.next_token()
        }
        Ok(Statement::Expr(expression))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression> {
        let mut left_exp = match &self.current_token.kind {
            // Prefix operators
            TokenKind::Ident(ident) => Expression::Ident(Identifier::new(ident.clone())),
            TokenKind::Int(i) => {
                Expression::IntegerLiteral(i.parse().expect("Failed parsing Token::Int(i)"))
            }
            TokenKind::True => Expression::Boolean(true),
            TokenKind::False => Expression::Boolean(false),
            TokenKind::LParen => self.parse_grouped_expression()?,
            TokenKind::If => self.parse_if_expression()?,
            TokenKind::Function => self.parse_function_literal()?,
            TokenKind::Minus | TokenKind::Bang => self.parse_prefix_expression()?,
            TokenKind::String(s) => Expression::StringLiteral(s.into()),
            TokenKind::LBracket => {
                Expression::ArrayLiteral(self.parse_expression_list(TokenKind::RBracket)?)
            },
            TokenKind::LBrace => self.parse_hash_literal()?,
            _ => miette::bail!("Unexpected Token: {}", &self.current_token.kind),
        };

        while self.peek_token.kind != TokenKind::Semicolon && precedence < self.peek_precedence() {
            self.next_token();
            match &self.current_token.kind {
                // Infix operators
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Asterisk
                | TokenKind::Equal
                | TokenKind::NotEqual
                | TokenKind::LessThan
                | TokenKind::GreaterThan => {
                    if let Ok(expr) = self.parse_infix_expression(left_exp.clone()) {
                        left_exp = expr;
                    }
                }
                TokenKind::LParen => {
                    if let Ok(expr) = self.parse_call_expression(left_exp.clone()) {
                        left_exp = expr;
                    }
                }
                TokenKind::LBracket => {
                    if let Ok(expr) = self.parse_index_expression(left_exp.clone()) {
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
        let operator = current_token.kind.to_string();

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
        let operator = current_token.kind.to_string();
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

        if self.peek_token.kind != TokenKind::RParen {
            let Span { start, end } = self.peek_token.span;
            return Err(miette::miette!(
                severity = miette::Severity::Error,
                labels = vec![miette::LabeledSpan::at(start..end, "here")],
                //url = "https://example.com",
                help = "Use `)` to end the grouping",
                "Expected `)`"
            )
            .with_source_code(self.lexer.source_code().to_string()));
        }

        self.next_token();

        expression
    }

    fn parse_if_expression(&mut self) -> Result<Expression> {
        //let token = self.current_token.clone();
        if self.peek_token.kind != TokenKind::LParen {
            let Span { start, end } = self.peek_token.span;
            return Err(miette::miette!(
                severity = miette::Severity::Error,
                labels = vec![miette::LabeledSpan::at(start..end, "here")],
                //url = "https://example.com",
                help = "Use parentheses around condition",
                "Expected `(`"
            )
            .with_source_code(self.lexer.source_code().to_string()));
        }
        self.next_token(); // jump over LParen
        self.next_token();

        let condition = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token.kind != TokenKind::RParen {
            let Span { start, end } = self.peek_token.span;
            return Err(miette::miette!(
                severity = miette::Severity::Error,
                labels = vec![miette::LabeledSpan::at(start..end, "here")],
                //url = "https://example.com",
                help = "Use parentheses around condition",
                "Expected `)`"
            )
            .with_source_code(self.lexer.source_code().to_string()));
        }
        self.next_token(); // jump over RParen

        if self.peek_token.kind != TokenKind::LBrace {
            miette::bail!("Expected Left Brace at beginning of block");
        }
        self.next_token(); // jump over LBrace

        let consequence = self.parse_block_statement()?;

        let alternative = if self.peek_token.kind == TokenKind::Else {
            self.next_token(); // jump over the else
            if self.peek_token.kind != TokenKind::LBrace {
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

        while self.current_token.kind != TokenKind::RBrace
            && self.current_token.kind != TokenKind::Eof
        {
            if let Ok(stmt) = self.parse_statement() {
                block_statement.push(stmt);
            };
            self.next_token();
        }

        Ok(block_statement)
    }

    fn parse_function_literal(&mut self) -> Result<Expression> {
        if self.peek_token.kind != TokenKind::LParen {
            miette::bail!("Expeced LParen after `fn`");
        }
        self.next_token();

        let parameters = self.parse_function_parameters()?;

        if self.peek_token.kind != TokenKind::LBrace {
            miette::bail!("Expeced LBrace after parameter list");
        }
        self.next_token();

        let body = self.parse_block_statement()?;

        Ok(Expression::FunctionLiteral { parameters, body })
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<Identifier>> {
        let mut identifiers = Vec::new();

        if self.peek_token.kind == TokenKind::RParen {
            self.next_token();
            return Ok(identifiers);
        }
        self.next_token();

        let identifier = Identifier::new(self.current_token.kind.to_string());
        identifiers.push(identifier);

        while self.peek_token.kind == TokenKind::Comma {
            self.next_token();
            self.next_token();
            identifiers.push(Identifier::new(self.current_token.kind.to_string()));
        }

        if self.peek_token.kind != TokenKind::RParen {
            miette::bail!("Expected RParen")
        }
        self.next_token();

        Ok(identifiers)
    }

    fn parse_call_expression(&mut self, function: Expression) -> Result<Expression> {
        let arguments = self.parse_expression_list(TokenKind::RParen)?;
        Ok(Expression::Call {
            function: Box::new(function),
            arguments,
        })
    }

    // This was replaced but I leave it in for completeness
    //fn parse_call_arguments(&mut self) -> Result<Vec<Expression>> {
    //    let mut args = Vec::new();
    //    if self.peek_token.kind == TokenKind::RParen {
    //        self.next_token();
    //        return Ok(args);
    //    }
    //    self.next_token();
    //
    //    if let Ok(expr) = self.parse_expression(Precedence::Lowest) {
    //        args.push(expr)
    //    }
    //
    //    while self.peek_token.kind == TokenKind::Comma {
    //        self.next_token();
    //        self.next_token();
    //        if let Ok(expr) = self.parse_expression(Precedence::Lowest) {
    //            args.push(expr)
    //        }
    //    }
    //
    //    if self.peek_token.kind != TokenKind::RParen {
    //        miette::bail!("Expected RParen");
    //    }
    //    self.next_token();
    //
    //    Ok(args)
    //}

    fn parse_expression_list(&mut self, end: TokenKind) -> Result<Vec<Expression>> {
        let mut list = Vec::new();

        if self.peek_token.kind == end {
            self.next_token();
            return Ok(list);
        }
        self.next_token();

        list.push(self.parse_expression(Precedence::Lowest)?);

        while self.peek_token.kind == TokenKind::Comma {
            self.next_token();
            self.next_token();
            list.push(self.parse_expression(Precedence::Lowest)?);
        }

        if self.peek_token.kind != end {
            return Err(miette::miette!(
                "Expected {}, got {}",
                end,
                self.peek_token.kind
            ));
        }

        self.next_token();
        Ok(list)
    }

    fn parse_index_expression(&mut self, left: Expression) -> Result<Expression> {
        self.next_token();
        let index = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token.kind != TokenKind::RBracket {
            return Err(miette::miette!(
                "Expected RBracket, got {}",
                self.peek_token.kind
            ));
        }

        self.next_token();

        Ok(Expression::IndexExpr {
            left: Box::new(left),
            index: Box::new(index),
        })
    }

    fn parse_hash_literal(&mut self) -> Result<Expression> {
        let mut pairs = Vec::new();

        while self.peek_token.kind != TokenKind::RBrace {
            self.next_token();
            let key = self.parse_expression(Precedence::Lowest)?;

            if self.peek_token.kind != TokenKind::Colon {
                return Err(miette::miette!("Expected Colon"));
            }
            self.next_token();
            self.next_token();

            let value = self.parse_expression(Precedence::Lowest)?;
            pairs.push((key, value));

            if self.peek_token.kind != TokenKind::RBrace && self.peek_token.kind != TokenKind::Comma {
                return Err(miette::miette!("Expected RBrace or Comma"))
            }

            if self.peek_token.kind == TokenKind::Comma {
                self.next_token();
            }
        }

        if self.peek_token.kind != TokenKind::RBrace {
            return Err(miette::miette!("Expected RBrace"))
        }

        self.next_token();

        Ok(Expression::HashLiteral(pairs))
    }
}

#[cfg(test)]
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
                token: Token::new(TokenKind::Let, 0, 2),
                name: "x".into(),
                value: Expression::IntegerLiteral(5),
            }
        );
        assert_eq!(
            program[1],
            Statement::Let {
                token: Token::new(TokenKind::Let, 11, 13),
                name: "y".into(),
                value: Expression::Boolean(true),
            }
        );
        assert_eq!(
            program[2],
            Statement::Let {
                token: Token::new(TokenKind::Let, 25, 27),
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
                token: Token::new(TokenKind::Return, 0, 5),
                value: Expression::IntegerLiteral(5),
            }
        );
        assert_eq!(
            program[1],
            Statement::Return {
                token: Token::new(TokenKind::Return, 10, 15),
                value: Expression::IntegerLiteral(10),
            }
        );
        assert_eq!(
            program[2],
            Statement::Return {
                token: Token::new(TokenKind::Return, 21, 26),
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
                token: Token::new(TokenKind::Bang, 0, 0),
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
                token: Token::new(TokenKind::Minus, 0, 0),
                operator: "-".into(),
                right: Box::new(Expression::IntegerLiteral(5)),
            })
        );

        let program = program_from_input("!true;");
        assert_eq!(program.len(), 1);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::Prefix {
                token: Token::new(TokenKind::Bang, 0, 0),
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
                token: Token::new(TokenKind::Plus, 2, 2),
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
                token: Token::new(TokenKind::Minus, 2, 2),
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
                token: Token::new(TokenKind::Asterisk, 2, 2),
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
                token: Token::new(TokenKind::Slash, 2, 2),
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
                token: Token::new(TokenKind::GreaterThan, 2, 2),
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
                token: Token::new(TokenKind::LessThan, 2, 2),
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
                token: Token::new(TokenKind::Equal, 2, 3),
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
                token: Token::new(TokenKind::NotEqual, 2, 3),
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
                token: Token::new(TokenKind::Equal, 5, 6),
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
                token: Token::new(TokenKind::NotEqual, 5, 6),
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
        // Indexing
        assert_eq!(
            program_from_input("a * [1, 2, 3, 4][b * c] * d").to_string(),
            "((a * ([1, 2, 3, 4][(b * c)])) * d)"
        );
        assert_eq!(
            program_from_input("add(a * b[2], b[1], 2 * [1, 2][1])").to_string(),
            "add((a * (b[2])), (b[1]), (2 * ([1, 2][1])))"
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
                    token: Token::new(TokenKind::LessThan, 6, 6),
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
                    token: Token::new(TokenKind::LessThan, 6, 6),
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
            token: Token::new(TokenKind::Plus, 13, 13),
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
                        token: Token::new(TokenKind::Asterisk, 9, 9),
                        operator: "*".to_string(),
                        left: Box::new(Expression::IntegerLiteral(2)),
                        right: Box::new(Expression::IntegerLiteral(3)),
                    },
                    Expression::Infix {
                        token: Token::new(TokenKind::Plus, 16, 16),
                        operator: "+".to_string(),
                        left: Box::new(Expression::IntegerLiteral(4)),
                        right: Box::new(Expression::IntegerLiteral(5)),
                    },
                ]
            })
        );
    }

    #[test]
    fn test_string_literal_expression() {
        let program = program_from_input(r#""hello world";"#);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::StringLiteral("hello world".into()))
        );
    }

    #[test]
    fn test_parsing_array_literals() {
        let program = program_from_input("[1, 2 * 2, 3 + 3]");
        assert_eq!(
            program[0],
            Statement::Expr(Expression::ArrayLiteral(vec![
                Expression::IntegerLiteral(1),
                Expression::Infix {
                    token: Token::new(TokenKind::Asterisk, 6, 6),
                    operator: "*".into(),
                    left: Box::new(Expression::IntegerLiteral(2)),
                    right: Box::new(Expression::IntegerLiteral(2)),
                },
                Expression::Infix {
                    token: Token::new(TokenKind::Plus, 13, 13),
                    operator: "+".into(),
                    left: Box::new(Expression::IntegerLiteral(3)),
                    right: Box::new(Expression::IntegerLiteral(3)),
                },
            ]))
        )
    }

    #[test]
    fn test_parsing_index_expressions() {
        let program = program_from_input("myArray[1 + 1]");
        assert_eq!(
            program[0],
            Statement::Expr(Expression::IndexExpr {
                left: Box::new(Expression::Ident(Identifier::new("myArray".into()))),
                index: Box::new(Expression::Infix {
                    token: Token::new(TokenKind::Plus, 10, 10),
                    operator: "+".into(),
                    left: Box::new(Expression::IntegerLiteral(1)),
                    right: Box::new(Expression::IntegerLiteral(1)),
                })
            })
        )
    }

    #[test]
    fn test_parsing_hash_literal_string_keys() {
        let program = program_from_input(r#"{"one": 1, "two": 2, "three": 3}"#);
        dbg!(&program);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::HashLiteral(vec![
                (
                    Expression::StringLiteral("one".into()),
                    Expression::IntegerLiteral(1)
                ),
                (
                    Expression::StringLiteral("two".into()),
                    Expression::IntegerLiteral(2)
                ),
                (
                    Expression::StringLiteral("three".into()),
                    Expression::IntegerLiteral(3)
                ),
            ]))
        );
    }

    #[test]
    fn test_parsing_emtpy_hash_literal() {
        let program = program_from_input(r#"{}"#);

        assert_eq!(program[0], Statement::Expr(Expression::HashLiteral(vec![])));
    }

    #[test]
    fn test_parsing_hash_literal_with_expressions() {
        let program = program_from_input(r#"{"one": 0 + 1, "two": 10 - 8, "three": 15 / 5}"#);
        assert_eq!(
            program[0],
            Statement::Expr(Expression::HashLiteral(vec![
                (
                    Expression::StringLiteral("one".into()),
                    Expression::Infix {
                        token: Token::new(TokenKind::Plus, 10, 10),
                        operator: "+".into(),
                        left: Box::new(Expression::IntegerLiteral(0)),
                        right: Box::new(Expression::IntegerLiteral(1)),
                    }
                ),
                (
                    Expression::StringLiteral("two".into()),
                    Expression::Infix {
                        token: Token::new(TokenKind::Minus, 25, 25),
                        operator: "-".into(),
                        left: Box::new(Expression::IntegerLiteral(10)),
                        right: Box::new(Expression::IntegerLiteral(8)),
                    }
                ),
                (
                    Expression::StringLiteral("three".into()),
                    Expression::Infix {
                        token: Token::new(TokenKind::Slash, 42, 42),
                        operator: "/".into(),
                        left: Box::new(Expression::IntegerLiteral(15)),
                        right: Box::new(Expression::IntegerLiteral(5)),
                    }
                ),
            ]))
        );
    }
}
