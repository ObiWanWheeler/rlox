use crate::expr::{Expr, Visitor};

pub struct AstPrinter {}

impl Visitor<String> for AstPrinter {
    fn visit_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Binary {
                left,
                right,
                operator,
            } => format!(
                "({} {} {})",
                operator.raw,
                self.visit_expr(left),
                self.visit_expr(right)
            ),
            Expr::Grouping { expression } => format!("(group {})", self.visit_expr(expression)),
            Expr::Literal { value } => format!("{:?}", value.clone()),
            Expr::Unary { operator, right } => {
                format!("({}{})", operator.raw, self.visit_expr(right))
            }
        }
    }
}
