//! Tiny native window shell for the first GUI milestone.
//!
//! This deliberately uses only std/tokio so the core remains buildable without
//! a GUI toolkit. A platform frontend can later consume BasicUi::state().

use super::BasicUi;
use std::io::{self, Write};

pub fn print_status(ui: &BasicUi) {
    // GUI frontends should call BasicUi::state() asynchronously. This helper
    // exists as a deterministic fallback for console launches.
    let _ = io::stdout().flush();
    let _ = ui;
}
