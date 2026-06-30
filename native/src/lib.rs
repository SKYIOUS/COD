#[macro_use]
extern crate napi_derive;

pub mod fuzzy;
pub mod diff;
pub mod hash;
pub mod encoding;
pub mod jsonc;
pub mod welcome;
pub mod color;
pub mod tokenize;
pub mod search;
pub mod render;

pub use fuzzy::*;
pub use diff::*;
pub use hash::*;
pub use encoding::*;
pub use jsonc::*;
pub use welcome::*;
pub use color::*;
pub use tokenize::*;
pub use search::*;
pub use render::*;

#[napi]
pub fn hello() -> String {
    "Hello from COD Native!".to_string()
}
