use crate::ast::Visitor;
use ariadne::{sources, Color, Label, Report, ReportKind};
use chumsky::{prelude::Input, Parser};
//use clap::Parser;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

mod ast;
mod codegen;
mod lexer;
mod optimizer;
mod parser;
mod rewrite;

#[derive(Debug, clap::Parser)]
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
    //let args = Opt::parse();

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .finish();
    // Ignore error if it's already been set
    let _ = tracing::subscriber::set_global_default(subscriber);

    // let file = std::fs::read_to_string("bug_612641.abs").unwrap();
    // let lexer = lexer::Lexer::<crate::lexer::asm::Token>::new(&file);

    // if args.debug.any_set() {
    //     if args.debug.print_tokens {
    //         for token in lexer {
    //             tracing::info!("{:?}", token);
    //         }
    //     }

    //     return;
    // }

    // let parser = parser::asm::grammar::FunctionsParser::new();
    // let out = parser.parse(lexer).unwrap();

    // println!("{:#?}", out);

    let file = std::fs::read_to_string("test1.as").unwrap();

    let (tokens, mut errs) = rewrite::lexer().parse(file.as_str()).into_output_errors();

    let parse_errs = if let Some(tokens) = &tokens {
        let (ast, parse_errs) = rewrite::expr_parser()
            .map_with(|ast, e| (ast, e.span()))
            .parse(
                tokens
                    .as_slice()
                    .spanned((file.len()..file.len()).into())
                    .into(),
            )
            .into_output_errors();

        if let Some(v) = &ast {
            tracing::info!("{:#?}", v);
        }

        parse_errs
    } else {
        Vec::new()
    };

    errs.into_iter()
        .map(|e| e.map_token(|c| c.to_string()))
        .chain(
            parse_errs
                .into_iter()
                .map(|e| e.map_token(|tok| tok.to_string())),
        )
        .for_each(|e| {
            Report::build(ReportKind::Error, "TODO", e.span().start)
                .with_message(e.to_string())
                .with_label(
                    Label::new(("TODO".clone(), e.span().into_range()))
                        .with_message(e.reason().to_string())
                        .with_color(Color::Red),
                )
                .finish()
                .print(sources([("TODO".clone(), file.clone())]))
                .unwrap()
        })
}
