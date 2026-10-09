//! Realce de sintaxe próprio (100% Rust, sem dependências de C).
//!
//! Um lexer determinístico por linguagem cobre a maioria dos casos:
//! comentários (linha e bloco), strings (simples, triplas e template),
//! números, palavras-chave, builtins, tipos, funções, macros, operadores
//! e pontuação. Produz, para cada linha, uma lista de [`Span`]s contíguos
//! que cobre todo o texto (incluindo espaços), prontos para colorir.

use openbrains_common::LanguageId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Plain,
    Comment,
    String,
    Number,
    Keyword,
    Builtin,
    Type,
    Function,
    Macro,
    Operator,
    Punctuation,
    /// Identificador comum.
    Ident,
    /// Título/heading (Markdown).
    Heading,
}

/// Um trecho colorido de uma linha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub kind: TokenKind,
    /// Coluna inicial em *caracteres* dentro da linha.
    pub start: usize,
    /// Comprimento em *caracteres*.
    pub len: usize,
}

/// Especificação de linguagem usada pelo lexer.
pub struct LangSpec {
    pub line_comments: &'static [&'static str],
    pub block_comment: Option<(&'static str, &'static str)>,
    pub allow_backtick: bool,
    pub allow_triple: bool,
    pub string_delims: &'static [char],
    pub keywords: &'static [&'static str],
    pub builtins: &'static [&'static str],
    pub caps_are_types: bool,
    pub macros: bool,
    pub attr_prefixes: &'static [char],
}

fn empty<T>() -> &'static [T] {
    &[]
}

