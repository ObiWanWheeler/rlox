use std::vec::IntoIter;

use crate::{
    common::{Token, TokenType},
    expr::{Expr, LiteralType},
    lox, token,
};

pub struct Parser {
    tokens: std::iter::Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    fn consume_token(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_next_token(&vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.consume_token().unwrap();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while self.match_next_token(&vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.consume_token().unwrap();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.match_next_token(&vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.consume_token().unwrap();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.match_next_token(&vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.consume_token().unwrap();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_next_token(&vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.consume_token().unwrap();
            Ok(Expr::Unary {
                operator,
                right: Box::new(self.unary()?),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        match self.consume_token().unwrap() {
            Token {
                token_type: TokenType::False,
                ..
            } => Ok(Expr::Literal {
                value: LiteralType::Bool(false),
            }),
            Token {
                token_type: TokenType::True,
                ..
            } => Ok(Expr::Literal {
                value: LiteralType::Bool(true),
            }),
            Token {
                token_type: TokenType::Number,
                raw,
                ..
            } => Ok(Expr::Literal {
                value: LiteralType::Number(raw.parse::<f32>().unwrap()),
            }),
            Token {
                token_type: TokenType::LeftParen,
                ..
            } => {
                let expr = self.expression()?;
                self.require_consume(TokenType::RightParen, "Expect ')'")?;

                Ok(Expr::Grouping {
                    expression: Box::new(expr),
                })
            }
            t => Err(self.error(&t, "Expected expression")),
        }
    }

    fn match_next_token(&mut self, types: &[TokenType]) -> bool {
        match self.tokens.peek() {
            None => false,
            Some(t) => types.contains(&t.token_type),
        }
    }

    fn require_consume(
        &mut self,
        required: TokenType,
        error_message: &str,
    ) -> Result<(), ParseError> {
        match self.consume_token() {
            Some(t) if t.token_type == required => Ok(()),
            Some(t) => Err(self.error(&t, error_message)),
            None => Err(self.error(&token!(EOF, "", (0, 0)), error_message)),
        }
    }

    fn error(&self, token: &Token, message: &str) -> ParseError {
        println!(
            "parser: {} at line {} column {}",
            message, token.line, token.column
        );
        lox::report_error();
        ParseError
    }

    fn synchronize(&mut self) {
        while !self.match_next_token(&vec![
            TokenType::SemiColon,
            TokenType::Class,
            TokenType::Funct,
            TokenType::Var,
            TokenType::For,
            TokenType::If,
            TokenType::While,
            TokenType::Print,
            TokenType::Return,
        ]) {
            self.consume_token();
        }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        match self.expression() {
            Err(_) => None,
            Ok(expr) => Some(expr),
        }
    }
}

struct ParseError;
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error")
    }
}