#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Illegal,
    Eof,

    Ident(String),
    Int(String),
    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,

    LessThan,
    GreaterThan,
    Equal,
    NotEqual,

    Comma,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,

    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
}

impl Token {
    pub fn lookup_ident(self) -> Token {
        if let Token::Ident(ident) = &self {
            match ident.as_str() {
                "fn" => Token::Function,
                "let" => Token::Let,
                "true" => Token::True,
                "false" => Token::False,
                "if" => Token::If,
                "else" => Token::Else,
                "return" => Token::Return,
                _ => self,
            }
        } else {
            self
        }
    }
}
