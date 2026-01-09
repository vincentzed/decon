# decontaminate

Fast contamination detection for ML training data. Python bindings for [decon](https://github.com/vincentzed/decon).

## Installation

```bash
pip install decontaminate
```

## Usage

```python
import decon

config = decon.Config(
    training_dir="/path/to/training/data",
    evals_dir="/path/to/eval/references",
    report_output_dir="/path/to/output",
)
report_dir = decon.detect(config)
```

## API

The Python API is a thin PyO3 wrapper over the Rust implementation. See [`src/lib.rs`](https://github.com/vincentzed/decon/blob/main/crates/decon-py/src/lib.rs) for all `Config` parameters and available functions:

- `detect()`, `review()`, `compare()`, `evals()`, `server()`
- `Tokenizer` (encode/decode with cl100k, o200k, etc.)
- `clean_text()` (text normalization)

## Documentation

Full documentation: https://github.com/vincentzed/decon
