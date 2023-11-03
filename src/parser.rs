pub mod as3;
pub mod asm;
pub mod common;

use std::convert::Infallible;
use std::str::FromStr;

use crate::lexer::{Span, Token};
use chumsky::pratt::*;
use chumsky::prelude::*;

pub type ParserInput<'tokens, 'src> =
    chumsky::input::SpannedInput<Token<'src>, Span, &'tokens [(Token<'src>, Span)]>;

/// A type that has a span attached.
pub type Spanned<T> = (T, Span);

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

#[derive(Clone, Debug)]
pub enum UnaryOp {
    Negate,
}

/// An expression node.
#[derive(Debug)]
pub enum Expr<'src> {
    /// While parsing, an error occurred.
    Error,
    /// A simple value.
    Value(Value<'src>),
    // // A list of expressions, this can be used for function call arguments, arrays and such.
    // List(Vec<Spanned<Self>>),
    /// Expression that references a local variable.
    Local(&'src str),

    //Call,
    /// A binary operation. (A operation which has two sides).
    Binary(Box<Self>, BinaryOp, Box<Self>),
    /// An unary operation.
    Unary(UnaryOp, Box<Expr<'src>>),
}

/// A statement node.
#[derive(Debug)]
pub enum Stmt<'src> {
    Var(Spanned<TypedIdent<'src>>, Spanned<Expr<'src>>),
    If {
        cond: Box<Spanned<Expr<'src>>>,
        body: Vec<Spanned<Expr<'src>>>,
        // /// This if statements else block. It may or may not exist.
        // else_block: Option<Box<Spanned<Self>>>,
    },
}

#[derive(Debug)]
pub enum Type {
    Any,
    Int,
    Uint,
    Number,
    Bool,
    String,
    // FIXME: Make this a reference?
    Other(String),
}

impl FromStr for Type {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "int" => Ok(Type::Int),
            "uint" => Ok(Type::Uint),
            "Number" => Ok(Type::Number),
            "Boolean" => Ok(Type::Bool),
            "String" => Ok(Type::String),
            x => Ok(Type::Other(x.to_owned())),
        }
    }
}

#[derive(Debug)]
pub struct TypedIdent<'src> {
    pub name: &'src str,
    /// Type of the identifier. If this is [`None`] then that means no type was provided.
    pub ty: Option<Type>,
}

#[derive(Debug)]
pub struct Fn<'src> {
    pub name: &'src str,
    pub args: Vec<Spanned<TypedIdent<'src>>>,
    pub body: Vec<Spanned<Stmt<'src>>>,
}

pub fn typed_ident_parser<'tokens, 'src: 'tokens>() -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    Spanned<TypedIdent<'src>>,
    extra::Err<Rich<'tokens, Token<'src>, Span>>,
> + Clone {
    // FIXME: Deduplicate identifier parser, this is also in expr_parser.
    let ident = select! { Token::Ident(ident) => ident }.labelled("identifier");
    let ty_parser = ident.map(|x| x.parse::<Type>().unwrap());

    let typed_ident = ident
        .then_ignore(just(Token::Ctrl(':')))
        .then(ty_parser)
        .map(|(name, ty)| TypedIdent { name, ty: Some(ty) })
        .map_with(|expr, e| (expr, e.span()));

    let implicit_any = ident
        .map(|name| TypedIdent { name, ty: None })
        .map_with(|expr, e| (expr, e.span()));

    typed_ident.or(implicit_any).labelled("typed identifier")
}

pub fn expr_parser<'tokens, 'src: 'tokens>() -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    Spanned<Expr<'src>>,
    extra::Err<Rich<'tokens, Token<'src>, Span>>,
> + Clone {
    let value = select! {
        Token::Null => Expr::Value(Value::Null),
        Token::Undefined => Expr::Value(Value::Undefined),
        Token::Bool(x) => Expr::Value(Value::Bool(x)),
        Token::Int(x) => Expr::Value(Value::Int(x)),
        Token::Str(x) => Expr::Value(Value::Str(x)),
    }
    .labelled("value");

    // FIXME: deduplicate
    let ident = select! { Token::Ident(ident) => ident }.labelled("identifier");
    let op = |x| just(Token::Op(x));

    let inline_expr = recursive(|expr| {
        let atom = value
            .or(ident.map(Expr::Local))
            // TODO: Would be nice if we preserved span info here, but oh well, pratt parsing does not like it.
            // Atoms can also just be normal expressions, but surrounded with parentheses
            .or(expr
                .clone()
                .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')'))))
            .boxed();

        let pratt = atom.pratt((
            // Negation.
            prefix(3, op("-"), |rhs| {
                Expr::Unary(UnaryOp::Negate, Box::new(rhs))
            }),
            infix(left(2), op("*"), |l, r| {
                Expr::Binary(Box::new(l), BinaryOp::Mul, Box::new(r))
            }),
            infix(left(2), op("/"), |l, r| {
                Expr::Binary(Box::new(l), BinaryOp::Div, Box::new(r))
            }),
            // - and + bind the weakest, meaning that even if they occur earlier they are last executed.
            infix(left(1), op("+"), |l, r| {
                Expr::Binary(Box::new(l), BinaryOp::Add, Box::new(r))
            }),
            infix(left(1), op("-"), |l, r| {
                Expr::Binary(Box::new(l), BinaryOp::Sub, Box::new(r))
            }),
        ));

        pratt.labelled("expression").as_context()
    });

    inline_expr.map_with(|expr, e| (expr, e.span()))
}

pub fn stmt_parser<'tokens, 'src: 'tokens>() -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    Spanned<Stmt<'src>>,
    extra::Err<Rich<'tokens, Token<'src>, Span>>,
> + Clone {
    // FIXME: Deduplicate identifier parser, this is also in expr_parser.
    let typed_identifier = typed_ident_parser();
    // Expression parser.
    let expr = expr_parser();

    let var = just(Token::Var)
        .ignore_then(typed_identifier)
        .then_ignore(just(Token::Op("=")))
        .then(expr)
        .then_ignore(just(Token::Ctrl(';')))
        .map(|(name, val)| Stmt::Var(name, val))
        .map_with(|expr, e| (expr, e.span()));

    var
}
pub fn fn_parser<'tokens, 'src: 'tokens>() -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    Spanned<Fn<'src>>,
    extra::Err<Rich<'tokens, Token<'src>, Span>>,
> + Clone {
    // FIXME: Deduplicate identifier parser, this is also in expr_parser.
    let ident = select! { Token::Ident(ident) => ident }.labelled("identifier");
    let typed_identifier = typed_ident_parser();
    let stmt = stmt_parser();

    // Argument lists are just identifiers separated by commas, surrounded by parentheses
    let args = typed_identifier
        .separated_by(just(Token::Ctrl(',')))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
        .labelled("function args");

    // Block parser.
    let block = stmt
        .clone()
        .repeated()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}')))
        .labelled("block");

    // FIXME: Add return types.
    let function_parser = just(Token::Fn)
        .ignore_then(ident)
        .then(args)
        .then(block)
        .map(|((name, args), body)| Fn { name, args, body })
        .map_with(|expr, e| (expr, e.span()));

    function_parser
}
