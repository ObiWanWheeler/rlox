use std::{cell::RefCell, rc::Rc};

use crate::{
    common::{LoxType, Token, TokenType, LoxFunction},
    environment::Environment,
    expr, lox, stmt, native_functions::Clock,
};

pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new(None)));
        globals.borrow_mut().define("clock".to_string(), LoxType::Function(Rc::new(Clock)));

        Self {
            globals: Rc::clone(&globals),
            environment: globals,
        }
    }

    fn execute(&mut self, stmt: &stmt::Stmt) -> Result<(), RuntimeException> {
        stmt::Visitor::visit_stmt(self, stmt)
    }

    pub fn execute_block(&mut self, statements: &[stmt::Stmt], environment: Rc<RefCell<Environment>>) -> Result<(), RuntimeException> {
        let prev = Rc::clone(&self.environment);
        self.environment = environment;

        for stmt in statements {
            self.execute(stmt)?;
        }

        self.environment = prev;
        Ok(())
    }

    fn evaluate(&mut self, expression: &expr::Expr) -> Result<LoxType, RuntimeException> {
        expr::Visitor::visit_expr(self, expression)
    }

    fn is_truthy(object: LoxType) -> bool {
        match object {
            LoxType::Nil => false,
            LoxType::Bool(value) => value,
            _ => true,
        }
    }

    pub fn globals(&self) -> Rc<RefCell<Environment>> {
        Rc::clone(&self.globals)
    }

    pub fn interpret(&mut self, statements: &[stmt::Stmt]) {
        for stmt in statements {
            if let Err(_) = self.execute(stmt) {
                return;
            }
        }
    }
}

impl expr::Visitor<LoxType, RuntimeException> for Interpreter {
    fn visit_expr(&mut self, expr: &expr::Expr) -> Result<LoxType, RuntimeException> {
        match expr {
            expr::Expr::Literal { value } => Ok(value.clone()),
            expr::Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;

                match operator.token_type {
                    TokenType::Or => {
                        if Interpreter::is_truthy(left) {
                            return Ok(LoxType::Bool(true));
                        }
                    }
                    TokenType::And => {
                        if !Interpreter::is_truthy(left) {
                            return Ok(LoxType::Bool(false));
                        }
                    }
                    _ => {
                        return Err(RuntimeException::report(
                            operator.clone(),
                            &format!("invalid operator {} in logical expression", operator.raw),
                        ))
                    }
                };

                self.evaluate(right)
            }

            expr::Expr::Binary {
                left,
                right,
                operator,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                // TODO factor out Errs into function
                match operator.token_type {
                    TokenType::Plus => match (left, right) {
                        (LoxType::Number(left), LoxType::Number(right)) => {
                            Ok(LoxType::Number(left + right))
                        }
                        (LoxType::Strang(left), right) => {
                            Ok(LoxType::Strang(left + &right.to_string()))
                        }
                        (left, LoxType::Strang(right)) => {
                            Ok(LoxType::Strang(left.to_string() + &right))
                        }
                        (left, right) => Err(RuntimeException::report(
                            operator.clone(),
                            &format!("invalid operands {:?}, {:?} for +", left, right),
                        )),
                    },
                    TokenType::Minus => match (left, right) {
                        (LoxType::Number(left), LoxType::Number(right)) => {
                            Ok(LoxType::Number(left - right))
                        }
                        (left, right) => Err(RuntimeException::report(
                            operator.clone(),
                            &format!("invalid operands {:?}, {:?} for -", left, right),
                        )),
                    },
                    TokenType::Slash => match (left, right) {
                        (LoxType::Number(left), LoxType::Number(right)) => {
                            if right == 0f32 {
                                // divide by 0 error
                                return Err(RuntimeException::report(
                                    operator.clone(),
                                    &format!("cannot divide by 0 in {:?} / {:?}", left, 0f32),
                                ));
                            }
                            Ok(LoxType::Number(left / right))
                        }
                        (left, right) => Err(RuntimeException::report(
                            operator.clone(),
                            &format!("invalid operands {:?}, {:?} for / ", left, right),
                        )),
                    },
                    TokenType::Star => match (left, right) {
                        (LoxType::Number(left), LoxType::Number(right)) => {
                            Ok(LoxType::Number(left * right))
                        }
                        (left, right) => Err(RuntimeException::report(
                            operator.clone(),
                            &format!("invalid operands {:?}, {:?} for * ", left, right),
                        )),
                    },
                    TokenType::Greater => Ok(LoxType::Bool(left > right)),
                    TokenType::GreaterEqual => Ok(LoxType::Bool(left >= right)),
                    TokenType::Less => Ok(LoxType::Bool(left < right)),
                    TokenType::LessEqual => Ok(LoxType::Bool(left <= right)),
                    TokenType::BangEqual => Ok(LoxType::Bool(!(left == right))),
                    TokenType::EqualEqual => Ok(LoxType::Bool(left == right)),
                    _ => Err(RuntimeException::report(
                        operator.clone(),
                        &format!("Invalid binary operand {:?}", operator),
                    )),
                }
            }
            expr::Expr::Grouping { expression } => Ok(self.evaluate(expression)?),
            expr::Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::Minus => match right {
                        LoxType::Number(value) => Ok(LoxType::Number(-value)),
                        _ => Err(RuntimeException::report(
                            operator.clone(),
                            &format!(
                                "Unary operator Minus '-' not supported on type of {:?}",
                                right
                            ),
                        )),
                    },
                    TokenType::Bang => {
                        return Ok(LoxType::Bool(!Interpreter::is_truthy(right)));
                    }
                    _ => Err(RuntimeException::report(
                        operator.clone(),
                        &format!(
                            "Unary operator Bang '!' not supported on type of {:?}",
                            right
                        ),
                    )),
                }
            }
            expr::Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.evaluate(callee)?;

                let mut args = vec![];
                for arg in arguments.iter() {
                    args.push(self.evaluate(arg)?);
                }

                match callee {
                    LoxType::Function(f) => {
                        if arguments.len() != f.arity() {
                            Err(RuntimeException::report(
                                paren.clone(),
                                &format!(
                                    "Expected {} arguments, found {} in {:?}",
                                    f.arity(),
                                    arguments.len(),
                                    arguments
                                ),
                            ))
                        } else {
                            f.call(self, args)
                        }
                    }
                    _ => Err(RuntimeException::report(
                        paren.clone(),
                        &format!(
                            "Unable to call {:?}. Only functions and classes may be called",
                            callee
                        ),
                    )),
                }
            }
            expr::Expr::Variable { name } => self.environment.borrow().get(name),
            expr::Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;
                self.environment.borrow_mut().assign(name, value.clone())?;
                Ok(value)
            }
        }
    }
}

