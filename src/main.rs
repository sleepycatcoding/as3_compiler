use crate::ast::Visitor;

mod ast;
mod codegen;
mod lexer;
mod optimizer;
mod parser;

fn main() {
    println!("Hello, world!");

    // let file = std::fs::read_to_string("Test.as").unwrap();
    // let lexer = lexer::Lexer::new(&file);
    // let parser = parser::as3::grammar::PackageParser::new();
    // let ast = parser.parse(lexer).unwrap();

    // println!("{:?}", ast);

    // //let mut optimizer = optimizer::AstOptimizer::default();

    // //let new_ast = optimizer.fold_package(ast);

    // let function = &ast.classes[0].functions[0];

    // println!("Expr: {:?}", function);

    // let mut gen = codegen::CodeGenerator::default();

    // gen.visit_function(function).unwrap();

    // println!("Expr {:#?}", gen);

    let file = std::fs::read_to_string("bug_612641.abs").unwrap();
    let lexer = lexer::Lexer::<crate::lexer::asm::Token>::new(&file);
    let parser = parser::asm::grammar::FunctionsParser::new();

    // for token in lexer {
    //     println!("{:?}", token);
    // }

    let out = parser.parse(lexer).unwrap();

    println!("{:#?}", out);

    //println!("{:?}", new_ast);
}
