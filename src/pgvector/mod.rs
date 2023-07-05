//! pgvector support for Rust
//!
//! [View the docs](https://github.com/pgvector/pgvector-rust)

mod vector;
pub use vector::Vector;

mod postgres_ext;
mod sqlx_ext;
