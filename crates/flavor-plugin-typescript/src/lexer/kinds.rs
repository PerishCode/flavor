use crate::internal::grammar::{self as kind, Kind};

pub(super) fn keyword_kind(text: &str) -> Kind {
    match text {
        "abstract" => kind::KEYWORD_ABSTRACT,
        "as" => kind::KEYWORD_AS,
        "async" => kind::KEYWORD_ASYNC,
        "await" => kind::KEYWORD_AWAIT,
        "break" => kind::KEYWORD_BREAK,
        "case" => kind::KEYWORD_CASE,
        "catch" => kind::KEYWORD_CATCH,
        "class" => kind::KEYWORD_CLASS,
        "const" => kind::KEYWORD_CONST,
        "continue" => kind::KEYWORD_CONTINUE,
        "declare" => kind::KEYWORD_DECLARE,
        "default" => kind::KEYWORD_DEFAULT,
        "delete" => kind::KEYWORD_DELETE,
        "do" => kind::KEYWORD_DO,
        "else" => kind::KEYWORD_ELSE,
        "enum" => kind::KEYWORD_ENUM,
        "export" => kind::KEYWORD_EXPORT,
        "extends" => kind::KEYWORD_EXTENDS,
        "false" => kind::KEYWORD_FALSE,
        "finally" => kind::KEYWORD_FINALLY,
        "for" => kind::KEYWORD_FOR,
        "from" => kind::KEYWORD_FROM,
        "function" => kind::KEYWORD_FUNCTION,
        "get" => kind::KEYWORD_GET,
        "if" => kind::KEYWORD_IF,
        "implements" => kind::KEYWORD_IMPLEMENTS,
        "infer" => kind::KEYWORD_INFER,
        "in" => kind::KEYWORD_IN,
        "instanceof" => kind::KEYWORD_INSTANCEOF,
        "import" => kind::KEYWORD_IMPORT,
        "interface" => kind::KEYWORD_INTERFACE,
        "keyof" => kind::KEYWORD_KEYOF,
        "let" => kind::KEYWORD_LET,
        "module" => kind::KEYWORD_MODULE,
        "namespace" => kind::KEYWORD_NAMESPACE,
        "new" => kind::KEYWORD_NEW,
        "null" => kind::KEYWORD_NULL,
        "override" => kind::KEYWORD_OVERRIDE,
        "of" => kind::KEYWORD_OF,
        "private" => kind::KEYWORD_PRIVATE,
        "protected" => kind::KEYWORD_PROTECTED,
        "public" => kind::KEYWORD_PUBLIC,
        "readonly" => kind::KEYWORD_READONLY,
        "return" => kind::KEYWORD_RETURN,
        "satisfies" => kind::KEYWORD_SATISFIES,
        "set" => kind::KEYWORD_SET,
        "static" => kind::KEYWORD_STATIC,
        "super" => kind::KEYWORD_SUPER,
        "switch" => kind::KEYWORD_SWITCH,
        "this" => kind::KEYWORD_THIS,
        "throw" => kind::KEYWORD_THROW,
        "true" => kind::KEYWORD_TRUE,
        "try" => kind::KEYWORD_TRY,
        "type" => kind::KEYWORD_TYPE,
        "typeof" => kind::KEYWORD_TYPEOF,
        "unique" => kind::KEYWORD_UNIQUE,
        "void" => kind::KEYWORD_VOID,
        "while" => kind::KEYWORD_WHILE,
        "yield" => kind::KEYWORD_YIELD,
        _ => kind::IDENTIFIER,
    }
}

pub(super) fn punctuators() -> &'static [(&'static str, Kind)] {
    &[
        ("===", kind::EQUALS_EQUALS_EQUALS),
        ("!==", kind::BANG_EQUALS_EQUALS),
        ("...", kind::DOT_DOT_DOT),
        ("=>", kind::ARROW),
        ("==", kind::EQUALS_EQUALS),
        ("!=", kind::BANG_EQUALS),
        ("<=", kind::LESS_THAN_EQUALS),
        (">=", kind::GREATER_THAN_EQUALS),
        ("++", kind::PLUS_PLUS),
        ("--", kind::MINUS_MINUS),
        ("+=", kind::PLUS_EQUALS),
        ("-=", kind::MINUS_EQUALS),
        ("*=", kind::STAR_EQUALS),
        ("/=", kind::SLASH_EQUALS),
        ("%=", kind::PERCENT_EQUALS),
        ("&&", kind::AMPERSAND_AMPERSAND),
        ("||", kind::PIPE_PIPE),
        ("??", kind::QUESTION_QUESTION),
        ("?.", kind::QUESTION_DOT),
    ]
}

pub(super) fn is_regex_prefix(kind: Kind) -> bool {
    matches!(
        kind,
        kind::OPEN_PAREN
            | kind::OPEN_BRACE
            | kind::OPEN_BRACKET
            | kind::EQUALS
            | kind::COMMA
            | kind::COLON
            | kind::SEMICOLON
            | kind::QUESTION
            | kind::BANG
            | kind::ARROW
            | kind::KEYWORD_RETURN
            | kind::KEYWORD_THROW
            | kind::KEYWORD_CASE
            | kind::KEYWORD_AWAIT
            | kind::KEYWORD_DELETE
            | kind::KEYWORD_INSTANCEOF
            | kind::KEYWORD_TYPEOF
            | kind::KEYWORD_VOID
            | kind::KEYWORD_YIELD
            | kind::PLUS
            | kind::MINUS
            | kind::STAR
            | kind::SLASH
            | kind::PERCENT
            | kind::PIPE
            | kind::AMPERSAND
            | kind::AMPERSAND_AMPERSAND
            | kind::PIPE_PIPE
            | kind::QUESTION_QUESTION
            | kind::EQUALS_EQUALS
            | kind::EQUALS_EQUALS_EQUALS
            | kind::BANG_EQUALS
            | kind::BANG_EQUALS_EQUALS
            | kind::LESS_THAN_EQUALS
            | kind::GREATER_THAN
            | kind::GREATER_THAN_EQUALS
            | kind::PLUS_EQUALS
            | kind::MINUS_EQUALS
            | kind::STAR_EQUALS
            | kind::SLASH_EQUALS
            | kind::PERCENT_EQUALS
    )
}

pub(super) fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '$'
}

pub(super) fn is_identifier_part(ch: char) -> bool {
    is_identifier_start(ch) || ch.is_ascii_digit()
}

pub(super) fn is_hex_digit(ch: char) -> bool {
    ch.is_ascii_hexdigit()
}

pub(super) fn is_bin_digit(ch: char) -> bool {
    matches!(ch, '0' | '1')
}

pub(super) fn is_oct_digit(ch: char) -> bool {
    matches!(ch, '0'..='7')
}

pub(super) fn is_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\n' | '\r' | '\t')
}
