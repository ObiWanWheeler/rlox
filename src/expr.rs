use crate::common::{Token, LiteralType};

pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },

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
        name: Token,
    },
}

pub trait Visitor<R, E> {
    fn visit_expr(&mut self, expr: &Expr) -> Result<R, E>;
}
