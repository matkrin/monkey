use crate::token::{Span, Token, TokenKind};

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    read_position: usize,
    ch: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Self {
            input,
            position: 0,
            read_position: 0,
            ch: None,
        };
        lexer.read_char();
        lexer
    }

    pub fn source_code(&self) -> &str {
        self.input
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_some_and(|c| c.is_ascii_whitespace()) {
            self.read_char();
        }
    }

    fn read_char(&mut self) {
        let input_len = self.input.chars().count();
        if self.read_position >= input_len {
            self.ch = None;
        } else {
            self.ch = self.input.chars().nth(self.read_position);
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.read_position)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.ch {
            Some('=') if self.peek_char() == Some('=') => {
                let start = self.position;
                self.read_char();
                let end = self.position;
                Token::new(TokenKind::Equal, start, end)
            }
            Some('=') => Token::new(TokenKind::Assign, self.position, self.position),
            Some('+') => Token::new(TokenKind::Plus, self.position, self.position),
            Some('-') => Token::new(TokenKind::Minus, self.position, self.position),
            Some('!') if self.peek_char() == Some('=') => {
                let start = self.position;
                self.read_char();
                let end = self.position;
                Token::new(TokenKind::NotEqual, start, end)
            }
            Some('!') => Token::new(TokenKind::Bang, self.position, self.position),
            Some('/') => Token::new(TokenKind::Slash, self.position, self.position),
            Some('*') => Token::new(TokenKind::Asterisk, self.position, self.position),
            Some('<') => Token::new(TokenKind::LessThan, self.position, self.position),
            Some('>') => Token::new(TokenKind::GreaterThan, self.position, self.position),
            Some(';') => Token::new(TokenKind::Semicolon, self.position, self.position),
            Some(',') => Token::new(TokenKind::Comma, self.position, self.position),
            Some('(') => Token::new(TokenKind::LParen, self.position, self.position),
            Some(')') => Token::new(TokenKind::RParen, self.position, self.position),
            Some('{') => Token::new(TokenKind::LBrace, self.position, self.position),
            Some('}') => Token::new(TokenKind::RBrace, self.position, self.position),
            Some(c) if is_letter(c) => {
                let (ident, span) = self.read_identfier();
                let token_kind = TokenKind::Ident(ident).lookup_ident();
                return Token::new(token_kind, span.start, span.end);
            }
            Some(c) if is_digit(c) => {
                let (number, span) = self.read_number();
                let token_kind = TokenKind::Int(number);
                return Token {
                    kind: token_kind,
                    span,
                };
            }
            Some(_) => Token::new(TokenKind::Illegal, self.position, self.position),
            None => Token::new(TokenKind::Eof, self.position, self.position),
        };

        self.read_char();

        token
    }

    fn read_identfier(&mut self) -> (String, Span) {
        let current_position = self.position;
        while self.ch.is_some_and(is_letter) {
            self.read_char();
        }
        (
            self.input[current_position..self.position].to_string(),
            Span {
                start: current_position,
                end: self.position - 1,
            },
        )
    }

    fn read_number(&mut self) -> (String, Span) {
        let current_position = self.position;
        while self.ch.is_some_and(is_digit) {
            self.read_char();
        }
        (
            self.input[current_position..self.position].to_string(),
            Span {
                start: current_position,
                end: self.position - 1,
            },
        )
    }
}

