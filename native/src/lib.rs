#[macro_use]
extern crate napi_derive;

pub mod fuzzy;
pub mod diff;
pub mod hash;

pub use fuzzy::*;
pub use diff::*;
pub use hash::*;

#[napi]
pub fn hello() -> String {
    "Hello from COD Native!".to_string()
}
