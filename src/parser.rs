use crate::{
    ast::{Program, Statement},
    lexer::Lexer,
    token::Token,
};
use miette::Result;

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
            t => miette::bail!("Cannot parse token of type {} yet", t),
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
        })
    }

    fn parse_return_statement(&mut self) -> Result<Statement> {
        let current_token = self.current_token.clone();
        self.next_token();

        // skip until semicolon for now
        while self.current_token != Token::Semicolon {
            self.next_token()
        }

        Ok(Statement::Return { token: current_token, value: todo!() })
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_let_statement() {
        let input = "let x = 5;
let y = 10;
let foobar = 838383;
";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(program.len(), 3);
        assert_eq!(program[0], Statement::Let { token: Token::Let, name: "x".into() });
        assert_eq!(program[1], Statement::Let { token: Token::Let, name: "y".into() });
        assert_eq!(program[2], Statement::Let { token: Token::Let, name: "foobar".into() });
    }

    #[test]
    fn test_return_statement() {
        let input = "return 5;
return 10;
return 993322;
";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(program.len(), 3);
        assert_eq!(program[0], Statement::Return { token: Token::Return, value: todo!() });
        assert_eq!(program[0], Statement::Return { token: Token::Return, value: todo!() });
        assert_eq!(program[0], Statement::Return { token: Token::Return, value: todo!() });
    }
}
