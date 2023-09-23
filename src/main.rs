use lalrpop_util::lalrpop_mod;

use crate::ast::Visitor;

mod ast;
mod codegen;
mod lexer;
mod optimizer;
mod parser;

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

    let function = &ast.classes[0].functions[0];

    println!("Expr: {:?}", function);

    let mut gen = codegen::CodeGenerator::default();

    gen.visit_function(function).unwrap();

    println!("Expr {:#?}", gen);

    //println!("{:?}", new_ast);
}
