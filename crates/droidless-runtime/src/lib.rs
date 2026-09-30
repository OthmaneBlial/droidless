//! DROIDLESS's own register interpreter and Android compatibility layer.
mod activities;
mod collections;
mod components;
mod framework;
pub mod heap;
mod interpreter;
mod preferences;
mod storage;
pub mod ui;
mod vm;
pub use vm::{Runtime, Trace};
