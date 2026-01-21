//! Input validation and sanitization module.
//!
//! Provides centralized validation for text fields including:
//! - Length constraints
//! - Whitespace trimming and normalization
//! - Phone number format validation
//! - Email format validation
//! - Log-safe sanitization

pub mod constraints;
pub mod sanitize;

pub use constraints::*;
pub use sanitize::*;
