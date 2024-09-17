use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, start: usize, end: usize) -> Self {
        Self {
            kind,
            span: Span { start, end },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
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

impl TokenKind {
    pub fn lookup_ident(self) -> TokenKind {
        if let TokenKind::Ident(ident) = &self {
            match ident.as_str() {
                "fn" => TokenKind::Function,
                "let" => TokenKind::Let,
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                "if" => TokenKind::If,
                "else" => TokenKind::Else,
                "return" => TokenKind::Return,
                _ => self,
            }
        } else {
            self
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TokenKind::Illegal => write!(f, "Illegal"),
            TokenKind::Eof => write!(f, "Eof"),
            TokenKind::Ident(x) => write!(f, "{}", x),
            TokenKind::Int(x) => write!(f, "{}", x),
            TokenKind::Assign => write!(f, "="),
            TokenKind::Plus => write!(f, "+",),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::Asterisk => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::LessThan => write!(f, "<"),
            TokenKind::GreaterThan => write!(f, ">"),
            TokenKind::Equal => write!(f, "=="),
            TokenKind::NotEqual => write!(f, "!="),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Function => write!(f, "fn"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Return => write!(f, "return"),
        }
    }
}
