#!/bin/bash
#
# Release script for decontaminate
# Based on: https://github.com/allenai/python-package-template
#
# Usage:
#   1. Update version in crates/decon-py/pyproject.toml
#   2. Run: ./scripts/release.sh
#

set -e

# Extract version from pyproject.toml
VERSION=$(grep '^version = ' crates/decon-py/pyproject.toml | head -1 | cut -d'"' -f2)
TAG="v$VERSION"

echo "╔════════════════════════════════════════╗"
echo "║     decontaminate Release Script       ║"
echo "╚════════════════════════════════════════╝"
echo ""
echo "  Version: $VERSION"
echo "  Tag:     $TAG"
echo ""

# Check if tag already exists
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "❌ Error: Tag $TAG already exists!"
    echo ""
    echo "To delete and recreate:"
    echo "  git tag -d $TAG"
    echo "  git push origin :refs/tags/$TAG"
    exit 1
fi

# Check for uncommitted changes
if [[ -n $(git status --porcelain) ]]; then
    echo "⚠️  You have uncommitted changes:"
    git status --short
    echo ""
fi

read -p "Create release $TAG? [Y/n] " prompt

if [[ $prompt == "y" || $prompt == "Y" || $prompt == "yes" || $prompt == "Yes" || $prompt == "" ]]; then
    # Commit any staged changes
    if [[ -n $(git status --porcelain) ]]; then
        git add -A
        git commit -m "Bump version to $VERSION for release" || true
    fi
    
    # Push to main
    echo "📤 Pushing to main..."
    git push origin main
    
    # Create and push tag
    echo "🏷️  Creating tag $TAG..."
    git tag -a "$TAG" -m "Release $TAG"
    
    echo "📤 Pushing tag..."
    git push origin "$TAG"
    
    echo ""
    echo "✅ Release $TAG triggered!"
    echo ""
    echo "📦 Watch the build: https://github.com/vincentzed/decon/actions"
    echo "🐍 PyPI package:    https://pypi.org/project/decontaminate/"
else
    echo "❌ Cancelled"
    exit 1
fi

