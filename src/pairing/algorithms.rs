//! Pairing algorithm implementations live in focused child modules.
//!
//! Each implementation owns an immutable snapshot input, validation,
//! algorithm-specific errors, and diagnostic proposal output. Match publication
//! and table assignment remain downstream so adding an algorithm does not
//! duplicate them or give solver output order sporting meaning.

pub mod blossom_v1;
