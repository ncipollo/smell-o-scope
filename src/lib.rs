//! Track down and visualize code smell. The `smell-o-scope` binary is a thin
//! CLI over this library.

pub mod cli;
pub mod feature;
pub mod render;
#[cfg(test)]
mod testing;
