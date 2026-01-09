"""
Parity tests between Python bindings and Rust implementation.

These tests verify that the Python API produces identical results to the Rust API.
Test cases are derived from the Rust tests in:
- decon-core/src/common/tokenizer.rs
- decon-core/src/common/text.rs
- decon-core/src/common/detection_config.rs
"""

import pytest
import decon


# =============================================================================
# Tokenizer Tests (from decon-core/src/common/tokenizer.rs)
# =============================================================================

class TestTokenizerNew:
    """
    Mirrors Rust test: test_tokenizer_new
    Tests all tiktoken tokenizers and custom tokenizers.
    """

    def test_r50k(self):
        tok = decon.Tokenizer("r50k")
        assert tok.name == "r50k"

    def test_p50k(self):
        tok = decon.Tokenizer("p50k")
        assert tok.name == "p50k"

    def test_p50k_edit(self):
        tok = decon.Tokenizer("p50k_edit")
        assert tok.name == "p50k_edit"

    def test_cl100k(self):
        tok = decon.Tokenizer("cl100k")
        assert tok.name == "cl100k"

    def test_o200k(self):
        tok = decon.Tokenizer("o200k")
        assert tok.name == "o200k"

    def test_uniseg(self):
        """Custom tokenizer - uniseg doesn't use BPE."""
        tok = decon.Tokenizer("uniseg")
        assert tok.name == "uniseg"

    def test_unknown_fallback(self):
        """Unknown tokenizer falls back to character-level."""
        tok = decon.Tokenizer("unknown")
        assert tok.name == "unknown"


class TestTokenizerEncode:
    """
    Mirrors Rust test: test_encode
    Tests BPE tokenizers produce exact token sequences.
    """

    def test_p50k_hello_world(self):
        """Rust: assert_eq!(p50k.encode("hello world"), vec![31373, 995]);"""
        tok = decon.Tokenizer("p50k")
        assert tok.encode("hello world") == [31373, 995]

    def test_cl100k_hello_world(self):
        """Rust: assert_eq!(cl100k.encode("hello world"), vec![15339, 1917]);"""
        tok = decon.Tokenizer("cl100k")
        assert tok.encode("hello world") == [15339, 1917]

    def test_o200k_hello_world_length(self):
        """Rust: assert_eq!(o200k_tokens.len(), 2);"""
        tok = decon.Tokenizer("o200k")
        tokens = tok.encode("hello world")
        assert len(tokens) == 2

    def test_uniseg_hello_world(self):
        """Rust: assert_eq!(uniseg_tokens.len(), 3); // "hello", " ", "world" """
        tok = decon.Tokenizer("uniseg")
        tokens = tok.encode("hello world")
        assert len(tokens) == 3

    def test_unknown_abc(self):
        """Rust: assert_eq!(unknown.encode("abc"), vec![97, 98, 99]); // ASCII values"""
        tok = decon.Tokenizer("unknown")
        assert tok.encode("abc") == [97, 98, 99]


class TestTokenizerIsSpaceToken:
    """
    Mirrors Rust test: test_is_space_token
    Tests space token detection across different tokenizers.
    """

    def test_cl100k_space_token(self):
        """Rust: assert!(cl100k.is_space_token(space_token_cl));"""
        tok = decon.Tokenizer("cl100k")
        space_token = tok.encode(" ")[0]
        assert tok.is_space_token(space_token)
        assert not tok.is_space_token(100)  # arbitrary non-space

    def test_o200k_space_token(self):
        """Rust: assert!(o200k.is_space_token(space_token_o200k));"""
        tok = decon.Tokenizer("o200k")
        space_token = tok.encode(" ")[0]
        assert tok.is_space_token(space_token)

    def test_p50k_space_token(self):
        """Rust: assert!(p50k.is_space_token(space_token_p50k));"""
        tok = decon.Tokenizer("p50k")
        space_token = tok.encode(" ")[0]
        assert tok.is_space_token(space_token)
        assert not tok.is_space_token(100)

    def test_r50k_space_token(self):
        """Rust: assert!(r50k.is_space_token(space_token_r50k));"""
        tok = decon.Tokenizer("r50k")
        space_token = tok.encode(" ")[0]
        assert tok.is_space_token(space_token)

    def test_uniseg_always_false(self):
        """Rust: assert!(!uniseg.is_space_token(0)); assert!(!uniseg.is_space_token(12345));"""
        tok = decon.Tokenizer("uniseg")
        assert not tok.is_space_token(0)
        assert not tok.is_space_token(12345)

    def test_unknown_ascii_space(self):
        """Rust: assert!(unknown.is_space_token(32)); // ASCII space"""
        tok = decon.Tokenizer("unknown")
        assert tok.is_space_token(32)  # ASCII space
        assert not tok.is_space_token(65)  # 'A'


