#!/usr/bin/env bash
# Install git hooks for development
set -euo pipefail
REPO_ROOT="$(git rev-parse --show-toplevel)"
ln -sf "../../scripts/pre-commit" "${REPO_ROOT}/.git/hooks/pre-commit"
chmod +x "${REPO_ROOT}/.git/hooks/pre-commit"
echo "Pre-commit hook installed."
