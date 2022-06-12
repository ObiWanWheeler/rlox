use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use phf::phf_map;

use crate::{
    environment::Environment,
    interpreter::{Interpreter, RuntimeException},
    stmt::Stmt,
};

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
   "meth" => TokenType::Meth,
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
    Meth,
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
    Class(LoxClass),
    Instance(LoxInstance),
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
            Self::Class(c) => match other {
                Self::Class(c2) => c.eq(c2),
                _ => false,
            },
            Self::Instance(i) => match other {
                Self::Instance(i2) => i.eq(i2),
                _ => false,
            },
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
            Self::Function(f) => f.to_string(),
            Self::Class(c) => c.to_string(),
            Self::Instance(i) => i.to_string(),
        }
    }
}

pub trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Rc<RefCell<LoxType>>>,
    ) -> Result<Rc<RefCell<LoxType>>, RuntimeException>;
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

impl ToString for dyn LoxCallable {
    fn to_string(&self) -> String {
        format!("function <{}>", self.arity())
    }
}

pub struct LoxFunction {
    name: Token,
    parameters: Vec<Token>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(
        name: Token,
        parameters: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
    ) -> Self {
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
        arguments: Vec<Rc<RefCell<LoxType>>>,
    ) -> Result<Rc<RefCell<LoxType>>, RuntimeException> {
        let mut environment = Environment::new(Some(Rc::clone(&self.closure)));

        for (param, arg) in self.parameters.iter().zip(arguments.into_iter()) {
            environment.define(param.raw.clone(), arg);
        }

        match interpreter.execute_block(&self.body, Rc::new(RefCell::new(environment))) {
            Err(err) => {
                if err.token.token_type == TokenType::Return {
                    match err.value {
                        None => return Ok(Rc::new(RefCell::new(LoxType::Nil))),
                        Some(v) => return Ok(v),
                    }
                }
            }
            _ => {}
        }
        Ok(Rc::new(RefCell::new(LoxType::Nil)))
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

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct LoxClass {
    name: String,
}

impl LoxClass {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl ToString for LoxClass {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

impl LoxCallable for LoxClass {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _: &mut Interpreter,
        _: Vec<Rc<RefCell<LoxType>>>,
    ) -> Result<Rc<RefCell<LoxType>>, RuntimeException> {
        Ok(Rc::new(RefCell::new(LoxType::Instance(LoxInstance::new(
            self.clone(),
        )))))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance {
    class_: LoxClass,
    fields: HashMap<String, Rc<RefCell<LoxType>>>,
}

impl LoxInstance {
    pub fn new(class_: LoxClass) -> Self {
        Self {
            class_,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Rc<RefCell<LoxType>>, RuntimeException> {
        match self.fields.get(&name.raw) {
            Some(v) => Ok(Rc::clone(v)),
            None => Err(RuntimeException::report(
                name.clone(),
                &format!(
                    "Property {} does not exist on {}",
                    name.raw,
                    self.to_string()
                ),
            )),
        }
    }

    pub fn set(&mut self, name: &Token, value: Rc<RefCell<LoxType>>) {
        self.fields.insert(name.raw.to_string(), value);
    }
}

impl PartialOrd for LoxInstance {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl ToString for LoxInstance {
    fn to_string(&self) -> String {
        format!("{} instance", self.class_.to_string())
    }
}

pub fn is_punctuation(c: &char) -> bool {
    let punctuation = vec!['(', ')', '{', '}', '[', ']', ';', ',', '\'', '"', '.'];
    punctuation.contains(c)
}