impl LangSpec {
    pub fn for_lang(lang: LanguageId) -> LangSpec {
        match lang {
            LanguageId::Rust => LangSpec {
                line_comments: &["//"],
                block_comment: Some(("/*", "*/")),
                allow_backtick: false,
                allow_triple: false,
                string_delims: &['"'],
                keywords: &[
                    "as",
                    "async",
                    "await",
                    "break",
                    "const",
                    "continue",
                    "crate",
                    "dyn",
                    "else",
                    "enum",
                    "extern",
                    "false",
                    "fn",
                    "for",
                    "if",
                    "impl",
                    "in",
                    "let",
                    "loop",
                    "match",
                    "mod",
                    "move",
                    "mut",
                    "pub",
                    "ref",
                    "return",
                    "self",
                    "Self",
                    "static",
                    "struct",
                    "super",
                    "trait",
                    "true",
                    "type",
                    "unsafe",
                    "use",
                    "where",
                    "while",
                    "macro_rules",
                ],
                builtins: &[
                    "i8",
                    "i16",
                    "i32",
                    "i64",
                    "i128",
                    "isize",
                    "u8",
                    "u16",
                    "u32",
                    "u64",
                    "u128",
                    "usize",
                    "f32",
                    "f64",
                    "bool",
                    "char",
                    "str",
                    "String",
                    "Vec",
                    "Box",
                    "Option",
                    "Result",
                    "Some",
                    "None",
                    "Ok",
                    "Err",
                    "println",
                    "print",
                    "eprintln",
                    "eprint",
                    "write",
                    "writeln",
                    "format",
                    "vec",
                    "panic",
                    "assert",
                    "assert_eq",
                    "assert_ne",
                    "matches",
                    "dbg",
                    "todo",
                    "unimplemented",
                    "derive",
                    "Copy",
                    "Clone",
                    "Debug",
                    "Default",
                    "Iterator",
                    "IntoIterator",
                    "Send",
                    "Sync",
                ],
                caps_are_types: true,
                macros: true,
                attr_prefixes: &['#'],
            },
            LanguageId::Python => LangSpec {
                line_comments: &["#"],
                block_comment: None,
                allow_backtick: false,
                allow_triple: true,
                string_delims: &['"', '\''],
                keywords: &[
                    "and", "as", "assert", "async", "await", "break", "class", "continue", "def",
                    "del", "elif", "else", "except", "finally", "for", "from", "global", "if",
                    "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise",
                    "return", "try", "while", "with", "yield", "match", "case", "True", "False",
                    "None",
                ],
                builtins: &[
                    "print",
                    "len",
                    "range",
                    "list",
                    "dict",
                    "set",
                    "tuple",
                    "int",
                    "float",
                    "str",
                    "bool",
                    "open",
                    "input",
                    "sum",
                    "min",
                    "max",
                    "abs",
                    "sorted",
                    "enumerate",
                    "zip",
                    "map",
                    "filter",
                    "type",
                    "isinstance",
                    "issubclass",
                    "super",
                    "hasattr",
                    "getattr",
                    "setattr",
                    "delattr",
                    "repr",
                    "format",
                    "any",
                    "all",
                    "round",
                    "self",
                    "cls",
                ],
                caps_are_types: false,
                macros: false,
                attr_prefixes: &['@'],
            },
            LanguageId::JavaScript
            | LanguageId::TypeScript
            | LanguageId::JavaScriptReact
            | LanguageId::TypeScriptReact => LangSpec {
                line_comments: &["//"],
                block_comment: Some(("/*", "*/")),
                allow_backtick: true,
                allow_triple: false,
                string_delims: &['"', '\''],
                keywords: &[
                    "var",
                    "let",
                    "const",
                    "function",
                    "return",
                    "if",
                    "else",
                    "for",
                    "while",
                    "do",
                    "switch",
                    "case",
                    "default",
                    "break",
                    "continue",
                    "new",
                    "delete",
                    "typeof",
                    "instanceof",
                    "in",
                    "of",
                    "void",
                    "class",
                    "extends",
                    "super",
                    "this",
                    "null",
                    "undefined",
                    "true",
                    "false",
                    "async",
                    "await",
                    "import",
                    "export",
                    "from",
                    "as",
                    "static",
                    "get",
                    "set",
                    "try",
                    "catch",
                    "finally",
                    "throw",
                    "yield",
                    "interface",
                    "type",
                    "enum",
                    "implements",
                    "readonly",
                    "namespace",
                    "declare",
                    "keyof",
                    "infer",
                    "satisfies",
                    "abstract",
                    "private",
                    "protected",
                    "public",
                    "is",
                ],
                builtins: &[
                    "console",
                    "Math",
                    "JSON",
                    "Object",
                    "Array",
                    "String",
                    "Number",
                    "Boolean",
                    "Promise",
                    "Map",
                    "Set",
                    "WeakMap",
                    "Date",
                    "RegExp",
                    "Error",
                    "Symbol",
                    "window",
                    "document",
                    "globalThis",
                    "process",
                    "require",
                    "module",
                    "exports",
                    "parseInt",
                    "parseFloat",
                    "isNaN",
                    "setTimeout",
                    "setInterval",
                    "fetch",
                    "string",
                    "number",
                    "boolean",
                    "any",
                    "unknown",
                    "never",
                    "object",
                    "bigint",
                ],
                caps_are_types: true,
                macros: false,
                attr_prefixes: &['@'],
            },
            LanguageId::C => LangSpec {
                line_comments: &["//"],
                block_comment: Some(("/*", "*/")),
                allow_backtick: false,
                allow_triple: false,
                string_delims: &['"', '\''],
                keywords: &[
                    "auto", "break", "case", "char", "const", "continue", "default", "do",
                    "double", "else", "enum", "extern", "float", "for", "goto", "if", "inline",
                    "int", "long", "register", "restrict", "return", "short", "signed", "sizeof",
                    "static", "struct", "switch", "typedef", "union", "unsigned", "void",
                    "volatile", "while", "_Bool",
                ],
                builtins: &[
                    "printf", "scanf", "fprintf", "sprintf", "malloc", "calloc", "realloc", "free",
                    "memcpy", "memset", "strlen", "strcpy", "strcmp", "strcat", "NULL", "true",
                    "false", "size_t", "int8_t", "uint8_t", "int32_t", "uint32_t",
                ],
                caps_are_types: false,
                macros: false,
                attr_prefixes: &['#'],
            },
            LanguageId::Cpp => LangSpec {
                line_comments: &["//"],
                block_comment: Some(("/*", "*/")),
                allow_backtick: false,
                allow_triple: false,
                string_delims: &['"', '\''],
                keywords: &[
                    "alignas",
                    "alignof",
                    "and",
                    "asm",
                    "auto",
                    "bool",
                    "break",
                    "case",
                    "catch",
                    "char",
                    "class",
                    "const",
                    "constexpr",
                    "continue",
                    "decltype",
                    "default",
                    "delete",
                    "do",
                    "double",
                    "dynamic_cast",
                    "else",
                    "enum",
                    "explicit",
                    "export",
                    "extern",
                    "false",
                    "float",
                    "for",
                    "friend",
                    "goto",
                    "if",
                    "inline",
                    "int",
                    "long",
                    "mutable",
                    "namespace",
                    "new",
                    "noexcept",
                    "nullptr",
                    "operator",
                    "private",
                    "protected",
                    "public",
                    "register",
                    "return",
                    "short",
                    "signed",
                    "sizeof",
                    "static",
                    "static_cast",
                    "struct",
                    "switch",
                    "template",
                    "this",
                    "throw",
                    "true",
                    "try",
                    "typedef",
                    "typeid",
                    "typename",
                    "union",
                    "unsigned",
                    "using",
                    "virtual",
                    "void",
                    "volatile",
                    "while",
                    "override",
                    "final",
                ],
                builtins: &[
                    "std",
                    "cout",
                    "cin",
                    "cerr",
                    "endl",
                    "vector",
                    "string",
                    "map",
                    "set",
                    "unordered_map",
                    "make_shared",
                    "make_unique",
                    "move",
                    "forward",
                    "size_t",
                    "int32_t",
                    "uint32_t",
                ],
                caps_are_types: true,
                macros: false,
                attr_prefixes: &['#'],
            },
            LanguageId::Go => LangSpec {
                line_comments: &["//"],
                block_comment: Some(("/*", "*/")),
                allow_backtick: true,
                allow_triple: false,
                string_delims: &['"', '\''],
                keywords: &[
                    "break",
                    "case",
                    "chan",
                    "const",
                    "continue",
                    "default",
                    "defer",
                    "else",
                    "fallthrough",
                    "for",
                    "func",
                    "go",
                    "goto",
                    "if",
                    "import",
                    "interface",
                    "map",
                    "package",
                    "range",
                    "return",
                    "select",
                    "struct",
                    "switch",
                    "type",
                    "var",
                    "true",
                    "false",
                    "iota",
                    "nil",
                ],
                builtins: &[
                    "make", "new", "len", "cap", "append", "copy", "delete", "panic", "recover",
                    "print", "println", "close", "complex", "real", "imag", "int", "int8", "int16",
                    "int32", "int64", "uint", "uint8", "uint16", "uint32", "uint64", "uintptr",
                    "float32", "float64", "string", "error", "bool", "byte", "rune",
                ],
                caps_are_types: true,
                macros: false,
                attr_prefixes: &['@'],
            },
            LanguageId::Java => LangSpec {
                line_comments: &["//"],
                block_comment: Some(("/*", "*/")),
                allow_backtick: false,
                allow_triple: false,
                string_delims: &['"', '\''],
                keywords: &[
                    "abstract",
                    "assert",
                    "boolean",
                    "break",
                    "byte",
                    "case",
                    "catch",
                    "char",
                    "class",
                    "const",
                    "continue",
                    "default",
                    "do",
                    "double",
                    "else",
                    "enum",
                    "extends",
                    "final",
                    "finally",
                    "float",
                    "for",
                    "goto",
                    "if",
                    "implements",
                    "import",
                    "instanceof",
                    "int",
                    "interface",
                    "long",
                    "native",
                    "new",
                    "package",
                    "private",
                    "protected",
                    "public",
                    "return",
                    "short",
                    "static",
                    "strictfp",
                    "super",
                    "switch",
                    "synchronized",
                    "this",
                    "throw",
                    "throws",
                    "transient",
                    "try",
                    "void",
                    "volatile",
                    "while",
                    "true",
                    "false",
                    "null",
                    "var",
                    "record",
                    "sealed",
                    "yield",
                ],
                builtins: &[
                    "System",
                    "String",
                    "Integer",
                    "Object",
                    "List",
                    "Map",
                    "ArrayList",
                    "HashMap",
                    "HashSet",
                    "Optional",
                    "Stream",
                ],
                caps_are_types: true,
                macros: false,
                attr_prefixes: &['@'],
            },
            LanguageId::CSharp => LangSpec {
                line_comments: &["//"],
                block_comment: Some(("/*", "*/")),
                allow_backtick: false,
                allow_triple: false,
                string_delims: &['"', '\''],
                keywords: &[
                    "abstract",
                    "as",
                    "base",
                    "bool",
                    "break",
                    "byte",
                    "case",
                    "catch",
                    "char",
                    "checked",
                    "class",
                    "const",
                    "continue",
                    "decimal",
                    "default",
                    "delegate",
                    "do",
                    "double",
                    "else",
                    "enum",
                    "event",
                    "explicit",
                    "extern",
                    "false",
                    "finally",
                    "fixed",
                    "float",
                    "for",
                    "foreach",
                    "goto",
                    "if",
                    "implicit",
                    "in",
                    "int",
                    "interface",
                    "internal",
                    "is",
                    "lock",
                    "long",
                    "namespace",
                    "new",
                    "null",
                    "object",
                    "operator",
                    "out",
                    "override",
                    "params",
                    "private",
                    "protected",
                    "public",
                    "readonly",
                    "ref",
                    "return",
                    "sbyte",
                    "sealed",
                    "short",
                    "sizeof",
                    "stackalloc",
                    "static",
                    "string",
                    "struct",
                    "switch",
                    "this",
                    "throw",
                    "true",
                    "try",
                    "typeof",
                    "uint",
                    "ulong",
                    "unchecked",
                    "unsafe",
                    "ushort",
                    "using",
                    "var",
                    "virtual",
                    "void",
                    "volatile",
                    "while",
                    "async",
                    "await",
                    "nameof",
                    "record",
                ],
                builtins: &[
                    "Console",
                    "List",
                    "Dictionary",
                    "Task",
                    "String",
                    "Int32",
                    "Math",
                    "Convert",
                    "DateTime",
                ],
                caps_are_types: true,
                macros: false,
                attr_prefixes: &['[', '@'],
            },
            LanguageId::Sql => LangSpec {
                line_comments: &["--"],
                block_comment: Some(("/*", "*/")),
                allow_backtick: false,
                allow_triple: false,
                string_delims: &['\'', '"'],
                keywords: &[
                    "SELECT",
                    "FROM",
                    "WHERE",
                    "INSERT",
                    "INTO",
                    "VALUES",
                    "UPDATE",
                    "SET",
                    "DELETE",
                    "CREATE",
                    "TABLE",
                    "DROP",
                    "ALTER",
                    "ADD",
                    "COLUMN",
                    "PRIMARY",
                    "KEY",
                    "FOREIGN",
                    "REFERENCES",
                    "INDEX",
                    "VIEW",
                    "JOIN",
                    "INNER",
                    "LEFT",
                    "RIGHT",
                    "OUTER",
                    "FULL",
                    "ON",
                    "GROUP",
                    "BY",
                    "ORDER",
                    "HAVING",
                    "AS",
                    "AND",
                    "OR",
                    "NOT",
                    "NULL",
                    "DISTINCT",
                    "UNION",
                    "ALL",
                    "LIMIT",
                    "OFFSET",
                    "CASE",
                    "WHEN",
                    "THEN",
                    "ELSE",
                    "END",
                    "IN",
                    "LIKE",
                    "BETWEEN",
                    "IS",
                    "EXISTS",
                    "COUNT",
                    "SUM",
                    "AVG",
                    "MIN",
                    "MAX",
                    "ASC",
                    "DESC",
                    "DEFAULT",
                    "CONSTRAINT",
                    "UNIQUE",
                    "CHECK",
                ],
                builtins: empty(),
                caps_are_types: false,
                macros: false,
                attr_prefixes: &['&'],
            },
            LanguageId::Shell => LangSpec {
                line_comments: &["#"],
                block_comment: None,
                allow_backtick: false,
                allow_triple: false,
                string_delims: &['"', '\''],
                keywords: &[
                    "if", "then", "else", "elif", "fi", "for", "in", "do", "done", "while",
                    "until", "case", "esac", "function", "return", "break", "continue", "local",
                    "export", "source", "alias", "eval", "exec", "shift", "trap", "readonly",
                ],
                builtins: &[
                    "echo", "printf", "cd", "pwd", "read", "test", "exit", "set", "unset", "ls",
                    "cat", "grep", "sed", "awk", "find", "xargs", "true", "false",
                ],
                caps_are_types: false,
                macros: false,
                attr_prefixes: &['$'],
            },
            LanguageId::Yaml => LangSpec {
                line_comments: &["#"],
                block_comment: None,
                allow_backtick: false,
                allow_triple: false,
                string_delims: &['"', '\''],
                keywords: empty(),
                builtins: &["true", "false", "null", "yes", "no", "on", "off"],
                caps_are_types: false,
                macros: false,
                attr_prefixes: &['-'],
            },
            LanguageId::Toml => LangSpec {
                line_comments: &["#"],
                block_comment: None,
                allow_backtick: false,
                allow_triple: false,
                string_delims: &['"', '\''],
                keywords: empty(),
                builtins: &["true", "false"],
                caps_are_types: false,
                macros: false,
                attr_prefixes: &['['],
            },
            LanguageId::Json => LangSpec {
                line_comments: empty(),
                block_comment: None,
                allow_backtick: false,
                allow_triple: false,
                string_delims: &['"'],
                keywords: &["true", "false", "null"],
                builtins: empty(),
                caps_are_types: false,
                macros: false,
                attr_prefixes: empty(),
            },
            LanguageId::Css | LanguageId::Scss => LangSpec {
                line_comments: &["//"],
                block_comment: Some(("/*", "*/")),
                allow_backtick: false,
                allow_triple: false,
                string_delims: &['"', '\''],
                keywords: empty(),
                builtins: empty(),
                caps_are_types: false,
                macros: false,
                attr_prefixes: &['@', '#'],
            },
            LanguageId::Html => LangSpec {
                line_comments: empty(),
                block_comment: Some(("<!--", "-->")),
                allow_backtick: false,
                allow_triple: false,
                string_delims: &['"', '\''],
                keywords: empty(),
                builtins: empty(),
                caps_are_types: false,
                macros: false,
                attr_prefixes: empty(),
            },
            LanguageId::Markdown => LangSpec {
                line_comments: empty(),
                block_comment: None,
                allow_backtick: true,
                allow_triple: false,
                string_delims: &['"'],
                keywords: empty(),
                builtins: empty(),
                caps_are_types: false,
                macros: false,
                attr_prefixes: &['#'],
            },
            LanguageId::PlainText => LangSpec {
                line_comments: empty(),
                block_comment: None,
                allow_backtick: false,
                allow_triple: false,
                string_delims: empty(),
                keywords: empty(),
                builtins: empty(),
                caps_are_types: false,
                macros: false,
                attr_prefixes: empty(),
            },
        }
    }
}

