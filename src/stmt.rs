use crate::{common::Token, expr::Expr};

pub enum Stmt {
    Expression { expression: Expr },

    Print { expression: Expr },

    Var { name: Token, initializer: Option<Expr> },
}

pub trait Visitor<R, E> {
    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<R, E>;
}
