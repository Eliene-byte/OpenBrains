//! Shell da aplicação: janela, painéis, abas, file tree, paleta de comandos
//! e painel de IA. Integra o widget de editor customizado.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::{App, CreationContext, Frame};
use egui::{self, Context, Galley, Id, Key, Margin, Modifiers, RichText, Vec2};
use openbrains_ai::{AiDiagnostic, AiEngine, Completion};
use openbrains_common::LanguageId;
use openbrains_editor_core::{Buffer, Cursor, Span};

use crate::editor_widget;
use crate::theme::Theme;

const MAX_TABS: usize = 32;

/// Ordena dois cursores (a <= b).
pub fn order_cursors(a: Cursor, b: Cursor) -> (Cursor, Cursor) {
    if (a.line, a.col) <= (b.line, b.col) {
        (a, b)
    } else {
        (b, a)
    }
}

#[derive(Clone)]
pub struct Snap {
    lines: Vec<String>,
    cursor: Cursor,
    anchor: Cursor,
}

pub struct CompletionState {
    pub visible: bool,
    pub items: Vec<Completion>,
    pub index: usize,
    pub start: usize,
}

impl Default for CompletionState {
    fn default() -> Self {
        CompletionState {
            visible: false,
            items: Vec::new(),
            index: 0,
            start: 0,
        }
    }
}

pub struct Tab {
    pub path: Option<PathBuf>,
    pub buffer: Buffer,
    pub cursor: Cursor,
    pub anchor: Cursor,
    pub scroll_y: f32,
    pub want_focus: bool,
    pub lang: LanguageId,
    pub saved_version: u64,
    pub undo: Vec<Snap>,
    pub redo: Vec<Snap>,
    pub typing_run: bool,
    pub comp: CompletionState,
    pub hl: Vec<Vec<Span>>,
    pub hl_version: u64,
    pub galleys: std::collections::HashMap<usize, (u64, Arc<Galley>)>,
    pub diagnostics: Vec<AiDiagnostic>,
    pub diag_version: u64,
}

impl Tab {
    pub fn untitled() -> Self {
        Tab {
            path: None,
            buffer: Buffer::new(),
            cursor: Cursor::new(0, 0),
            anchor: Cursor::new(0, 0),
            scroll_y: 0.0,
            want_focus: true,
            lang: LanguageId::PlainText,
            saved_version: 0,
            undo: Vec::new(),
            redo: Vec::new(),
            typing_run: false,
            comp: CompletionState::default(),
            hl: vec![Vec::new()],
            hl_version: 0,
            galleys: std::collections::HashMap::new(),
            diagnostics: Vec::new(),
            diag_version: u64::MAX,
        }
    }

    pub fn from_file(path: PathBuf, text: String) -> Self {
        let lang = LanguageId::from_path(&path);
        let mut t = Tab::untitled();
        t.buffer = Buffer::from_text(&text);
        t.path = Some(path);
        t.lang = lang;
        t.saved_version = t.buffer.version();
        // Força o realce a ser computado na primeira exibição.
        t.hl_version = u64::MAX;
        t.diag_version = u64::MAX;
        t
    }

