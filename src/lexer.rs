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
            Some('[') => Token::new(TokenKind::LBracket, self.position, self.position),
            Some(']') => Token::new(TokenKind::RBracket, self.position, self.position),
            Some('"') => {
                let (literal, span) = self.read_string();
                let token_kind = TokenKind::String(literal);
                Token::new(token_kind, span.start, span.end)
            }
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

    fn read_string(&mut self) -> (String, Span) {
        let current_position = self.position + 1;
        loop {
            self.read_char();
            if self.ch.is_some_and(|c| c == '"') {
                break;
            }
        }
        (
            self.input[current_position..self.position].to_string(),
            Span {
                start: current_position - 1,
                end: self.position,
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
        let input = r#"let five = 5;
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
"foobar"
"foo bar"
[1, 2];
"#;

        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Let, 0, 2));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Ident("five".into()), 4, 7)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Assign, 9, 9));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Int("5".into()), 11, 11)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Semicolon, 12, 12));

        assert_eq!(lexer.next_token(), Token::new(TokenKind::Let, 14, 16));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Ident("ten".into()), 18, 20)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Assign, 22, 22));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Int("10".into()), 24, 25)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Semicolon, 26, 26));

        assert_eq!(lexer.next_token(), Token::new(TokenKind::Let, 29, 31));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Ident("add".into()), 33, 35)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Assign, 37, 37));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Function, 39, 40));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::LParen, 41, 41));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Ident("x".into()), 42, 42)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Comma, 43, 43));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Ident("y".into()), 45, 45)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::RParen, 46, 46));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::LBrace, 48, 48));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Ident("x".into()), 52, 52)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Plus, 54, 54));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Ident("y".into()), 56, 56)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Semicolon, 57, 57));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::RBrace, 59, 59));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Semicolon, 60, 60));

        // let result = add(five, ten);
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Let, 63, 65));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Ident("result".into()), 67, 72)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Assign, 74, 74));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Ident("add".into()), 76, 78)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::LParen, 79, 79));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Ident("five".into()), 80, 83)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Comma, 84, 84));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Ident("ten".into()), 86, 88)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::RParen, 89, 89));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Semicolon, 90, 90));
        // !-/*5;
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Bang, 92, 92));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Minus, 93, 93));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Slash, 94, 94));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Asterisk, 95, 95));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Int("5".into()), 96, 96)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Semicolon, 97, 97));
        // 5 < 10 > 5;
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Int("5".into()), 99, 99)
        );
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::LessThan, 101, 101)
        );
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Int("10".into()), 103, 104)
        );
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::GreaterThan, 106, 106)
        );
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Int("5".into()), 108, 108)
        );
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Semicolon, 109, 109)
        );
        // if (5 < 10) {
        //     return true;
        // } else {
        //     return false;
        // }
        assert_eq!(lexer.next_token(), Token::new(TokenKind::If, 112, 113));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::LParen, 115, 115));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Int("5".into()), 116, 116)
        );
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::LessThan, 118, 118)
        );
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Int("10".into()), 120, 121)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::RParen, 122, 122));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::LBrace, 124, 124));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Return, 130, 135));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::True, 137, 140));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Semicolon, 141, 141)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::RBrace, 143, 143));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Else, 145, 148));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::LBrace, 150, 150));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Return, 156, 161));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::False, 163, 167));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Semicolon, 168, 168)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::RBrace, 170, 170));
        //
        // 10 == 10;
        // 10 != 9;
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Int("10".into()), 173, 174)
        );
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Equal, 176, 177));
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Int("10".into()), 179, 180)
        );
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Semicolon, 181, 181)
        );
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Int("10".into()), 183, 184)
        );
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::NotEqual, 186, 187)
        );
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Int("9".into()), 189, 189)
        );
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::Semicolon, 190, 190)
        );
        // "foobar"
        // "foo bar"
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::String("foobar".into()), 192, 199)
        );
        assert_eq!(
            lexer.next_token(),
            Token::new(TokenKind::String("foo bar".into()), 201, 209)
        );
        // [1, 2];
        assert_eq!(lexer.next_token(), Token::new(TokenKind::LBracket, 211, 211));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Int("1".into()), 212, 212));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Comma, 213, 213));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Int("2".into()), 215, 215));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::RBracket, 216, 216));
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Semicolon, 217, 217));
        //
        assert_eq!(lexer.next_token(), Token::new(TokenKind::Eof, 219, 219));
    }
}
