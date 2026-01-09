//! Python bindings for the decon contamination detection library.
//!
//! This module provides PyO3-based Python bindings for the core decon functionality.

use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use std::path::{Path, PathBuf};

use decon_core::common::{
    create_default_config, default_punctuation_chars, generate_temp_dir, is_quiet_mode,
    read_config as rust_read_config, validate_config, Config as RustConfig,
    SUPPORTED_DATA_EXTENSIONS,
};
use decon_core::common::text::clean_text as rust_clean_text;
use decon_core::common::tokenizer::OmniTokenizer as RustTokenizer;
use decon_core::compare::args::CompareArgs;
use decon_core::detect::detection::contamination_detect as rust_contamination_detect;
use decon_core::detect::scoring::calculate_minimum_question_idf_threshold;
use decon_core::evals::args::EvalsArgs;
use decon_core::review::args::ReviewArgs;
use decon_core::server::run_server;
use decon_core::{execute_compare, execute_evals, execute_review};
use tokio::runtime::Runtime;

fn update_minimum_question_idf_threshold(config: &mut RustConfig) {
    config.minimum_question_idf_threshold =
        calculate_minimum_question_idf_threshold(config.contamination_score_threshold);
}

fn normalize_sample_every_m_tokens(config: &mut RustConfig) {
    if config.sample_every_m_tokens < 1 {
        eprintln!(
            "Warning: sample_every_m_tokens was {}, adjusting to 1 (minimum value)",
            config.sample_every_m_tokens
        );
        config.sample_every_m_tokens = 1;
    }

    if config.sample_every_m_tokens > 100 {
        eprintln!(
            "Warning: sample_every_m_tokens is set to {}, which is quite large. This may cause contamination to be missed. Consider using a smaller value.",
            config.sample_every_m_tokens
        );
    }
}

fn print_detect_header(config: &RustConfig) {
    let report_dir_str = config.report_output_dir.display().to_string();
    print!("Contamination report output directory: {}", report_dir_str);

    if report_dir_str.contains("/tmp/decon-") || report_dir_str.contains("\\decon-") {
        println!(" (set report directory with --report-output-dir)");
    } else {
        println!();
    }

    if config.purify {
        if let Some(ref cleaned_dir) = config.cleaned_output_dir {
            let cleaned_dir_str = cleaned_dir.display().to_string();
            print!("Cleaned output directory: {}", cleaned_dir_str);

            if cleaned_dir_str.contains("/tmp/decon-cleaned-")
                || cleaned_dir_str.contains("\\decon-cleaned-")
            {
                println!(" (set cleaned directory with --cleaned-output-dir)");
            } else {
                println!();
            }
        }
    }

    if config.verbose && !is_quiet_mode() {
        println!("Using Simple contamination detection...");
        println!("  N-gram size: {}", config.ngram_size);
        println!("  Sample every M tokens: {}", config.sample_every_m_tokens);
        println!(
            "  Question max consecutive misses: {}",
            config.question_max_consecutive_misses
        );
        println!(
            "  Contamination score threshold: {}",
            config.contamination_score_threshold
        );
        println!("  Tokenizer: {}", config.tokenizer_str);
        println!("  Worker threads: {}", config.worker_threads);

        println!("\nReference Preprocessing:");
        println!(
            "  Deduplication: {}",
            if config.eval_dedup { "enabled" } else { "disabled" }
        );
        if config.eval_min_token_length > 0 {
            println!(
                "  Minimum length: {} tokens",
                config.eval_min_token_length
            );
        }
        if config.eval_min_unique_word_count > 0 {
            println!(
                "  Minimum unique words: {}",
                config.eval_min_unique_word_count
            );
        }

        println!("\nInput and Output:");
        println!("  Training directory: {}", config.training_dir.display());
        println!("  Content key: {}", config.content_key);
        println!("  Evaluation directory: {}", config.evals_dir.display());
        println!("  Report output dir: {}", config.report_output_dir.display());
        if let Some(ref cleaned_dir) = config.cleaned_output_dir {
            println!("  Cleaned output dir: {}", cleaned_dir.display());
        }
        println!("  Purify: {}", config.purify);
        println!();
    }
}

