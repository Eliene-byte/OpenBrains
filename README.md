# ◆ OpenBrains

Um **IDE / editor de código ultra-leve, 100% Rust**, com um motor de **IA determinístico** embutido (offline, sem depender de serviços externos).

O objetivo: uma ferramenta no espírito de um "VS Code", mas extremamente leve, escrita inteiramente em Rust, compilada e testada automaticamente no **GitHub Actions** para Windows, Linux e macOS.

## ✨ Funcionalidades (v0.1)

- **Editor customizado** desenhado pixel-a-pixel (egui):
  - Gutter com números de linha, linha atual destacada, seleção, caret com blink
  - Auto-indent, auto-fechamento de pares `() [] {} '' "" \`\``
  - Undo/redo, navegação por palavras, página, Home/End
  - Copiar/Recortar/Colar integrados ao clipboard do sistema
- **Realce de sintaxe próprio** (sem C, sem dependência externa pesada) para
  Rust, Python, JavaScript/TypeScript(+JSX/TSX), C/C++, Go, Java, C#, SQL, Shell,
  JSON, TOML, YAML, HTML, CSS/SCSS, Markdown e texto puro.
- **Explorador de arquivos** com árvore, abas de arquivos, indicador de modificação.
- **Paleta de comandos** (`Ctrl+Shift+P` / `Ctrl+P`) com comandos e abertura rápida de arquivos.
- **Painel de IA** com:
  - **Diagnósticos determinísticos**: espaços no fim da linha, linhas longas,
    `TODO/FIXME`, brackets desbalanceados, statements de debug esquecidos, dicas por linguagem.
  - **Autocomplete** que combina o **modelo de conhecimento por linguagem**
    (keywords, tipos, builtins, snippets) com os **símbolos do próprio arquivo**.
- **Temas** claro e escuro, fonte ajustável, configuração persistida em `openbrains-config.json`.
- **Status bar** com linguagem, posição do cursor, contagem de diagnósticos etc.

## ⌨️ Atalhos

| Ação | Atalho |
|------|--------|
| Paleta de comandos | `Ctrl+Shift+P` ou `Ctrl+P` |
| Salvar | `Ctrl+S` |
| Salvar como | `Ctrl+Shift+S` |
| Novo arquivo | `Ctrl+N` |
| Abrir pasta | `Ctrl+O` |
| Fechar aba | `Ctrl+W` |
| Ir para linha | `Ctrl+G` |
| Desfazer / Refazer | `Ctrl+Z` / `Ctrl+Shift+Z` |
| Alternar comentário | `Ctrl+/` |
| Autocomplete | `Ctrl+Space` |
| Selecionar tudo | `Ctrl+A` |
| Fechar paleta/prompt | `Esc` |

## 🧠 Sobre a "IA"

O motor de IA (`crates/ai`) é **determinístico e 100% offline**: não baixa modelos
nem chama APIs. Ele é formado por:

- `complete` — sugestões de autocomplete a partir do modelo da linguagem +
  símbolos do arquivo (funções, variáveis).
- `analyze` — diagnósticos por regras.

A arquitetura é **plugável**: no futuro é possível adicionar um backend de LLM
local (ex.: `candle`) sem mudar quem consome o motor.

## 🏗️ Arquitetura (workspace Cargo)

```
crates/
  common/       -> detecção de linguagem (sem deps)
  editor-core/  -> buffer de texto, cursor, realce de sintaxe (Rust puro)
  ai/           -> motor de IA determinístico (completion + análise)
  app/          -> GUI (eframe/egui), binário `openbrains`
```

## 🚀 Compilar e rodar

Requer Rust estável. Em Windows, instale as **Build Tools do Visual Studio**
(workload "Desenvolvimento para desktop com C++").

```bash
cargo run --release
```

Binário resultante: `target/release/openbrains`.

## 🤖 CI

O GitHub Actions (`.github/workflows/ci.yml`) executa, em Windows/Ubuntu/macOS:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo build --release`

## 📄 Licença

MIT (veja o arquivo `LICENSE`).
