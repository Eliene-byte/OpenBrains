//! Tipos compartilhados entre os crates do OpenBrains.
//!
//! Sem dependências externas: apenas detecção de linguagem por caminho/extensão
//! e metadados básicos usados pelo motor de IA, pelo highlighter e pela UI.

use std::path::Path;

/// Identificador de linguagem de programação.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LanguageId {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    JavaScriptReact,
    TypeScriptReact,
    Json,
    Toml,
    Yaml,
    Markdown,
    Html,
    Css,
    Scss,
    C,
    Cpp,
    Go,
    Java,
    CSharp,
    Shell,
    Sql,
    PlainText,
}

impl LanguageId {
    /// Detecta a linguagem a partir do caminho do arquivo.
    pub fn from_path(path: &Path) -> LanguageId {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            return LanguageId::from_ext(ext);
        }
        match path.file_name().and_then(|n| n.to_str()) {
            Some("Dockerfile") | Some("Containerfile") => LanguageId::Shell,
            Some("Makefile") | Some("makefile") | Some("GNUmakefile") => LanguageId::Shell,
            Some(".gitignore") | Some(".env") => LanguageId::PlainText,
            _ => LanguageId::PlainText,
        }
    }

    /// Detecta a linguagem a partir da extensão (sem ponto).
    pub fn from_ext(ext: &str) -> LanguageId {
        match ext.to_ascii_lowercase().as_str() {
            "rs" => LanguageId::Rust,
            "py" | "pyi" | "pyw" => LanguageId::Python,
            "js" | "mjs" | "cjs" => LanguageId::JavaScript,
            "ts" | "mts" | "cts" => LanguageId::TypeScript,
            "jsx" => LanguageId::JavaScriptReact,
            "tsx" => LanguageId::TypeScriptReact,
            "json" | "jsonc" => LanguageId::Json,
            "toml" => LanguageId::Toml,
            "yaml" | "yml" => LanguageId::Yaml,
            "md" | "markdown" => LanguageId::Markdown,
            "html" | "htm" | "xhtml" => LanguageId::Html,
            "css" => LanguageId::Css,
            "scss" | "sass" => LanguageId::Scss,
            "c" | "h" => LanguageId::C,
            "cpp" | "cc" | "cxx" | "c++" | "hpp" | "hh" | "hxx" => LanguageId::Cpp,
            "go" => LanguageId::Go,
            "java" => LanguageId::Java,
            "cs" => LanguageId::CSharp,
            "sh" | "bash" | "zsh" | "fish" => LanguageId::Shell,
            "sql" => LanguageId::Sql,
            _ => LanguageId::PlainText,
        }
    }

    /// Nome legível da linguagem.
    pub fn name(self) -> &'static str {
        match self {
            LanguageId::Rust => "Rust",
            LanguageId::Python => "Python",
            LanguageId::JavaScript => "JavaScript",
            LanguageId::TypeScript => "TypeScript",
            LanguageId::JavaScriptReact => "JavaScript React",
            LanguageId::TypeScriptReact => "TypeScript React",
            LanguageId::Json => "JSON",
            LanguageId::Toml => "TOML",
            LanguageId::Yaml => "YAML",
            LanguageId::Markdown => "Markdown",
            LanguageId::Html => "HTML",
            LanguageId::Css => "CSS",
            LanguageId::Scss => "SCSS",
            LanguageId::C => "C",
            LanguageId::Cpp => "C++",
            LanguageId::Go => "Go",
            LanguageId::Java => "Java",
            LanguageId::CSharp => "C#",
            LanguageId::Shell => "Shell",
            LanguageId::Sql => "SQL",
            LanguageId::PlainText => "Plain Text",
        }
    }

    /// Prefixo de comentário de linha (para toggle de comentário, análise, etc.).
    pub fn line_comment(self) -> &'static str {
        match self {
            LanguageId::Python | LanguageId::Shell | LanguageId::Toml | LanguageId::Yaml => "#",
            LanguageId::Sql => "--",
            _ => "//",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_by_extension() {
        assert_eq!(LanguageId::from_ext("rs"), LanguageId::Rust);
        assert_eq!(LanguageId::from_ext("PY"), LanguageId::Python);
        assert_eq!(LanguageId::from_ext("tsx"), LanguageId::TypeScriptReact);
        assert_eq!(LanguageId::from_ext("unknown"), LanguageId::PlainText);
    }

    #[test]
    fn detects_by_filename() {
        assert_eq!(
            LanguageId::from_path(Path::new("Makefile")),
            LanguageId::Shell
        );
        assert_eq!(
            LanguageId::from_path(Path::new("src/main.rs")),
            LanguageId::Rust
        );
    }
}
