//! Autocomplete determinístico: combina o modelo da linguagem com os
//! identificadores já presentes no arquivo (a parte "inteligente").

use openbrains_common::LanguageId;

use crate::model::model_for;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Keyword,
    Builtin,
    Type,
    Snippet,
    Function,
    Variable,
}

impl CompletionKind {
    /// Prioridade de ordenação (menor = mais relevante).
    fn rank(self) -> u8 {
        match self {
            CompletionKind::Keyword => 0,
            CompletionKind::Builtin => 1,
            CompletionKind::Type => 2,
            CompletionKind::Snippet => 3,
            CompletionKind::Function => 4,
            CompletionKind::Variable => 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Completion {
    pub label: String,
    pub detail: String,
    pub kind: CompletionKind,
    pub insert: String,
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Extrai o prefixo de palavra imediatamente antes de (row, col) em chars.
fn current_prefix(lines: &[String], row: usize, col: usize) -> String {
    let line = match lines.get(row) {
        Some(l) => l,
        None => return String::new(),
    };
    let chars: Vec<char> = line.chars().collect();
    let mut i = col.min(chars.len());
    let mut start = i;
    while start > 0 && is_word(chars[start - 1]) {
        start -= 1;
    }
    i = i.min(chars.len());
    let _ = i;
    chars[start..col.min(chars.len())].iter().collect()
}

/// Coleta identificadores do arquivo; retorna (não-funções, funções).
fn collect_locals(lines: &[String]) -> (Vec<String>, Vec<String>) {
    use std::collections::BTreeSet;
    let mut vars: BTreeSet<String> = BTreeSet::new();
    let mut funcs: BTreeSet<String> = BTreeSet::new();
    for line in lines {
        let chars: Vec<char> = line.chars().collect();
        let n = chars.len();
        let mut i = 0;
        while i < n {
            if chars[i].is_alphabetic() || chars[i] == '_' {
                let start = i;
                while i < n && is_word(chars[i]) {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                // pula espaços para ver se é uma chamada
                let mut k = i;
                while k < n && chars[k] == ' ' || chars[k] == '\t' {
                    k += 1;
                }
                if chars.get(k) == Some(&'(') {
                    funcs.insert(word);
                } else {
                    vars.insert(word);
                }
            } else {
                i += 1;
            }
        }
    }
    (vars.into_iter().collect(), funcs.into_iter().collect())
}

/// Gera sugestões de completion para (row, col).
pub fn complete(
    lines: &[String],
    row: usize,
    col: usize,
    lang: LanguageId,
    limit: usize,
) -> Vec<Completion> {
    if let Some(c) = lines
        .get(row)
        .and_then(|l| l.chars().nth(col.saturating_sub(1)))
    {
        // não sugerir no meio de um número solto
        let _ = c;
    }
    let prefix = current_prefix(lines, row, col);

    let mut out: Vec<Completion> = Vec::new();

    // Do modelo da linguagem.
    if let Some(model) = model_for(lang) {
        for kw in model.keywords {
            if starts(&prefix, kw) {
                out.push(Completion {
                    label: kw.to_string(),
                    detail: "keyword".into(),
                    kind: CompletionKind::Keyword,
                    insert: kw.to_string(),
                });
            }
        }
        for ty in model.types {
            if starts(&prefix, ty) {
                out.push(Completion {
                    label: ty.to_string(),
                    detail: "type".into(),
                    kind: CompletionKind::Type,
                    insert: ty.to_string(),
                });
            }
        }
        for bi in model.builtins {
            if starts(&prefix, bi) {
                out.push(Completion {
                    label: bi.to_string(),
                    detail: "builtin".into(),
                    kind: CompletionKind::Builtin,
                    insert: bi.to_string(),
                });
            }
        }
        for sn in model.snippets {
            if starts(&prefix, sn.trigger) {
                out.push(Completion {
                    label: format!("{}  ({})", sn.trigger, sn.label),
                    detail: "snippet".into(),
                    kind: CompletionKind::Snippet,
                    insert: sn.body.to_string(),
                });
            }
        }
    }

    // Identificadores locais (variáveis e funções do arquivo).
    let (vars, funcs) = collect_locals(lines);
    for f in funcs {
        if starts(&prefix, &f) && f != prefix {
            out.push(Completion {
                label: f.clone(),
                detail: "função local".into(),
                kind: CompletionKind::Function,
                insert: f,
            });
        }
    }
    for v in vars {
        if starts(&prefix, &v) && v != prefix {
            out.push(Completion {
                label: v.clone(),
                detail: "símbolo local".into(),
                kind: CompletionKind::Variable,
                insert: v,
            });
        }
    }

    // Dedup por insert text mantendo o de maior relevância.
    out.sort_by(|a, b| {
        a.kind
            .rank()
            .cmp(&b.kind.rank())
            .then(a.label.len().cmp(&b.label.len()))
            .then(a.label.cmp(&b.label))
    });
    let mut seen = std::collections::HashSet::new();
    out.retain(|c| seen.insert(c.insert.clone()));
    out.truncate(limit);
    out
}

fn starts(prefix: &str, candidate: &str) -> bool {
    if prefix.is_empty() {
        return true;
    }
    candidate.starts_with(prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_lines(s: &str) -> Vec<String> {
        s.lines().map(|l| l.to_string()).collect()
    }

    #[test]
    fn prefix_empty_returns_something() {
        let lines = to_lines("fn ");
        let c = complete(&lines, 0, 3, LanguageId::Rust, 20);
        assert!(!c.is_empty());
    }

    #[test]
    fn completion_from_locals() {
        let src = "fn helper() {}\nhel";
        let lines = to_lines(src);
        let c = complete(&lines, 1, 3, LanguageId::Rust, 20);
        assert!(c.iter().any(|x| x.label == "helper"));
        assert!(c.iter().any(|x| x.kind == CompletionKind::Function));
    }

    #[test]
    fn keyword_completion() {
        let lines = to_lines("stru");
        let c = complete(&lines, 0, 4, LanguageId::Rust, 20);
        assert!(c.iter().any(|x| x.label == "struct"));
    }
}
