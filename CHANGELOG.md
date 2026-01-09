# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [v0.3.0.post1](https://github.com/vincentzed/decon/releases/tag/v0.3.0.post1) - 2026-01-09

### Added

### Changed

### Fixed

### Removed

## [v0.3.0](https://github.com/vincentzed/decon/releases/tag/v0.3.0) - 2025-01-09

### Added

- 🐍 Python bindings via PyO3 (`pip install decontaminate`)
- `decon.Config` for configuring detection runs
- `decon.detect()` function for contamination detection
- `decon.Tokenizer` class with encode/decode methods
- `decon.clean_text()` for text normalization
- Support for tokenizers: r50k, p50k, cl100k, o200k, uniseg
- Comprehensive documentation in `doc/python.md` and `doc/building.md`
- GitHub Actions release workflow with multi-platform wheel builds
- Release script (`scripts/release.sh`) based on Allen AI template

### Changed

- Restructured as Rust workspace with `decon-core`, `decon-cli`, `decon-py` crates
- Package renamed to `decontaminate` on PyPI (import as `import decon`)

