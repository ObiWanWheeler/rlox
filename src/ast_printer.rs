use std::string::ParseError;

use crate::expr::{Expr, Visitor};

pub struct AstPrinter {}

impl Visitor<String, ParseError> for AstPrinter {
    fn visit_expr(&mut self, expr: &Expr) -> Result<String, ParseError> {
        match expr {
            Expr::Binary {
                left,
                right,
                operator,
            } => Ok(format!(
                "({} {} {})",
                operator.raw,
                self.visit_expr(left)?,
                self.visit_expr(right)?)
            ),
            Expr::Grouping { expression } => Ok(format!("(group {})", self.visit_expr(expression)?)),
            Expr::Literal { value } => Ok(format!("{:?}", value.clone())),
            Expr::Unary { operator, right } => {
                Ok(format!("({}{})", operator.raw, self.visit_expr(right)?))
            }
            Expr::Variable { name } => {
                Ok(name.raw.clone())
            }
        }
    }
}