    pub fn name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Sem título".to_string())
    }

    pub fn is_dirty(&self) -> bool {
        self.buffer.version() != self.saved_version
    }

    pub fn snap(&self) -> Snap {
        Snap {
            lines: self.buffer.lines.clone(),
            cursor: self.cursor,
            anchor: self.anchor,
        }
    }

    pub fn restore(&mut self, s: Snap) {
        self.buffer.lines = s.lines;
        self.buffer.version = self.buffer.version.wrapping_add(1);
        self.cursor = s.cursor;
        self.anchor = s.anchor;
        self.cursor.clamp(&self.buffer);
        self.anchor.clamp(&self.buffer);
    }

    pub fn begin_edit(&mut self) {
        self.undo.push(self.snap());
        self.redo.clear();
        self.typing_run = false;
        if self.undo.len() > 500 {
            self.undo.remove(0);
        }
    }

    pub fn undo(&mut self) {
        if let Some(s) = self.undo.pop() {
            self.redo.push(self.snap());
            self.restore(s);
        }
    }

    pub fn redo(&mut self) {
        if let Some(s) = self.redo.pop() {
            self.undo.push(self.snap());
            self.restore(s);
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum PromptKind {
    OpenFolder,
    SaveAs,
    GoToLine,
}

const COMMANDS: &[&str] = &[
    "Novo Arquivo",
    "Abrir Pasta...",
    "Salvar",
    "Salvar Como...",
    "Fechar Aba",
    "Ir para Linha...",
    "Alternar Barra Lateral",
    "Alternar Painel de IA",
    "Alternar Tema",
    "Alternar Comentario",
    "Aumentar Fonte",
    "Diminuir Fonte",
];

const CONFIG_PATH: &str = "openbrains-config.json";

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Config {
    theme_dark: Option<bool>,
    font_size: Option<f32>,
    workspace: Option<String>,
    show_sidebar: Option<bool>,
    show_ai: Option<bool>,
}

pub struct EditorApp {
    tabs: Vec<Tab>,
    active: usize,
    workspace: PathBuf,
    show_sidebar: bool,
    show_ai: bool,
    expanded: HashSet<PathBuf>,
    workspace_files: Vec<PathBuf>,
    files_dirty: bool,
    theme: Theme,
    ai: AiEngine,
    palette: Option<PaletteState>,
    prompt: Option<PromptState>,
    status: String,
}

struct PaletteState {
    query: String,
    index: usize,
    want_focus: bool,
}

struct PromptState {
    title: String,
    buffer: String,
    kind: PromptKind,
    want_focus: bool,
}

impl EditorApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let config = std::fs::read_to_string(CONFIG_PATH)
            .ok()
            .and_then(|s| serde_json::from_str::<Config>(&s).ok())
            .unwrap_or_default();

        let dark = config.theme_dark.unwrap_or(true);
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut theme = theme;
        if let Some(fs) = config.font_size {
            theme.font_size = fs;
        }

        let workspace = config
            .workspace
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let mut app = EditorApp {
            tabs: vec![Tab::untitled()],
            active: 0,
            workspace,
            show_sidebar: config.show_sidebar.unwrap_or(true),
            show_ai: config.show_ai.unwrap_or(false),
            expanded: HashSet::new(),
            workspace_files: Vec::new(),
            files_dirty: true,
            theme,
            ai: AiEngine::new(),
            palette: None,
            prompt: None,
            status: String::from("Bem-vindo ao OpenBrains"),
        };

        // Fonte monoespaçada por padrão na UI padrão (opcional).
        let _ = &cc.egui_ctx;
        app.refresh_files();
        app
    }

    fn save_config(&self) {
        let cfg = Config {
            theme_dark: Some(self.theme.dark),
            font_size: Some(self.theme.font_size),
            workspace: Some(self.workspace.to_string_lossy().to_string()),
            show_sidebar: Some(self.show_sidebar),
            show_ai: Some(self.show_ai),
        };
        if let Ok(s) = serde_json::to_string_pretty(&cfg) {
            let _ = std::fs::write(CONFIG_PATH, s);
        }
    }

    fn set_status(&mut self, msg: impl Into<String>) {
        self.status = msg.into();
    }

    // ---------- Arquivos / pastas ----------

    fn refresh_files(&mut self) {
        let mut v: Vec<PathBuf> = Vec::new();
        let mut stack = vec![self.workspace.clone()];
        let mut count = 0usize;
        while let Some(dir) = stack.pop() {
            if let Ok(rd) = std::fs::read_dir(&dir) {
                let mut items: Vec<PathBuf> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
                items.sort();
                for p in items {
                    let name = p
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if name == "target"
                        || name == ".git"
                        || name == "node_modules"
                        || name.starts_with('.')
                    {
                        continue;
                    }
                    if p.is_dir() {
                        stack.push(p);
                    } else {
                        v.push(p);
                        count += 1;
                        if count > 5000 {
                            break;
                        }
                    }
                }
            }
            if count > 5000 {
                break;
            }
        }
        self.workspace_files = v;
        self.files_dirty = false;
    }

    fn open_file(&mut self, path: PathBuf) {
        // foca aba já aberta
        for (i, t) in self.tabs.iter().enumerate() {
            if t.path.as_ref() == Some(&path) {
                self.active = i;
                self.tabs[i].want_focus = true;
                return;
            }
        }
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let tab = Tab::from_file(path.clone(), text);
                if self.tabs.len() >= MAX_TABS {
                    self.tabs.remove(0);
                }
                self.tabs.push(tab);
                self.active = self.tabs.len() - 1;
                self.set_status(format!("Aberto {}", path.display()));
            }
            Err(e) => self.set_status(format!("Erro ao abrir {}: {e}", path.display())),
        }
    }

    fn new_untitled(&mut self) {
        if self.tabs.len() >= MAX_TABS {
            self.tabs.remove(0);
        }
        self.tabs.push(Tab::untitled());
        self.active = self.tabs.len() - 1;
    }

    fn close_tab(&mut self, i: usize) {
        if i < self.tabs.len() {
            self.tabs.remove(i);
            if self.tabs.is_empty() {
                self.tabs.push(Tab::untitled());
            }
            if self.active >= self.tabs.len() {
                self.active = self.tabs.len() - 1;
            }
        }
    }

    fn save_active(&mut self) {
        let path = match self.tabs.get(self.active).and_then(|t| t.path.clone()) {
            Some(p) => p,
            None => {
                self.prompt_save_as();
                return;
            }
        };
        if let Some(tab) = self.tabs.get_mut(self.active) {
            match std::fs::write(&path, tab.buffer.to_text()) {
                Ok(()) => {
                    tab.saved_version = tab.buffer.version();
                    self.set_status(format!("Salvo {}", path.display()));
                }
                Err(e) => self.set_status(format!("Erro ao salvar: {e}")),
            }
        }
    }

    fn prompt_save_as(&mut self) {
        let default = self
            .tabs
            .get(self.active)
            .and_then(|t| t.path.clone())
            .unwrap_or_else(|| self.workspace.join("novo.txt"));
        self.prompt = Some(PromptState {
            title: "Salvar como (caminho)".into(),
            buffer: default.to_string_lossy().to_string(),
            kind: PromptKind::SaveAs,
            want_focus: true,
        });
    }

    fn prompt_open_folder(&mut self) {
        self.prompt = Some(PromptState {
            title: "Abrir pasta (caminho)".into(),
            buffer: self.workspace.to_string_lossy().to_string(),
            kind: PromptKind::OpenFolder,
            want_focus: true,
        });
    }

    fn prompt_goto_line(&mut self) {
        self.prompt = Some(PromptState {
            title: "Ir para linha".into(),
            buffer: String::new(),
            kind: PromptKind::GoToLine,
            want_focus: true,
        });
    }

    fn switch_theme(&mut self) {
        self.theme = if self.theme.dark {
            let mut t = Theme::light();
            t.font_size = self.theme.font_size;
            t
        } else {
            let mut t = Theme::dark();
            t.font_size = self.theme.font_size;
            t
        };
        self.save_config();
    }

    fn change_font(&mut self, delta: f32) {
        self.theme.font_size = (self.theme.font_size + delta).clamp(8.0, 40.0);
        self.save_config();
    }

    fn toggle_comment(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active) {
            tab.begin_edit();
            let prefix = tab.lang.line_comment().to_string();
            tab.buffer.toggle_line_comment(&mut tab.cursor, &prefix);
        }
    }

    fn run_command(&mut self, label: &str) {
        match label {
            "Novo Arquivo" => self.new_untitled(),
            "Abrir Pasta..." => self.prompt_open_folder(),
            "Salvar" => self.save_active(),
            "Salvar Como..." => self.prompt_save_as(),
            "Fechar Aba" => {
                let i = self.active;
                self.close_tab(i);
            }
            "Ir para Linha..." => self.prompt_goto_line(),
            "Alternar Barra Lateral" => self.show_sidebar = !self.show_sidebar,
            "Alternar Painel de IA" => self.show_ai = !self.show_ai,
            "Alternar Tema" => self.switch_theme(),
            "Alternar Comentario" => self.toggle_comment(),
            "Aumentar Fonte" => self.change_font(1.0),
            "Diminuir Fonte" => self.change_font(-1.0),
            _ => {}
        }
        self.save_config();
    }

    // ---------- Atalhos globais ----------

    fn handle_shortcuts(&mut self, ctx: &Context) {
        if self.palette.is_some() || self.prompt.is_some() {
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
                self.palette = None;
                self.prompt = None;
            }
            return;
        }
        ctx.input_mut(|i| {
            // Combinações com Shift ANTES das sem shift (matches_logically ignora shift).
            if i.consume_key(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::P) {
                self.open_palette();
            }
            if i.consume_key(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::S) {
                self.prompt_save_as();
            }
            if i.consume_key(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::Z) {
                if let Some(t) = self.tabs.get_mut(self.active) {
                    t.redo();
                }
            }
            if i.consume_key(Modifiers::COMMAND, Key::P) {
                self.open_palette();
            }
            if i.consume_key(Modifiers::COMMAND, Key::S) {
                self.save_active();
            }
            if i.consume_key(Modifiers::COMMAND, Key::N) {
                self.new_untitled();
            }
            if i.consume_key(Modifiers::COMMAND, Key::O) {
                self.prompt_open_folder();
            }
            if i.consume_key(Modifiers::COMMAND, Key::W) {
                let i = self.active;
                self.close_tab(i);
            }
            if i.consume_key(Modifiers::COMMAND, Key::G) {
                self.prompt_goto_line();
            }
            if i.consume_key(Modifiers::COMMAND, Key::Z) {
                if let Some(t) = self.tabs.get_mut(self.active) {
                    t.undo();
                }
            }
            if i.consume_key(Modifiers::COMMAND, Key::Slash) {
                self.toggle_comment();
            }
        });
    }

    fn open_palette(&mut self) {
        if self.files_dirty {
            self.refresh_files();
        }
        self.palette = Some(PaletteState {
            query: String::new(),
            index: 0,
            want_focus: true,
        });
    }

    // ---------- File tree ----------

    fn collect_entries(&self) -> Vec<(PathBuf, usize, bool)> {
        let mut out = Vec::new();
        let mut budget = 8000usize;
        collect_dir(&self.workspace, 0, &self.expanded, &mut out, &mut budget);
        out
    }
}

