use crate::{
    common::{Token, TokenType},
    environment::Environment,
    expr::{self, LiteralType},
    lox, stmt,
};

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    fn execute(&mut self, stmt: &stmt::Stmt) -> Result<(), RuntimeException> {
        stmt::Visitor::visit_stmt(self, stmt)
    }
    fn evaluate(&mut self, expression: &expr::Expr) -> Result<LiteralType, RuntimeException> {
        expr::Visitor::visit_expr(self, expression)
    }

    fn is_truthy(object: LiteralType) -> bool {
        match object {
            LiteralType::Nil => false,
            LiteralType::Bool(value) => value,
            _ => true,
        }
    }

    pub fn interpret(&mut self, statements: &[stmt::Stmt]) {
        for stmt in statements {
            if let Err(_) = self.execute(stmt) {
                return;
            }
        }
    }
}

impl expr::Visitor<LiteralType, RuntimeException> for Interpreter {
    fn visit_expr(&mut self, expr: &expr::Expr) -> Result<LiteralType, RuntimeException> {
        match expr {
            expr::Expr::Literal { value } => Ok(value.clone()),
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
                        (LiteralType::Number(left), LiteralType::Number(right)) => {
                            Ok(LiteralType::Number(left + right))
                        }
                        (LiteralType::Strang(left), right) => {
                            Ok(LiteralType::Strang(left + &right.to_string()))
                        }
                        (left, LiteralType::Strang(right)) => {
                            Ok(LiteralType::Strang(left.to_string() + &right))
                        }
                        (left, right) => Err(RuntimeException::report(
                            operator.clone(),
                            &format!("invalid operands {:?}, {:?} for +", left, right),
                        )),
                    },
                    TokenType::Minus => match (left, right) {
                        (LiteralType::Number(left), LiteralType::Number(right)) => {
                            Ok(LiteralType::Number(left - right))
                        }
                        (left, right) => Err(RuntimeException::report(
                            operator.clone(),
                            &format!("invalid operands {:?}, {:?} for -", left, right),
                        )),
                    },
                    TokenType::Slash => match (left, right) {
                        (LiteralType::Number(left), LiteralType::Number(right)) => {
                            if right == 0f32 {
                                // divide by 0 error
                                return Err(RuntimeException::report(
                                    operator.clone(),
                                    &format!("cannot divide by 0 in {:?} / {:?}", left, 0f32),
                                ));
                            }
                            Ok(LiteralType::Number(left / right))
                        }
                        (left, right) => Err(RuntimeException::report(
                            operator.clone(),
                            &format!("invalid operands {:?}, {:?} for / ", left, right),
                        )),
                    },
                    TokenType::Star => match (left, right) {
                        (LiteralType::Number(left), LiteralType::Number(right)) => {
                            Ok(LiteralType::Number(left * right))
                        }
                        (left, right) => Err(RuntimeException::report(
                            operator.clone(),
                            &format!("invalid operands {:?}, {:?} for * ", left, right),
                        )),
                    },
                    TokenType::Greater => Ok(LiteralType::Bool(left > right)),
                    TokenType::GreaterEqual => Ok(LiteralType::Bool(left >= right)),
                    TokenType::Less => Ok(LiteralType::Bool(left < right)),
                    TokenType::LessEqual => Ok(LiteralType::Bool(left <= right)),
                    TokenType::BangEqual => Ok(LiteralType::Bool(!(left == right))),
                    TokenType::EqualEqual => Ok(LiteralType::Bool(left == right)),
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
                        LiteralType::Number(value) => Ok(LiteralType::Number(-value)),
                        _ => Err(RuntimeException::report(
                            operator.clone(),
                            &format!(
                                "Unary operator Minus '-' not supported on type of {:?}",
                                right
                            ),
                        )),
                    },
                    TokenType::Bang => {
                        return Ok(LiteralType::Bool(!Interpreter::is_truthy(right)));
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
            expr::Expr::Variable { name } => self.environment.get(name),
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
            stmt::Stmt::Print { expression } => {
                let val = self.evaluate(expression)?;
                println!("{}", val.to_string());
                Ok(())
            }
            stmt::Stmt::Var { name, initializer } => {
                let mut val = LiteralType::Nil;
                if let Some(init) = initializer {
                    val = self.evaluate(init)?;
                }

                self.environment.define(name.raw.clone(), val);
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
