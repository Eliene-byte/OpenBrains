//! Cursor do editor. Posições são (linha, coluna em *caracteres*).

use crate::buffer::Buffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cursor {
    pub line: usize,
    pub col: usize,
}

impl Cursor {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }

    /// Limita o cursor aos limites válidos do buffer.
    pub fn clamp(&mut self, buf: &Buffer) {
        if buf.line_count() == 0 {
            self.line = 0;
            self.col = 0;
            return;
        }
        if self.line >= buf.line_count() {
            self.line = buf.line_count() - 1;
        }
        let max = buf.line_char_len(self.line);
        if self.col > max {
            self.col = max;
        }
    }

    pub fn move_left(&mut self, buf: &Buffer) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.line > 0 {
            self.line -= 1;
            self.col = buf.line_char_len(self.line);
        }
    }

    pub fn move_right(&mut self, buf: &Buffer) {
        let max = buf.line_char_len(self.line);
        if self.col < max {
            self.col += 1;
        } else if self.line + 1 < buf.line_count() {
            self.line += 1;
            self.col = 0;
        }
    }

    pub fn move_up(&mut self, buf: &Buffer) {
        if self.line > 0 {
            self.line -= 1;
            self.clamp(buf);
        }
    }

    pub fn move_down(&mut self, buf: &Buffer) {
        if self.line + 1 < buf.line_count() {
            self.line += 1;
            self.clamp(buf);
        }
    }

    pub fn home(&mut self) {
        self.col = 0;
    }

    pub fn end(&mut self, buf: &Buffer) {
        self.col = buf.line_char_len(self.line);
    }

    pub fn move_word_left(&mut self, buf: &Buffer) {
        if self.col == 0 {
            self.move_left(buf);
            return;
        }
        let chars: Vec<char> = buf.line(self.line).chars().collect();
        let mut i = self.col;
        // pula não-palavra para trás
        while i > 0 && !is_word_char(chars[i - 1]) {
            i -= 1;
        }
        // pula palavra para trás
        while i > 0 && is_word_char(chars[i - 1]) {
            i -= 1;
        }
        self.col = i;
    }

    pub fn move_word_right(&mut self, buf: &Buffer) {
        let chars: Vec<char> = buf.line(self.line).chars().collect();
        let n = chars.len();
        let mut i = self.col;
        while i < n && is_word_char(chars[i]) {
            i += 1;
        }
        while i < n && !is_word_char(chars[i]) {
            i += 1;
        }
        if i == self.col && self.col >= n {
            // está no fim da linha: pula para a próxima
            self.move_right(buf);
            return;
        }
        self.col = i;
    }
}

/// Um caractere de "palavra" para navegação por palavras (alfanumérico ou `_`).
pub fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Retorna o início de "palavra" anterior na mesma linha (para backspace-word).
pub fn word_start_col(line: &str, col: usize) -> usize {
    let chars: Vec<char> = line.chars().collect();
    let mut i = col.min(chars.len());
    if i == 0 {
        return 0;
    }
    while i > 0 && !is_word_char(chars[i - 1]) {
        i -= 1;
    }
    while i > 0 && is_word_char(chars[i - 1]) {
        i -= 1;
    }
    i
}

/// Retorna o fim de "palavra" seguinte na mesma linha (para delete-word).
pub fn word_end_col(line: &str, col: usize) -> usize {
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut i = col.min(n);
    while i < n && is_word_char(chars[i]) {
        i += 1;
    }
    while i < n && !is_word_char(chars[i]) {
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;

    fn buf(s: &str) -> Buffer {
        Buffer::from_text(s)
    }

    #[test]
    fn basic_moves() {
        let b = buf("hello\nworld");
        let mut c = Cursor::new(0, 0);
        for _ in 0..5 {
            c.move_right(&b);
        }
        assert_eq!(c, Cursor::new(0, 5));
        c.move_right(&b);
        assert_eq!(c, Cursor::new(1, 0));
        c.move_left(&b);
        assert_eq!(c, Cursor::new(0, 5));
        c.move_left(&b);
        assert_eq!(c, Cursor::new(0, 4));
    }

    #[test]
    fn word_navigation() {
        let b = buf("foo bar   baz");
        let mut c = Cursor::new(0, 0);
        c.move_word_right(&b);
        assert_eq!(c.col, 4);
    }
}
