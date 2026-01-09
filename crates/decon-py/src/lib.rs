//! Python bindings for the decon contamination detection library.
//!
//! This module provides PyO3-based Python bindings for the core decon functionality.

use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use std::path::PathBuf;

use decon_core::common::detection_config::{create_default_config, Config as RustConfig};
use decon_core::common::text::{clean_text as rust_clean_text, default_punctuation_chars};
use decon_core::common::tokenizer::OmniTokenizer as RustTokenizer;
use decon_core::detect::config::execute_detect;
use decon_core::detect::args::DetectArgs;
use decon_core::detect::common_args::CommonDetectionArgs;

// =============================================================================
// PyConfig - Python wrapper for Config
// =============================================================================

/// Configuration for contamination detection.
///
/// Example:
///     config = decon.Config(
///         training_dir="/path/to/training",
///         evals_dir="/path/to/evals",
///         report_output_dir="/path/to/reports",
///     )
#[pyclass(name = "Config")]
#[derive(Clone)]
pub struct PyConfig {
    inner: RustConfig,
}

#[pymethods]
impl PyConfig {
    #[new]
    #[pyo3(signature = (
        training_dir,
        evals_dir,
        report_output_dir,
        *,
        ngram_size = 5,
        tokenizer = "cl100k",
        contamination_score_threshold = 0.8,
        content_key = "text",
        verbose = false,
        purify = false,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        training_dir: &str,
        evals_dir: &str,
        report_output_dir: &str,
        ngram_size: usize,
        tokenizer: &str,
        contamination_score_threshold: f32,
        content_key: &str,
        verbose: bool,
        purify: bool,
    ) -> PyResult<Self> {
        let mut config = create_default_config();

        config.training_dir = PathBuf::from(training_dir);
        config.evals_dir = PathBuf::from(evals_dir);
        config.report_output_dir = PathBuf::from(report_output_dir);
        config.ngram_size = ngram_size;
        config.tokenizer_str = tokenizer.to_string();
        config.contamination_score_threshold = contamination_score_threshold;
        config.content_key = content_key.to_string();
        config.verbose = verbose;
        config.purify = purify;

        Ok(PyConfig { inner: config })
    }

    #[getter]
    fn training_dir(&self) -> String {
        self.inner.training_dir.display().to_string()
    }

    #[getter]
    fn evals_dir(&self) -> String {
        self.inner.evals_dir.display().to_string()
    }

    #[getter]
    fn report_output_dir(&self) -> String {
        self.inner.report_output_dir.display().to_string()
    }

    #[getter]
    fn mode(&self) -> String {
        self.inner.mode.clone()
    }

    #[getter]
    fn ngram_size(&self) -> usize {
        self.inner.ngram_size
    }

    #[getter]
    fn tokenizer(&self) -> String {
        self.inner.tokenizer_str.clone()
    }

    #[getter]
    fn contamination_score_threshold(&self) -> f32 {
        self.inner.contamination_score_threshold
    }

    #[getter]
    fn content_key(&self) -> String {
        self.inner.content_key.clone()
    }

    #[getter]
    fn sample_every_m_tokens(&self) -> usize {
        self.inner.sample_every_m_tokens
    }

    #[getter]
    fn verbose(&self) -> bool {
        self.inner.verbose
    }

    #[getter]
    fn purify(&self) -> bool {
        self.inner.purify
    }

    fn __repr__(&self) -> String {
        format!(
            "Config(training_dir='{}', evals_dir='{}', report_output_dir='{}', ngram_size={}, tokenizer='{}', threshold={})",
            self.inner.training_dir.display(),
            self.inner.evals_dir.display(),
            self.inner.report_output_dir.display(),
            self.inner.ngram_size,
            self.inner.tokenizer_str,
            self.inner.contamination_score_threshold,
        )
    }
}

// =============================================================================
// PyTokenizer - Python wrapper for OmniTokenizer
// =============================================================================

/// Tokenizer for encoding and decoding text.
///
/// Supports multiple tokenizers: r50k, p50k, p50k_edit, cl100k, o200k, uniseg.
///
/// Example:
///     tok = decon.Tokenizer("cl100k")
///     tokens = tok.encode("hello world")  # [15339, 1917]
///     text = tok.decode(tokens)  # "hello world"
#[pyclass(name = "Tokenizer")]
pub struct PyTokenizer {
    inner: RustTokenizer,
    tokenizer_name: String,
}

