use crate::common::{LoxType, Token};

#[derive(Debug, Clone, PartialEq)]
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

    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Box<Vec<Expr>>
    },

    Get {
        object: Box<Expr>,
        name: Token,
    },

    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>
    },

    Grouping {
        expression: Box<Expr>,
    },

    Literal {
        value: LoxType,
    },

    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
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
