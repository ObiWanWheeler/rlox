use thiserror::Error;
use phf::phf_map;

macro_rules! token {
    ($self: ident, $tok_type: tt, $raw: expr) => {
        Token::new(TokenType::$tok_type, $raw.to_string(), $self.cursor_offset)
    };
}


static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
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



pub struct Lexer<'a> {
    source: std::iter::Peekable<std::str::Chars<'a>>,
    tokens: Vec<Result<Token, LexerError>>,
    cursor_offset: u32,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.chars().peekable(),
            tokens: vec![],
            cursor_offset: 0,
        }
    }

    fn match_next(&self, want: char) -> bool {
        if let Some(next) = self.source.peek() {
            return *next == want;
        }
        return false;
    }

    fn skip_comment(&mut self) {
        while !self.is_at_end() && *self.source.peek().unwrap() != '\n' {
            self.consume_char();
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() && self.source.peek().unwrap().is_whitespace() {
            self.consume_char();
        }
    }

    fn consume_char(&self) -> Option<char> {
        self.cursor_offset += 1;
        self.source.next()
    }

    fn parse_string(&mut self) -> Result<Token, LexerError> {
        //TODO add escape sequences, \n , \t etc.
        let buf = String::new();
        loop {
            match self.consume_char() {
                None => return Err(LexerError::UnclosedStringLiteral { literal: buf }),
                Some(c) if c == '"' => return Ok(token!(self, Strang, buf)),
                Some(c) => buf.push(c),
            }
        }
    }

    fn parse_num(&mut self, start: char) -> Result<Token, LexerError> {
        let mut buf = String::from(start);
        let mut seen_dp = false;

        loop {
            match self.source.peek() {
                None => return Ok(token!(self, Number, buf)),
                Some(c) if *c == '.' => {
                    if seen_dp {
                        // can't have two decimal points
                        // push for error
                        buf.push(*c);
                        return Err(LexerError::InvalidNumberLiteral {
                            literal: buf,
                            symbol: *c,
                        });
                    } else {
                        seen_dp = true;
                        buf.push(self.consume_char().unwrap());
                        // make sure . followed by a number
                        match self.source.peek() {
                            None => {
                                return Err(LexerError::InvalidNumberLiteral {
                                    literal: buf,
                                    symbol: '.',
                                })
                            }
                            Some(c) if !c.is_digit(10) => {
                                return Err(LexerError::InvalidNumberLiteral {
                                    literal: buf,
                                    symbol: '.',
                                })
                            }
                            Some(c) => buf.push(self.consume_char().unwrap()),
                        }
                    }
                }
                Some(c) if !c.is_digit(10) => {
                    return Err(LexerError::InvalidNumberLiteral {
                        literal: buf,
                        symbol: *c,
                    });
                }
                Some(c) if c.is_whitespace() => break,
                Some(c) => buf.push(self.consume_char().unwrap()),
            }
        }
        Ok(token!(self, Number, buf))
    }

    fn parse_identifier(&mut self, start: char) -> Result<Token, LexerError> {
        let mut buf = String::from(start);

        loop {
            match self.source.peek() {
                None => break,
                Some(c) if c.is_ascii_alphanumeric() => buf.push(self.consume_char().unwrap()),
                Some(c) => return Err(LexerError::InvalidIdentifier { identifier: buf, symbol: *c })
            }
        }

        // check if it's a keyword
        if let Some(token_type) = KEYWORDS.get(&buf).cloned() {
            // it is a keyword
            return Ok(Token::new(token_type, buf, self.cursor_offset));
        }
        else {
            // it's a plain ol' identifier
            return Ok(token!(self, Identifier, buf));
        }
    }

    fn lex_token(&mut self) {
        if let Some(c) = self.consume_char() {
            match c {
                '(' => self.tokens.push(Ok(token!(self, LeftParen, "("))),
                ')' => self.tokens.push(Ok(token!(self, RightParen, ")"))),
                '{' => self.tokens.push(Ok(token!(self, LeftBrace, "{"))),
                '}' => self.tokens.push(Ok(token!(self, RightBrace, "}"))),
                ',' => self.tokens.push(Ok(token!(self, Comma, ","))),

                '.' => self.tokens.push(Ok(token!(self, Dot, "."))),
                '-' => self.tokens.push(Ok(token!(self, Minus, "-"))),
                '+' => self.tokens.push(Ok(token!(self, Plus, "+"))),
                '*' => self.tokens.push(Ok(token!(self, Star, "*"))),
                ';' => self.tokens.push(Ok(token!(self, SemiColon, ";"))),
                '!' => {
                    if self.match_next('=') {
                        self.consume_char();
                        self.tokens.push(Ok(token!(self, BangEqual, "!=")));
                    } else {
                        self.tokens.push(Ok(token!(self, Bang, "!")));
                    }
                }
                '<' => {
                    if self.match_next('=') {
                        self.consume_char();
                        self.tokens.push(Ok(token!(self, LessEqual, "<=")));
                    } else {
                        self.tokens.push(Ok(token!(self, Less, "<")));
                    }
                }
                '>' => {
                    if self.match_next('=') {
                        self.consume_char();
                        self.tokens.push(Ok(token!(self, GreaterEqual, ">=")));
                    } else {
                        self.tokens.push(Ok(token!(self, Greater, ">")));
                    }
                }
                '=' => {
                    if self.match_next('=') {
                        self.consume_char();
                        self.tokens.push(Ok(token!(self, EqualEqual, "==")));
                    } else {
                        self.tokens.push(Ok(token!(self, Equal, "=")));
                    }
                }
                '/' => {
                    if self.match_next('/') {
                        // it's a comment, carry on till end of line
                        self.skip_comment();
                    } else {
                        self.tokens.push(Ok(token!(self, Slash, "/")));
                    }
                }
                '"' => self.tokens.push(self.parse_string()),
                c if c.is_whitespace() => self.skip_whitespace(),
                '0'..='9' => self.tokens.push(self.parse_num(c)),
                c if c.is_ascii_alphabetic() || c == '_' => self.tokens.push(self.parse_identifier(c)),

                _ => self
                    .tokens
                    .push(Err(LexerError::UnrecognisedSymbol { symbol: c })),
            }
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.source.peek() == None
    }

    // don't have to reference self, as lexer is effectively useless after this has been called
    // so we may take ownership
    pub fn collect_tokens(mut self) -> Vec<Result<Token, LexerError>> {
        while !self.is_at_end() {
            self.lex_token();
        }

        self.tokens.push(Ok(token!(self, EOF, "")));
        self.tokens
    }
}

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("unrecognised symbol {symbol}")]
    UnrecognisedSymbol { symbol: char },

    #[error("invalid string literal {literal}. Expected \" found end of file")]
    UnclosedStringLiteral { literal: String },

    #[error("invalid numeric literal {literal}. invalid symbol {symbol}")]
    InvalidNumberLiteral { literal: String, symbol: char },

    #[error("invalid identifier {identifier}. invalid symbol {symbol}")]
    InvalidIdentifier { identifier: String, symbol: char }
}

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

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    raw: String,
    cursor_offset: u32,
}

impl Token {
    pub fn new(token_type: TokenType, raw: String, cursor_offset: u32) -> Self {
        Self {
            token_type,
            raw,
            cursor_offset,
        }
    }
}
