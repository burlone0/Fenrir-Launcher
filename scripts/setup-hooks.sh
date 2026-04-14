#!/bin/sh
# Install Fenrir git hooks into .git/hooks/.
# Run once after cloning: sh scripts/setup-hooks.sh

set -e

HOOKS_DIR=".git/hooks"
SCRIPT_DIR="$(dirname "$0")"

if [ ! -d "$HOOKS_DIR" ]; then
    echo "error: run this script from the repository root" >&2
    exit 1
fi

cat > "$HOOKS_DIR/pre-commit" <<'EOF'
#!/bin/sh
set -e

echo "--- pre-commit: fmt ---"
cargo fmt --all --check

echo "--- pre-commit: clippy ---"
cargo clippy --all-targets -- -D warnings

echo "--- pre-commit: test ---"
cargo test --all --quiet

echo "--- pre-commit: ok ---"
EOF

chmod +x "$HOOKS_DIR/pre-commit"
echo "hooks installed."