# =============================================================================
# Clean Text Tests (from decon-core/src/common/text.rs)
# =============================================================================

class TestCleanTextWithDefaultPunctuation:
    """
    Mirrors Rust test: test_clean_text_with_default_punctuation
    """

    def test_hello_world(self):
        """
        Rust:
            let input = "  Hello, world! This is a test.  ";
            let expected = "hello world this is a test";
        """
        result = decon.clean_text("  Hello, world! This is a test.  ")
        assert result == "hello world this is a test"


class TestCleanTextWithCustomPunctuation:
    """
    Mirrors Rust test: test_clean_text_with_custom_punctuation
    """

    def test_pipe_as_punctuation(self):
        """
        Rust:
            let input = "Remove|these|character!s";
            let expected = "remove these character!s";
            let punctuation = "|";
        """
        result = decon.clean_text("Remove|these|character!s", punctuation_chars="|")
        assert result == "remove these character!s"


class TestCleanTextMultipleWhitespace:
    """
    Mirrors Rust test: test_clean_text_multiple_whitespace
    """

    def test_whitespace_normalization(self):
        """
        Rust:
            let input = "lots   of   whitespace";
            let expected = "lots of whitespace";
        """
        result = decon.clean_text("lots   of   whitespace")
        assert result == "lots of whitespace"


class TestCleanTextUppercase:
    """
    Mirrors Rust test: test_clean_text_uppercase
    """

    def test_lowercase_conversion(self):
        """
        Rust:
            let input = "UPPERCASE";
            let expected = "uppercase";
        """
        result = decon.clean_text("UPPERCASE")
        assert result == "uppercase"


class TestCleanTextEmptyString:
    """
    Mirrors Rust test: test_clean_text_empty_string
    """

    def test_empty_input(self):
        """
        Rust:
            let input = "";
            let expected = "";
        """
        result = decon.clean_text("")
        assert result == ""


class TestCleanTextNoChanges:
    """
    Mirrors Rust test: test_clean_text_no_changes
    """

    def test_unchanged_input(self):
        """
        Rust:
            let input = "this should not change";
            let expected = "this should not change";
        """
        result = decon.clean_text("this should not change")
        assert result == "this should not change"


# =============================================================================
# Config Tests (from decon-core/src/common/detection_config.rs)
# =============================================================================

class TestConfigDefaults:
    """
    Tests that Config defaults match the Rust create_default_config() function.
    Values from detection_config.rs:
        mode: "simple"
        content_key: "text"
        tokenizer_str: "cl100k"
        ngram_size: 5
        sample_every_m_tokens: 10
        contamination_score_threshold: 0.8
        verbose: false
        purify: false
    """

    def test_mode(self):
        config = decon.Config(
            training_dir="/tmp/t", evals_dir="/tmp/e", report_output_dir="/tmp/r"
        )
        assert config.mode == "simple"

    def test_content_key(self):
        config = decon.Config(
            training_dir="/tmp/t", evals_dir="/tmp/e", report_output_dir="/tmp/r"
        )
        assert config.content_key == "text"

    def test_tokenizer(self):
        config = decon.Config(
            training_dir="/tmp/t", evals_dir="/tmp/e", report_output_dir="/tmp/r"
        )
        assert config.tokenizer == "cl100k"

    def test_ngram_size(self):
        config = decon.Config(
            training_dir="/tmp/t", evals_dir="/tmp/e", report_output_dir="/tmp/r"
        )
        assert config.ngram_size == 5

    def test_sample_every_m_tokens(self):
        config = decon.Config(
            training_dir="/tmp/t", evals_dir="/tmp/e", report_output_dir="/tmp/r"
        )
        assert config.sample_every_m_tokens == 10

    def test_contamination_score_threshold(self):
        config = decon.Config(
            training_dir="/tmp/t", evals_dir="/tmp/e", report_output_dir="/tmp/r"
        )
        assert config.contamination_score_threshold == pytest.approx(0.8, rel=1e-5)

    def test_verbose(self):
        config = decon.Config(
            training_dir="/tmp/t", evals_dir="/tmp/e", report_output_dir="/tmp/r"
        )
        assert config.verbose == False

    def test_purify(self):
        config = decon.Config(
            training_dir="/tmp/t", evals_dir="/tmp/e", report_output_dir="/tmp/r"
        )
        assert config.purify == False