fn collect_dir(
    dir: &Path,
    depth: usize,
    expanded: &HashSet<PathBuf>,
    out: &mut Vec<(PathBuf, usize, bool)>,
    budget: &mut usize,
) {
    if *budget == 0 {
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut items: Vec<(PathBuf, bool)> = rd
        .filter_map(|e| e.ok())
        .map(|e| {
            let p = e.path();
            let is_dir = p.is_dir();
            (p, is_dir)
        })
        .filter(|(p, _)| {
            p.file_name()
                .map(|n| !n.to_string_lossy().starts_with('.'))
                .unwrap_or(false)
        })
        .collect();
    items.sort_by(|a, b| match (a.1, b.1) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.0.cmp(&b.0),
    });
    for (p, is_dir) in items {
        if *budget == 0 {
            break;
        }
        *budget -= 1;
        out.push((p.clone(), depth, is_dir));
        if is_dir && expanded.contains(&p) {
            collect_dir(&p, depth + 1, expanded, out, budget);
        }
    }
}

impl App for EditorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        let ctx = ui.ctx().clone();
        self.handle_shortcuts(&ctx);
        self.theme.apply(&ctx);

        // Barra superior.
        egui::Panel::top(Id::new("ob_menu"))
            .frame(self.menu_frame())
            .show(&mut *ui, |ui| self.menu_bar(ui));

        // Barra de status.
        egui::Panel::bottom(Id::new("ob_status"))
            .frame(self.menu_frame())
            .show(&mut *ui, |ui| self.status_bar(ui));

        if self.show_sidebar {
            egui::Panel::left(Id::new("ob_sidebar"))
                .frame(self.panel_frame())
                .resizable(true)
                .default_size(230.0)
                .show(&mut *ui, |ui| self.sidebar(ui));
        }

        if self.show_ai {
            egui::Panel::right(Id::new("ob_ai"))
                .frame(self.panel_frame())
                .resizable(true)
                .default_size(320.0)
                .show(&mut *ui, |ui| self.ai_panel(ui));
        }

        egui::CentralPanel::default_margins()
            .frame(self.central_frame())
            .show(&mut *ui, |ui| self.central(ui));

        self.overlays(&ctx);
    }
}

