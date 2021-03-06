use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    common::{LoxCallable, LoxClass, LoxFunction, LoxType, Token, TokenType},
    environment::Environment,
    expr, lox,
    native_functions::Clock,
    stmt,
};

pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    locals: HashMap<Token, usize>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new(None)));
        globals
            .borrow_mut()
            .define("clock".to_string(), Rc::new(RefCell::new(LoxType::Function(Rc::new(Clock)))));

        Self {
            globals: Rc::clone(&globals),
            environment: globals,
            locals: HashMap::new(),
        }
    }

    fn execute(&mut self, stmt: &stmt::Stmt) -> Result<(), RuntimeException> {
        stmt::Visitor::visit_stmt(self, stmt)
    }

    pub fn execute_block(
        &mut self,
        statements: &[stmt::Stmt],
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), RuntimeException> {
        let prev = Rc::clone(&self.environment);
        self.environment = environment;

        for stmt in statements {
            match self.execute(stmt) {
                Err(e) => {
                    self.environment = prev;
                    return Err(e);
                }
                _ => {}
            };
        }

        self.environment = prev;
        Ok(())
    }

    fn evaluate(&mut self, expression: &expr::Expr) -> Result<Rc<RefCell<LoxType>>, RuntimeException> {
        expr::Visitor::visit_expr(self, expression)
    }

    fn is_truthy(object: &LoxType) -> bool {
        match object {
            LoxType::Nil => false,
            LoxType::Bool(value) => *value,
            _ => true,
        }
    }

    pub fn globals(&self) -> Rc<RefCell<Environment>> {
        Rc::clone(&self.globals)
    }

    pub fn resolve(&mut self, name: Token, depth: usize) {
        self.locals.insert(name, depth);
    }

    pub fn lookup_variable(&mut self, name: &Token) -> Result<Rc<RefCell<LoxType>>, RuntimeException> {
        let distance = self.locals.get(name);
        match distance {
            Some(d) => self.environment.borrow().get_at(*d, &name),
            None => self.globals.borrow().get(&name),
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

impl expr::Visitor<Rc<RefCell<LoxType>>, RuntimeException> for Interpreter {
    fn visit_expr(&mut self, expr: &expr::Expr) -> Result<Rc<RefCell<LoxType>>, RuntimeException> {
        match expr {
            expr::Expr::Literal { value } => Ok(Rc::new(RefCell::new(value.clone()))),
            expr::Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;

                match operator.token_type {
                    TokenType::Or => {
                        if Interpreter::is_truthy(&*left.borrow()) {
                            return Ok(Rc::new(RefCell::new(LoxType::Bool(true))));
                        }
                    }
                    TokenType::And => {
                        if !Interpreter::is_truthy(&*left.borrow()) {
                            return Ok(Rc::new(RefCell::new(LoxType::Bool(false))));
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
                    TokenType::Plus => match (&*left.borrow(), &*right.borrow()) {
                        (LoxType::Number(left), LoxType::Number(right)) => {
                            Ok(Rc::new(RefCell::new(LoxType::Number(left + right))))
                        }
                        (LoxType::Strang(left), right) => {
                            Ok(Rc::new(RefCell::new(LoxType::Strang(left.to_string() + &right.to_string()))))
                        }
                        (left, LoxType::Strang(right)) => {
                            Ok(Rc::new(RefCell::new(LoxType::Strang(left.to_string() + &right))))
                        }
                        (left, right) => Err(RuntimeException::report(
                            operator.clone(),
                            &format!("invalid operands {:?}, {:?} for +", left, right),
                        )),
                    },
                    TokenType::Minus => match (&*left.borrow(), &*right.borrow()) {
                        (LoxType::Number(left), LoxType::Number(right)) => {
                            Ok(Rc::new(RefCell::new(LoxType::Number(left - right))))
                        }
                        (left, right) => Err(RuntimeException::report(
                            operator.clone(),
                            &format!("invalid operands {:?}, {:?} for -", left, right),
                        )),
                    },
                    TokenType::Slash => match (&*left.borrow(), &*right.borrow()) {
                        (LoxType::Number(left), LoxType::Number(right)) => {
                            if *right == 0f32 {
                                // divide by 0 error
                                return Err(RuntimeException::report(
                                    operator.clone(),
                                    &format!("cannot divide by 0 in {:?} / {:?}", left, 0f32),
                                ));
                            }
                            Ok(Rc::new(RefCell::new(LoxType::Number(left / right))))
                        }
                        (left, right) => Err(RuntimeException::report(
                            operator.clone(),
                            &format!("invalid operands {:?}, {:?} for / ", left, right),
                        )),
                    },
                    TokenType::Star => match (&*left.borrow(), &*right.borrow()) {
                        (LoxType::Number(left), LoxType::Number(right)) => {
                            Ok(Rc::new(RefCell::new(LoxType::Number(left * right))))
                        }
                        (left, right) => Err(RuntimeException::report(
                            operator.clone(),
                            &format!("invalid operands {:?}, {:?} for * ", left, right),
                        )),
                    },
                    TokenType::Greater => Ok(Rc::new(RefCell::new(LoxType::Bool(left > right)))),
                    TokenType::GreaterEqual => Ok(Rc::new(RefCell::new(LoxType::Bool(left >= right)))),
                    TokenType::Less => Ok(Rc::new(RefCell::new(LoxType::Bool(left < right)))),
                    TokenType::LessEqual => Ok(Rc::new(RefCell::new(LoxType::Bool(left <= right)))),
                    TokenType::BangEqual => Ok(Rc::new(RefCell::new(LoxType::Bool(!(left == right))))),
                    TokenType::EqualEqual => Ok(Rc::new(RefCell::new(LoxType::Bool(left == right)))),
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
                    TokenType::Minus => match &*right.borrow() {
                        LoxType::Number(value) => Ok(Rc::new(RefCell::new(LoxType::Number(-value)))),
                        _ => Err(RuntimeException::report(
                            operator.clone(),
                            &format!(
                                "Unary operator Minus '-' not supported on type of {:?}",
                                right
                            ),
                        )),
                    },
                    TokenType::Bang => {
                        return Ok(Rc::new(RefCell::new(LoxType::Bool(!Interpreter::is_truthy(&*right.borrow())))));
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
                
                let x = &*callee.borrow();
                match x {
                    LoxType::Function(f) => {
                        if args.len() != f.arity() {
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
                    LoxType::Class(c) => {
                        if args.len() != c.arity() {
                            Err(RuntimeException::report(
                                paren.clone(),
                                &format!(
                                    "Expected {} arguments, found {} in {:?}",
                                    c.arity(),
                                    arguments.len(),
                                    arguments
                                ),
                            ))
                        }
                        else {
                            c.call(self, args)
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
            expr::Expr::Variable { name } => self.lookup_variable(name),
            expr::Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;
                let distance = self.locals.get(name);
                match distance {
                    Some(d) => self
                        .environment
                        .borrow_mut()
                        .assign_at(*d, name, value.clone())?,
                    None => self.globals.borrow_mut().assign(name, value.clone())?,
                };
                Ok(value)
            }
            expr::Expr::Get { object, name } => {
                let object = self.evaluate(object)?;
                let x = &*object.borrow();
                match x {
                    LoxType::Instance(inst) => {
                        inst.get(name)
                    }
                    _ => Err(RuntimeException::report(name.clone(), &format!("Unable to access property {} on {:?}. Not an instance. Only instances have properties.", name.raw, object)))
                }
            },
            expr::Expr::Set { object, name, value } => {
                let object = self.evaluate(object)?;
                let x = &mut *object.borrow_mut();
                match x {
                    LoxType::Instance(ref mut inst) => {
                        let value = self.evaluate(value)?;
                        inst.set(name, value.clone());
                        Ok(value)
                    } 
                    _ => Err(RuntimeException::report(name.clone(), &format!("Unable to set property on {} on {:?}. Not an instance. Only instances have properties.", name.raw, object)))
                }
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
                if Interpreter::is_truthy(&*condition.borrow()) {
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
                while Interpreter::is_truthy(&*self.evaluate(condition)?.borrow()) {
                    if let Err(err) = self.execute(then_branch) {
                        if err.token.token_type == TokenType::Break {
                            break;
                        }
                    }
                }
                if let Some(finally_branch) = finally_branch {
                    self.execute(finally_branch)?;
                }
                Ok(())
            }
            stmt::Stmt::Break { token } => Err(RuntimeException {
                token: token.clone(),
                message: "break".to_string(),
                value: None,
            }),
            stmt::Stmt::Print { expression } => {
                let val = self.evaluate(expression)?;
                println!("{}", &*val.borrow().to_string());
                Ok(())
            }
            stmt::Stmt::Var { name, initializer } => {
                let mut val = Rc::new(RefCell::new(LoxType::Nil));
                if let Some(init) = initializer {
                    val = self.evaluate(init)?;
                }

                self.environment.borrow_mut().define(name.raw.clone(), val);
                Ok(())
            }
            stmt::Stmt::Function {
                name,
                parameters,
                body,
            } => {
                let function = LoxFunction::new(
                    name.clone(),
                    parameters.to_vec(),
                    body.to_vec(),
                    Rc::clone(&self.environment),
                );
                self.environment
                    .borrow_mut()
                    .define(name.raw.clone(), Rc::new(RefCell::new(LoxType::Function(Rc::new(function)))));
                Ok(())
            }
            stmt::Stmt::Return {
                token,
                return_value,
            } => {
                let rv: Rc<RefCell<LoxType>>;
                if let Some(val) = return_value {
                    rv = self.evaluate(val)?;
                } else {
                    rv = Rc::new(RefCell::new(LoxType::Nil));
                }
                Err(RuntimeException {
                    token: token.clone(),
                    message: "return".to_string(),
                    value: Some(rv),
                })
            }
            stmt::Stmt::Block { statements } => {
                let block_env = Environment::new(Some(Rc::clone(&self.environment)));
                self.execute_block(&statements, Rc::new(RefCell::new(block_env)))?;
                Ok(())
            }
            stmt::Stmt::Class { name, .. } => {
                self.environment
                    .borrow_mut()
                    .define(name.raw.to_string(), Rc::new(RefCell::new(LoxType::Nil)));
                let class_ = Rc::new(RefCell::new(LoxType::Class(LoxClass::new(name.raw.to_string()))));
                self.environment.borrow_mut().assign(&name, class_)?;
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub struct RuntimeException {
    pub token: Token,
    pub message: String,
    pub value: Option<Rc<RefCell<LoxType>>>,
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
            value: None,
        }
    }
}
