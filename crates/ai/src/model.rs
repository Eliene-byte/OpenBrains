//! Modelo de conhecimento por linguagem ("IA" determinística).
//!
//! Cada linguagem tem um conjunto de palavras-chave, tipos, builtins e
//! snippets. Nada de download de modelo, nada de rede: pura lógica Rust.

use openbrains_common::LanguageId;

/// Um snippet de código: gatilho -> corpo.
pub struct Snippet {
    pub trigger: &'static str,
    pub label: &'static str,
    pub body: &'static str,
}

/// Modelo de conhecimento de uma linguagem.
pub struct LanguageModel {
    pub lang: LanguageId,
    pub keywords: &'static [&'static str],
    pub types: &'static [&'static str],
    pub builtins: &'static [&'static str],
    pub snippets: &'static [Snippet],
}

const RUST_SNIPPETS: &[Snippet] = &[
    Snippet {
        trigger: "fn",
        label: "fn name()",
        body: "fn name() {\n    \n}",
    },
    Snippet {
        trigger: "p",
        label: "println!",
        body: "println!(\"{}\", );",
    },
    Snippet {
        trigger: "for",
        label: "for .. in",
        body: "for item in iter {\n    \n}",
    },
    Snippet {
        trigger: "if",
        label: "if",
        body: "if cond {\n    \n}",
    },
    Snippet {
        trigger: "match",
        label: "match",
        body: "match value {\n    _ => {}\n}",
    },
];

const PY_SNIPPETS: &[Snippet] = &[
    Snippet {
        trigger: "def",
        label: "def fn()",
        body: "def name(args):\n    ",
    },
    Snippet {
        trigger: "for",
        label: "for in",
        body: "for item in iterable:\n    ",
    },
    Snippet {
        trigger: "if",
        label: "if",
        body: "if cond:\n    ",
    },
    Snippet {
        trigger: "cls",
        label: "class",
        body: "class Name:\n    def __init__(self):\n        ",
    },
];

const JS_SNIPPETS: &[Snippet] = &[
    Snippet {
        trigger: "fn",
        label: "function",
        body: "function name(args) {\n  \n}",
    },
    Snippet {
        trigger: "af",
        label: "arrow fn",
        body: "const name = (args) => {\n  \n};",
    },
    Snippet {
        trigger: "log",
        label: "console.log",
        body: "console.log();",
    },
    Snippet {
        trigger: "for",
        label: "for..of",
        body: "for (const item of iterable) {\n  \n}",
    },
];

fn rust_model() -> LanguageModel {
    LanguageModel {
        lang: LanguageId::Rust,
        keywords: &[
            "fn", "let", "mut", "pub", "struct", "enum", "impl", "trait", "match", "if", "else",
            "for", "while", "loop", "return", "use", "mod", "const", "static", "as", "where",
            "async", "await", "unsafe", "dyn", "move", "ref",
        ],
        types: &[
            "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
            "f32", "f64", "bool", "char", "str", "String", "Vec", "Option", "Result", "Box",
            "HashMap", "HashSet",
        ],
        builtins: &[
            "println!",
            "print!",
            "format!",
            "vec!",
            "panic!",
            "assert!",
            "assert_eq!",
            "dbg!",
            "Some",
            "None",
            "Ok",
            "Err",
            "Clone",
            "Copy",
            "Debug",
            "Default",
            "Iterator",
            "IntoIterator",
            "to_string",
            "clone",
            "iter",
            "into_iter",
            "collect",
            "push",
            "len",
            "unwrap",
            "expect",
        ],
        snippets: RUST_SNIPPETS,
    }
}

fn py_model() -> LanguageModel {
    LanguageModel {
        lang: LanguageId::Python,
        keywords: &[
            "def", "class", "return", "if", "elif", "else", "for", "while", "import", "from", "as",
            "with", "try", "except", "finally", "lambda", "yield", "async", "await", "pass",
            "break", "continue", "global", "nonlocal", "True", "False", "None",
        ],
        types: &[
            "int", "float", "str", "bool", "list", "dict", "set", "tuple", "bytes",
        ],
        builtins: &[
            "print",
            "len",
            "range",
            "enumerate",
            "zip",
            "map",
            "filter",
            "sorted",
            "sum",
            "min",
            "max",
            "abs",
            "open",
            "input",
            "isinstance",
            "super",
            "getattr",
            "setattr",
            "hasattr",
            "str",
            "int",
            "float",
            "list",
            "dict",
        ],
        snippets: PY_SNIPPETS,
    }
}

