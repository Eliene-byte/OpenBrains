//! Tema visual do editor (cores + tamanho de fonte). Tema escuro por padrão.

use egui::Color32;
use openbrains_editor_core::TokenKind;

pub struct Theme {
    pub dark: bool,
    pub font_size: f32,
    // fundos
    pub bg: Color32,
    pub panel_bg: Color32,
    pub gutter_bg: Color32,
    pub current_line: Color32,
    pub selection: Color32,
    pub caret: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub gutter_text: Color32,
    pub gutter_text_active: Color32,
    pub accent: Color32,
    pub hover: Color32,
    pub active: Color32,
    pub tab_active: Color32,
    pub tab_inactive: Color32,
    // tokens
    pub comment: Color32,
    pub string: Color32,
    pub number: Color32,
    pub keyword: Color32,
    pub builtin: Color32,
    pub type_: Color32,
    pub function: Color32,
    pub macro_: Color32,
    pub operator: Color32,
    pub punctuation: Color32,
    pub heading: Color32,
    // diagnosticos
    pub error: Color32,
    pub warning: Color32,
    pub info: Color32,
    pub hint: Color32,
}

fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

impl Theme {
    pub fn dark() -> Self {
        Theme {
            dark: true,
            font_size: 14.0,
            bg: rgb(24, 25, 30),
            panel_bg: rgb(30, 32, 38),
            gutter_bg: rgb(26, 27, 33),
            current_line: rgb(32, 34, 41),
            selection: Color32::from_rgba_unmultiplied(60, 110, 180, 90),
            caret: rgb(235, 235, 240),
            border: rgb(45, 48, 58),
            text: rgb(214, 218, 224),
            text_dim: rgb(140, 145, 155),
            gutter_text: rgb(95, 100, 112),
            gutter_text_active: rgb(200, 205, 215),
            accent: rgb(86, 156, 214),
            hover: rgb(45, 48, 58),
            active: rgb(56, 60, 72),
            tab_active: rgb(24, 25, 30),
            tab_inactive: rgb(30, 32, 38),
            comment: rgb(106, 122, 106),
            string: rgb(206, 145, 120),
            number: rgb(181, 206, 168),
            keyword: rgb(198, 120, 221),
            builtin: rgb(86, 156, 214),
            type_: rgb(78, 201, 176),
            function: rgb(220, 220, 170),
            macro_: rgb(120, 200, 200),
            operator: rgb(171, 178, 191),
            punctuation: rgb(171, 178, 191),
            heading: rgb(86, 156, 214),
            error: rgb(244, 88, 88),
            warning: rgb(224, 175, 68),
            info: rgb(96, 160, 220),
            hint: rgb(120, 190, 130),
        }
    }

    pub fn light() -> Self {
        Theme {
            dark: false,
            font_size: 14.0,
            bg: rgb(252, 252, 252),
            panel_bg: rgb(243, 243, 243),
            gutter_bg: rgb(240, 240, 240),
            current_line: rgb(238, 240, 244),
            selection: Color32::from_rgba_unmultiplied(173, 214, 255, 140),
            caret: rgb(20, 20, 20),
            border: rgb(220, 222, 228),
            text: rgb(40, 42, 48),
            text_dim: rgb(120, 124, 134),
            gutter_text: rgb(160, 164, 172),
            gutter_text_active: rgb(60, 64, 72),
            accent: rgb(0, 102, 204),
            hover: rgb(232, 234, 238),
            active: rgb(214, 218, 226),
            tab_active: rgb(252, 252, 252),
            tab_inactive: rgb(243, 243, 243),
            comment: rgb(0, 128, 0),
            string: rgb(163, 21, 21),
            number: rgb(9, 134, 88),
            keyword: rgb(0, 0, 255),
            builtin: rgb(0, 112, 184),
            type_: rgb(38, 127, 153),
            function: rgb(121, 94, 38),
            macro_: rgb(38, 127, 153),
            operator: rgb(60, 62, 70),
            punctuation: rgb(60, 62, 70),
            heading: rgb(0, 102, 204),
            error: rgb(200, 30, 30),
            warning: rgb(180, 120, 0),
            info: rgb(0, 102, 204),
            hint: rgb(0, 130, 60),
        }
    }

    pub fn color_for(&self, kind: TokenKind) -> Color32 {
        use TokenKind::*;
        match kind {
            Plain | Ident => self.text,
            Comment => self.comment,
            String => self.string,
            Number => self.number,
            Keyword => self.keyword,
            Builtin => self.builtin,
            Type => self.type_,
            Function => self.function,
            Macro => self.macro_,
            Operator => self.operator,
            Punctuation => self.punctuation,
            Heading => self.heading,
        }
    }

    /// Aplica o tema aos visuais do egui (painéis, botões, etc.).
    pub fn apply(&self, ctx: &egui::Context) {
        let mut v = if self.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        v.panel_fill = self.panel_bg;
        v.window_fill = self.panel_bg;
        v.extreme_bg_color = self.bg;
        v.faint_bg_color = self.panel_bg;
        v.widgets.noninteractive.bg_fill = self.panel_bg;
        v.widgets.inactive.bg_fill = self.panel_bg;
        v.widgets.hovered.bg_fill = self.hover;
        v.widgets.active.bg_fill = self.active;
        v.widgets.noninteractive.fg_stroke.color = self.text;
        v.widgets.inactive.fg_stroke.color = self.text;
        v.widgets.hovered.fg_stroke.color = self.text;
        v.widgets.active.fg_stroke.color = self.text;
        v.hyperlink_color = self.accent;
        ctx.set_visuals(v);
    }
}
