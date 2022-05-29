use crate::common::Token;

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
}

#[derive(Debug)]
pub enum LiteralType {
    Number(f32),
    Strang(String),
    Bool(bool),
    Nil,
}

pub trait Visitor<R> {
    fn visit_expr(&mut self, expr: &Expr) -> R;
}
