use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{common::Token, expr, interpreter::Interpreter, lox, stmt};

pub struct Resolver {
    interpreter: Rc<RefCell<Interpreter>>,
    scopes: Vec<HashMap<String, bool>>,
    current_scope: ScopeType,
}

impl Resolver {
    pub fn new(interpreter: Rc<RefCell<Interpreter>>) -> Self {
        Self {
            interpreter,
            scopes: vec![],
            current_scope: ScopeType::None,
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

    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        match self
            .scopes
            .last_mut()
            .unwrap()
            .insert(name.raw.to_string(), false)
        {
            None => {}
            Some(_) => {
                // variable name already declared in this scope.
                self.error(name.clone(), "Already a variable with this name in scope");
            }
        }
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.raw.to_string(), true);
    }

    fn resolve_local(&mut self, token: Token) -> Result<(), ResolverError> {
        for (i, scope) in self.scopes.iter().enumerate() {
            if scope.contains_key(&token.raw) {
                self.interpreter
                    .borrow_mut()
                    .resolve(token, self.scopes.len() - 1 - i);
                return Ok(());
            }
        }
        Ok(())
    }

    fn error(&self, token: Token, message: &str) -> ResolverError {
        println!(
            "Resolver: {} caused by {} at line {} column {}",
            message, token.raw, token.line, token.column
        );
        lox::report_error();
        ResolverError::new(token, message.to_string())
    }

    pub fn resolve(&mut self, statements: &[stmt::Stmt]) {
        for stmt in statements {
            match self.resolve_statement(stmt) {
                Err(_) => {}
                Ok(_) => {}
            }
        }
    }
}

impl expr::Visitor<(), ResolverError> for Resolver {
    fn visit_expr(&mut self, expr: &expr::Expr) -> Result<(), ResolverError> {
        match expr {
            expr::Expr::Variable { name } => {
                if !self.scopes.is_empty()
                    && self.scopes.last().unwrap().get(&name.raw).unwrap_or(&true) == &false
                {
                    Err(self.error(
                        name.clone(),
                        "Cannot use a variable in it's own initializer",
                    ))
                } else {
                    self.resolve_local(name.clone())?;
                    Ok(())
                }
            }
            expr::Expr::Assign { name, value } => {
                self.resolve_expr(value)?;
                self.resolve_local(name.clone())?;
                Ok(())
            }
            expr::Expr::Binary { left, right, .. } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
                Ok(())
            }
            expr::Expr::Call {
                callee, arguments, ..
            } => {
                self.resolve_expr(callee)?;
                for arg in (*arguments).iter() {
                    self.resolve_expr(arg)?;
                }
                Ok(())
            }
            expr::Expr::Grouping { expression } => self.resolve_expr(expression),
            expr::Expr::Literal { .. } => Ok(()),
            expr::Expr::Logical { left, right, .. } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
                Ok(())
            }
            expr::Expr::Unary { right, .. } => self.resolve_expr(right),
            expr::Expr::Get { object, .. } => self.resolve_expr(object),
            expr::Expr::Set { object, value, .. } => {
                self.resolve_expr(object)?;
                self.resolve_expr(value)?;
                Ok(())
            }
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
                self.declare(name);
                if let Some(init) = initializer {
                    self.resolve_expr(init)?;
                }
                self.define(name);
                Ok(())
            }
            stmt::Stmt::Function {
                name,
                parameters,
                body,
            } => {
                self.declare(name);
                self.define(name);

                let enclosing_scope_type = self.current_scope;
                self.current_scope = ScopeType::Function;

                self.begin_scope();

                for param in parameters {
                    self.declare(param);
                    self.define(param);
                }

                for stmt in (*body).iter() {
                    self.resolve_statement(stmt)?;
                }

                self.end_scope();
                self.current_scope = enclosing_scope_type;

                Ok(())
            }
            stmt::Stmt::Expression { expression } => self.resolve_expr(expression),
            stmt::Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition)?;
                self.resolve_statement(then_branch)?;
                if let Some(b) = else_branch {
                    self.resolve_statement(b)?;
                }
                Ok(())
            }
            stmt::Stmt::While {
                condition,
                then_branch,
                finally_branch,
            } => {
                let enclosing_scope_type = self.current_scope;
                self.current_scope = ScopeType::Loop;

                self.resolve_expr(condition)?;
                self.resolve_statement(then_branch)?;
                if let Some(b) = finally_branch {
                    self.resolve_statement(b)?;
                }

                self.current_scope = enclosing_scope_type;
                Ok(())
            }
            stmt::Stmt::Print { expression } => self.resolve_expr(expression),
            stmt::Stmt::Break { token } => {
                if let ScopeType::Loop = self.current_scope {
                    Ok(())
                } else {
                    Err(self.error(token.clone(), "Can only break from inside a loop."))
                }
            }
            stmt::Stmt::Return {
                return_value,
                token,
            } => {
                if let ScopeType::Function = self.current_scope {
                    if let Some(val) = return_value {
                        self.resolve_expr(val)?;
                    }
                    Ok(())
                } else {
                    Err(self.error(token.clone(), "Can only return from a function."))
                }
            }
            stmt::Stmt::Class { name, .. } => {
                self.declare(name);
                self.define(name);
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
        Self { token, message }
    }
}

#[derive(Clone, Copy)]
enum ScopeType {
    None,
    Function,
    Loop,
}
