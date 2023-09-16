use lalrpop_util::lalrpop_mod;

mod ast;
mod lexer;

lalrpop_mod!(pub grammar);

fn main() {
    println!("Hello, world!");

    println!(
        "{:#?}",
        grammar::ExprParser::new()
            .parse("(10 + 10 * (100 * 100))")
            .unwrap()
    );

    grammar::FileParser::new()
        .parse(include_str!("../Test.as"))
        .unwrap();
}
