use std::fmt::write;

use ariadne::{sources, Color, Label, Report, ReportKind};
use chumsky::prelude::*;

pub type Span = SimpleSpan<usize>;

pub struct Error {
    span: Span,
    msg: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value<'src> {
    Null,
    Undefined,
    Bool(bool),
    Int(i32),
    Str(&'src str),
    Array(Vec<Self>),
    // FIXME: TODO
    //Func(&'src str),
}

impl<'src> Value<'src> {
    pub fn int(self, span: Span) -> Result<i32, Error> {
        if let Value::Int(x) = self {
            Ok(x)
        } else {
            Err(Error {
                span,
                msg: format!("'{}' is not a integer", self),
            })
        }
    }
}

impl<'src> std::fmt::Display for Value<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Undefined => write!(f, "undefined"),
            Self::Bool(x) => write!(f, "{}", x),
            Self::Int(x) => write!(f, "{}", x),
            Self::Str(x) => write!(f, "{}", x),
            Self::Array(xs) => write!(
                f,
                "[{}]",
                xs.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
}

/// A type that has a span attached.
pub type Spanned<T> = (T, Span);

/// An expression node.
#[derive(Debug)]
pub enum Expr<'src> {
    Error,
    Value(Value<'src>),
    // A list of expressions, this can be used for function call arguments, arrays and such.
    List(Vec<Spanned<Self>>),
    Local(&'src str),
    Var(&'src str, Box<Spanned<Self>>, Box<Spanned<Self>>),
    // FIXME: Remove this and use a vec instead.
    // If second value is Some that means this is a block that has a next expression.
    Then(Box<Spanned<Self>>, Option<Box<Spanned<Self>>>),
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
    //Call,
    If {
        cond: Box<Spanned<Self>>,
        body: Box<Spanned<Self>>,
        /// This if statements else block. It may or may not exist.
        else_block: Option<Box<Spanned<Self>>>,
    },
}

/// A function node.
#[derive(Debug)]
pub struct Func<'src> {
    // FIXME: Replace with a more concrete type.
    args: Vec<&'src str>,
    span: Span,
    body: Spanned<Expr<'src>>,
}

use crate::lexer::Token;

pub type ParserInput<'tokens, 'src> =
    chumsky::input::SpannedInput<Token<'src>, Span, &'tokens [(Token<'src>, Span)]>;

pub fn expr_parser<'tokens, 'src: 'tokens>() -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    Spanned<Expr<'src>>,
    extra::Err<Rich<'tokens, Token<'src>, Span>>,
> + Clone {
    recursive(|expr| {
        let inline_expr = recursive(|inline_expr| {
            let val = select! {
                Token::Null => Expr::Value(Value::Null),
                Token::Undefined => Expr::Value(Value::Undefined),
                Token::Bool(x) => Expr::Value(Value::Bool(x)),
                Token::Int(x) => Expr::Value(Value::Int(x)),
                Token::Str(x) => Expr::Value(Value::Str(x)),
            };
            //.labelled("value");

            let ident = select! { Token::Ident(ident) => ident };

            // A var expression.
            let var = just(Token::Var)
                .ignore_then(ident)
                .then_ignore(just(Token::Op("=")))
                .then(inline_expr)
                .then_ignore(just(Token::Ctrl(';')))
                .then(expr.clone())
                .map(|((name, val), body)| Expr::Var(name, Box::new(val), Box::new(body)));

            // // A list of expressions
            // let items = expr
            //     .clone()
            //     .separated_by(just(Token::Ctrl(',')))
            //     .allow_trailing()
            //     .collect::<Vec<_>>();

            // let list = items.clone().map(Expr)

            // 'Atoms' are expressions that contain no ambiguity
            let atom = val
                .or(ident.map(Expr::Local))
                .or(var)
                .map_with(|expr, e| (expr, e.span()))
                .or(expr
                    .clone()
                    // Atoms can also just be normal expressions, but surrounded with parentheses
                    .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')'))))
                .recover_with(via_parser(nested_delimiters(
                    Token::Ctrl('('),
                    Token::Ctrl(')'),
                    [
                        (Token::Ctrl('['), Token::Ctrl(']')),
                        (Token::Ctrl('{'), Token::Ctrl('}')),
                    ],
                    |span| (Expr::Error, span),
                )))
                .boxed();

            // FIXME: Function calls.

            // Product ops (multiply and divide) have equal precedence
            let op = just(Token::Op("*"))
                .to(BinaryOp::Mul)
                .or(just(Token::Op("/")).to(BinaryOp::Div));
            let product = atom
                .clone()
                .foldl_with(op.then(atom).repeated(), |a, (op, b), e| {
                    (Expr::Binary(Box::new(a), op, Box::new(b)), e.span())
                });

            // Sum ops (add and subtract) have equal precedence
            let op = just(Token::Op("+"))
                .to(BinaryOp::Add)
                .or(just(Token::Op("-")).to(BinaryOp::Sub));
            let sum = product
                .clone()
                .foldl_with(op.then(product).repeated(), |a, (op, b), e| {
                    (Expr::Binary(Box::new(a), op, Box::new(b)), e.span())
                });

            // Comparison ops (equal, not-equal) have equal precedence
            let op = just(Token::Op("=="))
                .to(BinaryOp::Eq)
                .or(just(Token::Op("!=")).to(BinaryOp::NotEq));
            let compare = sum
                .clone()
                .foldl_with(op.then(sum).repeated(), |a, (op, b), e| {
                    (Expr::Binary(Box::new(a), op, Box::new(b)), e.span())
                });

            compare
        });

        let block = expr
            .clone()
            .delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}'))) // Attempt to recover anything that looks like a block but contains errors
            .recover_with(via_parser(nested_delimiters(
                Token::Ctrl('{'),
                Token::Ctrl('}'),
                [
                    (Token::Ctrl('('), Token::Ctrl(')')),
                    (Token::Ctrl('['), Token::Ctrl(']')),
                ],
                |span| (Expr::Error, span),
            )));

        let if_ = recursive(|if_| {
            just(Token::If)
                .then_ignore(just(Token::Ctrl('(')))
                .ignore_then(expr.clone())
                .then_ignore(just(Token::Ctrl(')')))
                .then(block.clone())
                .then(
                    just(Token::Else)
                        .ignore_then(block.clone().or(if_))
                        .or_not(),
                )
                .map_with(|((cond, a), b), e| {
                    (
                        Expr::If {
                            cond: Box::new(cond),
                            body: Box::new(a),
                            else_block: b.map(Box::new),
                        },
                        e.span(),
                    )
                })
        });

        if_.or(inline_expr.clone())
            .recover_with(skip_then_retry_until(
                any().ignored(),
                one_of([
                    Token::Ctrl(';'),
                    Token::Ctrl('}'),
                    Token::Ctrl(')'),
                    Token::Ctrl(']'),
                ])
                .ignored(),
            ))
            .foldl_with(
                just(Token::Ctrl(';')).ignore_then(expr.or_not()).repeated(),
                |a, b, e| {
                    let span: Span = e.span();
                    (Expr::Then(Box::new(a), b.map(Box::new)), span)
                },
            )
    })
}
