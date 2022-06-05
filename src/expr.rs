use crate::common::Token;

#[derive(Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        right: Box<Expr>,
        operator: Token,
    },

    Grouping {
        expression: Box<Expr>,
    },

    Literal {
        value: LiteralType,
    },

    Unary {
        operator: Token,
        right: Box<Expr>,
    },

    Variable {
        name: Token
    }

}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum LiteralType {
    Number(f32),
    Strang(String),
    Bool(bool),
    Nil,
}



impl ToString for LiteralType {
    fn to_string(&self) -> String {
        match self {
            Self::Number(v) => v.to_string(),
            Self::Strang(v) => v.to_string(),
            Self::Bool(v) => v.to_string(),
            Self::Nil => "nil".to_string(),
        }
    }
}

pub trait Visitor<R, E> {
    fn visit_expr(&mut self, expr: &Expr) -> Result<R, E>;
}