class TestConfigCustomValues:
    """Tests setting custom config values."""

    def test_all_custom_values(self):
        config = decon.Config(
            training_dir="/custom/train",
            evals_dir="/custom/evals",
            report_output_dir="/custom/reports",
            ngram_size=7,
            tokenizer="p50k",
            contamination_score_threshold=0.9,
            content_key="content",
            verbose=True,
            purify=True,
        )

        assert config.training_dir == "/custom/train"
        assert config.evals_dir == "/custom/evals"
        assert config.report_output_dir == "/custom/reports"
        assert config.ngram_size == 7
        assert config.tokenizer == "p50k"
        assert config.contamination_score_threshold == pytest.approx(0.9, rel=1e-5)
        assert config.content_key == "content"
        assert config.verbose == True
        assert config.purify == True


# =============================================================================
# Tokenizer Decode Roundtrip Tests
# =============================================================================

class TestTokenizerDecodeRoundtrip:
    """Tests that encode -> decode produces original text."""

    def test_cl100k_roundtrip(self):
        tok = decon.Tokenizer("cl100k")
        original = "hello world"
        assert tok.decode(tok.encode(original)) == original

    def test_p50k_roundtrip(self):
        tok = decon.Tokenizer("p50k")
        original = "The quick brown fox"
        assert tok.decode(tok.encode(original)) == original

    def test_unicode_roundtrip(self):
        tok = decon.Tokenizer("cl100k")
        original = "Hello 世界"
        assert tok.decode(tok.encode(original)) == original

    def test_empty_roundtrip(self):
        tok = decon.Tokenizer("cl100k")
        assert tok.decode(tok.encode("")) == ""


# =============================================================================
# Module Attributes Tests
# =============================================================================

class TestModuleAttributes:
    """Tests for module-level attributes and exports."""

    def test_version(self):
        assert hasattr(decon, "__version__")
        assert decon.__version__ == "0.3.0"

    def test_exports(self):
        """Verify all expected exports are available."""
        assert hasattr(decon, "Config")
        assert hasattr(decon, "Tokenizer")
        assert hasattr(decon, "detect")
        assert hasattr(decon, "clean_text")
        assert hasattr(decon, "default_config")


# =============================================================================
# Edge Cases
# =============================================================================

class TestEdgeCases:
    """Additional edge case tests for robustness."""

    def test_tokenizer_long_text(self):
        """Test tokenizing longer text."""
        tok = decon.Tokenizer("cl100k")
        text = "This is a longer piece of text that should be tokenized correctly. " * 10
        tokens = tok.encode(text)
        decoded = tok.decode(tokens)
        assert decoded == text

    def test_tokenizer_special_chars(self):
        """Test tokenizing text with special characters."""
        tok = decon.Tokenizer("cl100k")
        text = "Special chars: @#$%^&*()_+-=[]{}|;':\",./<>?"
        tokens = tok.encode(text)
        assert len(tokens) > 0

    def test_clean_text_unicode_punctuation(self):
        """Test clean_text with unicode punctuation (em dash, curly quotes)."""
        # Default punctuation includes: \u{201c} \u{201d} \u{2018} \u{2019} \u{2014}
        result = decon.clean_text("Hello\u2014World")  # em dash
        assert "hello" in result.lower()
        assert "world" in result.lower()

    def test_config_paths_preserved(self):
        """Test that paths are preserved exactly."""
        config = decon.Config(
            training_dir="/path/with spaces/train",
            evals_dir="/path/with spaces/evals",
            report_output_dir="/path/with spaces/reports",
        )
        assert config.training_dir == "/path/with spaces/train"
        assert config.evals_dir == "/path/with spaces/evals"
        assert config.report_output_dir == "/path/with spaces/reports"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
