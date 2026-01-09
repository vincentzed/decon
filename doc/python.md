# Python Bindings

This document shows how to use decon from Python. Each example includes the equivalent CLI command for reference.

> **Source code**: See [`crates/decon-py/src/lib.rs`](../crates/decon-py/src/lib.rs) for full API signatures.

## Installation

```bash
pip install decon
```

Or build from source:

```bash
cd crates/decon-py
maturin develop --release
```

---

## Basic Detection

Run contamination detection on a training dataset against an eval reference set.

**CLI equivalent:**
```bash
decon detect \
    --training-dir tests/fixtures/training \
    --evals-dir tests/fixtures/reference \
    --report-output-dir /tmp/decon-results
```

**Python:**
```python
import decon

config = decon.Config(
    training_dir="tests/fixtures/training",
    evals_dir="tests/fixtures/reference",
    report_output_dir="/tmp/decon-results",
)

report_dir = decon.detect(config)
print(f"Results written to: {report_dir}")
```

---

## Detection with Custom Parameters

Adjust sensitivity and other parameters.

**CLI equivalent:**
```bash
decon detect \
    --training-dir tests/fixtures/training \
    --evals-dir tests/fixtures/reference \
    --report-output-dir /tmp/decon-results \
    --contamination-score-threshold 0.9 \
    --ngram-size 7 \
    --tokenizer o200k \
    --content-key text \
    --verbose
```

**Python:**
```python
import decon

config = decon.Config(
    training_dir="tests/fixtures/training",
    evals_dir="tests/fixtures/reference",
    report_output_dir="/tmp/decon-results",
    contamination_score_threshold=0.9,  # Higher threshold = stricter detection
    ngram_size=7,                        # Larger n-grams = fewer false positives
    tokenizer="o200k",                   # GPT-4 tokenizer
    content_key="text",                  # JSON field containing document text
    verbose=True,                        # Print progress
)

report_dir = decon.detect(config)
```

---

## Creating Cleaned Datasets

Remove contaminated documents and output a cleaned copy.

**CLI equivalent:**
```bash
decon detect \
    --training-dir tests/fixtures/training \
    --evals-dir tests/fixtures/reference \
    --report-output-dir /tmp/decon-results \
    --purify
```

**Python:**
```python
import decon

config = decon.Config(
    training_dir="tests/fixtures/training",
    evals_dir="tests/fixtures/reference",
    report_output_dir="/tmp/decon-results",
    purify=True,  # Output cleaned dataset alongside reports
)

report_dir = decon.detect(config)
# Cleaned files written to report_output_dir
```

---

## Tokenizer Utilities

Use the same tokenizers as decon for preprocessing or analysis.

**Python only** (no CLI equivalent):
```python
import decon

# Available tokenizers: r50k, p50k, p50k_edit, cl100k, o200k, uniseg
tokenizer = decon.Tokenizer("cl100k")  # GPT-3.5/4 tokenizer

# Encode text to tokens
tokens = tokenizer.encode("The quick brown fox jumps over the lazy dog.")
print(f"Tokens: {tokens}")
# Tokens: [791, 4062, 14198, 39935, 35308, 927, 279, 16053, 5765, 13]

# Decode back to text
text = tokenizer.decode(tokens)
print(f"Text: {text}")
# Text: The quick brown fox jumps over the lazy dog.

# Check token count
print(f"Token count: {len(tokens)}")
# Token count: 10
```

### Comparing Tokenizers

```python
import decon

text = "Hello, world! How are you today?"

for name in ["r50k", "p50k", "cl100k", "o200k"]:
    tok = decon.Tokenizer(name)
    tokens = tok.encode(text)
    print(f"{name:8} -> {len(tokens):2} tokens: {tokens}")

# r50k     ->  9 tokens: [15496, 11, 995, 0, 1374, 389, 345, 1909, 30]
# p50k     ->  9 tokens: [15496, 11, 995, 0, 1374, 389, 345, 1909, 30]
# cl100k   ->  9 tokens: [9906, 11, 1917, 0, 2650, 527, 499, 3432, 30]
# o200k    ->  9 tokens: [13225, 11, 2375, 0, 3253, 553, 498, 4024, 30]
```

---

## Text Cleaning

Normalize text the same way decon does internally before matching.

