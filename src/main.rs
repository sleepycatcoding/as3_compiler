use crate::ast::Folder;
use lalrpop_util::lalrpop_mod;

mod ast;
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

    let mut optimizer = optimizer::AstOptimizer::default();

    let new_ast = optimizer.fold_package(ast);

    println!("{:?}", new_ast);
}
