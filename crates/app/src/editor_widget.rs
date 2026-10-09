//! Widget de editor de código customizado, desenhado pixel a pixel com egui.
//!
//! Inclui: gutter com números de linha, realce de sintaxe, linha atual,
//! seleção, caret com blink, scroll, auto-pairs, auto-indent, undo/redo
//! e popup de autocomplete (motor de IA).

use std::sync::Arc;

use egui::{
    self, Align2, Event, FontId, Key, Margin, Modifiers, PointerButton, Pos2, Rect, RichText,
    Sense, Stroke, Vec2,
};
use openbrains_ai::AiEngine;
use openbrains_common::LanguageId;
use openbrains_editor_core::{highlight, Cursor, Span};

use super::{order_cursors, Tab};

const TOP_PAD: f32 = 8.0;
const LEFT_PAD: f32 = 6.0;
const GUTTER_EXTRA: f32 = 26.0;
const INDENT: &str = "    ";

fn char_width(advance: f32, c: char) -> f32 {
    if c.is_ascii() {
        advance
    } else {
        advance * 2.0
    }
}

fn col_to_x(line: &str, col: usize, advance: f32) -> f32 {
    let mut x = 0.0;
    for (i, c) in line.chars().enumerate() {
        if i >= col {
            break;
        }
        x += char_width(advance, c);
    }
    x
}

fn x_to_col(line: &str, dx: f32, advance: f32) -> usize {
    if dx <= 0.0 {
        return 0;
    }
    let mut x = 0.0;
    for (i, c) in line.chars().enumerate() {
        let w = char_width(advance, c);
        if x + w * 0.5 > dx {
            return i;
        }
        x += w;
    }
    line.chars().count()
}

fn char_byte(s: &str, idx: usize) -> usize {
    s.char_indices()
        .nth(idx)
        .map(|(b, _)| b)
        .unwrap_or_else(|| s.len())
}

fn matching(c: char) -> Option<char> {
    match c {
        '(' => Some(')'),
        '[' => Some(']'),
        '{' => Some('}'),
        '"' => Some('"'),
        '\'' => Some('\''),
        '`' => Some('`'),
        _ => None,
    }
}

fn is_closing(c: char) -> bool {
    matches!(c, ')' | ']' | '}' | '"' | '\'' | '`')
}

fn indent_extra(prefix_line: &str, lang: LanguageId) -> &'static str {
    let t = prefix_line.trim_end();
    let ends = |c: char| t.ends_with(c);
    if ends('{') || ends('(') || ends('[') {
        return INDENT;
    }
    if lang == LanguageId::Python && ends(':') {
        return INDENT;
    }
    ""
}

