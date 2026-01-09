# Building & Releasing

This document covers building, testing, and releasing the `decontaminate` Python package.

> **Package name**: `decontaminate` on PyPI, import as `import decon`
>
> **Source files**:
> - Rust bindings: [`crates/decon-py/src/lib.rs`](../crates/decon-py/src/lib.rs)
> - Python package: [`crates/decon-py/python/decon/`](../crates/decon-py/python/decon/)
> - Build config: [`crates/decon-py/pyproject.toml`](../crates/decon-py/pyproject.toml)

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Development Build](#development-build)
3. [Building Wheels](#building-wheels)
4. [Running Tests](#running-tests)
5. [Release Process](#release-process)
6. [Project Structure](#project-structure)
7. [Troubleshooting](#troubleshooting)

---

## Prerequisites

| Requirement | Version | Notes |
|-------------|---------|-------|
| **Rust** | 1.88+ | Install via [rustup](https://rustup.rs/) |
| **Python** | 3.12+ | Required for bindings |
| **maturin** | 1.4+ | PyO3 build tool |

### Install maturin

```bash
# With uv (recommended)
uv pip install maturin

# With pip
pip install maturin

# With pipx (isolated install)
pipx install maturin
```

---

## Development Build

Build and install bindings into a virtual environment for development.

### Quick Start

```bash
cd crates/decon-py
uv venv --python 3.12
source .venv/bin/activate  # or `.venv\Scripts\activate` on Windows

# Build and install (debug mode, faster compile)
maturin develop

# Or with optimizations (slower compile, faster runtime)
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

### Single Platform

```bash
cd crates/decon-py

# Build wheel for current platform
maturin build --release

# Output: target/wheels/decontaminate-0.3.0-cp312-cp312-*.whl
```

### Cross-Platform Builds

```bash
# Linux (manylinux)
maturin build --release --manylinux auto

# macOS Intel
maturin build --release --target x86_64-apple-darwin

# macOS Apple Silicon
maturin build --release --target aarch64-apple-darwin

# Windows
maturin build --release --target x86_64-pc-windows-msvc
```

### Install from Wheel

```bash
pip install target/wheels/decontaminate-0.3.0-cp312-cp312-*.whl
```

---

## Running Tests

### Python Tests

```bash
cd crates/decon-py
pip install pytest
pytest tests/ -v
```

### Rust Tests

```bash
# From repo root
cargo test --workspace
```

---

## Release Process

Based on [Allen AI's release process](https://github.com/allenai/python-package-template/blob/main/RELEASE_PROCESS.md).

### One-Time Setup: PyPI Trusted Publishing

Configure PyPI to trust GitHub Actions (no API tokens needed):

1. Go to https://pypi.org/manage/project/decontaminate/settings/publishing/
2. Add a new publisher:
   | Field | Value |
   |-------|-------|
   | Owner | `vincentzed` |
   | Repository | `decon` |
   | Workflow name | `release.yml` |
   | Environment | `pypi` |

### Release Steps

#### 1. Update the version

Edit version in `crates/decon-py/pyproject.toml`:

```toml
[project]
version = "X.Y.Z"  # e.g., "0.4.0"
```

Optionally sync Cargo.toml versions:
```bash
# crates/decon-py/Cargo.toml
# crates/decon-core/Cargo.toml  
# crates/decon-cli/Cargo.toml
```

#### 2. Commit and push

```bash
git add -A
git commit -m "Bump version to X.Y.Z for release"
git push origin main
```

#### 3. Create and push version tag

```bash
# Create annotated tag
git tag -a vX.Y.Z -m "Release vX.Y.Z"

# Push tag to trigger release workflow
git push origin vX.Y.Z
```

#### 4. Monitor the release

GitHub Actions will automatically:
1. ✅ Build wheels for Linux, macOS (Intel + ARM), Windows
2. ✅ Build source distribution
3. ✅ Publish to PyPI via trusted publishing

Watch progress at: https://github.com/vincentzed/decon/actions

### Quick Release Script

```bash
#!/bin/bash
# scripts/release.sh

set -e

VERSION=$(grep 'version = ' crates/decon-py/pyproject.toml | head -1 | cut -d'"' -f2)
TAG="v$VERSION"

read -p "Creating new release for $TAG. Continue? [Y/n] " prompt

if [[ $prompt == "y" || $prompt == "Y" || $prompt == "yes" || $prompt == "Yes" || $prompt == "" ]]; then
    git add -A
    git commit -m "Bump version to $TAG for release" || true
    git push origin main
    echo "Creating new git tag $TAG"
    git tag -a "$TAG" -m "Release $TAG"
    git push origin "$TAG"
    echo "✅ Release triggered! Watch: https://github.com/vincentzed/decon/actions"
else
    echo "Cancelled"
    exit 1
fi
```

### Version Tag Format

Tags **must** start with `v` to trigger the release workflow:

| Format | Example | Valid? |
|--------|---------|--------|
| `vX.Y.Z` | `v0.3.0`, `v1.0.0` | ✅ |
| `vX.Y.Z-beta` | `v0.4.0-beta` | ✅ |
| `X.Y.Z` | `0.3.0` | ❌ |

### Versioning Guidelines

Follow [Semantic Versioning](https://semver.org/):

| Change Type | Bump | Example |
|-------------|------|---------|
| Bug fix | PATCH | `0.3.0` → `0.3.1` |
| New feature (backwards compatible) | MINOR | `0.3.1` → `0.4.0` |
| Breaking API change | MAJOR | `0.4.0` → `1.0.0` |

### Fixing a Failed Release

If GitHub Actions fails after tagging:

```bash
# Delete tag locally and remotely
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z

# Fix the issue, then re-tag
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

### Manual Release (fallback)

If CI fails, release manually:

```bash
cd crates/decon-py
source ../../.venv/bin/activate

# Build
maturin build --release

# Upload (requires PyPI API token)
pip install twine
twine upload target/wheels/*.whl
```

---

## Project Structure

```
crates/decon-py/
├── Cargo.toml              # Rust crate config
├── pyproject.toml          # Python package config (maturin)
├── LICENSE                 # Apache-2.0
├── src/
│   └── lib.rs              # PyO3 bindings (Rust)
├── python/
│   └── decon/
│       └── __init__.py     # Python re-exports
└── tests/
    └── test_parity.py      # Python tests
```

### Key Configuration Files

**pyproject.toml**:
```toml
[build-system]
requires = ["maturin>=1.4,<2.0"]
build-backend = "maturin"

[project]
name = "decontaminate"
version = "0.3.0"
requires-python = ">=3.12"

[tool.maturin]
python-source = "python"
module-name = "decon._decon"
strip = true
```

**Cargo.toml**:
```toml
[lib]
name = "_decon"
crate-type = ["cdylib"]

[dependencies]
decon-core = { path = "../decon-core" }
pyo3 = { version = "0.27", features = ["extension-module"] }
```

---

## CI/CD

The GitHub Actions workflow (`.github/workflows/release.yml`) builds for:

| Platform | Target | Runner |
|----------|--------|--------|
| Linux x64 | `manylinux` | `ubuntu-latest` |
| macOS Intel | `x86_64-apple-darwin` | `macos-13` |
| macOS ARM | `aarch64-apple-darwin` | `macos-14` |
| Windows x64 | `x86_64-pc-windows-msvc` | `windows-latest` |

---

## Troubleshooting

### `maturin develop` fails with "can't find Rust"

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Import error: `undefined symbol`

Rebuild with `maturin develop --release` to ensure ABI compatibility.

### Python version mismatch

maturin builds for the active Python interpreter:

```bash
python --version  # Should be 3.12+
maturin develop
```

### "Version already exists" on PyPI

PyPI doesn't allow re-uploading. Bump the version number and re-release.

### GitHub Actions "trusted publishing" fails

Ensure PyPI has the correct publisher configuration (see One-Time Setup above).

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
