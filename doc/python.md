# Python Bindings

Python bindings for decon via [PyO3](https://pyo3.rs/).

## Installation

```bash
pip install decontaminate
```

> The PyPI package is `decontaminate`, but the import is `import decon`.

## Quick Example

```python
import decon

# Run contamination detection
config = decon.Config(
    training_dir="path/to/training",
    evals_dir="path/to/evals",
    report_output_dir="/tmp/decon-results",
)
report_dir = decon.detect(config)

# Tokenizer (same tokenizers used internally)
tok = decon.Tokenizer("cl100k")
tokens = tok.encode("hello world")  # [15339, 1917]

# Text normalization (same as internal preprocessing)
cleaned = decon.clean_text("Hello, World!")  # "hello world"
```

## API Reference

The Python API is a thin wrapper over the Rust implementation. All parameters and their defaults are defined in [`crates/decon-py/src/lib.rs`](../crates/decon-py/src/lib.rs).

Key sections in `lib.rs`:
- **`PyConfig`** (line ~230): All `Config` parameters with defaults in the `#[pyo3(signature = ...)]` block
- **`PyTokenizer`** (line ~740): Tokenizer with `encode()`, `decode()`, `is_space_token()`
- **Functions** (line ~830+): `detect()`, `clean_text()`, `review()`, `compare()`, `evals()`, `server()`

The Rust parameter names map directly to Python kwargs (e.g., `ngram_size` in Rust = `ngram_size=` in Python).