#[pymethods]
impl PyTokenizer {
    #[new]
    #[pyo3(signature = (name = "cl100k"))]
    fn new(name: &str) -> PyResult<Self> {
        let tokenizer = RustTokenizer::new(name)
            .map_err(|e| PyValueError::new_err(format!("Failed to create tokenizer: {}", e)))?;
        Ok(PyTokenizer {
            inner: tokenizer,
            tokenizer_name: name.to_string(),
        })
    }

    /// Get the tokenizer name.
    #[getter]
    fn name(&self) -> &str {
        &self.tokenizer_name
    }

    /// Encode text to token IDs.
    fn encode(&self, text: &str) -> Vec<usize> {
        self.inner.encode(text)
    }

    /// Decode token IDs back to text.
    fn decode(&self, tokens: Vec<usize>) -> String {
        self.inner.decode(&tokens)
    }

    /// Check if a token represents a space character.
    fn is_space_token(&self, token: usize) -> bool {
        self.inner.is_space_token(token)
    }

    fn __repr__(&self) -> String {
        format!("Tokenizer(name='{}')", self.tokenizer_name)
    }
}

// =============================================================================
// Functions
// =============================================================================

/// Run contamination detection with the given configuration.
///
/// This function runs the full detection pipeline and writes results to
/// the report_output_dir specified in the config.
///
/// Args:
///     config: Configuration for the detection run.
///
/// Returns:
///     The path to the report output directory.
///
/// Example:
///     config = decon.Config(...)
///     report_dir = decon.detect(config)
#[pyfunction]
fn detect(py: Python<'_>, config: &PyConfig) -> PyResult<String> {
    let report_dir = config.inner.report_output_dir.clone();

    // Create DetectArgs from config
    let common = CommonDetectionArgs {
        training_dir: Some(config.inner.training_dir.clone()),
        evals_dir: Some(config.inner.evals_dir.clone()),
        report_output_dir: Some(config.inner.report_output_dir.clone()),
        cleaned_output_dir: config.inner.cleaned_output_dir.clone(),
        tokenizer: Some(config.inner.tokenizer_str.clone()),
        ngram_size: Some(config.inner.ngram_size),
        sample_every_m_tokens: Some(config.inner.sample_every_m_tokens),
        contamination_score_threshold: Some(config.inner.contamination_score_threshold),
        content_key: Some(config.inner.content_key.clone()),
        verbose: config.inner.verbose,
        purify: config.inner.purify,
        config: None,
        question_max_consecutive_misses: None,
        short_answer_token_threshold: None,
        short_answer_window_length: None,
        min_long_answer_window: None,
        answer_ngram_size: None,
        passage_max_consecutive_misses: None,
        passage_ngram_size: None,
        worker_threads: None,
        eval_dedup: false,
        index_passages: None,
        index_answers: None,
        eval_min_token_length: None,
        eval_min_unique_word_count: None,
        perfect_match_decay_start: None,
        perfect_match_decay_end: None,
    };
    let args = DetectArgs { common };

    // Release GIL for CPU-intensive parallel work
    py.allow_threads(|| {
        execute_detect(&args)
            .map_err(|e| PyIOError::new_err(format!("Detection failed: {}", e)))
    })?;

    Ok(report_dir.display().to_string())
}

/// Clean text by normalizing punctuation and whitespace.
///
/// Converts to lowercase, replaces punctuation with spaces, and normalizes whitespace.
///
/// Args:
///     text: The text to clean.
///     punctuation_chars: Optional custom punctuation characters to replace.
///
/// Returns:
///     The cleaned text.
///
/// Example:
///     cleaned = decon.clean_text("Hello, World!")  # "hello world"
#[pyfunction]
#[pyo3(signature = (text, punctuation_chars = None))]
fn clean_text(text: &str, punctuation_chars: Option<&str>) -> String {
    let default_punct = default_punctuation_chars();
    let punct = punctuation_chars.unwrap_or(&default_punct);
    rust_clean_text(text, punct)
}

/// Create a default configuration.
///
/// Note: You must set training_dir, evals_dir, and report_output_dir before using.
///
/// Returns:
///     A Config with default values.
#[pyfunction]
fn default_config() -> PyConfig {
    PyConfig {
        inner: create_default_config(),
    }
}

// =============================================================================
// Module Definition
// =============================================================================

/// Python module for decon contamination detection.
#[pymodule]
fn _decon(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyConfig>()?;
    m.add_class::<PyTokenizer>()?;
    m.add_function(wrap_pyfunction!(detect, m)?)?;
    m.add_function(wrap_pyfunction!(clean_text, m)?)?;
    m.add_function(wrap_pyfunction!(default_config, m)?)?;

    // Version info
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
