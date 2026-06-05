//! Filesystem layer: the *only* part of the crate that writes output, and it does so
//! safely.
//!
//! * [`output_tree`] validates untrusted zone/link names and maps them to paths strictly
//!   inside the explicit output root, then writes zone files and links.
//! * [`atomic_write`] performs the temp-file-then-rename dance so output is never observed
//!   half-written and is never silently clobbered.

pub mod atomic_write;
pub mod output_tree;

pub use atomic_write::write_atomic;
pub use output_tree::{is_contained, safe_relative_path, write_link, write_zone_file};
