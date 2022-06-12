use std::vec::IntoIter;

use crate::{
    common::{LoxType, Token, TokenType, LOX_MAX_ARGUMENT_COUNT},
    expr::Expr,
    lox,
    stmt::Stmt,
    token,
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

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.match_next_token(&[TokenType::Var]) {
            self.var_declaration()
        } else if self.match_next_token(&[TokenType::Funct]) {
            self.function_declaration()
        } else if self.match_next_token(&[TokenType::Class]) {
            self.class_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        // consume var token
        self.consume_token();
        let name = self.require_consume(TokenType::Identifier, "Expected variable name")?;
        let mut initializer = None;
        if self.match_next_token(&[TokenType::Equal]) {
            // consume = token
            self.consume_token();
            initializer = Some(self.expression()?);
        }

        self.require_consume(
            TokenType::SemiColon,
            "Expect ';' after variable declaration",
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    fn function_declaration(&mut self) -> Result<Stmt, ParseError> {
        // consume funct token
        self.consume_token();
        self.function()
    }

    fn function(&mut self) -> Result<Stmt, ParseError> {
        let name = self.require_consume(
            TokenType::Identifier,
            "Expect 'funct' keyword be followed by function name",
        )?;

        self.require_consume(TokenType::LeftParen, "Expect '(' after function name")?;

        let mut parameters = vec![];
        while !self.match_next_token(&[TokenType::RightParen, TokenType::EOF]) {
            // still have args
            parameters.push(self.consume_token().unwrap());
            if parameters.len() > LOX_MAX_ARGUMENT_COUNT {
                let next_tok = self.consume_token().unwrap();
                self.error(&next_tok, "Exceeded max parameter count");
            }
            if self.match_next_token(&[TokenType::RightParen]) {
                break;
            }
            self.require_consume(TokenType::Comma, "Expect parameters are comma seperated")?;
        }

        self.require_consume(
            TokenType::RightParen,
            "Expect function parameter list to be closed with ')'",
        )?;

        Ok(Stmt::Function {
            name,
            parameters,
            body: self.block()?,
        })
    }

    fn class_declaration(&mut self) -> Result<Stmt, ParseError> {
        // consume class token
        self.consume_token();

        let name = self.require_consume(
            TokenType::Identifier,
            "Expect class name after 'class' keyword",
        )?;
        self.require_consume(TokenType::LeftBrace, "Expect '{' to open class body")?;

        let mut methods = vec![];
        while self.match_next_token(&[TokenType::Meth]) {
            // more methods to come
            // consume meth token
            self.consume_token();
            methods.push(self.function()?);
        }

        self.require_consume(TokenType::RightBrace, "Expect '}' to close class body")?;

        Ok(Stmt::Class {
            name,
            methods: Box::new(methods),
        })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_next_token(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_next_token(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_next_token(&[TokenType::For]) {
            self.for_statement()
        } else if self.match_next_token(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_next_token(&[TokenType::Break]) {
            self.break_statement()
        } else if self.match_next_token(&[TokenType::Return]) {
            self.return_statement()
        } else if self.match_next_token(&[TokenType::LeftBrace]) {
            Ok(Stmt::Block {
                statements: self.block()?,
            })
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        // consume the if token
        self.consume_token();
        self.require_consume(TokenType::LeftParen, "Expect '(' to open 'if' condition")?;
        let condition = self.expression()?;
        self.require_consume(TokenType::RightParen, "Expect ')' to close 'if' condition")?;
        let then_branch = Box::new(self.statement()?);
        let mut else_branch = None;
        if self.match_next_token(&[TokenType::Else]) {
            // consume the else token
            self.consume_token();
            else_branch = Some(Box::new(self.statement()?));
        }
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        // consume the while token
        self.consume_token();
        self.require_consume(TokenType::LeftParen, "Expect '(' to open 'while' condition")?;
        let condition = self.expression()?;
        self.require_consume(
            TokenType::RightParen,
            "Expect ')' to close 'while' condition",
        )?;
        let then_branch = Box::new(self.statement()?);
        let mut finally_branch = None;
        if self.match_next_token(&[TokenType::Finally]) {
            // consume the finally token
            self.consume_token();
            finally_branch = Some(Box::new(self.statement()?));
        }
        Ok(Stmt::While {
            condition,
            then_branch,
            finally_branch,
        })
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        // consume the for token
        self.consume_token();
        self.require_consume(TokenType::LeftParen, "Expect '(' to open 'for' clause")?;

        let initializer;
        if self.match_next_token(&[TokenType::SemiColon]) {
            initializer = None;
        } else if self.match_next_token(&[TokenType::Var]) {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expression_statement()?);
        }

        let mut condition = None;
        if !self.match_next_token(&[TokenType::SemiColon]) {
            condition = Some(self.expression()?);
        }
        self.require_consume(
            TokenType::SemiColon,
            "Expect ';' after 'for' loop condition",
        )?;

        let mut increment = None;
        if !self.match_next_token(&[TokenType::RightParen]) {
            increment = Some(self.expression()?);
        }

        self.require_consume(TokenType::RightParen, "Expect ')' to close 'for' clause")?;

        let mut body = self.statement()?;

        if increment.is_some() {
            body = Stmt::Block {
                statements: Box::new(vec![
                    body,
                    Stmt::Expression {
                        expression: increment.unwrap(),
                    },
                ]),
            };
        }

        if condition.is_some() {
            body = Stmt::While {
                condition: condition.unwrap(),
                then_branch: Box::new(body),
                finally_branch: None,
            };
        }

        if initializer.is_some() {
            body = Stmt::Block {
                statements: Box::new(vec![initializer.unwrap(), body]),
            };
        }

        Ok(body)
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        // consume print token
        self.consume_token();
        let value = self.expression()?;
        self.require_consume(TokenType::SemiColon, "Expect ';' after value")?;
        Ok(Stmt::Print { expression: value })
    }

    fn break_statement(&mut self) -> Result<Stmt, ParseError> {
        let break_ = self.require_consume(TokenType::Break, "Expect 'break'")?;
        self.require_consume(TokenType::SemiColon, "Expect ';' after break")?;
        Ok(Stmt::Break { token: break_ })
    }

    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let return_ = self.require_consume(TokenType::Return, "Expect 'return'")?;
        let mut return_value = None;
        if !self.match_next_token(&[TokenType::SemiColon]) {
            // not void
            return_value = Some(self.expression()?);
        }
        self.require_consume(TokenType::SemiColon, "Expect ';' after return statement")?;
        Ok(Stmt::Return {
            token: return_,
            return_value,
        })
    }
    fn block(&mut self) -> Result<Box<Vec<Stmt>>, ParseError> {
        // consume { token
        self.consume_token();

        let mut statements = vec![];

        while !self.match_next_token(&[TokenType::RightBrace]) && !self.is_done() {
            statements.push(self.declaration()?);
        }

        // only errors if the self.is_done() causes the loop to terminate, i.e unclosed brace
        self.require_consume(TokenType::RightBrace, "Expect '}' to close a block")?;
        Ok(Box::new(statements))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expression = self.expression()?;
        self.require_consume(TokenType::SemiColon, "Expect ';' after expression")?;
        Ok(Stmt::Expression { expression })
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.or()?;

        if self.match_next_token(&[TokenType::Equal]) {
            let equals = self.consume_token().unwrap();
            let value = self.assignment()?;

            if let Expr::Variable { name } = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            } else if let Expr::Get { object, name } = expr {
                return Ok(Expr::Set {
                    object,
                    name,
                    value: Box::new(value),
                });
            }

            self.error(&equals, "Invalid assignment target.");
        }
        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;

        while self.match_next_token(&[TokenType::Or]) {
            let operator = self.consume_token().unwrap();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.match_next_token(&[TokenType::And]) {
            let operator = self.consume_token().unwrap();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_next_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
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

        while self.match_next_token(&[
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

        while self.match_next_token(&[TokenType::Minus, TokenType::Plus]) {
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

        while self.match_next_token(&[TokenType::Slash, TokenType::Star]) {
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
        if self.match_next_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.consume_token().unwrap();
            Ok(Expr::Unary {
                operator,
                right: Box::new(self.unary()?),
            })
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_next_token(&[TokenType::LeftParen]) {
                // consume left paren
                let left_paren = self.consume_token().unwrap();
                // it's a function call
                let mut arguments = vec![];
                while !self.match_next_token(&[TokenType::RightParen]) {
                    // still have args
                    arguments.push(self.expression()?);
                    if arguments.len() > LOX_MAX_ARGUMENT_COUNT {
                        self.error(&left_paren, "Exceeded max argument count");
                    }
                    if self.match_next_token(&[TokenType::RightParen]) {
                        break;
                    }
                    self.require_consume(TokenType::Comma, "Expect arguments are comma seperated")?;
                }
                expr = Expr::Call {
                    callee: Box::new(expr),
                    paren: self.require_consume(
                        TokenType::RightParen,
                        "Expect ')' closing function call",
                    )?,
                    arguments: Box::new(arguments),
                };
            } else if self.match_next_token(&[TokenType::Dot]) {
                // it's a instance access
                // consume the dot
                self.consume_token();
                let name = self.require_consume(
                    TokenType::Identifier,
                    "Expect identifier after '.' operator on object",
                )?;
                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        match self.consume_token().unwrap() {
            Token {
                token_type: TokenType::False,
                ..
            } => Ok(Expr::Literal {
                value: LoxType::Bool(false),
            }),
            Token {
                token_type: TokenType::True,
                ..
            } => Ok(Expr::Literal {
                value: LoxType::Bool(true),
            }),
            Token {
                token_type: TokenType::Nil,
                ..
            } => Ok(Expr::Literal {
                value: LoxType::Nil,
            }),
            Token {
                token_type: TokenType::Number,
                raw,
                ..
            } => Ok(Expr::Literal {
                value: LoxType::Number(raw.parse::<f32>().unwrap()),
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
            Token {
                token_type: TokenType::Strang,
                raw,
                ..
            } => Ok(Expr::Literal {
                value: LoxType::Strang(raw),
            }),
            t if t.token_type == TokenType::Identifier => Ok(Expr::Variable { name: t }),
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
    ) -> Result<Token, ParseError> {
        match self.consume_token() {
            Some(t) if t.token_type == required => Ok(t),
            Some(t) => Err(self.error(&t, error_message)),
            None => Err(self.error(&token!(EOF, "", (0, 0)), error_message)),
        }
    }

    fn error(&self, token: &Token, message: &str) -> ParseError {
        println!(
            "parser: {} caused by {:?}, at line {} column {}",
            message, token.token_type, token.line, token.column
        );
        lox::report_error();
        ParseError
    }

    fn synchronize(&mut self) {
        while !self.is_done()
            && !self.match_next_token(&vec![
                TokenType::EOF,
                TokenType::SemiColon,
                TokenType::Class,
                TokenType::Funct,
                TokenType::Var,
                TokenType::For,
                TokenType::If,
                TokenType::While,
                TokenType::Print,
                TokenType::Return,
            ])
        {
            self.consume_token();
        }
    }

    pub fn is_done(&mut self) -> bool {
        match self.tokens.peek() {
            None => true,
            Some(tok) => tok.token_type == TokenType::EOF,
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while !self.is_done() {
            match self.declaration() {
                Ok(decl) => statements.push(decl),
                Err(_) => self.synchronize(),
            }
        }
        statements
    }
}

struct ParseError;