**Python only** (no CLI equivalent):
```python
import decon

# clean_text: lowercase, replace punctuation with spaces, normalize whitespace
raw = "  Hello, World!   This is a TEST...  "
cleaned = decon.clean_text(raw)
print(f"'{raw}' -> '{cleaned}'")
# '  Hello, World!   This is a TEST...  ' -> 'hello world this is a test'

# Useful for preprocessing before comparison
doc1 = decon.clean_text("The answer is: 42!")
doc2 = decon.clean_text("the answer is 42")
print(f"Match: {doc1 == doc2}")
# Match: True
```

---

## Complete Example Script

A runnable script that demonstrates the full workflow:

```python
#!/usr/bin/env python3
"""
Example: Run decon contamination detection from Python.

Usage:
    python example_detect.py --training-dir /path/to/data --evals-dir /path/to/evals
"""
import argparse
import json
import os
from pathlib import Path

import decon


def main():
    parser = argparse.ArgumentParser(description="Run decon contamination detection")
    parser.add_argument("--training-dir", required=True, help="Directory with training JSONL files")
    parser.add_argument("--evals-dir", required=True, help="Directory with eval reference JSONL files")
    parser.add_argument("--output-dir", default="/tmp/decon-results", help="Output directory for reports")
    parser.add_argument("--threshold", type=float, default=0.8, help="Contamination score threshold")
    parser.add_argument("--purify", action="store_true", help="Create cleaned dataset")
    parser.add_argument("--verbose", action="store_true", help="Enable verbose output")
    args = parser.parse_args()

    print(f"decon version: {decon.__version__}")
    print(f"Training dir:  {args.training_dir}")
    print(f"Evals dir:     {args.evals_dir}")
    print(f"Output dir:    {args.output_dir}")
    print(f"Threshold:     {args.threshold}")
    print()

    # Configure detection
    config = decon.Config(
        training_dir=args.training_dir,
        evals_dir=args.evals_dir,
        report_output_dir=args.output_dir,
        contamination_score_threshold=args.threshold,
        purify=args.purify,
        verbose=args.verbose,
    )

    # Run detection (parallelized across all CPU cores)
    print("Running contamination detection...")
    report_dir = decon.detect(config)
    print(f"Done! Results written to: {report_dir}")

    # Read and summarize results
    results_file = Path(report_dir) / "contamination_results.jsonl"
    if results_file.exists():
        with open(results_file) as f:
            results = [json.loads(line) for line in f]
        print(f"\nFound {len(results)} contaminated matches")

        # Group by eval suite
        by_suite = {}
        for r in results:
            suite = r.get("eval_suite", "unknown")
            by_suite[suite] = by_suite.get(suite, 0) + 1

        if by_suite:
            print("\nContamination by eval suite:")
            for suite, count in sorted(by_suite.items(), key=lambda x: -x[1]):
                print(f"  {suite}: {count}")


if __name__ == "__main__":
    main()
```

### Running the Example

```bash
# Using test fixtures included in the repo
python example_detect.py \
    --training-dir tests/fixtures/training \
    --evals-dir tests/fixtures/reference \
    --verbose

# With custom data
python example_detect.py \
    --training-dir /path/to/my/training/data \
    --evals-dir /path/to/my/evals \
    --threshold 0.9 \
    --purify
```

---

## API Reference

### `decon.Config`

Configuration object for detection runs.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `training_dir` | `str` | _required_ | Directory containing training JSONL files |
| `evals_dir` | `str` | _required_ | Directory containing eval reference JSONL files |
| `report_output_dir` | `str` | _required_ | Directory for output reports |
| `ngram_size` | `int` | `5` | N-gram size for matching |
| `tokenizer` | `str` | `"cl100k"` | Tokenizer: `r50k`, `p50k`, `cl100k`, `o200k`, `uniseg` |
| `contamination_score_threshold` | `float` | `0.8` | Score threshold for contamination (0.0-1.0) |
| `content_key` | `str` | `"text"` | JSON field containing document text |
| `verbose` | `bool` | `False` | Print progress during detection |
| `purify` | `bool` | `False` | Output cleaned dataset with contamination removed |

### `decon.detect(config) -> str`

Run contamination detection. Returns the path to the report output directory.

### `decon.Tokenizer(name="cl100k")`

Create a tokenizer instance.

| Method | Description |
|--------|-------------|
| `encode(text) -> list[int]` | Encode text to token IDs |
| `decode(tokens) -> str` | Decode token IDs to text |
| `is_space_token(token) -> bool` | Check if token is whitespace |
| `name -> str` | Get tokenizer name |

### `decon.clean_text(text, punctuation_chars=None) -> str`

Normalize text: lowercase, replace punctuation with spaces, collapse whitespace.

### `decon.__version__`

Package version string (e.g., `"0.3.0"`).
