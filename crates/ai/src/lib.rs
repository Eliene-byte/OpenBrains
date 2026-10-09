//! Motor de "IA" ultra-leve e 100% Rust do OpenBrains.
//!
//! Tudo aqui e deterministico e offline: complecao baseada em modelo por
//! linguagem + simbolos do arquivo, e analise de diagnosticos por regras.
//! A arquitetura e plugavel: no futuro pode-se adicionar um backend de LLM
//! local (ex.: candle) sem mudar os consumidores.

pub mod analyze;
pub mod complete;
pub mod model;

use openbrains_common::LanguageId;

use analyze::{analyze as analyze_impl, Diagnostic, Severity};
use complete::complete as complete_impl;

/// Motor principal de inteligencia do editor.
#[derive(Debug, Default)]
pub struct AiEngine;

impl AiEngine {
    pub fn new() -> Self {
        AiEngine
    }

    /// Sugestoes de autocomplete para a posicao (row, col) em chars.
    pub fn complete(
        &self,
        lang: LanguageId,
        lines: &[String],
        row: usize,
        col: usize,
        limit: usize,
    ) -> Vec<complete::Completion> {
        complete_impl(lines, row, col, lang, limit)
    }

    /// Diagnosticos do arquivo.
    pub fn analyze(&self, lang: LanguageId, lines: &[String]) -> Vec<Diagnostic> {
        analyze_impl(lang, lines)
    }
}

pub use analyze::Diagnostic as AiDiagnostic;
pub use complete::{Completion, CompletionKind};
pub type AiSeverity = Severity;
