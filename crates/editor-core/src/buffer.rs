//! Buffer de texto simples e leve: um `Vec<String>` por linha.
//!
//! Para arquivos-fonte típicos isso é mais do que suficiente e mantém o
//! binário pequeno. Todas as operações incrementam `version`, que a UI usa
//! para invalidar o cache de realce de sintaxe.

use crate::cursor::{word_end_col, word_start_col, Cursor};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct Buffer {
    pub lines: Vec<String>,
    pub version: u64,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            lines: vec![String::new()],
            version: 0,
        }
    }

    /// Cria o buffer a partir de um texto, normalizando CRLF/CR para `\n`.
    pub fn from_text(text: &str) -> Self {
        let mut lines: Vec<String> = Vec::new();
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        for raw in normalized.split('\n') {
            lines.push(raw.to_string());
        }
        if lines.is_empty() {
            lines.push(String::new());
        }
        Buffer { lines, version: 0 }
    }

    pub fn to_text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn line(&self, i: usize) -> &str {
        self.lines.get(i).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn line_string(&self, i: usize) -> String {
        self.lines.get(i).cloned().unwrap_or_default()
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn line_char_len(&self, line: usize) -> usize {
        self.lines.get(line).map(|l| l.chars().count()).unwrap_or(0)
    }

    fn touch(&mut self) {
        self.version = self.version.wrapping_add(1);
    }

    fn byte_of(&self, line: usize, col: usize) -> usize {
        let l = &self.lines[line];
        l.char_indices()
            .nth(col)
            .map(|(b, _)| b)
            .unwrap_or_else(|| l.len())
    }

    /// Insere texto (possivelmente multilinha) na posição do cursor.
    pub fn insert_text(&mut self, cur: &mut Cursor, text: &str) {
        if text.is_empty() {
            return;
        }
        cur.clamp(self);
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        let mut parts = normalized.split('\n');
        let first = parts.next().unwrap_or("");
        let rest: Vec<&str> = parts.collect();

        let byte = self.byte_of(cur.line, cur.col);
        self.lines[cur.line].insert_str(byte, first);
        let col_after_first = cur.col + first.chars().count();

        if rest.is_empty() {
            cur.col = col_after_first;
            self.touch();
            return;
        }

        // Separa o resto da linha atual (cauda) para reencaixar depois.
        let tail_byte = self.byte_of(cur.line, col_after_first);
        let tail = self.lines[cur.line].split_off(tail_byte);

        let n = rest.len();
        let mut inserts: Vec<String> = Vec::with_capacity(n);
        for (idx, part) in rest.iter().enumerate() {
            if idx == n - 1 {
                inserts.push(format!("{part}{tail}"));
            } else {
                inserts.push((*part).to_string());
            }
        }
        self.lines.splice(cur.line + 1..cur.line + 1, inserts);
        cur.line += n;
        cur.col = rest[n - 1].chars().count();
        self.touch();
    }

    pub fn insert_char(&mut self, cur: &mut Cursor, c: char) {
        cur.clamp(self);
        let byte = self.byte_of(cur.line, cur.col);
        self.lines[cur.line].insert(byte, c);
        cur.col += 1;
        self.touch();
    }

    /// Insere uma nova linha, quebrando a linha atual na coluna do cursor.
    pub fn insert_newline(&mut self, cur: &mut Cursor) {
        cur.clamp(self);
        let byte = self.byte_of(cur.line, cur.col);
        let tail = self.lines[cur.line].split_off(byte);
        let idx = cur.line + 1;
        self.lines.insert(idx, tail);
        cur.line = idx;
        cur.col = 0;
        self.touch();
    }

    /// Backspace: apaga o grafema anterior ou une linhas.
    pub fn backspace(&mut self, cur: &mut Cursor) {
        cur.clamp(self);
        if cur.col > 0 {
            let line = &self.lines[cur.line];
            let chars: Vec<char> = line.chars().collect();
            let remove_len = {
                let prefix: String = chars[..cur.col].iter().collect();
                prefix
                    .graphemes(true)
                    .last()
                    .map(|g| g.chars().count())
                    .unwrap_or(1)
                    .max(1)
            };
            let start = cur.col - remove_len;
            let new_line: String = chars[..start]
                .iter()
                .chain(chars[cur.col..].iter())
                .collect();
            self.lines[cur.line] = new_line;
            cur.col = start;
            self.touch();
        } else if cur.line > 0 {
            let prev_len = self.line_char_len(cur.line - 1);
            let cur_line = self.lines.remove(cur.line);
            self.lines[cur.line - 1].push_str(&cur_line);
            cur.line -= 1;
            cur.col = prev_len;
            self.touch();
        }
    }

    /// Delete: apaga o grafema seguinte ou une com a próxima linha.
    pub fn delete(&mut self, cur: &mut Cursor) {
        cur.clamp(self);
        let len = self.line_char_len(cur.line);
        if cur.col < len {
            let line = &self.lines[cur.line];
            let chars: Vec<char> = line.chars().collect();
            let remove_len = {
                let suffix: String = chars[cur.col..].iter().collect();
                suffix
                    .graphemes(true)
                    .next()
                    .map(|g| g.chars().count())
                    .unwrap_or(1)
                    .max(1)
            };
            let end = (cur.col + remove_len).min(chars.len());
            let new_line: String = chars[..cur.col].iter().chain(chars[end..].iter()).collect();
            self.lines[cur.line] = new_line;
            self.touch();
        } else if cur.line + 1 < self.line_count() {
            let next = self.lines.remove(cur.line + 1);
            self.lines[cur.line].push_str(&next);
            self.touch();
        }
    }

    /// Backspace por palavra (Ctrl+Backspace).
    pub fn backspace_word(&mut self, cur: &mut Cursor) {
        cur.clamp(self);
        if cur.col == 0 {
            self.backspace(cur);
            return;
        }
        let line = self.line_string(cur.line);
        let start = word_start_col(&line, cur.col);
        if start == cur.col {
            self.backspace(cur);
            return;
        }
        let chars: Vec<char> = line.chars().collect();
        let new_line: String = chars[..start]
            .iter()
            .chain(chars[cur.col..].iter())
            .collect();
        self.lines[cur.line] = new_line;
        cur.col = start;
        self.touch();
    }

    /// Delete por palavra (Ctrl+Delete).
    pub fn delete_word(&mut self, cur: &mut Cursor) {
        cur.clamp(self);
        let line = self.line_string(cur.line);
        let len = self.line_char_len(cur.line);
        let end = word_end_col(&line, cur.col);
        if end == cur.col {
            // nada a remover na linha: une com a próxima
            if cur.col >= len && cur.line + 1 < self.line_count() {
                self.delete(cur);
            }
            return;
        }
        let chars: Vec<char> = line.chars().collect();
        let new_line: String = chars[..cur.col].iter().chain(chars[end..].iter()).collect();
        self.lines[cur.line] = new_line;
        self.touch();
    }

    /// Alterna comentário de linha (usando o prefixo da linguagem) na linha do cursor.
    /// Retorna `true` se a linha ficou comentada.
    pub fn toggle_line_comment(&mut self, cur: &mut Cursor, prefix: &str) -> bool {
        cur.clamp(self);
        let line = self.line_string(cur.line);
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];
        let was_commented = trimmed.starts_with(prefix);
        let new_text = if let Some(rest) = trimmed.strip_prefix(prefix) {
            let rest = rest.strip_prefix(' ').unwrap_or(rest);
            format!("{indent}{rest}")
        } else {
            format!("{indent}{prefix} {trimmed}")
        };
        self.lines[cur.line] = new_text;
        self.touch();
        // Retorna `true` se a linha ficou comentada (i.e. foi adicionado um comentário).
        !was_commented
    }

    /// Texto contido no intervalo `[a, b]` (assumindo `a <= b`).
    pub fn text_of_range(&self, a: Cursor, b: Cursor) -> String {
        if a.line == b.line {
            let line = self.line_string(a.line);
            let chars: Vec<char> = line.chars().collect();
            return chars[a.col..b.col.min(chars.len())].iter().collect();
        }
        let first = {
            let line = self.line_string(a.line);
            let chars: Vec<char> = line.chars().collect();
            chars[a.col.min(chars.len())..].iter().collect::<String>()
        };
        let last = {
            let line = self.line_string(b.line);
            let chars: Vec<char> = line.chars().collect();
            chars[..b.col.min(chars.len())].iter().collect::<String>()
        };
        let mut out = first;
        for row in (a.line + 1)..b.line {
            out.push('\n');
            out.push_str(&self.line_string(row));
        }
        out.push('\n');
        out.push_str(&last);
        out
    }

    /// Remove o intervalo `[a, b]` (assumindo `a <= b`) e retorna a nova posição do cursor.
    pub fn delete_range(&mut self, a: Cursor, b: Cursor) -> Cursor {
        if a.line == b.line && a.col == b.col {
            return a;
        }
        if a.line == b.line {
            let line = self.line_string(a.line);
            let chars: Vec<char> = line.chars().collect();
            let new_line: String = chars[..a.col.min(chars.len())]
                .iter()
                .chain(chars[b.col.min(chars.len())..].iter())
                .collect();
            self.lines[a.line] = new_line;
            self.touch();
            return a;
        }
        let merged = {
            let first_chars: Vec<char> = self.line_string(a.line).chars().collect();
            let last_chars: Vec<char> = self.line_string(b.line).chars().collect();
            let mut s: String = first_chars[..a.col.min(first_chars.len())].iter().collect();
            s.extend(last_chars[b.col.min(last_chars.len())..].iter());
            s
        };
        self.lines[a.line] = merged;
        let remove_start = a.line + 1;
        let remove_end = b.line + 1;
        self.lines
            .drain(remove_start..remove_end.min(self.lines.len()));
        self.touch();
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_text_and_back() {
        let b = Buffer::from_text("a\r\nb\nc");
        assert_eq!(b.line_count(), 3);
        assert_eq!(b.to_text(), "a\nb\nc");
    }

    #[test]
    fn insert_and_newline() {
        let mut b = Buffer::new();
        let mut c = Cursor::new(0, 0);
        b.insert_text(&mut c, "hello");
        c.end(&b);
        b.insert_newline(&mut c);
        b.insert_text(&mut c, "world");
        assert_eq!(b.to_text(), "hello\nworld");
        assert_eq!(c, Cursor::new(1, 5));
    }

    #[test]
    fn backspace_joins_lines() {
        let mut b = Buffer::from_text("ab\ncd");
        let mut c = Cursor::new(1, 0);
        b.backspace(&mut c);
        assert_eq!(b.to_text(), "abcd");
        assert_eq!(c, Cursor::new(0, 2));
    }

    #[test]
    fn toggle_comment() {
        let mut b = Buffer::from_text("    let x = 1;");
        let mut c = Cursor::new(0, 0);
        let added = b.toggle_line_comment(&mut c, "//");
        assert!(added);
        assert_eq!(b.line(0), "    // let x = 1;");
        let removed = b.toggle_line_comment(&mut c, "//");
        assert!(!removed);
        assert_eq!(b.line(0), "    let x = 1;");
    }

    #[test]
    fn version_bumps() {
        let mut b = Buffer::new();
        let mut c = Cursor::new(0, 0);
        let v0 = b.version();
        b.insert_char(&mut c, 'x');
        assert_eq!(b.version(), v0 + 1);
    }
}
