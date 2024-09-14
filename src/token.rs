#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Illegal,
    Eof,

    Ident(String),
    Int(String),
    Assign,
    Plus,

    Comma,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,

    Function,
    Let,
}

impl Token {
    pub fn lookup_ident(self) -> Token {
        if let Token::Ident(ident) = &self {
            match ident.as_str() {
                "fn" => Token::Function,
                "let" => Token::Let,
                _ => self,
            }
        } else {
            self
        }
    }
}
