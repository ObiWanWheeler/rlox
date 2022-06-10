use crate::{common::{*, self}, lexer_error, lox, token};
use thiserror::Error;

pub struct Lexer<'a> {
    source: std::iter::Peekable<std::str::Chars<'a>>,
    tokens: Vec<Token>,
    line: u32,
    column: u32,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.chars().peekable(),
            tokens: vec![],
            line: 1,
            column: 1,
        }
    }

    fn match_next(&mut self, want: char) -> bool {
        if let Some(next) = self.source.peek() {
            return *next == want;
        }
        return false;
    }

    fn skip_line_comment(&mut self) {
        while !self.is_at_end() && *self.source.peek().unwrap() != '\n' {
            self.consume_char();
        }
    }

    fn skip_block_comment(&mut self) {
        // cursor starts on  first *, consume that
        self.consume_char();

        loop {
            match self.consume_char() {
                None => break,
                Some(c) if c == '*' => {
                    match self.source.peek() {
                        None => {}
                        Some(c) if *c == '/' => {
                            // end of block comment
                            self.consume_char();
                            break;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() && self.source.peek().unwrap().is_whitespace() {
            self.consume_char();
        }
    }

    fn consume_char(&mut self) -> Option<char> {
        if let Some(c) = self.source.peek() {
            if *c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        self.source.next()
    }

    fn parse_string(&mut self) -> Result<Token, LexerError> {
        //TODO add escape sequences, \n , \t etc.
        let mut buf = String::new();
        loop {
            match self.consume_char() {
                None => {
                    return Err(self.error(LexerErrorKind::UnclosedStringLiteral { literal: buf }))
                }
                Some(c) if c == '"' => return Ok(token!(Strang, buf, (self.line, self.column))),
                Some(c) => buf.push(c),
            }
        }
    }

    fn parse_num(&mut self, start: char) -> Result<Token, LexerError> {
        let mut buf = String::from(start);
        let mut seen_dp = false;

        loop {
            match self.source.peek() {
                None => return Ok(token!(Number, buf, (self.line, self.column))),
                Some(c) if *c == '.' => {
                    if seen_dp {
                        // can't have two decimal points
                        // push for error
                        buf.push(*c);
                        return Err(self.error(LexerErrorKind::InvalidNumberLiteral {
                            literal: buf,
                            symbol: '.',
                        }));
                    } else {
                        seen_dp = true;
                        buf.push(self.consume_char().unwrap());
                        // make sure . followed by a number
                        match self.source.peek() {
                            None => {
                                return Err(self.error(LexerErrorKind::InvalidNumberLiteral {
                                    literal: buf,
                                    symbol: '.',
                                }));
                            }
                            Some(c) if !c.is_digit(10) => {
                                let err = self.error(LexerErrorKind::InvalidNumberLiteral {
                                    literal: buf,
                                    symbol: '.',
                                });
                                return Err(err);
                            }
                            Some(_) => buf.push(self.consume_char().unwrap()),
                        }
                    }
                }
                Some(c) if c.is_whitespace() || common::is_punctuation(c) => break,
                Some(c) if c.is_ascii_alphabetic() => {
                    let kind = LexerErrorKind::InvalidNumberLiteral {
                        literal: buf,
                        symbol: *c,
                    };
                    println!(
                        "line {} column {}: {}",
                        self.line,
                        self.column,
                        kind.to_string()
                    );
                    lox::report_error();
                    return Err(lexer_error!(kind, (self.line, self.column)));
                }
                Some(_) => buf.push(self.consume_char().unwrap()),
            }
        }
        Ok(token!(Number, buf, (self.line, self.column)))
    }

    fn parse_identifier(&mut self, start: char) -> Result<Token, LexerError> {
        let mut buf = String::from(start);

        loop {
            match self.source.peek() {
                None => break,
                Some(c) if c.is_ascii_alphanumeric() || *c == '_' => buf.push(self.consume_char().unwrap()),
                Some(_) => break,
            }
        }

        // check if it's a keyword
        // it is a keyword
        if let Some(token_type) = KEYWORDS.get(&buf).cloned() {
            return Ok(Token {
                token_type,
                raw: buf,
                line: self.line,
                column: self.column,
            });
        } else {
            // it's a plain ol' identifier
            return Ok(token!(Identifier, buf, (self.line, self.column)));
        }
    }

    fn lex_token(&mut self) {
        if let Some(c) = self.consume_char() {
            match c {
                '(' => self
                    .tokens
                    .push(token!(LeftParen, "(", (self.line, self.column))),
                ')' => self
                    .tokens
                    .push(token!(RightParen, ")", (self.line, self.column))),
                '{' => self
                    .tokens
                    .push(token!(LeftBrace, "{", (self.line, self.column))),
                '}' => self
                    .tokens
                    .push(token!(RightBrace, "}", (self.line, self.column))),
                ',' => self
                    .tokens
                    .push(token!(Comma, ",", (self.line, self.column))),
                '.' => self.tokens.push(token!(Dot, ".", (self.line, self.column))),
                '-' => self
                    .tokens
                    .push(token!(Minus, "-", (self.line, self.column))),
                '+' => self
                    .tokens
                    .push(token!(Plus, "+", (self.line, self.column))),
                '*' => self
                    .tokens
                    .push(token!(Star, "*", (self.line, self.column))),
                ';' => self
                    .tokens
                    .push(token!(SemiColon, ";", (self.line, self.column))),
                '!' => {
                    if self.match_next('=') {
                        self.consume_char();
                        self.tokens
                            .push(token!(BangEqual, "!=", (self.line, self.column)));
                    } else {
                        self.tokens
                            .push(token!(Bang, "!", (self.line, self.column)));
                    }
                }
                '<' => {
                    if self.match_next('=') {
                        self.consume_char();
                        self.tokens
                            .push(token!(LessEqual, "<=", (self.line, self.column)));
                    } else {
                        self.tokens
                            .push(token!(Less, "<", (self.line, self.column)));
                    }
                }
                '>' => {
                    if self.match_next('=') {
                        self.consume_char();
                        self.tokens
                            .push(token!(GreaterEqual, ">=", (self.line, self.column)));
                    } else {
                        self.tokens
                            .push(token!(Greater, ">", (self.line, self.column)));
                    }
                }
                '=' => {
                    if self.match_next('=') {
                        self.consume_char();
                        self.tokens
                            .push(token!(EqualEqual, "==", (self.line, self.column)));
                    } else {
                        self.tokens
                            .push(token!(Equal, "=", (self.line, self.column)));
                    }
                }
                '/' => {
                    if self.match_next('/') {
                        // it's a comment, carry on till end of line
                        self.skip_line_comment();
                    } else if self.match_next('*') {
                        // it's a line comment
                        self.skip_block_comment();
                    } else {
                        self.tokens
                            .push(token!(Slash, "/", (self.line, self.column)));
                    }
                }
                '"' => {
                    let string_tok = self.parse_string();
                    match string_tok {
                        Ok(tok) => self.tokens.push(tok),
                        Err(e) => {
                            self.error(e.kind);
                        }
                    }
                }
                c if c.is_whitespace() => self.skip_whitespace(),
                '0'..='9' => {
                    let num_tok = self.parse_num(c);
                    match num_tok {
                        Ok(tok) => self.tokens.push(tok),
                        Err(e) => {
                            self.error(e.kind);
                        }
                    }
                }
                c if c.is_ascii_alphabetic() || c == '_' => {
                    let ident_tok = self.parse_identifier(c);
                    match ident_tok {
                        Ok(tok) => self.tokens.push(tok),
                        Err(e) => {
                            self.error(e.kind);
                        }
                    }
                }

                _ => {
                    self.error(LexerErrorKind::UnrecognisedSymbol { symbol: c });
                }
            }
        }
    }

    pub fn is_at_end(&mut self) -> bool {
        self.source.peek() == None
    }

    fn error(&self, kind: LexerErrorKind) -> LexerError {
        println!(
            "lexer: line {} column {}: {}",
            self.line,
            self.column,
            kind.to_string()
        );
        lox::report_error();
        lexer_error!(kind, (self.line, self.column))
    }

    // don't have to reference self, as lexer is effectively useless after this has been called
    // so we may take ownership
    pub fn collect_tokens(mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.lex_token();
        }

        self.tokens.push(token!(EOF, "", (self.line, self.column)));

        self.tokens
    }
}

#[derive(Debug)]
pub struct LexerError {
    pub kind: LexerErrorKind,
    pub line: u32,
    pub column: u32,
}

#[derive(Error, Debug)]
pub enum LexerErrorKind {
    #[error("unrecognised symbol {symbol}")]
    UnrecognisedSymbol { symbol: char },

    #[error("invalid string literal {literal}. Expected \" found end of file")]
    UnclosedStringLiteral { literal: String },

    #[error("invalid numeric literal {literal}. invalid symbol {symbol}")]
    InvalidNumberLiteral { literal: String, symbol: char },
}
