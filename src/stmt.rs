use crate::common::Token;
use crate::expr::Expr;

pub enum Stmt {
    Block {
        statements: Vec<Box<Stmt>>,
    },

    Expression {
        expression: Expr,
    },

    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>
    },

    Print {
        expression: Expr,
    },

    Var {
        name: Token,
        initializer: Option<Expr>,
    },
}

pub trait Visitor<R, E> {
    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<R, E>;
}