fn is_letter(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

fn is_digit(character: char) -> bool {
    character.is_ascii_digit()
}

mod tests {
    use super::*;

    #[test]
    fn test_next_token() {
        let input = "let five = 5;
let ten = 10;

let add = fn(x, y) {
  x + y;
};

let result = add(five, ten);
!-/*5;
5 < 10 > 5;

if (5 < 10) {
    return true;
} else {
    return false;
}

10 == 10;
10 != 9;
";

        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Let, 0, 2));
        //assert_eq!(lexer.next_token(), TokenKind::Ident("five".into()));
        //assert_eq!(lexer.next_token(), TokenKind::Assign);
        //assert_eq!(lexer.next_token(), TokenKind::Int("5".into()));
        //assert_eq!(lexer.next_token(), TokenKind::Semicolon);
        //
        //assert_eq!(lexer.next_token(), TokenKind::Let);
        //assert_eq!(lexer.next_token(), TokenKind::Ident("ten".into()));
        //assert_eq!(lexer.next_token(), TokenKind::Assign);
        //assert_eq!(lexer.next_token(), TokenKind::Int("10".into()));
        //assert_eq!(lexer.next_token(), TokenKind::Semicolon);
        //
        //assert_eq!(lexer.next_token(), TokenKind::Let);
        //assert_eq!(lexer.next_token(), TokenKind::Ident("add".into()));
        //assert_eq!(lexer.next_token(), TokenKind::Assign);
        //assert_eq!(lexer.next_token(), TokenKind::Function);
        //assert_eq!(lexer.next_token(), TokenKind::LParen);
        //assert_eq!(lexer.next_token(), TokenKind::Ident("x".into()));
        //assert_eq!(lexer.next_token(), TokenKind::Comma);
        //assert_eq!(lexer.next_token(), TokenKind::Ident("y".into()));
        //assert_eq!(lexer.next_token(), TokenKind::RParen);
        //assert_eq!(lexer.next_token(), TokenKind::LBrace);
        //assert_eq!(lexer.next_token(), TokenKind::Ident("x".into()));
        //assert_eq!(lexer.next_token(), TokenKind::Plus);
        //assert_eq!(lexer.next_token(), TokenKind::Ident("y".into()));
        //assert_eq!(lexer.next_token(), TokenKind::Semicolon);
        //assert_eq!(lexer.next_token(), TokenKind::RBrace);
        //assert_eq!(lexer.next_token(), TokenKind::Semicolon);
        //
        //assert_eq!(lexer.next_token(), TokenKind::Let);
        //assert_eq!(lexer.next_token(), TokenKind::Ident("result".into()));
        //assert_eq!(lexer.next_token(), TokenKind::Assign);
        //assert_eq!(lexer.next_token(), TokenKind::Ident("add".into()));
        //assert_eq!(lexer.next_token(), TokenKind::LParen);
        //assert_eq!(lexer.next_token(), TokenKind::Ident("five".into()));
        //assert_eq!(lexer.next_token(), TokenKind::Comma);
        //assert_eq!(lexer.next_token(), TokenKind::Ident("ten".into()));
        //assert_eq!(lexer.next_token(), TokenKind::RParen);
        //assert_eq!(lexer.next_token(), TokenKind::Semicolon);
        //
        //assert_eq!(lexer.next_token(), TokenKind::Bang);
        //assert_eq!(lexer.next_token(), TokenKind::Minus);
        //assert_eq!(lexer.next_token(), TokenKind::Slash);
        //assert_eq!(lexer.next_token(), TokenKind::Asterisk);
        //assert_eq!(lexer.next_token(), TokenKind::Int("5".into()));
        //assert_eq!(lexer.next_token(), TokenKind::Semicolon);
        //
        //assert_eq!(lexer.next_token(), TokenKind::Int("5".into()));
        //assert_eq!(lexer.next_token(), TokenKind::LessThan);
        //assert_eq!(lexer.next_token(), TokenKind::Int("10".into()));
        //assert_eq!(lexer.next_token(), TokenKind::GreaterThan);
        //assert_eq!(lexer.next_token(), TokenKind::Int("5".into()));
        //assert_eq!(lexer.next_token(), TokenKind::Semicolon);
        ////if (5 < 10) {
        ////    return true;
        ////} else {
        ////    return false;
        ////}
        //assert_eq!(lexer.next_token(), TokenKind::If);
        //assert_eq!(lexer.next_token(), TokenKind::LParen);
        //assert_eq!(lexer.next_token(), TokenKind::Int("5".into()));
        //assert_eq!(lexer.next_token(), TokenKind::LessThan);
        //assert_eq!(lexer.next_token(), TokenKind::Int("10".into()));
        //assert_eq!(lexer.next_token(), TokenKind::RParen);
        //assert_eq!(lexer.next_token(), TokenKind::LBrace);
        //assert_eq!(lexer.next_token(), TokenKind::Return);
        //assert_eq!(lexer.next_token(), TokenKind::True);
        //assert_eq!(lexer.next_token(), TokenKind::Semicolon);
        //assert_eq!(lexer.next_token(), TokenKind::RBrace);
        //assert_eq!(lexer.next_token(), TokenKind::Else);
        //assert_eq!(lexer.next_token(), TokenKind::LBrace);
        //assert_eq!(lexer.next_token(), TokenKind::Return);
        //assert_eq!(lexer.next_token(), TokenKind::False);
        //assert_eq!(lexer.next_token(), TokenKind::Semicolon);
        //assert_eq!(lexer.next_token(), TokenKind::RBrace);
        //
        ////10 == 10;
        ////10 != 9;
        //
        //assert_eq!(lexer.next_token(), TokenKind::Int("10".into()));
        //assert_eq!(lexer.next_token(), TokenKind::Equal);
        //assert_eq!(lexer.next_token(), TokenKind::Int("10".into()));
        //assert_eq!(lexer.next_token(), TokenKind::Semicolon);
        //assert_eq!(lexer.next_token(), TokenKind::Int("10".into()));
        //assert_eq!(lexer.next_token(), TokenKind::NotEqual);
        //assert_eq!(lexer.next_token(), TokenKind::Int("9".into()));
        //assert_eq!(lexer.next_token(), TokenKind::Semicolon);
        //
        //assert_eq!(lexer.next_token(), TokenKind::Eof);
    }
}
