#!/usr/bin/env zsh
# compare_outputs.sh — Convert each fixture with Rust, Go, hast (JS), and turndown and diff.
#
# Usage (from repo root):
#   ./benches/compare_outputs.sh

set -euo pipefail
cd "$(dirname "$0")/.."

FIXTURES_DIR="benches/fixtures"
FIXTURES=(article table lists code large)
JS_DIR="benches/compare/js"
GO_BIN="/tmp/html2markdown-go"

# Build the Rust converter
cargo build --bin convert --release -q
RUST_BIN="./target/release/convert"

# Build the Go CLI if missing
if [[ ! -x "$GO_BIN" ]]; then
  echo "==> Building Go binary…"
  (cd "../refs/html-to-markdown" && go build -o "$GO_BIN" ./cli/html2markdown/)
fi

TMPDIR_LOCAL=$(mktemp -d)
trap "rm -rf $TMPDIR_LOCAL" EXIT

any_diffs=0

for fixture in "${FIXTURES[@]}"; do
  input="$PWD/$FIXTURES_DIR/${fixture}.html"

  out_rust="$TMPDIR_LOCAL/${fixture}.rust.md"
  out_go="$TMPDIR_LOCAL/${fixture}.go.md"
  out_hast="$TMPDIR_LOCAL/${fixture}.hast.md"
  out_turndown="$TMPDIR_LOCAL/${fixture}.turndown.md"

  "$RUST_BIN" < "$input" > "$out_rust"
  "$GO_BIN"   < "$input" > "$out_go"
  (cd "$JS_DIR" && node convert.mjs hast     "$input") > "$out_hast"
  turndown_ok=0
  (cd "$JS_DIR" && node convert.mjs turndown "$input") > "$out_turndown" 2>/tmp/turndown_err && turndown_ok=1 || true

  echo "════════════════════════════════════════"
  echo "  fixture: $fixture"
  echo "════════════════════════════════════════"

  for pair in "go:$out_go" "hast:$out_hast" "turndown:$out_turndown"; do
    label="${pair%%:*}"
    other="${pair##*:}"
    if [[ "$label" == "turndown" && $turndown_ok -eq 0 ]]; then
      echo "  rust vs turndown  ✗ error: $(cat /tmp/turndown_err)"
      any_diffs=1
      continue
    fi
    diff_out=$(diff --unified=3 "$out_rust" "$other" || true)
    if [[ -z "$diff_out" ]]; then
      echo "  rust vs $label  ✓ identical"
    else
      echo "  rust vs $label  ✗ differs:"
      echo "$diff_out" | head -80
      any_diffs=1
    fi
  done

  echo ""
done

if [[ $any_diffs -eq 0 ]]; then
  echo "All outputs identical across all implementations."
else
  echo "Some differences found (see above)."
fi
