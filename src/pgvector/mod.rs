//! pgvector support for Rust
//!
//! More details on [pgvector](https://github.com/pgvector/pgvector)

#![doc = include_str!("./README.md")]

mod vector;
pub use vector::Vector;

mod postgres_ext;
mod sqlx_ext;
