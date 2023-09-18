use lalrpop_util::lalrpop_mod;

use crate::ast::Visitor;

mod ast;
mod codegen;
mod lexer;
mod optimizer;

lalrpop_mod!(pub grammar);

fn main() {
    println!("Hello, world!");

    let file = std::fs::read_to_string("Test.as").unwrap();
    let lexer = lexer::Lexer::new(&file);
    let parser = grammar::PackageParser::new();
    let ast = parser.parse(lexer).unwrap();

    println!("{:?}", ast);

    //let mut optimizer = optimizer::AstOptimizer::default();

    //let new_ast = optimizer.fold_package(ast);

    let expr = crate::ast::Expression::BinaryOperation {
        lhs: crate::ast::Expression::Integer(10).into(),
        operator: crate::ast::Operator::Add,
        rhs: crate::ast::Expression::BinaryOperation {
            lhs: ast::Expression::BinaryOperation {
                lhs: ast::Expression::BinaryOperation {
                    lhs: ast::Expression::Integer(10).into(),
                    operator: ast::Operator::Mul,
                    rhs: ast::Expression::Integer(1000).into(),
                }
                .into(),
                operator: ast::Operator::Div,
                rhs: ast::Expression::Integer(100).into(),
            }
            .into(),
            operator: ast::Operator::Mul,
            rhs: ast::Expression::BinaryOperation {
                lhs: ast::Expression::Integer(100).into(),
                operator: ast::Operator::Sub,
                rhs: ast::Expression::Integer(10000).into(),
            }
            .into(),
        }
        .into(),
    };

    println!("Expr: {:?}", expr);

    let mut gen = codegen::CodeGenerator {};

    println!("Expr {:#?}", gen.visit_expression(&expr).unwrap());

    //println!("{:?}", new_ast);
}
