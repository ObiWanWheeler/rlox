use crate::common::Token;
use crate::expr::Expr;

#[derive(Clone)]
pub enum Stmt {
    Block {
        statements: Box<Vec<Stmt>>,
    },

    Expression {
        expression: Expr,
    },

    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },

    While {
        condition: Expr,
        then_branch: Box<Stmt>,
        finally_branch: Option<Box<Stmt>>
    },

    Print {
        expression: Expr,
    },

    Var {
        name: Token,
        initializer: Option<Expr>,
    },

    Function {
        name: Token,
        parameters: Vec<Token>,
        body: Box<Vec<Stmt>>,
    }
}

pub trait Visitor<R, E> {
    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<R, E>;
}
