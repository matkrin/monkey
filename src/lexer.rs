use crate::token::Token;

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

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.ch {
            Some('=') => Token::Assign,
            Some(';') => Token::Semicolon,
            Some('(') => Token::LParen,
            Some(')') => Token::RParen,
            Some(',') => Token::Comma,
            Some('+') => Token::Plus,
            Some('{') => Token::LBrace,
            Some('}') => Token::RBrace,
            Some(c) if is_letter(c) => return Token::Ident(self.read_identfier()).lookup_ident(),
            Some(c) if is_digit(c) => return Token::Int(self.read_number()),
            Some(_) => Token::Illegal,
            None => Token::Eof,
        };

        self.read_char();

        token
    }

    fn read_identfier(&mut self) -> String {
        let current_position = self.position;
        while self.ch.is_some_and(is_letter) {
            self.read_char();
        }
        self.input[current_position..self.position].to_string()
    }

    fn read_number(&mut self) -> String {
        let current_position = self.position;
        while self.ch.is_some_and(is_digit) {
            self.read_char();
        }
        self.input[current_position..self.position].to_string()
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
";

        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::Ident("five".into()));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Int("5".into()));
        assert_eq!(lexer.next_token(), Token::Semicolon);

        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::Ident("ten".into()));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Int("10".into()));
        assert_eq!(lexer.next_token(), Token::Semicolon);

        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::Ident("add".into()));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Function);
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::Ident("x".into()));
        assert_eq!(lexer.next_token(), Token::Comma);
        assert_eq!(lexer.next_token(), Token::Ident("y".into()));
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::LBrace);
        assert_eq!(lexer.next_token(), Token::Ident("x".into()));
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Ident("y".into()));
        assert_eq!(lexer.next_token(), Token::Semicolon);
        assert_eq!(lexer.next_token(), Token::RBrace);
        assert_eq!(lexer.next_token(), Token::Semicolon);

        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::Ident("result".into()));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Ident("add".into()));
        assert_eq!(lexer.next_token(), Token::LParen);
        assert_eq!(lexer.next_token(), Token::Ident("five".into()));
        assert_eq!(lexer.next_token(), Token::Comma);
        assert_eq!(lexer.next_token(), Token::Ident("ten".into()));
        assert_eq!(lexer.next_token(), Token::RParen);
        assert_eq!(lexer.next_token(), Token::Semicolon);
        assert_eq!(lexer.next_token(), Token::Eof);
    }
}