fn run_detect_with_config(config: &RustConfig) -> Result<(), String> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(config.worker_threads)
        .build_global()
        .unwrap_or_else(|e| {
            eprintln!("Warning: Failed to set Rayon thread pool size: {}", e);
        });

    match config.mode.as_str() {
        "simple" => {
            print_detect_header(config);
            rust_contamination_detect(config).map_err(|e| e.to_string())
        }
        unknown_mode => Err(format!("Unsupported detection mode: {}", unknown_mode)),
    }
}

fn check_dir_for_data_files(dir: &Path) -> bool {
    fn check_dir_recursive(dir: &Path) -> bool {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        for ext in SUPPORTED_DATA_EXTENSIONS {
                            if name.ends_with(ext) {
                                return true;
                            }
                        }
                    }
                } else if path.is_dir() && check_dir_recursive(&path) {
                    return true;
                }
            }
        }
        false
    }

    check_dir_recursive(dir)
}

fn validate_server_config(config: &RustConfig) -> PyResult<()> {
    if !config.evals_dir.exists() {
        return Err(PyIOError::new_err(format!(
            "Evaluation directory does not exist: {}",
            config.evals_dir.display()
        )));
    }

    if !config.evals_dir.is_dir() {
        return Err(PyIOError::new_err(format!(
            "Evaluation path is not a directory: {}",
            config.evals_dir.display()
        )));
    }

    if !check_dir_for_data_files(&config.evals_dir) {
        return Err(PyIOError::new_err(format!(
            "Evaluation directory contains no data files: {}",
            config.evals_dir.display()
        )));
    }

    if let Some(parent) = config.report_output_dir.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                PyIOError::new_err(format!(
                    "Failed to create report output directory parent: {} (error: {})",
                    parent.display(),
                    e
                ))
            })?;
        }
    }

    if config.purify {
        if let Some(ref cleaned_dir) = config.cleaned_output_dir {
            if let Some(parent) = cleaned_dir.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        PyIOError::new_err(format!(
                            "Failed to create cleaned output directory parent: {} (error: {})",
                            parent.display(),
                            e
                        ))
                    })?;
                }
            }
        }
    }

    Ok(())
}

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
        mode = "simple",
        ngram_size = 5,
        tokenizer = "cl100k",
        hash_seed = 0,
        content_key = "text",
        cleaned_output_dir = None,
        exact_override = false,
        sample_every_m_tokens = 10,
        question_max_consecutive_misses = 11,
        ngram_bucket_lru_cache = 10000,
        punctuation_chars = None,
        worker_threads = None,
        window_size_increment = None,
        num_windows = None,
        window_step_size = None,
        short_answer_window_length = 50,
        min_long_answer_window = 100,
        short_answer_token_threshold = 3,
        answer_ngram_size = 3,
        min_passage_distance = 100,
        passage_max_consecutive_misses = 2,
        passage_ngram_size = 4,
        contamination_score_threshold = 0.8,
        purify = false,
        eval_dedup = true,
        eval_min_token_length = 20,
        eval_min_unique_word_count = 4,
        perfect_match_decay_start = 20,
        perfect_match_decay_end = 50,
        verbose = false,
        index_passages = true,
        index_answers = true,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        training_dir: &str,
        evals_dir: &str,
        report_output_dir: &str,
        mode: &str,
        ngram_size: usize,
        tokenizer: &str,
        hash_seed: usize,
        content_key: &str,
        cleaned_output_dir: Option<&str>,
        exact_override: bool,
        sample_every_m_tokens: usize,
        question_max_consecutive_misses: usize,
        ngram_bucket_lru_cache: usize,
        punctuation_chars: Option<&str>,
        worker_threads: Option<usize>,
        window_size_increment: Option<usize>,
        num_windows: Option<usize>,
        window_step_size: Option<usize>,
        short_answer_window_length: usize,
        min_long_answer_window: usize,
        short_answer_token_threshold: usize,
        answer_ngram_size: usize,
        min_passage_distance: usize,
        passage_max_consecutive_misses: usize,
        passage_ngram_size: usize,
        contamination_score_threshold: f32,
        purify: bool,
        eval_dedup: bool,
        eval_min_token_length: usize,
        eval_min_unique_word_count: usize,
        perfect_match_decay_start: Option<usize>,
        perfect_match_decay_end: Option<usize>,
        verbose: bool,
        index_passages: bool,
        index_answers: bool,
    ) -> PyResult<Self> {
        let mut config = create_default_config();

        config.training_dir = PathBuf::from(training_dir);
        config.evals_dir = PathBuf::from(evals_dir);
        config.report_output_dir = PathBuf::from(report_output_dir);
        config.mode = mode.to_string();
        config.ngram_size = ngram_size;
        config.tokenizer_str = tokenizer.to_string();
        config.hash_seed = hash_seed;
        config.content_key = content_key.to_string();
        config.cleaned_output_dir = cleaned_output_dir.map(PathBuf::from);
        config.exact_override = exact_override;
        config.sample_every_m_tokens = sample_every_m_tokens;
        config.question_max_consecutive_misses = question_max_consecutive_misses;
        config.ngram_bucket_lru_cache = ngram_bucket_lru_cache;
        if let Some(chars) = punctuation_chars {
            config.punctuation_chars = chars.to_string();
        }
        if let Some(worker_threads) = worker_threads {
            config.worker_threads = worker_threads;
        }
        config.window_size_increment = window_size_increment;
        config.num_windows = num_windows;
        config.window_step_size = window_step_size;
        config.short_answer_window_length = short_answer_window_length;
        config.min_long_answer_window = min_long_answer_window;
        config.short_answer_token_threshold = short_answer_token_threshold;
        config.answer_ngram_size = answer_ngram_size;
        config.min_passage_distance = min_passage_distance;
        config.passage_max_consecutive_misses = passage_max_consecutive_misses;
        config.passage_ngram_size = passage_ngram_size;
        config.contamination_score_threshold = contamination_score_threshold;
        config.purify = purify;
        config.eval_dedup = eval_dedup;
        config.eval_min_token_length = eval_min_token_length;
        config.eval_min_unique_word_count = eval_min_unique_word_count;
        config.perfect_match_decay_start = perfect_match_decay_start;
        config.perfect_match_decay_end = perfect_match_decay_end;
        config.verbose = verbose;
        config.index_passages = index_passages;
        config.index_answers = index_answers;

        update_minimum_question_idf_threshold(&mut config);

        Ok(PyConfig { inner: config })
    }

    #[getter]
    fn training_dir(&self) -> String {
        self.inner.training_dir.display().to_string()
    }

    #[setter]
    fn set_training_dir(&mut self, training_dir: &str) {
        self.inner.training_dir = PathBuf::from(training_dir);
    }

    #[getter]
    fn evals_dir(&self) -> String {
        self.inner.evals_dir.display().to_string()
    }

    #[setter]
    fn set_evals_dir(&mut self, evals_dir: &str) {
        self.inner.evals_dir = PathBuf::from(evals_dir);
    }

    #[getter]
    fn report_output_dir(&self) -> String {
        self.inner.report_output_dir.display().to_string()
    }

    #[setter]
    fn set_report_output_dir(&mut self, report_output_dir: &str) {
        self.inner.report_output_dir = PathBuf::from(report_output_dir);
    }

    #[getter]
    fn cleaned_output_dir(&self) -> Option<String> {
        self.inner
            .cleaned_output_dir
            .as_ref()
            .map(|p| p.display().to_string())
    }

    #[setter]
    fn set_cleaned_output_dir(&mut self, cleaned_output_dir: Option<String>) {
        self.inner.cleaned_output_dir = cleaned_output_dir.map(PathBuf::from);
    }

    #[getter]
    fn mode(&self) -> String {
        self.inner.mode.clone()
    }

    #[setter]
    fn set_mode(&mut self, mode: &str) {
        self.inner.mode = mode.to_string();
    }

    #[getter]
    fn ngram_size(&self) -> usize {
        self.inner.ngram_size
    }

    #[setter]
    fn set_ngram_size(&mut self, ngram_size: usize) {
        self.inner.ngram_size = ngram_size;
    }

    #[getter]
    fn tokenizer(&self) -> String {
        self.inner.tokenizer_str.clone()
    }

    #[setter]
    fn set_tokenizer(&mut self, tokenizer: &str) {
        self.inner.tokenizer_str = tokenizer.to_string();
    }

    #[getter]
    fn hash_seed(&self) -> usize {
        self.inner.hash_seed
    }

    #[setter]
    fn set_hash_seed(&mut self, hash_seed: usize) {
        self.inner.hash_seed = hash_seed;
    }

    #[getter]
    fn content_key(&self) -> String {
        self.inner.content_key.clone()
    }

    #[setter]
    fn set_content_key(&mut self, content_key: &str) {
        self.inner.content_key = content_key.to_string();
    }

    #[getter]
    fn exact_override(&self) -> bool {
        self.inner.exact_override
    }

    #[setter]
    fn set_exact_override(&mut self, exact_override: bool) {
        self.inner.exact_override = exact_override;
    }

    #[getter]
    fn sample_every_m_tokens(&self) -> usize {
        self.inner.sample_every_m_tokens
    }

    #[setter]
    fn set_sample_every_m_tokens(&mut self, sample_every_m_tokens: usize) {
        self.inner.sample_every_m_tokens = sample_every_m_tokens;
    }

    #[getter]
    fn question_max_consecutive_misses(&self) -> usize {
        self.inner.question_max_consecutive_misses
    }

    #[setter]
    fn set_question_max_consecutive_misses(&mut self, question_max_consecutive_misses: usize) {
        self.inner.question_max_consecutive_misses = question_max_consecutive_misses;
    }

    #[getter]
    fn ngram_bucket_lru_cache(&self) -> usize {
        self.inner.ngram_bucket_lru_cache
    }

    #[setter]
    fn set_ngram_bucket_lru_cache(&mut self, ngram_bucket_lru_cache: usize) {
        self.inner.ngram_bucket_lru_cache = ngram_bucket_lru_cache;
    }

    #[getter]
    fn punctuation_chars(&self) -> String {
        self.inner.punctuation_chars.clone()
    }

    #[setter]
    fn set_punctuation_chars(&mut self, punctuation_chars: Option<String>) {
        self.inner.punctuation_chars = punctuation_chars.unwrap_or_else(default_punctuation_chars);
    }

    #[getter]
    fn worker_threads(&self) -> usize {
        self.inner.worker_threads
    }

    #[setter]
    fn set_worker_threads(&mut self, worker_threads: usize) {
        self.inner.worker_threads = worker_threads;
    }

    #[getter]
    fn window_size_increment(&self) -> Option<usize> {
        self.inner.window_size_increment
    }

    #[setter]
    fn set_window_size_increment(&mut self, window_size_increment: Option<usize>) {
        self.inner.window_size_increment = window_size_increment;
    }

    #[getter]
    fn num_windows(&self) -> Option<usize> {
        self.inner.num_windows
    }

    #[setter]
    fn set_num_windows(&mut self, num_windows: Option<usize>) {
        self.inner.num_windows = num_windows;
    }

    #[getter]
    fn window_step_size(&self) -> Option<usize> {
        self.inner.window_step_size
    }

    #[setter]
    fn set_window_step_size(&mut self, window_step_size: Option<usize>) {
        self.inner.window_step_size = window_step_size;
    }

    #[getter]
    fn short_answer_window_length(&self) -> usize {
        self.inner.short_answer_window_length
    }

    #[setter]
    fn set_short_answer_window_length(&mut self, short_answer_window_length: usize) {
        self.inner.short_answer_window_length = short_answer_window_length;
    }

    #[getter]
    fn min_long_answer_window(&self) -> usize {
        self.inner.min_long_answer_window
    }

    #[setter]
    fn set_min_long_answer_window(&mut self, min_long_answer_window: usize) {
        self.inner.min_long_answer_window = min_long_answer_window;
    }

    #[getter]
    fn short_answer_token_threshold(&self) -> usize {
        self.inner.short_answer_token_threshold
    }

    #[setter]
    fn set_short_answer_token_threshold(&mut self, short_answer_token_threshold: usize) {
        self.inner.short_answer_token_threshold = short_answer_token_threshold;
    }

    #[getter]
    fn answer_ngram_size(&self) -> usize {
        self.inner.answer_ngram_size
    }

    #[setter]
    fn set_answer_ngram_size(&mut self, answer_ngram_size: usize) {
        self.inner.answer_ngram_size = answer_ngram_size;
    }

    #[getter]
    fn min_passage_distance(&self) -> usize {
        self.inner.min_passage_distance
    }

    #[setter]
    fn set_min_passage_distance(&mut self, min_passage_distance: usize) {
        self.inner.min_passage_distance = min_passage_distance;
    }

    #[getter]
    fn passage_max_consecutive_misses(&self) -> usize {
        self.inner.passage_max_consecutive_misses
    }

    #[setter]
    fn set_passage_max_consecutive_misses(&mut self, passage_max_consecutive_misses: usize) {
        self.inner.passage_max_consecutive_misses = passage_max_consecutive_misses;
    }

    #[getter]
    fn passage_ngram_size(&self) -> usize {
        self.inner.passage_ngram_size
    }

    #[setter]
    fn set_passage_ngram_size(&mut self, passage_ngram_size: usize) {
        self.inner.passage_ngram_size = passage_ngram_size;
    }

    #[getter]
    fn contamination_score_threshold(&self) -> f32 {
        self.inner.contamination_score_threshold
    }

    #[setter]
    fn set_contamination_score_threshold(&mut self, contamination_score_threshold: f32) {
        self.inner.contamination_score_threshold = contamination_score_threshold;
        update_minimum_question_idf_threshold(&mut self.inner);
    }

    #[getter]
    fn minimum_question_idf_threshold(&self) -> f32 {
        self.inner.minimum_question_idf_threshold
    }

    #[getter]
    fn purify(&self) -> bool {
        self.inner.purify
    }

    #[setter]
    fn set_purify(&mut self, purify: bool) {
        self.inner.purify = purify;
    }

    #[getter]
    fn eval_dedup(&self) -> bool {
        self.inner.eval_dedup
    }

    #[setter]
    fn set_eval_dedup(&mut self, eval_dedup: bool) {
        self.inner.eval_dedup = eval_dedup;
    }

    #[getter]
    fn eval_min_token_length(&self) -> usize {
        self.inner.eval_min_token_length
    }

    #[setter]
    fn set_eval_min_token_length(&mut self, eval_min_token_length: usize) {
        self.inner.eval_min_token_length = eval_min_token_length;
    }

    #[getter]
    fn eval_min_unique_word_count(&self) -> usize {
        self.inner.eval_min_unique_word_count
    }

    #[setter]
    fn set_eval_min_unique_word_count(&mut self, eval_min_unique_word_count: usize) {
        self.inner.eval_min_unique_word_count = eval_min_unique_word_count;
    }

    #[getter]
    fn perfect_match_decay_start(&self) -> Option<usize> {
        self.inner.perfect_match_decay_start
    }

    #[setter]
    fn set_perfect_match_decay_start(&mut self, perfect_match_decay_start: Option<usize>) {
        self.inner.perfect_match_decay_start = perfect_match_decay_start;
    }

    #[getter]
    fn perfect_match_decay_end(&self) -> Option<usize> {
        self.inner.perfect_match_decay_end
    }

    #[setter]
    fn set_perfect_match_decay_end(&mut self, perfect_match_decay_end: Option<usize>) {
        self.inner.perfect_match_decay_end = perfect_match_decay_end;
    }

    #[getter]
    fn verbose(&self) -> bool {
        self.inner.verbose
    }

    #[setter]
    fn set_verbose(&mut self, verbose: bool) {
        self.inner.verbose = verbose;
    }

    #[getter]
    fn index_passages(&self) -> bool {
        self.inner.index_passages
    }

    #[setter]
    fn set_index_passages(&mut self, index_passages: bool) {
        self.inner.index_passages = index_passages;
    }

    #[getter]
    fn index_answers(&self) -> bool {
        self.inner.index_answers
    }

    #[setter]
    fn set_index_answers(&mut self, index_answers: bool) {
        self.inner.index_answers = index_answers;
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

fn run_detection(py: Python<'_>, config: &PyConfig) -> PyResult<PathBuf> {
    let mut config = config.inner.clone();

    if config.training_dir.as_os_str().is_empty() {
        return Err(PyValueError::new_err(
            "training_dir is required. Provide it via Config or load it from a config file.",
        ));
    }

    update_minimum_question_idf_threshold(&mut config);
    normalize_sample_every_m_tokens(&mut config);

    if config.purify && config.cleaned_output_dir.is_none() {
        config.cleaned_output_dir = Some(generate_temp_dir("decon-cleaned"));
    }

    validate_config(&config)
        .map_err(|e| PyIOError::new_err(format!("Detection failed: {}", e)))?;

    let report_dir = config.report_output_dir.clone();

    py.detach(|| {
        run_detect_with_config(&config)
            .map_err(|e| PyIOError::new_err(format!("Detection failed: {}", e)))
    })?;

    Ok(report_dir)
}

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
    let report_dir = run_detection(py, config)?;
    Ok(report_dir.display().to_string())
}

