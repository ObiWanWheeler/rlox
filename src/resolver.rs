use std::collections::HashMap;

use crate::{
    expr,
    interpreter::Interpreter,
    stmt, lox, common::Token,
};

pub struct Resolver {
    interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            scopes: vec![],
        }
    }

    fn resolve_statement(&mut self, stmt: &stmt::Stmt) -> Result<(), ResolverError> {
        stmt::Visitor::visit_stmt(self, stmt)
    }

    fn resolve_expr(&mut self, expr: &expr::Expr) -> Result<(), ResolverError> {
        expr::Visitor::visit_expr(self, expr)
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::<String, bool>::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: String) {
        if self.scopes.is_empty() {
            return;
        }
        self.scopes.last_mut().unwrap().insert(name, false);
    }

    fn define(&mut self, name: String) {
        if self.scopes.is_empty() {
            return;
        }

        self.scopes.last_mut().unwrap().insert(name, true);
    }

    fn resolve_local(&mut self, expr: &expr::Expr, name: String) -> Result<(), ResolverError> {
        for (i, scope) in self.scopes.iter().enumerate() {
            if scope.contains_key(&name) {
                self.interpreter.resolve(expr, self.scopes.len() - 1 -i);
                return Ok(())
            }
        }
        Ok(())
    }

    fn error(&self, token: Token, message: &str) -> ResolverError {
        println!("{}", message);
        lox::report_error();
        ResolverError::new(token, message.to_string())
    }
}

impl expr::Visitor<(), ResolverError> for Resolver {
    fn visit_expr(&mut self, expr: &expr::Expr) -> Result<(), ResolverError> {
        match expr {
            expr::Expr::Variable { name } => {
                if !self.scopes.is_empty() 
                    && !self.scopes.last().unwrap().get(&name.raw).unwrap_or(&false) {
                    Err(self.error(name.clone(), "Cannot use a variable in it's own initializer"))
                }
                else {
                    self.resolve_local(expr, name.raw.clone())?;
                    Ok(())
                }
            }
            expr::Expr::Assign { name, value } => {
                self.resolve_expr(value)?;
                self.resolve_local(expr, name.raw.clone())?;
                Ok(())
            }
            expr::Expr::Binary { left, right, .. } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
                Ok(())
            },
            expr::Expr::Call { callee, arguments, .. } => {
                self.resolve_expr(callee)?;
                for arg in (*arguments).iter() {
                    self.resolve_expr(arg)?;
                }
                Ok(())
            },
            expr::Expr::Grouping { expression } => self.resolve_expr(expression),
            expr::Expr::Literal { .. }=> Ok(()),
            expr::Expr::Logical { left, right, .. } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
                Ok(())
            },
            expr::Expr::Unary { right, .. } => self.resolve_expr(right),
        }
    }
}

impl stmt::Visitor<(), ResolverError> for Resolver {
    fn visit_stmt(&mut self, stmt: &stmt::Stmt) -> Result<(), ResolverError> {
        match stmt {
            stmt::Stmt::Block { statements } => {
                self.begin_scope();
                for stmt in statements.iter() {
                    self.resolve_statement(stmt)?;
                }
                self.end_scope();
                Ok(())
            }
            stmt::Stmt::Var { name, initializer } => {
                self.declare(name.raw.clone());
                if let Some(init) = initializer {
                    self.resolve_expr(init)?;
                }
                self.define(name.raw.clone());
                Ok(())
            }
            stmt::Stmt::Function { name, parameters, body } => {
                self.declare(name.raw.clone());
                self.define(name.raw.clone());
                
                self.begin_scope();
                
                for param in parameters {
                    self.declare(param.raw.clone());
                    self.define(param.raw.clone());
                }

                for stmt in (*body).iter() {
                    self.resolve_statement(stmt)?;
                }

                self.end_scope();

                Ok(())
            }
            stmt::Stmt::Expression { expression } => self.resolve_expr(expression),
            stmt::Stmt::If { condition, then_branch, else_branch } => { 
                self.resolve_expr(condition)?; 
                self.resolve_statement(then_branch)?;
                if let Some(b) = else_branch { 
                    self.resolve_statement(b)?;
                } 
                Ok(()) 
            },
            stmt::Stmt::While { condition, then_branch, finally_branch } => {
                self.resolve_expr(condition)?;
                self.resolve_statement(then_branch)?;
                if let Some(b) = finally_branch {
                    self.resolve_statement(b)?;
                }
                Ok(())
            },
            stmt::Stmt::Print { expression } => self.resolve_expr(expression),
            stmt::Stmt::Break { .. } => Ok(()),
            stmt::Stmt::Return { return_value, .. } => {
                if let Some(val) = return_value {
                    self.resolve_expr(val)?;
                }
                Ok(())
            },
        }
    }
}

pub struct ResolverError {
    pub token: Token,
    pub message: String,
}

impl ResolverError {
    pub fn new(token: Token, message: String) -> Self {
        Self {
            token, message,
        }
    }
}
