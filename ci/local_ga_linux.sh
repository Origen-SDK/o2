#!/usr/bin/env bash
# Reproduce the Linux half of .github/workflows/regression_test.yml locally for
# one Python version. Mirrors the GA job order, including the per-project
# working directories that the workflow relies on.
#
#   ci/local_ga_linux.sh 3.11.11
#
# Requires pyenv with the requested version installed, a Rust toolchain, and UV
# at the version the CLI enforces. Set UV_BIN to prepend a specific UV install:
#
#   UV_BIN=/opt/uv-0.12.5 ci/local_ga_linux.sh 3.11.11
#
# Not used by CI. This exists so the matrix can be validated before pushing.
set -euo pipefail

MINIMUM_UV_VERSION="0.12.5"

PY_VERSION="${1:?usage: local_ga_linux.sh <pyenv-version>}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UV_BIN="${UV_BIN:-}"
LOG_DIR="${LOG_DIR:-$ROOT/.local-ga/$PY_VERSION}"

mkdir -p "$LOG_DIR"

export PYENV_VERSION="$PY_VERSION"
export PATH="${UV_BIN:+$UV_BIN:}$ROOT/rust/origen/target/debug:$(pyenv root)/versions/$PY_VERSION/bin:$HOME/.cargo/bin:$PATH"
export PYO3_PYTHON="$(pyenv root)/versions/$PY_VERSION/bin/python"
export RUST_BACKTRACE=full

step() { printf '\n=== %s\n' "$*"; }

# An older UV silently lacks flags used below (notably 'uv build --clear'), so
# fail with the same minimum the CLI enforces rather than midway through.
uv_version="$(uv --version 2>/dev/null | awk '{print $2}' || true)"
if [ -z "$uv_version" ]; then
    echo "UV was not found on PATH. Install >= $MINIMUM_UV_VERSION or set UV_BIN." >&2
    exit 1
fi
if [ "$(printf '%s\n%s\n' "$MINIMUM_UV_VERSION" "$uv_version" | sort -V | head -1)" != "$MINIMUM_UV_VERSION" ]; then
    echo "UV $uv_version is too old; >= $MINIMUM_UV_VERSION is required. Set UV_BIN to a newer install." >&2
    exit 1
fi

step "Python and tool versions"
python -V
uv --version
cargo --version

# A previous version's virtualenvs are not reusable across ABIs.
step "Clear stale virtualenvs"
find "$ROOT/test_apps" "$ROOT/python" -maxdepth 3 -name .venv -type d -prune -exec rm -rf {} + 2>/dev/null || true

step "Build Origen CLI"
cargo build --manifest-path "$ROOT/rust/origen/cli/Cargo.toml" --locked --bins

step "Build PyAPI"
origen develop_origen build

step "Build PyAPI - Metal"
origen develop_origen build --metal

if [[ "$PY_VERSION" == 3.11* ]]; then
    step "Run Poetry Migration Transaction Tests"
    cargo test --manifest-path "$ROOT/rust/origen/Cargo.toml" --locked -p cli migration

    step "Migrate Historical Poetry Application"
    python "$ROOT/ci/test_poetry_migration.py"
fi

step "Setup App Env"
cd "$ROOT/test_apps/python_app"
UV_PYTHON="$PYO3_PYTHON" origen env setup
origen -v

step "Stage CLI into the package"
cp "$ROOT/rust/origen/target/debug/origen" "$ROOT/python/origen/origen/__bin__/bin"

step "Run Python-App Unit Tests"
cd "$ROOT/test_apps/python_app"
PYTHONPATH="$ROOT/rust/pyapi/target:$ROOT/python/origen_metal" \
  origen exec pytest -vv 2>&1 | tee "$LOG_DIR/app.log" | tail -3

step "Run Python-App Diff Tests"
origen examples

step "Setup No-App Env"
cd "$ROOT/test_apps/python_no_app"
UV_PYTHON="$PYO3_PYTHON" uv sync --all-groups --no-editable

step "Copy Origen Library"
cp "$ROOT/rust/pyapi/target/debug/lib_origen.so" "$ROOT/python/origen/_origen.so"

step "Run Python-No-App Unit Tests"
cd "$ROOT/test_apps/python_no_app"
PYTHONPATH="$ROOT/rust/pyapi/target:$ROOT/python/origen_metal" \
  uv run --no-editable pytest -vv 2>&1 | tee "$LOG_DIR/no_app.log" | tail -3

step "Setup Python Env - Metal"
cd "$ROOT/python/origen_metal"
UV_PYTHON="$PYO3_PYTHON" uv sync --all-groups --no-editable

step "Run Python Unit Tests - Metal"
PYTHONPATH="$ROOT/rust/pyapi/target:$ROOT/python/origen_metal" \
  uv run --no-editable pytest -vv 2>&1 | tee "$LOG_DIR/metal.log" | tail -3

step "Build Origen Wheel"
cd "$ROOT/python/origen"
rm -rf "$ROOT/python/origen_metal/tmp"
uv build --wheel --clear --python "$PY_VERSION"
ls dist

printf '\n=== PASS %s\n' "$PY_VERSION"
