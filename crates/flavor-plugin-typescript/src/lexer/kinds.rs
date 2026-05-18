use crate::syntax_kind::TsSyntaxKind;

pub(super) fn keyword_kind(text: &str) -> TsSyntaxKind {
    match text {
        "abstract" => TsSyntaxKind::KeywordAbstract,
        "as" => TsSyntaxKind::KeywordAs,
        "async" => TsSyntaxKind::KeywordAsync,
        "await" => TsSyntaxKind::KeywordAwait,
        "break" => TsSyntaxKind::KeywordBreak,
        "case" => TsSyntaxKind::KeywordCase,
        "catch" => TsSyntaxKind::KeywordCatch,
        "class" => TsSyntaxKind::KeywordClass,
        "const" => TsSyntaxKind::KeywordConst,
        "continue" => TsSyntaxKind::KeywordContinue,
        "declare" => TsSyntaxKind::KeywordDeclare,
        "default" => TsSyntaxKind::KeywordDefault,
        "delete" => TsSyntaxKind::KeywordDelete,
        "do" => TsSyntaxKind::KeywordDo,
        "else" => TsSyntaxKind::KeywordElse,
        "enum" => TsSyntaxKind::KeywordEnum,
        "export" => TsSyntaxKind::KeywordExport,
        "extends" => TsSyntaxKind::KeywordExtends,
        "false" => TsSyntaxKind::KeywordFalse,
        "finally" => TsSyntaxKind::KeywordFinally,
        "for" => TsSyntaxKind::KeywordFor,
        "from" => TsSyntaxKind::KeywordFrom,
        "function" => TsSyntaxKind::KeywordFunction,
        "get" => TsSyntaxKind::KeywordGet,
        "if" => TsSyntaxKind::KeywordIf,
        "implements" => TsSyntaxKind::KeywordImplements,
        "infer" => TsSyntaxKind::KeywordInfer,
        "in" => TsSyntaxKind::KeywordIn,
        "instanceof" => TsSyntaxKind::KeywordInstanceof,
        "import" => TsSyntaxKind::KeywordImport,
        "interface" => TsSyntaxKind::KeywordInterface,
        "keyof" => TsSyntaxKind::KeywordKeyof,
        "let" => TsSyntaxKind::KeywordLet,
        "module" => TsSyntaxKind::KeywordModule,
        "namespace" => TsSyntaxKind::KeywordNamespace,
        "new" => TsSyntaxKind::KeywordNew,
        "null" => TsSyntaxKind::KeywordNull,
        "override" => TsSyntaxKind::KeywordOverride,
        "of" => TsSyntaxKind::KeywordOf,
        "private" => TsSyntaxKind::KeywordPrivate,
        "protected" => TsSyntaxKind::KeywordProtected,
        "public" => TsSyntaxKind::KeywordPublic,
        "readonly" => TsSyntaxKind::KeywordReadonly,
        "return" => TsSyntaxKind::KeywordReturn,
        "satisfies" => TsSyntaxKind::KeywordSatisfies,
        "set" => TsSyntaxKind::KeywordSet,
        "static" => TsSyntaxKind::KeywordStatic,
        "super" => TsSyntaxKind::KeywordSuper,
        "switch" => TsSyntaxKind::KeywordSwitch,
        "this" => TsSyntaxKind::KeywordThis,
        "throw" => TsSyntaxKind::KeywordThrow,
        "true" => TsSyntaxKind::KeywordTrue,
        "try" => TsSyntaxKind::KeywordTry,
        "type" => TsSyntaxKind::KeywordType,
        "typeof" => TsSyntaxKind::KeywordTypeof,
        "unique" => TsSyntaxKind::KeywordUnique,
        "void" => TsSyntaxKind::KeywordVoid,
        "while" => TsSyntaxKind::KeywordWhile,
        "yield" => TsSyntaxKind::KeywordYield,
        _ => TsSyntaxKind::Identifier,
    }
}

pub(super) fn punctuators() -> &'static [(&'static str, TsSyntaxKind)] {
    &[
        ("===", TsSyntaxKind::EqualsEqualsEquals),
        ("!==", TsSyntaxKind::BangEqualsEquals),
        ("...", TsSyntaxKind::DotDotDot),
        ("=>", TsSyntaxKind::Arrow),
        ("==", TsSyntaxKind::EqualsEquals),
        ("!=", TsSyntaxKind::BangEquals),
        ("<=", TsSyntaxKind::LessThanEquals),
        (">=", TsSyntaxKind::GreaterThanEquals),
        ("++", TsSyntaxKind::PlusPlus),
        ("--", TsSyntaxKind::MinusMinus),
        ("+=", TsSyntaxKind::PlusEquals),
        ("-=", TsSyntaxKind::MinusEquals),
        ("*=", TsSyntaxKind::StarEquals),
        ("/=", TsSyntaxKind::SlashEquals),
        ("%=", TsSyntaxKind::PercentEquals),
        ("&&", TsSyntaxKind::AmpersandAmpersand),
        ("||", TsSyntaxKind::PipePipe),
        ("??", TsSyntaxKind::QuestionQuestion),
        ("?.", TsSyntaxKind::QuestionDot),
    ]
}

pub(super) fn is_regex_prefix(kind: TsSyntaxKind) -> bool {
    matches!(
        kind,
        TsSyntaxKind::OpenParen
            | TsSyntaxKind::OpenBrace
            | TsSyntaxKind::OpenBracket
            | TsSyntaxKind::Equals
            | TsSyntaxKind::Comma
            | TsSyntaxKind::Colon
            | TsSyntaxKind::Semicolon
            | TsSyntaxKind::Question
            | TsSyntaxKind::Bang
            | TsSyntaxKind::Arrow
            | TsSyntaxKind::KeywordReturn
            | TsSyntaxKind::KeywordThrow
            | TsSyntaxKind::KeywordCase
            | TsSyntaxKind::KeywordAwait
            | TsSyntaxKind::KeywordDelete
            | TsSyntaxKind::KeywordInstanceof
            | TsSyntaxKind::KeywordTypeof
            | TsSyntaxKind::KeywordVoid
            | TsSyntaxKind::KeywordYield
            | TsSyntaxKind::Plus
            | TsSyntaxKind::Minus
            | TsSyntaxKind::Star
            | TsSyntaxKind::Slash
            | TsSyntaxKind::Percent
            | TsSyntaxKind::Pipe
            | TsSyntaxKind::Ampersand
            | TsSyntaxKind::AmpersandAmpersand
            | TsSyntaxKind::PipePipe
            | TsSyntaxKind::QuestionQuestion
            | TsSyntaxKind::EqualsEquals
            | TsSyntaxKind::EqualsEqualsEquals
            | TsSyntaxKind::BangEquals
            | TsSyntaxKind::BangEqualsEquals
            | TsSyntaxKind::LessThanEquals
            | TsSyntaxKind::GreaterThan
            | TsSyntaxKind::GreaterThanEquals
            | TsSyntaxKind::PlusEquals
            | TsSyntaxKind::MinusEquals
            | TsSyntaxKind::StarEquals
            | TsSyntaxKind::SlashEquals
            | TsSyntaxKind::PercentEquals
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