/// Run contamination detection with the given configuration (no return value).
#[pyfunction]
fn contamination_detect(py: Python<'_>, config: &PyConfig) -> PyResult<()> {
    let _ = run_detection(py, config)?;
    Ok(())
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
    let mut config = create_default_config();
    update_minimum_question_idf_threshold(&mut config);
    PyConfig { inner: config }
}

/// Load a configuration from a YAML file.
#[pyfunction]
fn read_config(path: &str) -> PyResult<PyConfig> {
    let mut config = rust_read_config(&PathBuf::from(path))
        .map_err(|e| PyIOError::new_err(format!("Failed to read config: {}", e)))?;
    update_minimum_question_idf_threshold(&mut config);
    Ok(PyConfig { inner: config })
}

/// Run the evals command to list or download evaluation datasets.
#[pyfunction]
#[pyo3(signature = (*, dir = None, stats = false, download = false, eval_name = None, output_dir = None, config_path = None))]
fn evals(
    py: Python<'_>,
    dir: Option<&str>,
    stats: bool,
    download: bool,
    eval_name: Option<&str>,
    output_dir: Option<&str>,
    config_path: Option<&str>,
) -> PyResult<()> {
    let args = EvalsArgs {
        dir: dir.map(PathBuf::from),
        stats,
        download,
        eval: eval_name.map(|s| s.to_string()),
        output_dir: output_dir.map(PathBuf::from),
        config: config_path.map(PathBuf::from),
    };

    py.detach(|| {
        execute_evals(&args)
            .map_err(|e| PyIOError::new_err(format!("Evals failed: {}", e)))
    })?;

    Ok(())
}

