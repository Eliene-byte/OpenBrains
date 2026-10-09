//! Núcleo do editor: buffer de texto, cursor e realce de sintaxe.
//! Totalmente independente de GUI, para ser testável e leve.

pub mod buffer;
pub mod cursor;
pub mod highlight;

pub use buffer::Buffer;
pub use cursor::Cursor;
pub use highlight::{highlight, Span, TokenKind};
