//! DROIDLESS's own register interpreter and Android compatibility layer.
mod activities;
mod components;
mod framework;
pub mod heap;
mod interpreter;
pub mod ui;
mod vm;
pub use vm::{Runtime, Trace};