/// Review contamination results in a report directory.
#[pyfunction]
#[pyo3(signature = (
    dir,
    *,
    stats = false,
    dump = false,
    top_eval_examples = None,
    dataset_counts = false,
    min_score = None,
    min_length = None,
    eval_name = None,
    sort_match_length_descending = false,
    sort_match_length_ascending = false,
    verbose = false,
))]
fn review(
    py: Python<'_>,
    dir: &str,
    stats: bool,
    dump: bool,
    top_eval_examples: Option<usize>,
    dataset_counts: bool,
    min_score: Option<f32>,
    min_length: Option<usize>,
    eval_name: Option<&str>,
    sort_match_length_descending: bool,
    sort_match_length_ascending: bool,
    verbose: bool,
) -> PyResult<()> {
    let args = ReviewArgs {
        dir: PathBuf::from(dir),
        stats,
        dump,
        top_eval_examples,
        dataset_counts,
        min_score,
        min_length,
        eval: eval_name.map(|s| s.to_string()),
        sort_match_length_descending,
        sort_match_length_ascending,
        verbose,
    };

    py.detach(|| {
        execute_review(&args)
            .map_err(|e| PyIOError::new_err(format!("Review failed: {}", e)))
    })?;

    Ok(())
}

/// Compare contamination results from two runs.
#[pyfunction]
#[pyo3(signature = (
    dir1,
    dir2,
    *,
    stats = false,
    common = false,
    only_in_first = false,
    only_in_second = false,
    min_score = None,
    eval_name = None,
    verbose = false,
))]
fn compare(
    py: Python<'_>,
    dir1: &str,
    dir2: &str,
    stats: bool,
    common: bool,
    only_in_first: bool,
    only_in_second: bool,
    min_score: Option<f32>,
    eval_name: Option<&str>,
    verbose: bool,
) -> PyResult<()> {
    let args = CompareArgs {
        dir1: PathBuf::from(dir1),
        dir2: PathBuf::from(dir2),
        stats,
        common,
        only_in_first,
        only_in_second,
        min_score,
        eval: eval_name.map(|s| s.to_string()),
        verbose,
    };

    py.detach(|| {
        execute_compare(&args)
            .map_err(|e| PyIOError::new_err(format!("Compare failed: {}", e)))
    })?;

    Ok(())
}

