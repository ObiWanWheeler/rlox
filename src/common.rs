use std::{cell::RefCell, fmt::Debug, rc::Rc};

use phf::phf_map;

use crate::{environment::Environment, interpreter::{Interpreter, RuntimeException}, stmt::Stmt};

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
   "break" => TokenType::Break,
   "class" => TokenType::Class,
   "else" => TokenType::Else,
   "false" => TokenType::False,
   "funct" => TokenType::Funct,
   "for" => TokenType::For,
   "finally" => TokenType::Finally,
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

pub const LOX_MAX_ARGUMENT_COUNT: usize = 255;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
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
    Break,
    Class,
    Else,
    False,
    Funct,
    Finally,
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
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Token {
    pub token_type: TokenType,
    pub raw: String,
    pub line: u32,
    pub column: u32,
}
#[derive(Debug, Clone, PartialOrd)]
pub enum LoxType {
    Number(f32),
    Strang(String),
    Bool(bool),
    Nil,
    Function(Rc<dyn LoxCallable>),
}

impl PartialEq for LoxType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Number(v) => match other {
                Self::Number(x) => *v == *x,
                Self::Strang(s) => match s.parse::<f32>() {
                    Ok(x) => *v == x,
                    Err(_) => false,
                },
                _ => false,
            },
            Self::Strang(s) => match other {
                Self::Strang(r) => s.eq(r),
                Self::Number(v) => match s.parse::<f32>() {
                    Ok(x) => *v == x,
                    _ => false,
                },
                _ => false,
            },
            Self::Bool(b) => match other {
                Self::Bool(c) => *b == *c,
                Self::Nil => *b == false,
                _ => false,
            },
            Self::Nil => match other {
                Self::Bool(false) => true,
                _ => false,
            },
            Self::Function(_) => false,
        }
    }
}

impl ToString for LoxType {
    fn to_string(&self) -> String {
        match self {
            Self::Number(v) => v.to_string(),
            Self::Strang(v) => v.to_string(),
            Self::Bool(v) => v.to_string(),
            Self::Nil => "nil".to_string(),
            Self::Function(f) => format!("function <{}>", f.arity()),
        }
    }
}

pub trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxType>,
    ) -> Result<LoxType, RuntimeException>;
}

impl Debug for dyn LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "function <{}>", self.arity())
    }
}

impl PartialEq for dyn LoxCallable {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl PartialOrd for dyn LoxCallable {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

pub struct LoxFunction {
    name: Token,
    parameters: Vec<Token>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Environment>>
}

impl LoxFunction {
    pub fn new(name: Token, parameters: Vec<Token>, body: Vec<Stmt>, closure: Rc<RefCell<Environment>>) -> Self {
        Self {
            name,
            parameters,
            body,
            closure,
        }
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.parameters.len()
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxType>,
    ) -> Result<LoxType, RuntimeException> {
        let mut environment = Environment::new(Some(Rc::clone(&self.closure)));

        for (param, arg) in self.parameters.iter().zip(arguments.into_iter()) {
            environment.define(param.raw.clone(), arg);
        }

        match interpreter.execute_block(&self.body, Rc::new(RefCell::new(environment))) {
            Err(err) => {
                if err.token.token_type == TokenType::Return {
                    match err.value {
                        None => return Ok(LoxType::Nil),
                        Some(v) => return Ok(v),
                    }
                }
            }
            _ => {}
        }
        // TODO add return types
        Ok(LoxType::Nil)
    }
}

impl ToString for LoxFunction {
    fn to_string(&self) -> String {
        format!(
            "<function> {:?} ({:?})",
            self.name.raw,
            self.parameters.iter().map(|tok| &tok.raw)
        )
    }
}

pub fn is_punctuation(c: &char) -> bool {
    let punctuation = vec!['(', ')', '{', '}', '[', ']', ';', ',', '\'', '"', '.'];
    punctuation.contains(c)
}
