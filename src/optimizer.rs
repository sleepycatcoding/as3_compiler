pub use ast::Optimizer as AstOptimizer;

mod ast {
    use crate::ast::{Expression, Folder, Operator};
    use std::borrow::Cow;

    /// Experimental Abstract Syntax Tree optimizer.
    ///
    /// # Goal
    ///
    /// Goal of the optimizer is to simplify expressions that can be evaluted at compile-time to reduce runtime evaluation costs.
    pub struct Optimizer {}

    impl Folder for Optimizer {
        fn fold_expression(&mut self, v: Box<Expression>) -> Box<Expression> {
            match *v {
                Expression::BinaryOperation { lhs, operator, rhs } => {
                    let lhs = self.fold_expression(lhs);
                    let rhs = self.fold_expression(rhs);

                    match (*lhs, *rhs) {
                        // Pre-evaluate integer operations.
                        (Expression::Integer(value1), Expression::Integer(value2)) => {
                            Box::new(Expression::Integer(match operator {
                                Operator::Add => value1 + value2,
                                // FIXME: Probably throw error here when second val is 0.
                                Operator::Div => value1 / value2,
                                Operator::Mul => value1 * value2,
                                Operator::Sub => value1 - value2,
                            }))
                        }
                        // Optimize string concat operations.
                        (Expression::String(lhs), rhs) => {
                            if let Ok(rhs) = TryInto::<Cow<'_, String>>::try_into(&rhs) {
                                let mut buf = String::new();

                                buf.push_str(&lhs);
                                buf.push_str(&rhs);

                                Box::new(Expression::String(buf))
                            } else {
                                Box::new(Expression::BinaryOperation {
                                    lhs: Box::new(Expression::String(lhs)),
                                    operator,
                                    rhs: Box::new(rhs),
                                })
                            }
                        }
                        // Needs to be evaluated at runtime.
                        (lhs, rhs) => Box::new(Expression::BinaryOperation {
                            lhs: Box::new(lhs),
                            operator,
                            rhs: Box::new(rhs),
                        }),
                    }
                }
                // Cannot optimize any further.
                Expression::Integer(v) => Box::new(Expression::Integer(v)),
                Expression::Variable(v) => Box::new(Expression::Variable(v)),
                Expression::String(v) => Box::new(Expression::String(v)),
            }
        }
    }

    impl Default for Optimizer {
        fn default() -> Self {
            Self {}
        }
    }
}