/// Run decon as an HTTP server for orchestrated pipelines.
#[pyfunction]
#[pyo3(signature = (config, *, port = 8080))]
fn server(py: Python<'_>, config: &PyConfig, port: u16) -> PyResult<()> {
    let mut config = config.inner.clone();

    update_minimum_question_idf_threshold(&mut config);
    normalize_sample_every_m_tokens(&mut config);

    if config.training_dir.as_os_str().is_empty() {
        config.training_dir = PathBuf::from(".");
    }

    validate_server_config(&config)?;

    py.detach(|| {
        let runtime = Runtime::new()
            .map_err(|e| PyIOError::new_err(format!("Failed to start server runtime: {}", e)))?;
        runtime
            .block_on(run_server(config, port))
            .map_err(|e| PyIOError::new_err(format!("Server failed: {}", e)))
    })?;

    Ok(())
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
    m.add_function(wrap_pyfunction!(contamination_detect, m)?)?;
    m.add_function(wrap_pyfunction!(clean_text, m)?)?;
    m.add_function(wrap_pyfunction!(default_config, m)?)?;
    m.add_function(wrap_pyfunction!(read_config, m)?)?;
    m.add_function(wrap_pyfunction!(evals, m)?)?;
    m.add_function(wrap_pyfunction!(review, m)?)?;
    m.add_function(wrap_pyfunction!(compare, m)?)?;
    m.add_function(wrap_pyfunction!(server, m)?)?;

    // Version info
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
