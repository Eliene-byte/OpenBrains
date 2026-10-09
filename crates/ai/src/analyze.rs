//! Analise deterministica de codigo (diagnosticos simples e uteis, custo zero).

use openbrains_common::LanguageId;
use openbrains_editor_core::highlight::{highlight, TokenKind};
use openbrains_editor_core::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: usize,
    pub col: usize,
    pub severity: Severity,
    pub message: String,
}

const MAX_LINE_LEN: usize = 120;

const TASK_WORDS: &[&str] = &["TODO", "FIXME", "XXX", "HACK", "NOTE", "BUG"];

/// Analisa o arquivo e retorna diagnosticos deterministas.
pub fn analyze(lang: LanguageId, lines: &[String]) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let text = lines.join("\n");
    let spans = highlight(lang, &text);

    let mut paren = 0i64;
    let mut bracket = 0i64;
    let mut brace = 0i64;

    for (row, line) in lines.iter().enumerate() {
        // Espacos no fim da linha.
        if line.len() != line.trim_end().len() {
            diags.push(Diagnostic {
                line: row,
                col: line.trim_end().chars().count(),
                severity: Severity::Info,
                message: "Espacos em branco no fim da linha.".into(),
            });
        }

        // Linha longa.
        let char_len = line.chars().count();
        if char_len > MAX_LINE_LEN {
            diags.push(Diagnostic {
                line: row,
                col: MAX_LINE_LEN,
                severity: Severity::Warning,
                message: format!("Linha longa ({char_len} > {MAX_LINE_LEN} caracteres)."),
            });
        }

        // Comentarios de tarefa.
        for w in TASK_WORDS {
            if let Some(pos) = line.find(*w) {
                diags.push(Diagnostic {
                    line: row,
                    col: line[..pos].chars().count(),
                    severity: Severity::Info,
                    message: format!("Comentario de tarefa: {w}"),
                });
                break;
            }
        }

        // Versao "mascarada" (strings e comentarios -> espacos) para analisar
        // apenas o codigo.
        let masked = mask_line(line, spans.get(row).map(|v| v.as_slice()).unwrap_or(&[]));

        for ch in masked.chars() {
            match ch {
                '(' => paren += 1,
                ')' => paren -= 1,
                '[' => bracket += 1,
                ']' => bracket -= 1,
                '{' => brace += 1,
                '}' => brace -= 1,
                _ => {}
            }
        }

        // Statements de debug provavelmente deixados no codigo.
        let patterns: &[&str] = match lang {
            LanguageId::Rust => &["dbg!(", "println!("],
            LanguageId::Python => &["print(", "breakpoint("],
            LanguageId::JavaScript
            | LanguageId::TypeScript
            | LanguageId::JavaScriptReact
            | LanguageId::TypeScriptReact => &["console.log(", "console.debug("],
            _ => &[],
        };
        for p in patterns {
            if let Some(pos) = masked.find(*p) {
                diags.push(Diagnostic {
                    line: row,
                    col: masked[..pos].chars().count(),
                    severity: Severity::Hint,
                    message: format!("Possivel statement de debug deixado no codigo: `{p}`."),
                });
            }
        }

        // Rust: println( deveria ser println!(.
        if lang == LanguageId::Rust {
            if let Some(pos) = masked.find("println(") {
                diags.push(Diagnostic {
                    line: row,
                    col: masked[..pos].chars().count(),
                    severity: Severity::Hint,
                    message: "Em Rust, use `println!(...)` (macro).".into(),
                });
            }
        }
    }

    if paren != 0 {
        diags.push(Diagnostic {
            line: 0,
            col: 0,
            severity: Severity::Warning,
            message: format!("Parenteses desbalanceados (saldo {paren})."),
        });
    }
    if bracket != 0 {
        diags.push(Diagnostic {
            line: 0,
            col: 0,
            severity: Severity::Warning,
            message: format!("Colchetes desbalanceados (saldo {bracket})."),
        });
    }
    if brace != 0 {
        diags.push(Diagnostic {
            line: 0,
            col: 0,
            severity: Severity::Warning,
            message: format!("Chaves desbalanceadas (saldo {brace})."),
        });
    }

    diags
}

/// Substitui spans de comentario/string por espacos, preservando posicoes (chars).
fn mask_line(original: &str, line_spans: &[Span]) -> String {
    let chars: Vec<char> = original.chars().collect();
    let mut masked: Vec<char> = chars.clone();
    for s in line_spans {
        if matches!(s.kind, TokenKind::Comment | TokenKind::String) {
            for idx in s.start..(s.start + s.len) {
                if idx < masked.len() {
                    masked[idx] = ' ';
                }
            }
        }
    }
    masked.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_lines(s: &str) -> Vec<String> {
        s.lines().map(|l| l.to_string()).collect()
    }

    #[test]
    fn detects_debug_leftover() {
        let d = analyze(LanguageId::Rust, &to_lines("fn m() { println!(\"x\"); }"));
        assert!(d.iter().any(|x| x.message.contains("debug")));
    }

    #[test]
    fn detects_trailing_ws() {
        let d = analyze(LanguageId::Rust, &to_lines("let x = 1;   "));
        assert!(d.iter().any(|x| x.message.contains("fim da linha")));
    }

    #[test]
    fn detects_todo() {
        let d = analyze(LanguageId::Rust, &to_lines("// TODO: melhorar"));
        assert!(d.iter().any(|x| x.message.contains("TODO")));
    }
}
