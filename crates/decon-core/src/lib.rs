//! Core library for contamination detection in text training data.
//!
//! This crate provides the fundamental algorithms and data structures
//! for detecting when training data contains text from evaluation datasets.

pub mod common;
pub mod compare;
pub mod detect;
pub mod evals;
pub mod review;
pub mod server;

// Re-export key types and functions for convenient access
pub use common::detection_config::{read_config, Config};
pub use common::text::clean_text;
pub use common::tokenizer::OmniTokenizer;

pub use detect::config::execute_detect;
pub use detect::contamination_entry::SimpleContaminationEntry;
pub use detect::detection::contamination_detect;
pub use detect::reference_index::SimpleReferenceIndex;

pub use compare::execute_compare;
pub use evals::execute_evals;
pub use review::execute_review;
pub use server::config::execute_server;
