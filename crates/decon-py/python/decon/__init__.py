"""
Decon - Fast contamination detection for ML training data.

This package provides Python bindings to the Rust decon library,
enabling high-performance detection of evaluation data leakage
in training datasets.

Example usage:
    import decon

    config = decon.Config(
        training_dir="/path/to/training",
        evals_dir="/path/to/evals",
        report_output_dir="/path/to/reports",
    )

    # Run detection (results written to report_output_dir)
    report_dir = decon.detect(config)

    # Tokenizer utilities
    tokenizer = decon.Tokenizer("cl100k")
    tokens = tokenizer.encode("hello world")
    text = tokenizer.decode(tokens)

    # Text cleaning
    cleaned = decon.clean_text("Hello,  World!")
"""

from decon._decon import (
    Config,
    Tokenizer,
    detect,
    contamination_detect,
    clean_text,
    default_config,
    read_config,
    evals,
    review,
    compare,
    server,
    __version__,
)

__all__ = [
    "Config",
    "Tokenizer",
    "detect",
    "contamination_detect",
    "clean_text",
    "default_config",
    "read_config",
    "evals",
    "review",
    "compare",
    "server",
    "__version__",
]
