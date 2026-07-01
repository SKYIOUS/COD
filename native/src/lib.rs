#[macro_use]
extern crate napi_derive;

pub mod color;
pub mod diff;
pub mod encoding;
pub mod fuzzy;
pub mod hash;
pub mod jsonc;
pub mod render;
pub mod search;
pub mod text_layout;
pub mod tokenize;
pub mod treesitter;
pub mod watcher;
pub mod welcome;

pub use color::*;
pub use diff::*;
pub use encoding::*;
pub use fuzzy::*;
pub use hash::*;
pub use jsonc::*;
pub use render::*;
pub use search::*;
pub use text_layout::*;
pub use tokenize::*;
pub use treesitter::*;
pub use watcher::*;
pub use welcome::*;

#[napi]
pub fn hello() -> String {
    "Hello from COD Native!".to_string()
}
