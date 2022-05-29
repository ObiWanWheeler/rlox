use phf::phf_map;

#[macro_export]
macro_rules! token {
    ($tok_type: tt, $raw: expr, ($line: expr, $column: expr)) => {
        Token {
            token_type: TokenType::$tok_type,
            raw: $raw.to_string(),
            line: $line,
            column: $column,
        }
    };
}

#[macro_export]
macro_rules! lexer_error {
    ($err_kind: expr, ($line: expr, $column: expr)) => {
        LexerError {
            kind: $err_kind,
            line: $line,
            column: $column,
        }
    };
}

pub static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
   "and" => TokenType::And,
   "class" => TokenType::Class,
   "else" => TokenType::Else,
   "false" => TokenType::False,
   "funct" => TokenType::Funct,
   "for" => TokenType::For,
   "if" => TokenType::If,
   "nil" => TokenType::Nil,
   "or" => TokenType::Or,
   "print" => TokenType::Print,
   "return" => TokenType::Return,
   "super" => TokenType::Super,
   "this" => TokenType::This,
   "true" => TokenType::True,
   "var" => TokenType::Var,
   "while" => TokenType::While,
};

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    SemiColon,

    // operators
    Minus,
    Plus,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // literals
    Identifier,
    Strang,
    Number,

    // keywords
    And,
    Class,
    Else,
    False,
    Funct,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub raw: String,
    pub line: u32,
    pub column: u32,
}