fn js_model(lang: LanguageId) -> LanguageModel {
    LanguageModel {
        lang,
        keywords: &[
            "function",
            "const",
            "let",
            "var",
            "return",
            "if",
            "else",
            "for",
            "while",
            "class",
            "extends",
            "new",
            "this",
            "import",
            "export",
            "from",
            "default",
            "async",
            "await",
            "try",
            "catch",
            "finally",
            "throw",
            "typeof",
            "instanceof",
            "yield",
            "true",
            "false",
            "null",
            "undefined",
        ],
        types: &[
            "string", "number", "boolean", "any", "unknown", "void", "object", "Promise", "Array",
            "Map", "Set",
        ],
        builtins: &[
            "console",
            "log",
            "Math",
            "JSON",
            "Object",
            "Array",
            "String",
            "Number",
            "Boolean",
            "Promise",
            "setTimeout",
            "setInterval",
            "fetch",
            "parseInt",
            "parseFloat",
        ],
        snippets: JS_SNIPPETS,
    }
}

fn c_model(lang: LanguageId) -> LanguageModel {
    LanguageModel {
        lang,
        keywords: &[
            "int", "char", "float", "double", "void", "long", "short", "unsigned", "signed",
            "const", "static", "struct", "union", "enum", "typedef", "if", "else", "for", "while",
            "do", "switch", "case", "default", "break", "continue", "return", "sizeof", "goto",
        ],
        types: &[
            "size_t", "int8_t", "uint8_t", "int32_t", "uint32_t", "int64_t", "uint64_t",
        ],
        builtins: &[
            "printf", "scanf", "malloc", "calloc", "free", "memcpy", "memset", "strlen", "strcpy",
            "NULL",
        ],
        snippets: &[],
    }
}

fn go_model() -> LanguageModel {
    LanguageModel {
        lang: LanguageId::Go,
        keywords: &[
            "func",
            "package",
            "import",
            "var",
            "const",
            "type",
            "struct",
            "interface",
            "map",
            "if",
            "else",
            "for",
            "range",
            "return",
            "go",
            "defer",
            "select",
            "switch",
            "case",
            "default",
            "chan",
            "true",
            "false",
            "nil",
        ],
        types: &[
            "int", "int8", "int16", "int32", "int64", "uint", "float32", "float64", "string",
            "bool", "byte", "rune", "error",
        ],
        builtins: &[
            "make", "new", "len", "cap", "append", "copy", "delete", "panic", "recover", "println",
            "print", "close",
        ],
        snippets: &[],
    }
}

/// Retorna o modelo de uma linguagem, se conhecido.
pub fn model_for(lang: LanguageId) -> Option<LanguageModel> {
    match lang {
        LanguageId::Rust => Some(rust_model()),
        LanguageId::Python => Some(py_model()),
        LanguageId::JavaScript => Some(js_model(LanguageId::JavaScript)),
        LanguageId::TypeScript | LanguageId::TypeScriptReact | LanguageId::JavaScriptReact => {
            Some(js_model(lang))
        }
        LanguageId::C | LanguageId::CSharp => Some(c_model(lang)),
        LanguageId::Cpp => Some(LanguageModel {
            lang: LanguageId::Cpp,
            keywords: &[
                "class",
                "struct",
                "namespace",
                "template",
                "typename",
                "public",
                "private",
                "protected",
                "virtual",
                "override",
                "const",
                "constexpr",
                "auto",
                "new",
                "delete",
                "return",
                "if",
                "else",
                "for",
                "while",
                "try",
                "catch",
                "throw",
                "using",
                "nullptr",
                "true",
                "false",
            ],
            types: &[
                "int", "char", "float", "double", "bool", "void", "size_t", "std", "vector",
                "string", "map",
            ],
            builtins: &[
                "std",
                "cout",
                "cin",
                "endl",
                "printf",
                "make_shared",
                "make_unique",
                "move",
            ],
            snippets: &[],
        }),
        LanguageId::Go => Some(go_model()),
        LanguageId::Java => Some(LanguageModel {
            lang: LanguageId::Java,
            keywords: &[
                "public",
                "private",
                "protected",
                "class",
                "interface",
                "extends",
                "implements",
                "static",
                "final",
                "void",
                "int",
                "return",
                "if",
                "else",
                "for",
                "while",
                "new",
                "this",
                "super",
                "true",
                "false",
                "null",
                "try",
                "catch",
            ],
            types: &[
                "String",
                "Integer",
                "Object",
                "List",
                "Map",
                "ArrayList",
                "HashMap",
            ],
            builtins: &["System", "out", "println", "print"],
            snippets: &[],
        }),
        LanguageId::Shell => Some(LanguageModel {
            lang: LanguageId::Shell,
            keywords: &[
                "if", "then", "else", "fi", "for", "in", "do", "done", "while", "case", "esac",
                "function",
            ],
            types: &[],
            builtins: &["echo", "cd", "export", "read", "test", "exit", "printf"],
            snippets: &[],
        }),
        _ => None,
    }
}