// ------------------------------- Lexer ------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum StrState {
    None,
    Triple(char),
    Backtick,
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident_part(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn matches_at(chars: &[char], i: usize, needle: &str) -> bool {
    let nc: Vec<char> = needle.chars().collect();
    if i + nc.len() > chars.len() {
        return false;
    }
    for (k, &want) in nc.iter().enumerate() {
        if chars[i + k] != want {
            return false;
        }
    }
    true
}

/// Faz o realce de `text` na linguagem `lang`.
/// Retorna um vetor com um `Vec<Span>` por linha.
pub fn highlight(lang: LanguageId, text: &str) -> Vec<Vec<Span>> {
    let spec = LangSpec::for_lang(lang);
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();

    let mut result: Vec<Vec<Span>> = vec![Vec::new()];
    let mut line = 0usize;
    let mut col = 0usize;
    let mut i = 0usize;

    let mut in_block: Option<&'static str> = None;
    let mut str_state = StrState::None;
    let mut in_html_tag = false;

    macro_rules! emit {
        ($kind:expr, $start:expr, $len:expr) => {
            if $len > 0 {
                result[line].push(Span {
                    kind: $kind,
                    start: $start,
                    len: $len,
                });
            }
        };
    }

    while i < n {
        let c = chars[i];

        // Nova linha.
        if c == '\n' {
            result.push(Vec::new());
            line += 1;
            col = 0;
            i += 1;
            continue;
        }

        // Estado: dentro de comentário de bloco.
        if let Some(end) = in_block {
            let start_col = col;
            let mut j = i;
            let mut found = false;
            loop {
                if j >= n || chars[j] == '\n' {
                    break;
                }
                if matches_at(&chars, j, end) {
                    let elen = end.chars().count();
                    j += elen;
                    found = true;
                    break;
                }
                j += 1;
            }
            let run = j - i;
            emit!(TokenKind::Comment, start_col, run);
            i = j;
            col = start_col + run;
            if found {
                in_block = None;
            }
            continue;
        }

        // Estado: string multilinha (tripla ou template/backtick).
        match str_state {
            StrState::Triple(q) => {
                let start_col = col;
                let mut j = i;
                let mut found = false;
                loop {
                    if j >= n {
                        break;
                    }
                    let ch = chars[j];
                    if ch == '\n' {
                        break;
                    }
                    if ch == q && matches_at(&chars, j, &q.to_string().repeat(3)) {
                        j += 3;
                        col += 3;
                        found = true;
                        break;
                    }
                    if ch == '\\' && j + 1 < n && chars[j + 1] != '\n' {
                        j += 2;
                        col += 2;
                        continue;
                    }
                    j += 1;
                    col += 1;
                }
                let run = j - i;
                emit!(TokenKind::String, start_col, run);
                i = j;
                if found {
                    str_state = StrState::None;
                }
                continue;
            }
            StrState::Backtick => {
                let start_col = col;
                let mut j = i;
                let mut found = false;
                loop {
                    if j >= n {
                        break;
                    }
                    let ch = chars[j];
                    if ch == '`' {
                        j += 1;
                        col += 1;
                        found = true;
                        break;
                    }
                    if ch == '\\' && j + 1 < n {
                        j += 2;
                        col += 2;
                        continue;
                    }
                    if ch == '\n' {
                        // templates atravessam linhas: para aqui e deixa o
                        // tratador de nova linha avançar a linha.
                        break;
                    }
                    j += 1;
                    col += 1;
                }
                let run = j - i;
                emit!(TokenKind::String, start_col, run);
                i = j;
                if found {
                    str_state = StrState::None;
                }
                continue;
            }
            StrState::None => {}
        }

        // Markdown: linha iniciada por '#..' é um heading.
        if lang == LanguageId::Markdown && c == '#' {
            let start_col = col;
            let mut j = i;
            while j < n && chars[j] != '\n' {
                j += 1;
                col += 1;
            }
            emit!(TokenKind::Heading, start_col, j - i);
            i = j;
            continue;
        }

        // Comentário de linha.
        if let Some(clen) = longest_match(&chars, i, spec.line_comments) {
            let start_col = col;
            let mut j = i;
            while j < n && chars[j] != '\n' {
                j += 1;
            }
            let run = j - i;
            emit!(TokenKind::Comment, start_col, run);
            i = j;
            col = start_col + run;
            let _ = clen;
            continue;
        }

        // Início de comentário de bloco.
        if let Some((open, close)) = spec.block_comment {
            if matches_at(&chars, i, open) {
                let olen = open.chars().count();
                emit!(TokenKind::Comment, col, olen);
                i += olen;
                col += olen;
                in_block = Some(close);
                continue;
            }
        }

        // Início de string tripla (Python """ / ''').
        if spec.allow_triple && (c == '"' || c == '\'') {
            let triple: String = std::iter::repeat(c).take(3).collect();
            if matches_at(&chars, i, &triple) {
                emit!(TokenKind::String, col, 3);
                i += 3;
                col += 3;
                str_state = StrState::Triple(c);
                continue;
            }
        }

        // Início de template/backtick.
        if spec.allow_backtick && c == '`' {
            emit!(TokenKind::String, col, 1);
            i += 1;
            col += 1;
            str_state = StrState::Backtick;
            continue;
        }

        // HTML: '<' abre tag, '>' fecha.
        if lang == LanguageId::Html {
            if c == '<' && !in_html_tag {
                emit!(TokenKind::Punctuation, col, 1);
                i += 1;
                col += 1;
                in_html_tag = true;
                continue;
            }
            if c == '>' && in_html_tag {
                emit!(TokenKind::Punctuation, col, 1);
                i += 1;
                col += 1;
                in_html_tag = false;
                continue;
            }
        }

        // String simples (mesma linha).
        if spec.string_delims.contains(&c) {
            let start_col = col;
            let quote = c;
            let mut j = i + 1;
            col += 1;
            loop {
                if j >= n || chars[j] == '\n' {
                    break;
                }
                if chars[j] == '\\' && j + 1 < n && chars[j + 1] != '\n' {
                    j += 2;
                    col += 2;
                    continue;
                }
                if chars[j] == quote {
                    j += 1;
                    col += 1;
                    break;
                }
                j += 1;
                col += 1;
            }
            let run = j - i;
            emit!(TokenKind::String, start_col, run);
            i = j;
            continue;
        }

        // Número.
        if c.is_ascii_digit() {
            let start_col = col;
            let mut j = i;
            while j < n && (chars[j].is_ascii_alphanumeric() || chars[j] == '.' || chars[j] == '_')
            {
                j += 1;
                col += 1;
            }
            // expoente com sinal: 1e-5
            if j > i
                && (chars[j - 1] == 'e' || chars[j - 1] == 'E')
                && j < n
                && (chars[j] == '+' || chars[j] == '-')
            {
                j += 1;
                col += 1;
                while j < n && chars[j].is_ascii_alphanumeric() {
                    j += 1;
                    col += 1;
                }
            }
            let run = j - i;
            emit!(TokenKind::Number, start_col, run);
            i = j;
            continue;
        }

        // Identificador / palavra-chave / tipo / função / macro.
        if is_ident_start(c) {
            let start_col = col;
            let mut j = i + 1;
            col += 1;
            while j < n && is_ident_part(chars[j]) {
                j += 1;
                col += 1;
            }
            let word: String = chars[i..j].iter().collect();
            let run = j - i;

            // procura próximo caractere não-espaço
            let mut k = j;
            while k < n && chars[k].is_whitespace() && chars[k] != '\n' {
                k += 1;
            }
            let next = chars.get(k).copied();

            let kind = classify_word(&spec, &word, next, in_html_tag);
            emit!(kind, start_col, run);
            i = j;
            continue;
        }

        // Prefixos de atributo/diretiva (@, #, $ ...).
        if spec.attr_prefixes.contains(&c) {
            emit!(TokenKind::Operator, col, 1);
            i += 1;
            col += 1;
            continue;
        }

        // Espaços em branco (Plain para cobertura total).
        if c.is_whitespace() {
            let start_col = col;
            let mut j = i;
            while j < n && chars[j].is_whitespace() && chars[j] != '\n' {
                j += 1;
                col += 1;
            }
            emit!(TokenKind::Plain, start_col, j - i);
            i = j;
            continue;
        }

        // Pontuação/operadores.
        let is_punct = matches!(c, '(' | ')' | '{' | '}' | '[' | ']' | ';' | ',' | '.' | ':');
        let kind = if is_punct {
            TokenKind::Punctuation
        } else {
            TokenKind::Operator
        };
        // agrupa operadores consecutivos
        let start_col = col;
        let mut j = i;
        while j < n {
            let ch = chars[j];
            let ch_punct = matches!(
                ch,
                '(' | ')' | '{' | '}' | '[' | ']' | ';' | ',' | '.' | ':'
            );
            if ch == '\n' || ch.is_whitespace() || is_ident_start(ch) || ch.is_ascii_digit() {
                break;
            }
            if ch_punct != is_punct {
                break;
            }
            if spec.attr_prefixes.contains(&ch) {
                break;
            }
            j += 1;
            col += 1;
        }
        emit!(kind, start_col, j - i);
        i = j;
    }

    result
}

fn longest_match(chars: &[char], i: usize, needles: &[&str]) -> Option<usize> {
    let mut best: Option<usize> = None;
    for nd in needles {
        if matches_at(chars, i, nd) {
            let len = nd.chars().count();
            if best.map_or(true, |b| len > b) {
                best = Some(len);
            }
        }
    }
    best
}

fn classify_word(spec: &LangSpec, word: &str, next: Option<char>, in_html_tag: bool) -> TokenKind {
    if in_html_tag {
        return TokenKind::Keyword;
    }
    if spec.keywords.contains(&word) {
        return TokenKind::Keyword;
    }
    if spec.builtins.contains(&word) {
        return TokenKind::Builtin;
    }
    if spec.caps_are_types
        && word
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    {
        return TokenKind::Type;
    }
    match next {
        Some('(') => TokenKind::Function,
        Some('!') if spec.macros => TokenKind::Macro,
        _ => TokenKind::Ident,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(_line: usize, spans: &[Span]) -> Vec<TokenKind> {
        spans.iter().map(|s| s.kind).collect()
    }

    #[test]
    fn rust_line_comment_and_keyword() {
        let text = "// oi\nlet x = 1;";
        let r = highlight(LanguageId::Rust, text);
        assert_eq!(r.len(), 2);
        assert_eq!(kinds(0, &r[0]), vec![TokenKind::Comment]);
        // "let" keyword, "x" ident, "=" operator, "1" number, ";" punctuation
        assert!(r[1].iter().any(|s| s.kind == TokenKind::Keyword));
        assert!(r[1].iter().any(|s| s.kind == TokenKind::Number));
    }

    #[test]
    fn block_comment_multiline() {
        let text = "a /* com\nment */ b";
        let r = highlight(LanguageId::C, text);
        assert_eq!(r.len(), 2);
        assert!(r[0].iter().any(|s| s.kind == TokenKind::Comment));
        assert!(r[1].iter().any(|s| s.kind == TokenKind::Comment));
    }

    #[test]
    fn spans_cover_whole_line() {
        let text = "fn main() { println!(\"hi\"); }";
        let r = highlight(LanguageId::Rust, text);
        // verifica cobertura contígua
        let mut expect = 0;
        for s in &r[0] {
            assert_eq!(s.start, expect, "span deve ser contígio");
            expect += s.len;
        }
        assert_eq!(expect, text.chars().count());
    }

    #[test]
    fn python_triple_string() {
        let text = "x = \"\"\"multi\nline\"\"\"";
        let r = highlight(LanguageId::Python, text);
        assert!(r[0].iter().any(|s| s.kind == TokenKind::String));
        assert!(r[1].iter().any(|s| s.kind == TokenKind::String));
    }

    #[test]
    fn markdown_heading() {
        let r = highlight(LanguageId::Markdown, "# Título");
        assert_eq!(kinds(0, &r[0]), vec![TokenKind::Heading]);
    }
}
