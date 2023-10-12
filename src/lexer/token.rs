use logos::{Lexer, Logos, Skip};

pub fn string_literal<'a, T: Logos<'a, Source = str>>(lex: &mut Lexer<'a, T>) -> Option<String> {
    let slice = lex.slice();

    // Remove double quotes. (left from parsing)
    slice[1..slice.len() - 1].parse().ok()
}

/// Update the line count and the char index.
fn newline_callback<'a, T: Logos<'a, Source = str, Extras = (usize, usize)>>(
    lex: &mut Lexer<'a, T>,
) -> Skip {
    lex.extras.0 += 1;
    lex.extras.1 = lex.span().end;
    Skip
}

pub mod asm {
    use super::{newline_callback, string_literal};
    use logos::{Lexer, Logos};
    use std::fmt;

    fn local_callback(lexer: &mut Lexer<Token>) -> Option<u32> {
        // First we remove Get/SetLocal from the token, then parse the rest as a number.
        let slice = &lexer.slice()[8..];

        // FIXME: Insert debug messages (easier for debugging).
        slice.parse().ok()
    }

    /// ActionScript assembly token.
    #[derive(Clone, Debug, Logos)]
    // Slash comments.
    #[logos(skip r"//[^\n]*")]
    // Block comments.
    #[logos(skip r"/\*(?:[^*]|\*[^/])*\*/")]
    #[logos(skip " ")]
    // Store current line and column for easier debugging.
    #[logos(extras = (usize, usize))]
    pub enum Token {
        #[token("function")]
        KeywordFunction,

        // Control Flow operations.
        #[token("returnvalue")]
        OpReturnValue,
        #[token("returnvoid")]
        OpReturnVoid,
        #[token("iftrue")]
        OpIfTrue,
        #[token("iffalse")]
        OpIfFalse,
        #[token("ifeq")]
        OpIfEq,

        #[regex("getlocal\\d", callback = local_callback)]
        OpGetLocal(u32),
        #[regex("setlocal\\d", callback = local_callback)]
        OpSetLocal(u32),

        // Push operations.
        #[token("pushstring")]
        OpPushString,
        #[token("pushfalse")]
        OpPushFalse,
        #[token("pushnull")]
        OpPushNull,
        #[token("pushnamespace")]
        OpPushNamespace,
        #[token("pushscope")]
        OpPushScope,

        #[token("pop")]
        OpPop,

        // Coercion operations.
        #[token("coerce_a")]
        OpCoerceA,
        #[token("coerce_s")]
        OpCoerceS,

        #[token("findproperty")]
        OpFindProperty,
        #[token("callpropvoid")]
        OpCallPropVoid,

        // New operations.
        #[token("newfunction")]
        OpNewFunction,

        #[token(".function_id")]
        IdFunction,

        #[regex("[_a-zA-Z][_0-9a-zA-Z]*", priority = 2, callback = |lex| lex.slice().parse().ok())]
        Identifier(String),
        #[regex("\\d+", |lex| lex.slice().parse().ok())]
        Integer(i32),
        #[regex(r#""(?:[^"]|\\")*""#, callback = string_literal)]
        String(String),

        #[token("(")]
        LParen,
        #[token(")")]
        RParen,
        #[token("{")]
        LCurlyBracket,
        #[token("}")]
        RCurlyBracket,
        #[token(",")]
        Comma,
        #[token(":")]
        Colon,

        #[regex(r"\n", newline_callback)]
        Newline,
    }

    impl fmt::Display for Token {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }
}

pub mod as3 {
    use super::{newline_callback, string_literal};
    use crate::ast::Visibility;
    use logos::Logos;
    use std::fmt;

    /// ActionScript 3 token.
    // NOTE: Useful regexes https://github.com/maciejhirsz/logos/issues/133
    #[derive(Clone, Debug, Logos)]
    // FIXME: Comments currently do not count towards line numbers.
    // Slash comments.
    #[logos(skip r"//[^\n]*")]
    // Block comments.
    #[logos(skip r"/\*(?:[^*]|\*[^/])*\*/")]
    #[logos(skip " ")]
    // Store current line and column for easier debugging.
    #[logos(extras = (usize, usize))]
    pub enum Token {
        #[token("var")]
        KeywordVar,
        #[token("package")]
        KeywordPackage,
        #[token("class")]
        KeywordClass,
        #[token("function")]
        KeywordFunction,

        #[regex("(public|protected|private)", callback = |lex| lex.slice().parse().ok())]
        KeywordVisibility(Visibility),

        #[regex("(true|false)", callback = |lex| lex.slice().parse().ok())]
        Bool(bool),
        #[regex("[_a-zA-Z][_0-9a-zA-Z]*", priority = 2, callback = |lex| lex.slice().parse().ok())]
        Identifier(String),
        #[regex("\\d+", |lex| lex.slice().parse().ok())]
        Integer(i32),
        #[regex(r#""(?:[^"]|\\")*""#, callback = string_literal)]
        String(String),

        #[token("(")]
        LParen,
        #[token(")")]
        RParen,
        #[token("{")]
        LCurlyBracket,
        #[token("}")]
        RCurlyBracket,
        #[token("=")]
        Assign,
        #[token(",")]
        Comma,
        #[token(":")]
        Colon,
        #[token(";")]
        Semicolon,

        #[token("+")]
        OperatorAdd,
        #[token("-")]
        OperatorSub,
        #[token("*")]
        OperatorMul,
        #[token("/")]
        OperatorDiv,

        #[regex(r"\n", newline_callback)]
        Newline,
    }

    impl fmt::Display for Token {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }
}
