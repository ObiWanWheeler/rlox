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
                self.visit_expr(left)?,
                operator.raw,
                self.visit_expr(right)?
            )),
            Expr::Logical {
                left,
                operator,
                right,
            } => Ok(format!(
                "(){} {} {})",
                self.visit_expr(left)?,
                operator.raw,
                self.visit_expr(right)?
            )),
            Expr::Grouping { expression } => {
                Ok(format!("(group {})", self.visit_expr(expression)?))
            }
            Expr::Literal { value } => Ok(format!("{:?}", value.clone())),
            Expr::Unary { operator, right } => {
                Ok(format!("({}{})", operator.raw, self.visit_expr(right)?))
            }
            Expr::Variable { name } => Ok(name.raw.clone()),
            Expr::Assign { name, value } => {
                Ok(format!("{} = {}", name.raw, self.visit_expr(value)?))
            }
            _ => Ok(format!("haven't bothered to implement this pp yet"))
        }
    }
}