/// Desenha o editor para a aba ativa.
pub fn show(ui: &mut egui::Ui, tab: &mut Tab, theme: &super::Theme, ai: &AiEngine) {
    let ctx = ui.ctx().clone();
    let now = ctx.time();
    let font = FontId::monospace(theme.font_size);

    let avail = ui.available_size_before_wrap();
    let (rect, response) = ui.allocate_at_least(avail, Sense::click_and_drag());

    // Métricas de fonte (monoespaçada).
    let probe =
        ui.painter()
            .layout_no_wrap("0000000000000000".to_string(), font.clone(), theme.text);
    let advance = probe.size().x / 16.0;
    let line_height = probe.size().y * 1.34;

    let n_lines = tab.buffer.line_count().max(1);
    let digits = n_lines.to_string().len().max(2) as f32;
    let gutter_w = digits * advance + GUTTER_EXTRA;
    let text_x = rect.min.x + gutter_w + LEFT_PAD;

    let lines_per_page = ((rect.height() - 2.0 * TOP_PAD) / line_height)
        .floor()
        .max(4.0) as usize;
    let max_scroll = ((n_lines as f32) * line_height - (rect.height() - TOP_PAD)).max(0.0);
    tab.scroll_y = tab.scroll_y.clamp(0.0, max_scroll);

    // ---- Mouse ----
    let hover_pos = ctx.pointer_hover_pos();
    if response.clicked() && !tab.comp.visible {
        if let Some(pos) = hover_pos {
            let c = pos_to_cursor(
                pos,
                rect,
                tab.scroll_y,
                text_x,
                advance,
                line_height,
                &tab.buffer,
            );
            tab.cursor = c;
            tab.anchor = c;
        }
        response.request_focus();
        tab.want_focus = false;
    }
    if response.dragged_by(PointerButton::Primary) {
        if let Some(pos) = hover_pos {
            let c = pos_to_cursor(
                pos,
                rect,
                tab.scroll_y,
                text_x,
                advance,
                line_height,
                &tab.buffer,
            );
            tab.cursor = c;
        }
    }
    if tab.want_focus {
        response.request_focus();
        tab.want_focus = false;
    }

    // ---- Scroll ----
    if let Some(pos) = hover_pos {
        if rect.contains(pos) {
            let dy = ctx.input(|i| i.smooth_scroll_delta.y);
            tab.scroll_y = (tab.scroll_y + dy).clamp(0.0, max_scroll);
        }
    }

    // ---- Teclado ----
    let focused = response.has_focus();
    if focused {
        ui.input_mut(|i| {
            let events = i.events.clone();
            for ev in &events {
                match ev {
                    Event::Text(t) => {
                        on_text(tab, ai, tab.lang, t, now);
                    }
                    Event::Paste(t) => {
                        commit(tab, false);
                        delete_selection(tab);
                        tab.buffer.insert_text(&mut tab.cursor, t);
                        tab.anchor = tab.cursor;
                        tab.comp.visible = false;
                        tab.last_edit = now;
                    }
                    Event::Copy => {
                        if tab.anchor != tab.cursor {
                            let (a, b) = order_cursors(tab.anchor, tab.cursor);
                            ctx.copy_text(tab.buffer.text_of_range(a, b));
                        }
                    }
                    Event::Cut => {
                        if tab.anchor != tab.cursor {
                            let (a, b) = order_cursors(tab.anchor, tab.cursor);
                            ctx.copy_text(tab.buffer.text_of_range(a, b));
                            commit(tab, false);
                            delete_selection(tab);
                            tab.last_edit = now;
                        }
                    }
                    Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => {
                        let base = if modifiers.ctrl || modifiers.command {
                            Modifiers::COMMAND
                        } else {
                            Modifiers::NONE
                        };
                        let _ = i.consume_key(base, *key);
                        let cmd = modifiers.command || modifiers.ctrl;
                        let shift = modifiers.shift;

                        // Navegação do popup quando visível.
                        if tab.comp.visible {
                            match key {
                                Key::ArrowUp => {
                                    if !tab.comp.items.is_empty() {
                                        let n = tab.comp.items.len();
                                        tab.comp.index = (tab.comp.index + n - 1) % n;
                                    }
                                    continue;
                                }
                                Key::ArrowDown => {
                                    if !tab.comp.items.is_empty() {
                                        let n = tab.comp.items.len();
                                        tab.comp.index = (tab.comp.index + 1) % n;
                                    }
                                    continue;
                                }
                                Key::Enter | Key::Tab => {
                                    accept_completion(tab, ai, now);
                                    continue;
                                }
                                Key::Escape => {
                                    tab.comp.visible = false;
                                    continue;
                                }
                                _ => {}
                            }
                        }

                        match key {
                            Key::A if cmd => {
                                tab.anchor = Cursor::new(0, 0);
                                tab.cursor = Cursor::new(
                                    tab.buffer.line_count().saturating_sub(1),
                                    tab.buffer
                                        .line_char_len(tab.buffer.line_count().saturating_sub(1)),
                                );
                            }
                            Key::Space if cmd => {
                                refresh_completion(tab, ai, tab.lang);
                            }
                            Key::ArrowLeft => {
                                select_begin(tab, shift);
                                if cmd {
                                    tab.cursor.move_word_left(&tab.buffer);
                                } else {
                                    tab.cursor.move_left(&tab.buffer);
                                }
                                pick_hide(tab);
                            }
                            Key::ArrowRight => {
                                select_begin(tab, shift);
                                if cmd {
                                    tab.cursor.move_word_right(&tab.buffer);
                                } else {
                                    tab.cursor.move_right(&tab.buffer);
                                }
                                pick_hide(tab);
                            }
                            Key::ArrowUp => {
                                select_begin(tab, shift);
                                tab.cursor.move_up(&tab.buffer);
                                pick_hide(tab);
                            }
                            Key::ArrowDown => {
                                select_begin(tab, shift);
                                tab.cursor.move_down(&tab.buffer);
                                pick_hide(tab);
                            }
                            Key::PageUp => {
                                select_begin(tab, shift);
                                for _ in 0..lines_per_page {
                                    tab.cursor.move_up(&tab.buffer);
                                }
                            }
                            Key::PageDown => {
                                select_begin(tab, shift);
                                for _ in 0..lines_per_page {
                                    tab.cursor.move_down(&tab.buffer);
                                }
                            }
                            Key::Home => {
                                select_begin(tab, shift);
                                tab.cursor.home();
                            }
                            Key::End => {
                                select_begin(tab, shift);
                                tab.cursor.end(&tab.buffer);
                            }
                            Key::Backspace => {
                                commit(tab, false);
                                if tab.anchor != tab.cursor {
                                    delete_selection(tab);
                                } else if cmd {
                                    tab.buffer.backspace_word(&mut tab.cursor);
                                } else if delete_pair_backward(tab) {
                                    // par apagado junto
                                } else {
                                    tab.buffer.backspace(&mut tab.cursor);
                                }
                                tab.anchor = tab.cursor;
                                tab.last_edit = now;
                                if tab.comp.visible {
                                    refresh_completion(tab, ai, tab.lang);
                                }
                            }
                            Key::Delete => {
                                commit(tab, false);
                                if tab.anchor != tab.cursor {
                                    delete_selection(tab);
                                } else if cmd {
                                    tab.buffer.delete_word(&mut tab.cursor);
                                } else if delete_pair_forward(tab) {
                                    // par apagado junto
                                } else {
                                    tab.buffer.delete(&mut tab.cursor);
                                }
                                tab.anchor = tab.cursor;
                                tab.last_edit = now;
                                if tab.comp.visible {
                                    refresh_completion(tab, ai, tab.lang);
                                }
                            }
                            Key::Enter => {
                                commit(tab, false);
                                delete_selection(tab);
                                newline_with_indent(tab, tab.lang);
                                tab.comp.visible = false;
                                tab.last_edit = now;
                            }
                            Key::Tab => {
                                commit(tab, false);
                                delete_selection(tab);
                                if shift {
                                    dedent_line(tab);
                                } else {
                                    tab.buffer.insert_text(&mut tab.cursor, INDENT);
                                }
                                tab.anchor = tab.cursor;
                                tab.comp.visible = false;
                                tab.last_edit = now;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    // ---- Recalcular realce ----
    if tab.hl_version != tab.buffer.version() {
        tab.hl = highlight(tab.lang, &tab.buffer.to_text());
        tab.hl_version = tab.buffer.version();
        tab.galleys.clear();
    }

    // ---- Manter caret visível ----
    keep_caret_visible(tab, line_height, rect.height(), TOP_PAD);

    // ---- Preparar galleys visíveis ----
    let first = (tab.scroll_y / line_height).floor().max(0.0) as usize;
    let last_raw = ((tab.scroll_y + rect.height()) / line_height).ceil() as usize + 1;
    let last = last_raw.min(n_lines.saturating_sub(1));
    let first = first.min(last);
    let mut galleys: Vec<Arc<egui::Galley>> = Vec::with_capacity(last - first + 1);
    for row in first..=last {
        galleys.push(galley_for(ui, tab, row, &font, theme));
    }

    // ---- Pintar ----
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, 0.0, theme.bg);

    // Gutter.
    let gutter_rect = Rect::from_min_size(rect.min, Vec2::new(gutter_w, rect.height()));
    painter.rect_filled(gutter_rect, 0.0, theme.gutter_bg);
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(gutter_rect.max.x - 1.0, rect.min.y),
            Vec2::new(1.0, rect.height()),
        ),
        0.0,
        theme.border,
    );

    let (sa, sb) = order_cursors(tab.anchor, tab.cursor);
    let caret_row = tab.cursor.line;

    for row in first..=last {
        let y = rect.min.y + TOP_PAD + row as f32 * line_height - tab.scroll_y;
        if y + line_height < rect.min.y || y > rect.max.y {
            continue;
        }
        let line = tab.buffer.line_string(row);

        // Linha atual.
        if row == caret_row {
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(rect.min.x, y),
                    Vec2::new(rect.width(), line_height),
                ),
                0.0,
                theme.current_line,
            );
        }

        // Seleção.
        if tab.anchor != tab.cursor && row >= sa.line && row <= sb.line {
            let (x1, x2) = if row == sa.line && row == sb.line {
                (
                    col_to_x(&line, sa.col, advance),
                    col_to_x(&line, sb.col, advance),
                )
            } else if row == sa.line {
                (col_to_x(&line, sa.col, advance), f32::INFINITY)
            } else if row == sb.line {
                (0.0, col_to_x(&line, sb.col, advance))
            } else {
                (0.0, f32::INFINITY)
            };
            let right = if x2.is_finite() {
                text_x + x2
            } else {
                rect.max.x
            };
            let sel_x1 = text_x + x1;
            if right > sel_x1 {
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(sel_x1, y),
                        Vec2::new(right - sel_x1, line_height),
                    ),
                    0.0,
                    theme.selection,
                );
            }
        }

        // Texto.
        let galley = &galleys[row - first];
        painter.galley(Pos2::new(text_x, y), galley.clone(), theme.text);

        // Número de linha.
        let num_color = if row == caret_row {
            theme.gutter_text_active
        } else {
            theme.gutter_text
        };
        painter.text(
            Pos2::new(gutter_rect.max.x - 10.0, y),
            Align2::RIGHT_TOP,
            (row + 1).to_string(),
            font.clone(),
            num_color,
        );
    }

    // Caret com blink.
    if focused {
        let t = ctx.time();
        let blink = ((t * 1.5) as i64) % 2 == 0 || (t - tab.last_edit) < 0.6;
        if blink {
            let line = tab.buffer.line_string(caret_row);
            let cx = text_x + col_to_x(&line, tab.cursor.col, advance);
            let cy = rect.min.y + TOP_PAD + caret_row as f32 * line_height - tab.scroll_y;
            let w = (advance * 0.12).max(1.6);
            painter.rect_filled(
                Rect::from_min_size(Pos2::new(cx, cy + 1.0), Vec2::new(w, line_height - 2.0)),
                0.0,
                theme.caret,
            );
        }
    }

    if focused {
        ctx.request_repaint();
    }

    // ---- Popup de completion ----
    draw_completion(ui, &ctx, tab, theme, ai, rect, text_x, line_height, advance);
}

fn pos_to_cursor(
    pos: Pos2,
    rect: Rect,
    scroll_y: f32,
    text_x: f32,
    advance: f32,
    line_height: f32,
    buffer: &openbrains_editor_core::Buffer,
) -> Cursor {
    let content_y = (pos.y - rect.min.y - TOP_PAD) + scroll_y;
    let mut row = (content_y / line_height).floor() as i64;
    if row < 0 {
        row = 0;
    }
    let n = buffer.line_count() as i64;
    if row >= n {
        row = n - 1;
    }
    let row = row as usize;
    let line = buffer.line_string(row);
    let col = x_to_col(&line, pos.x - text_x, advance);
    Cursor::new(row, col)
}

fn galley_for(
    ui: &mut egui::Ui,
    tab: &mut Tab,
    row: usize,
    font: &FontId,
    theme: &super::Theme,
) -> Arc<egui::Galley> {
    let cached = tab
        .galleys
        .get(&row)
        .map_or(false, |(v, _)| *v == tab.hl_version);
    if !cached {
        let line = tab.buffer.line_string(row);
        let spans: &[Span] = tab.hl.get(row).map(|v| v.as_slice()).unwrap_or(&[]);
        let mut job = egui::text::LayoutJob::default();
        for s in spans {
            let a = char_byte(&line, s.start);
            let b = char_byte(&line, s.start + s.len);
            let text = &line[a..b];
            if !text.is_empty() {
                job.append(
                    text,
                    0.0,
                    egui::text::TextFormat {
                        font_id: font.clone(),
                        color: theme.color_for(s.kind),
                        ..Default::default()
                    },
                );
            }
        }
        let g = ui.fonts_mut(|f| f.layout_job(job));
        tab.galleys.insert(row, (tab.hl_version, g));
    }
    tab.galleys.get(&row).unwrap().1.clone()
}

fn keep_caret_visible(tab: &mut Tab, line_height: f32, view_h: f32, top_pad: f32) {
    let caret_top = tab.cursor.line as f32 * line_height;
    let caret_bottom = caret_top + line_height;
    if caret_top < tab.scroll_y {
        tab.scroll_y = caret_top.max(0.0);
    } else if caret_bottom > tab.scroll_y + (view_h - top_pad) {
        tab.scroll_y = (caret_bottom - (view_h - top_pad)).max(0.0);
    }
}

// ---- Edição ----

fn commit(tab: &mut Tab, coalesce: bool) {
    if coalesce {
        if !tab.typing_run {
            tab.undo.push(tab.snap());
            tab.redo.clear();
            tab.typing_run = true;
        }
    } else {
        tab.undo.push(tab.snap());
        tab.redo.clear();
        tab.typing_run = false;
    }
    if tab.undo.len() > 500 {
        tab.undo.remove(0);
    }
}

fn select_begin(tab: &mut Tab, extend: bool) {
    if !extend {
        tab.anchor = tab.cursor;
    }
}

fn pick_hide(tab: &mut Tab) {
    tab.comp.visible = false;
}

fn delete_selection(tab: &mut Tab) {
    if tab.anchor == tab.cursor {
        return;
    }
    let (a, b) = order_cursors(tab.anchor, tab.cursor);
    let nc = tab.buffer.delete_range(a, b);
    tab.cursor = nc;
    tab.anchor = nc;
}

fn delete_pair_backward(tab: &mut Tab) -> bool {
    if tab.cursor.col == 0 {
        return false;
    }
    let line = tab.buffer.line_string(tab.cursor.line);
    let chars: Vec<char> = line.chars().collect();
    let prev = chars.get(tab.cursor.col - 1).copied();
    let next = chars.get(tab.cursor.col).copied();
    if let (Some(p), Some(n)) = (prev, next) {
        if matching(p) == Some(n) && p != n {
            tab.buffer.delete(&mut tab.cursor);
            tab.buffer.backspace(&mut tab.cursor);
            return true;
        }
    }
    false
}

fn delete_pair_forward(tab: &mut Tab) -> bool {
    let line = tab.buffer.line_string(tab.cursor.line);
    let chars: Vec<char> = line.chars().collect();
    let cur = chars.get(tab.cursor.col).copied();
    let next = chars.get(tab.cursor.col + 1).copied();
    if let (Some(c), Some(n)) = (cur, next) {
        if matching(c) == Some(n) && c != n {
            tab.buffer.delete(&mut tab.cursor);
            tab.buffer.delete(&mut tab.cursor);
            return true;
        }
    }
    false
}

fn newline_with_indent(tab: &mut Tab, lang: LanguageId) {
    let line = tab.buffer.line_string(tab.cursor.line);
    let indent: String = line
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect();
    let chars: Vec<char> = line.chars().collect();
    let prefix: String = chars[..tab.cursor.col.min(chars.len())].iter().collect();
    let extra = indent_extra(&prefix, lang);
    tab.buffer.insert_newline(&mut tab.cursor);
    let ins = format!("{indent}{extra}");
    tab.buffer.insert_text(&mut tab.cursor, &ins);
    tab.anchor = tab.cursor;
}

fn dedent_line(tab: &mut Tab) {
    let line = tab.buffer.line_string(tab.cursor.line);
    let mut chars: Vec<char> = line.chars().collect();
    let mut removed = 0;
    while removed < INDENT.len() && !chars.is_empty() && chars[0] == ' ' {
        chars.remove(0);
        removed += 1;
    }
    if removed > 0 {
        let new_line: String = chars.into_iter().collect();
        tab.buffer.lines[tab.cursor.line] = new_line;
        tab.buffer.version = tab.buffer.version.wrapping_add(1);
        tab.cursor.col = tab.cursor.col.saturating_sub(removed);
    }
}

fn on_text(tab: &mut Tab, ai: &AiEngine, lang: LanguageId, text: &str, now: f64) {
    if text.is_empty() {
        return;
    }
    commit(tab, true);
    delete_selection(tab);

    let single = text.chars().count() == 1;
    if single {
        let ch = text.chars().next().unwrap();
        commit(tab, true);
        if is_closing(ch) {
            let line = tab.buffer.line_string(tab.cursor.line);
            let chars: Vec<char> = line.chars().collect();
            if chars.get(tab.cursor.col) == Some(&ch) {
                tab.cursor.move_right(&tab.buffer);
            } else {
                tab.buffer.insert_char(&mut tab.cursor, ch);
            }
        } else if let Some(m) = matching(ch) {
            tab.buffer.insert_char(&mut tab.cursor, ch);
            tab.buffer.insert_char(&mut tab.cursor, m);
            tab.cursor.move_left(&tab.buffer);
        } else {
            tab.buffer.insert_char(&mut tab.cursor, ch);
        }
        tab.anchor = tab.cursor;
        tab.last_edit = now;
        if ch.is_alphanumeric() || ch == '_' {
            refresh_completion(tab, ai, lang);
        } else {
            tab.comp.visible = false;
        }
    } else {
        commit(tab, true);
        tab.buffer.insert_text(&mut tab.cursor, text);
        tab.anchor = tab.cursor;
        tab.last_edit = now;
        tab.comp.visible = false;
    }
}

fn current_prefix(tab: &Tab) -> (usize, usize) {
    // Retorna (start_col, len) do prefixo de palavra antes do cursor.
    let line = tab.buffer.line_string(tab.cursor.line);
    let chars: Vec<char> = line.chars().collect();
    let mut s = tab.cursor.col.min(chars.len());
    while s > 0 && (chars[s - 1].is_alphanumeric() || chars[s - 1] == '_') {
        s -= 1;
    }
    (s, tab.cursor.col - s)
}

fn refresh_completion(tab: &mut Tab, ai: &AiEngine, lang: LanguageId) {
    let (start, len) = current_prefix(tab);
    tab.comp.start = start;
    if len == 0 {
        tab.comp.visible = false;
        tab.comp.items.clear();
        return;
    }
    let items = ai.complete(lang, &tab.buffer.lines, tab.cursor.line, tab.cursor.col, 20);
    if items.is_empty() {
        tab.comp.visible = false;
    } else {
        tab.comp.items = items;
        tab.comp.index = 0;
        tab.comp.visible = true;
    }
}

fn accept_completion(tab: &mut Tab, ai: &AiEngine, now: f64) {
    let item = tab.comp.items.get(tab.comp.index).cloned();
    tab.comp.visible = false;
    if let Some(item) = item {
        commit(tab, false);
        let start = Cursor::new(tab.cursor.line, tab.comp.start.min(tab.cursor.col));
        let nc = tab.buffer.delete_range(start, tab.cursor);
        tab.cursor = nc;
        tab.buffer.insert_text(&mut tab.cursor, &item.insert);
        tab.anchor = tab.cursor;
        tab.last_edit = now;
        let _ = ai;
    }
    tab.comp.items.clear();
}

fn draw_completion(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    tab: &mut Tab,
    theme: &super::Theme,
    ai: &AiEngine,
    rect: Rect,
    text_x: f32,
    line_height: f32,
    advance: f32,
) {
    if !tab.comp.visible || tab.comp.items.is_empty() {
        tab.comp.visible = false;
        return;
    }
    let line = tab.buffer.line_string(tab.cursor.line);
    let caret_x = text_x + col_to_x(&line, tab.cursor.col, advance);
    let caret_y = rect.min.y + TOP_PAD + tab.cursor.line as f32 * line_height - tab.scroll_y;
    let pos = Pos2::new(
        caret_x.clamp(
            rect.min.x + 20.0,
            (rect.max.x - 220.0).max(rect.min.x + 20.0),
        ),
        (caret_y + line_height).min(rect.max.y - 20.0),
    );

    let items = tab.comp.items.clone();
    let mut accept_index: Option<usize> = None;

    egui::Area::new(egui::Id::new("ob_completion"))
        .fixed_pos(pos)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let frame = egui::Frame::NONE
                .fill(theme.panel_bg)
                .inner_margin(Margin::symmetric(2, 4))
                .stroke(Stroke::new(1.0, theme.border))
                .corner_radius(6.0);
            frame.show(ui, |ui| {
                ui.set_max_width(360.0);
                for (idx, it) in items.iter().take(12).enumerate() {
                    let selected = idx == tab.comp.index;
                    let color = if selected { theme.accent } else { theme.text };
                    let label = format!("{}  {}", it.label, it.detail);
                    let r = ui
                        .selectable_label(selected, RichText::new(label).color(color).monospace());
                    if r.clicked() {
                        accept_index = Some(idx);
                    }
                }
            });
        });

    if let Some(idx) = accept_index {
        tab.comp.index = idx;
        accept_completion(tab, ai, ctx.time());
    }
}