impl EditorApp {
    fn menu_frame(&self) -> egui::Frame {
        egui::Frame::NONE
            .fill(self.theme.panel_bg)
            .inner_margin(Margin::symmetric(10, 6))
    }

    fn panel_frame(&self) -> egui::Frame {
        egui::Frame::NONE
            .fill(self.theme.panel_bg)
            .inner_margin(Margin::symmetric(8, 8))
    }

    fn central_frame(&self) -> egui::Frame {
        egui::Frame::NONE
            .fill(self.theme.bg)
            .inner_margin(Margin::ZERO)
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("◆ OpenBrains")
                    .color(self.theme.accent)
                    .strong(),
            );
            ui.separator();
            if ui
                .add(egui::Button::new(RichText::new("Novo").small()))
                .clicked()
            {
                self.new_untitled();
            }
            if ui
                .add(egui::Button::new(RichText::new("Abrir Pasta").small()))
                .clicked()
            {
                self.prompt_open_folder();
            }
            if ui
                .add(egui::Button::new(RichText::new("Salvar").small()))
                .clicked()
            {
                self.save_active();
            }
            if ui
                .add(egui::Button::new(
                    RichText::new("Paleta  (Ctrl+Shift+P)").small(),
                ))
                .clicked()
            {
                self.open_palette();
            }
            let ai_label = if self.show_ai { "IA ●" } else { "IA ○" };
            if ui
                .add(egui::Button::new(RichText::new(ai_label).small()))
                .clicked()
            {
                self.show_ai = !self.show_ai;
            }
            let theme_label = if self.theme.dark {
                "Tema: Escuro"
            } else {
                "Tema: Claro"
            };
            if ui
                .add(egui::Button::new(RichText::new(theme_label).small()))
                .clicked()
            {
                self.switch_theme();
            }
        });
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let ws = self
                .workspace
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| self.workspace.to_string_lossy().to_string());
            ui.label(
                RichText::new(format!("📁 {ws}"))
                    .color(self.theme.text_dim)
                    .small(),
            );

            ui.separator();
            if let Some(tab) = self.tabs.get(self.active) {
                let dirty = if tab.is_dirty() { " •" } else { "" };
                ui.label(
                    RichText::new(format!("{}{}", tab.name(), dirty))
                        .color(self.theme.text)
                        .small(),
                );
                ui.separator();
                ui.label(
                    RichText::new(tab.lang.name())
                        .color(self.theme.text_dim)
                        .small(),
                );
                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "Ln {}, Col {}",
                        tab.cursor.line + 1,
                        tab.cursor.col + 1
                    ))
                    .color(self.theme.text_dim)
                    .small(),
                );
                ui.separator();
                let errs = tab
                    .diagnostics
                    .iter()
                    .filter(|d| d.severity == openbrains_ai::AiSeverity::Error)
                    .count();
                let warns = tab
                    .diagnostics
                    .iter()
                    .filter(|d| d.severity == openbrains_ai::AiSeverity::Warning)
                    .count();
                ui.label(
                    RichText::new(format!("⚠ {warns}  ⛔ {errs}"))
                        .color(self.theme.text_dim)
                        .small(),
                );
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(&self.status)
                        .color(self.theme.text_dim)
                        .small(),
                );
            });
        });
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("EXPLORADOR")
                    .color(self.theme.text_dim)
                    .strong(),
            );
            if ui.small_button("⟳").clicked() {
                self.refresh_files();
            }
        });
        ui.separator();
        let entries = self.collect_entries();
        let mut to_open: Option<PathBuf> = None;
        let mut to_toggle: Option<PathBuf> = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (path, depth, is_dir) in entries {
                let indent = depth as f32 * 14.0;
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let label = if is_dir {
                    let open = self.expanded.contains(&path);
                    let arrow = if open { "▾ " } else { "▸ " };
                    format!("{arrow}{name}")
                } else {
                    format!("  {name}")
                };
                ui.horizontal(|ui| {
                    ui.add_space(indent);
                    let color = if is_dir {
                        self.theme.text
                    } else {
                        self.theme.text_dim
                    };
                    let r = ui.add(egui::Button::new(RichText::new(label).color(color).small()));
                    if r.clicked() {
                        if is_dir {
                            to_toggle = Some(path.clone());
                        } else {
                            to_open = Some(path.clone());
                        }
                    }
                });
            }
        });
        if let Some(p) = to_toggle {
            if self.expanded.contains(&p) {
                self.expanded.remove(&p);
            } else {
                self.expanded.insert(p);
            }
        }
        if let Some(p) = to_open {
            self.open_file(p);
        }
    }

    fn ai_panel(&mut self, ui: &mut egui::Ui) {
        // Recalcula diagnóstico quando o buffer muda.
        if let Some(tab) = self.tabs.get_mut(self.active) {
            if tab.diag_version != tab.buffer.version() {
                tab.diagnostics = self.ai.analyze(tab.lang, &tab.buffer.lines);
                tab.diag_version = tab.buffer.version();
            }
        }

        ui.label(
            RichText::new("ASSISTENTE DE IA")
                .color(self.theme.accent)
                .strong(),
        );
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Some(tab) = self.tabs.get(self.active) {
                ui.label(
                    RichText::new(format!("Linguagem: {}", tab.lang.name()))
                        .color(self.theme.text_dim),
                );
                ui.add_space(4.0);

                ui.label(RichText::new("Diagnósticos").strong());
                if tab.diagnostics.is_empty() {
                    ui.label(
                        RichText::new("✓ Nenhum problema encontrado")
                            .color(self.theme.hint)
                            .small(),
                    );
                } else {
                    for d in &tab.diagnostics {
                        let color = match d.severity {
                            openbrains_ai::AiSeverity::Error => self.theme.error,
                            openbrains_ai::AiSeverity::Warning => self.theme.warning,
                            openbrains_ai::AiSeverity::Info => self.theme.info,
                            openbrains_ai::AiSeverity::Hint => self.theme.hint,
                        };
                        ui.horizontal_wrapped(|ui| {
                            ui.colored_label(color, format!("L{}:", d.line + 1));
                            ui.label(RichText::new(&d.message).small().color(self.theme.text));
                        });
                    }
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Sugestões para esta linha").strong());
                let items = self.ai.complete(
                    tab.lang,
                    &tab.buffer.lines,
                    tab.cursor.line,
                    tab.cursor.col,
                    10,
                );
                if items.is_empty() {
                    ui.label(RichText::new("—").color(self.theme.text_dim).small());
                } else {
                    for it in items.iter().take(10) {
                        ui.horizontal(|ui| {
                            ui.colored_label(self.theme.accent, &it.label);
                            ui.label(
                                RichText::new(format!("({})", it.detail))
                                    .small()
                                    .color(self.theme.text_dim),
                            );
                        });
                    }
                }
            }
        });
    }

    fn central(&mut self, ui: &mut egui::Ui) {
        // Aba(s) no topo.
        ui.horizontal_wrapped(|ui| {
            let mut switch: Option<usize> = None;
            let mut close: Option<usize> = None;
            for i in 0..self.tabs.len() {
                let name = self.tabs[i].name();
                let dirty = self.tabs[i].is_dirty();
                let title = if dirty { format!("● {name}") } else { name };
                let sel = i == self.active;
                let r = ui.selectable_label(sel, RichText::new(title).small());
                if r.clicked() {
                    switch = Some(i);
                }
                if ui.small_button("✕").clicked() {
                    close = Some(i);
                }
                ui.separator();
            }
            if let Some(i) = switch {
                self.active = i;
                if let Some(t) = self.tabs.get_mut(i) {
                    t.want_focus = true;
                }
            }
            if let Some(i) = close {
                self.close_tab(i);
            }
        });
        ui.separator();

        if let Some(tab) = self.tabs.get_mut(self.active) {
            crate::editor_widget::show(ui, tab, &self.theme, &self.ai);
        } else {
            ui.add_space(40.0);
            ui.label(
                RichText::new("Abra um arquivo pelo explorador ou pela paleta (Ctrl+Shift+P).")
                    .color(self.theme.text_dim),
            );
        }
    }

    fn overlays(&mut self, ctx: &Context) {
        // Paleta de comandos.
        if let Some(mut st) = self.palette.take() {
            let mut open = true;
            let mut run: Option<String> = None;
            let mut open_file_item: Option<PathBuf> = None;
            egui::Window::new("Paleta de Comandos")
                .collapsible(false)
                .resizable(false)
                .default_width(580.0)
                .anchor(egui::Align2::CENTER_TOP, Vec2::new(0.0, 70.0))
                .open(&mut open)
                .show(ctx, |ui| {
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut st.query)
                            .hint_text("Digite um comando ou nome de arquivo...")
                            .desired_width(560.0),
                    );
                    if st.want_focus {
                        resp.request_focus();
                        st.want_focus = false;
                    }
                    ui.separator();
                    let q = st.query.to_lowercase();

                    // Itens: comandos + arquivos.
                    let mut items: Vec<PaletteItem> = Vec::new();
                    for c in COMMANDS {
                        if q.is_empty() || c.to_lowercase().contains(&q) {
                            items.push(PaletteItem::Cmd(*c));
                        }
                    }
                    for p in &self.workspace_files {
                        if q.is_empty() || p.to_string_lossy().to_lowercase().contains(&q) {
                            items.push(PaletteItem::File(p.clone()));
                        }
                        if items.len() > 200 {
                            break;
                        }
                    }
                    if st.index >= items.len() {
                        st.index = 0;
                    }

                    egui::ScrollArea::vertical()
                        .max_height(320.0)
                        .show(ui, |ui| {
                            for (idx, item) in items.iter().enumerate() {
                                let sel = idx == st.index;
                                let (label, color) = match item {
                                    PaletteItem::Cmd(c) => (format!("› {c}"), self.theme.accent),
                                    PaletteItem::File(p) => (
                                        p.file_name()
                                            .map(|n| n.to_string_lossy().to_string())
                                            .unwrap_or_default(),
                                        self.theme.text,
                                    ),
                                };
                                let r = ui.selectable_label(sel, RichText::new(label).color(color));
                                if r.clicked() {
                                    st.index = idx;
                                    match item {
                                        PaletteItem::Cmd(c) => run = Some((*c).to_string()),
                                        PaletteItem::File(p) => open_file_item = Some(p.clone()),
                                    }
                                }
                            }
                        });

                    // Navegação/Enter.
                    let mut enter = false;
                    let mut up = false;
                    let mut down = false;
                    ui.input(|i| {
                        enter = i.key_pressed(Key::Enter);
                        up = i.key_pressed(Key::ArrowUp);
                        down = i.key_pressed(Key::ArrowDown);
                    });
                    if up && st.index > 0 {
                        st.index -= 1;
                    }
                    if down && st.index + 1 < items.len() {
                        st.index += 1;
                    }
                    if enter {
                        match items.get(st.index) {
                            Some(PaletteItem::Cmd(c)) => run = Some((*c).to_string()),
                            Some(PaletteItem::File(p)) => open_file_item = Some(p.clone()),
                            None => {}
                        }
                    }
                });
            if !open {
                self.palette = None;
            } else {
                self.palette = Some(st);
            }
            if let Some(cmd) = run {
                self.palette = None;
                self.run_command(&cmd);
            }
            if let Some(p) = open_file_item {
                self.palette = None;
                self.open_file(p);
            }
        }

        // Prompt.
        if let Some(mut st) = self.prompt.take() {
            let mut open = true;
            let mut done = false;
            egui::Window::new(st.title.clone())
                .collapsible(false)
                .resizable(false)
                .default_width(460.0)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::new(0.0, -40.0))
                .open(&mut open)
                .show(ctx, |ui| {
                    let resp =
                        ui.add(egui::TextEdit::singleline(&mut st.buffer).desired_width(440.0));
                    if st.want_focus {
                        resp.request_focus();
                        st.want_focus = false;
                    }
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() {
                            done = true;
                        }
                        if ui.button("Cancelar").clicked() {
                            open = false;
                        }
                    });
                    if ui.input(|i| i.key_pressed(Key::Enter)) {
                        done = true;
                    }
                });
            if !open {
                self.prompt = None;
            } else {
                if done {
                    self.prompt = None;
                    self.execute_prompt(st.kind, &st.buffer);
                } else {
                    self.prompt = Some(st);
                }
            }
        }
    }

    fn execute_prompt(&mut self, kind: PromptKind, value: &str) {
        let value = value.trim();
        match kind {
            PromptKind::OpenFolder => {
                let p = PathBuf::from(value);
                if p.is_dir() {
                    self.workspace = p;
                    self.expanded.clear();
                    self.refresh_files();
                    self.set_status(format!("Pasta: {}", self.workspace.display()));
                    self.save_config();
                } else {
                    self.set_status("Caminho de pasta inválido");
                }
            }
            PromptKind::SaveAs => {
                let p = PathBuf::from(value);
                if let Some(tab) = self.tabs.get_mut(self.active) {
                    match std::fs::write(&p, tab.buffer.to_text()) {
                        Ok(()) => {
                            tab.path = Some(p.clone());
                            tab.lang = LanguageId::from_path(&p);
                            tab.saved_version = tab.buffer.version();
                            self.set_status(format!("Salvo {}", p.display()));
                        }
                        Err(e) => self.set_status(format!("Erro ao salvar: {e}")),
                    }
                }
            }
            PromptKind::GoToLine => {
                if let Ok(n) = value.parse::<usize>() {
                    if let Some(tab) = self.tabs.get_mut(self.active) {
                        let line = n
                            .saturating_sub(1)
                            .min(tab.buffer.line_count().saturating_sub(1));
                        tab.cursor.line = line;
                        tab.cursor.end(&tab.buffer);
                        tab.anchor = tab.cursor;
                        tab.want_focus = true;
                    }
                }
            }
        }
    }
}

enum PaletteItem {
    Cmd(&'static str),
    File(PathBuf),
}
