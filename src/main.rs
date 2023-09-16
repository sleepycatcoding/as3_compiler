use lalrpop_util::lalrpop_mod;

mod ast;
mod lexer;

lalrpop_mod!(pub grammar);

fn main() {
    println!("Hello, world!");

    let file = std::fs::read_to_string("Test.as").unwrap();
    let lexer = lexer::Lexer::new(&file);
    let parser = grammar::PackageParser::new();
    let ast = parser.parse(lexer).unwrap();

    println!("{:?}", ast);
}
