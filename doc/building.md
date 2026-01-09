# Building Python Bindings

This document explains how to build and develop the decon Python bindings locally.

> **Source files**:
> - Rust bindings: [`crates/decon-py/src/lib.rs`](../crates/decon-py/src/lib.rs)
> - Python package: [`crates/decon-py/python/decon/`](../crates/decon-py/python/decon/)
> - Build config: [`crates/decon-py/pyproject.toml`](../crates/decon-py/pyproject.toml)

---

## Prerequisites

| Requirement | Version | Notes |
|-------------|---------|-------|
| **Rust** | 1.88+ | Install via [rustup](https://rustup.rs/) |
| **Python** | 3.12+ | Required for bindings |
| **maturin** | 1.4+ | PyO3 build tool |

### Install maturin

```bash
# With pip
pip install maturin

# With uv (recommended)
uv pip install maturin

# With pipx (isolated install)
pipx install maturin
```

---

## Development Build

Build and install bindings into a virtual environment for development.

### Option 1: Using uv (recommended)

```bash
# Create and activate virtual environment
cd crates/decon-py
uv venv --python 3.12
source .venv/bin/activate  # or `.venv\Scripts\activate` on Windows

# Build and install (debug mode, faster compile)
maturin develop

# Or with optimizations (slower compile, faster runtime)
maturin develop --release
```

### Option 2: Using pip

```bash
cd crates/decon-py
python -m venv .venv
source .venv/bin/activate

pip install maturin
maturin develop --release
```

### Verify Installation

```python
import decon
print(decon.__version__)  # 0.3.0
print(decon.Tokenizer("cl100k").encode("hello"))  # [15339]
```

---

## Building Wheels

Build distributable wheel files.

### Single Platform

```bash
cd crates/decon-py

# Build wheel for current platform
maturin build --release

# Output: target/wheels/decon-0.3.0-cp312-cp312-*.whl
```

### Install from Wheel

```bash
pip install target/wheels/decon-0.3.0-cp312-cp312-*.whl
```

---

## Cross-Platform Builds

Build wheels for multiple platforms using Docker (Linux) or native toolchains.

### Linux (manylinux)

```bash
# Build manylinux wheels using Docker
maturin build --release --manylinux auto

# Or specify a target
maturin build --release --target x86_64-unknown-linux-gnu
```

### macOS (Universal Binary)

```bash
# Build for Intel
maturin build --release --target x86_64-apple-darwin

# Build for Apple Silicon
maturin build --release --target aarch64-apple-darwin

# Build universal binary (requires both toolchains)
maturin build --release --target universal2-apple-darwin
```

### Windows

```bash
maturin build --release --target x86_64-pc-windows-msvc
```

---

## Running Tests

### Python Parity Tests

```bash
cd crates/decon-py

# Install test dependencies
pip install pytest

# Run tests
pytest tests/ -v

# Run specific test
pytest tests/test_parity.py::TestTokenizerEncode -v
```

### Rust Tests (core library)

```bash
# From repo root
cargo test --workspace
```

---

## Project Structure

```
crates/decon-py/
├── Cargo.toml              # Rust crate config
├── pyproject.toml          # Python package config (maturin)
├── src/
│   └── lib.rs              # PyO3 bindings (Rust)
├── python/
│   └── decon/
│       └── __init__.py     # Python re-exports
└── tests/
    └── test_parity.py      # Python tests mirroring Rust tests
```

### Key Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Declares `cdylib` crate type, depends on `decon-core` and `pyo3` |
| `pyproject.toml` | maturin config, Python package metadata, pytest config |
| `src/lib.rs` | PyO3 wrapper classes and `#[pymodule]` definition |
| `python/decon/__init__.py` | Re-exports from native `_decon` module |

---

## Build Configuration

### pyproject.toml

```toml
[build-system]
requires = ["maturin>=1.4,<2.0"]
build-backend = "maturin"

[project]
name = "decon"
version = "0.3.0"
requires-python = ">=3.12"

[tool.maturin]
python-source = "python"        # Location of Python package
module-name = "decon._decon"    # Native module import path
strip = true                    # Strip debug symbols from release
```

### Cargo.toml

```toml
[lib]
name = "_decon"                 # Must match module-name suffix
crate-type = ["cdylib"]         # Dynamic library for Python

[dependencies]
decon-core = { path = "../decon-core" }
pyo3 = { version = "0.27", features = ["extension-module"] }
```

---

## CI/CD

The GitHub Actions workflow (`.github/workflows/release.yml`) builds wheels for:

| Platform | Target |
|----------|--------|
| Linux x64 | `manylinux` (glibc compatible) |
| macOS Intel | `x86_64-apple-darwin` |
| macOS ARM | `aarch64-apple-darwin` |
| Windows x64 | `x86_64-pc-windows-msvc` |

Wheels are published to PyPI on release via [trusted publishing](https://docs.pypi.org/trusted-publishers/).

---

## Troubleshooting

### `maturin develop` fails with "can't find Rust"

Ensure Rust is installed and `cargo` is in your PATH:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Import error: `undefined symbol`

Rebuild with `maturin develop --release` to ensure ABI compatibility.

### Python version mismatch

maturin builds for the active Python interpreter. Ensure your venv uses Python 3.12+:

```bash
python --version  # Should be 3.12+
maturin develop
```

### PyO3 version incompatibility

If you see errors about unsupported Python versions, update PyO3 in the workspace `Cargo.toml`:

```toml
[workspace.dependencies]
pyo3 = { version = "0.27", features = ["extension-module"] }
```

---

## Performance Tips

1. **Use `--release` for benchmarks** — debug builds are 10-50x slower
2. **GIL is released** during `detect()` — multiple Python threads can call it
3. **Rayon parallelism** — detection uses all CPU cores automatically
4. **Warm-up tokenizers** — first call loads BPE data; subsequent calls are fast

```python
# Pre-warm tokenizer if timing is critical
tok = decon.Tokenizer("cl100k")
_ = tok.encode("warmup")

# Now tokenization is fast
tokens = tok.encode(large_text)
```