impl stmt::Visitor<(), RuntimeException> for Interpreter {
    fn visit_stmt(&mut self, stmt: &stmt::Stmt) -> Result<(), RuntimeException> {
        match stmt {
            stmt::Stmt::Expression { expression } => {
                self.evaluate(expression)?;
                Ok(())
            }
            stmt::Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.evaluate(condition)?;
                if Interpreter::is_truthy(condition) {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }
                Ok(())
            }
            stmt::Stmt::While {
                condition,
                then_branch,
                finally_branch,
            } => {
                while Interpreter::is_truthy(self.evaluate(condition)?) {
                    self.execute(then_branch)?;
                }
                if let Some(finally_branch) = finally_branch {
                    self.execute(finally_branch)?;
                }
                Ok(())
            }
            stmt::Stmt::Print { expression } => {
                let val = self.evaluate(expression)?;
                println!("{}", val.to_string());
                Ok(())
            }
            stmt::Stmt::Var { name, initializer } => {
                let mut val = LoxType::Nil;
                if let Some(init) = initializer {
                    val = self.evaluate(init)?;
                }

                self.environment.borrow_mut().define(name.raw.clone(), val);
                Ok(())
            }
            stmt::Stmt::Function { name, parameters, body } => {
               let function = LoxFunction::new(name.clone(), parameters.to_vec(), body.to_vec());
               self.environment.borrow_mut().define(name.raw.clone(), LoxType::Function(Rc::new(function)));
               Ok(())
            }
            stmt::Stmt::Block { statements } => {
                let block_env = Environment::new(Some(Rc::clone(&self.environment)));
                self.execute_block(&statements, Rc::new(RefCell::new(block_env)))?;
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub struct RuntimeException {
    pub token: Token,
    pub message: String,
}

impl RuntimeException {
    // alerts lox of runtime error and returns the error
    pub fn report(token: Token, message: &str) -> Self {
        println!(
            "{} caused by {:?} at {:?}:{:?}",
            message, token.token_type, token.line, token.column
        );
        lox::report_runtime_error();
        Self {
            token,
            message: message.to_string(),
        }
    }
}
