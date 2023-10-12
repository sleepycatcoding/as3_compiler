use crate::ast::Visitor;
use clap::Parser;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

mod ast;
mod codegen;
mod lexer;
mod optimizer;
mod parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Opt {
    #[clap(flatten, next_help_heading = "Debug Options")]
    debug: DebugFlags,
}

#[derive(Debug, clap::Args)]
#[group(multiple = false)]
struct DebugFlags {
    /// Prints out tokens of specified file, instead of compiling it.
    #[arg(long)]
    print_tokens: bool,
}

impl DebugFlags {
    /// Is set if any of the debug options are set.
    fn any_set(&self) -> bool {
        self.print_tokens
    }
}

fn main() {
    let args = Opt::parse();

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .finish();
    // Ignore error if it's already been set
    let _ = tracing::subscriber::set_global_default(subscriber);

    let file = std::fs::read_to_string("bug_612641.abs").unwrap();
    let lexer = lexer::Lexer::<crate::lexer::asm::Token>::new(&file);

    if args.debug.any_set() {
        if args.debug.print_tokens {
            for token in lexer {
                tracing::info!("{:?}", token);
            }
        }

        return;
    }

    let parser = parser::asm::grammar::FunctionsParser::new();
    let out = parser.parse(lexer).unwrap();

    println!("{:#?}", out);
}
