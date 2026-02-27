#!/usr/bin/env zsh
# compare.sh — Run benchmarks for all implementations and print a throughput table.
#
# Usage (from repo root):
#   ./benches/compare.sh
#
# Requirements:
#   - Rust toolchain (cargo)
#   - Go toolchain (go)
#   - Node.js >= 18 (node) + deps installed (npm install in benches/compare/js/)

set -euo pipefail
cd "$(dirname "$0")/.."

COMPARE_DIR="benches/compare"
TMPDIR_BENCH=$(mktemp -d)
trap "rm -rf $TMPDIR_BENCH" EXIT

# --------------------------------------------------------------------------- #
# 1. Rust — cargo bench (Criterion)                                            #
# --------------------------------------------------------------------------- #

echo "==> Running Rust benchmarks…"
cargo bench --bench conversion -- "full_pipeline" > "$TMPDIR_BENCH/rust.txt" 2>&1
cargo bench --bench conversion -- "html2md"      >> "$TMPDIR_BENCH/rust.txt" 2>&1

# --------------------------------------------------------------------------- #
# 2. Go — go test -bench                                                       #
# --------------------------------------------------------------------------- #

echo "==> Running Go benchmarks…"
(cd "$COMPARE_DIR/go" && go test -bench=BenchmarkConvert -benchtime=3s -benchmem) > "$TMPDIR_BENCH/go.txt" 2>&1

# --------------------------------------------------------------------------- #
# 3. JS — node bench.mjs (hast pipeline + turndown)                           #
# --------------------------------------------------------------------------- #

echo "==> Running JS benchmarks…"
(cd "$COMPARE_DIR/js" && node bench.mjs) > "$TMPDIR_BENCH/js.txt" 2>&1

# --------------------------------------------------------------------------- #
# 4. Parse + print table (Python)                                              #
# --------------------------------------------------------------------------- #

python3 - "$TMPDIR_BENCH/rust.txt" "$TMPDIR_BENCH/go.txt" "$TMPDIR_BENCH/js.txt" <<'PYEOF'
import sys, re

FIXTURES = ['article', 'table', 'lists', 'code', 'large']

rust_file, go_file, js_file = sys.argv[1], sys.argv[2], sys.argv[3]
rust_raw = open(rust_file).read()
go_raw   = open(go_file).read()
js_raw   = open(js_file).read()

# --- Rust: Criterion thrpt output ---
# benchmark name on one line, thrpt: [low mid high MiB/s] two lines later
rust = {}
html2md = {}
fixture = None
group = None
for line in rust_raw.splitlines():
    m = re.match(r'^(full_pipeline|html2md)/\w+/(\w+)', line)
    if m:
        group = m.group(1)
        fixture = m.group(2)
        continue
    if fixture and 'thrpt:' in line and 'MiB/s' in line:
        nums = re.findall(r'([\d.]+)\s+MiB/s', line)
        if len(nums) >= 2:
            target = rust if group == 'full_pipeline' else html2md
            target[fixture] = float(nums[1])  # median
        fixture = None

# --- Go: go test -bench output ---
# BenchmarkConvert/article-16   N   12345 ns/op   35.08 MB/s   ...
go = {}
for line in go_raw.splitlines():
    m = re.match(r'BenchmarkConvert/(\w+)-\d+\s+\d+\s+\d+\s+ns/op\s+([\d.]+)\s+MB/s', line)
    if m:
        # go reports decimal MB/s; convert to MiB/s
        go[m.group(1)] = float(m.group(2)) * 1e6 / (1024 * 1024)

# --- JS: node bench.mjs output ---
# hast/article              4.64 MiB/s  ...
# turndown/article         12.34 MiB/s  ...
hast = {}
turndown = {}
for line in js_raw.splitlines():
    m = re.match(r'hast/(\w+)\s+([\d.]+)\s+MiB/s', line)
    if m:
        hast[m.group(1)] = float(m.group(2))
    m = re.match(r'turndown/(\w+)\s+([\d.]+)\s+MiB/s', line)
    if m:
        turndown[m.group(1)] = float(m.group(2))
    m = re.match(r'turndown/(\w+)\s+ERROR', line)
    if m:
        turndown[m.group(1)] = None  # crashed

# --- Warn on missing results ---
for name, data in [('Rust', rust), ('html2md', html2md), ('Go', go), ('hast', hast)]:
    if not data:
        print(f"WARNING: No {name} results parsed — check output format", file=sys.stderr)

# --- Table (all values in MiB/s) ---
C = 14  # column width
print()
top = f"┌{'─'*13}┬{'─'*C}┬{'─'*C}┬{'─'*C}┬{'─'*C}┬{'─'*C}┐"
mid = f"├{'─'*13}┼{'─'*C}┼{'─'*C}┼{'─'*C}┼{'─'*C}┼{'─'*C}┤"
bot = f"└{'─'*13}┴{'─'*C}┴{'─'*C}┴{'─'*C}┴{'─'*C}┴{'─'*C}┘"
print(top)
print(f"│ {'fixture':<11} │ {'Rust':>{C-2}} │ {'html2md':>{C-2}} │ {'Go':>{C-2}} │ {'hast':>{C-2}} │ {'turndown':>{C-2}} │")
print(f"│ {'':11} │ {'(MiB/s)':>{C-2}} │ {'(MiB/s)':>{C-2}} │ {'(MiB/s)':>{C-2}} │ {'(MiB/s)':>{C-2}} │ {'(MiB/s)':>{C-2}} │")
print(mid)
for f in FIXTURES:
    r = f'{rust[f]:.1f}'       if f in rust                      else 'n/a'
    h2 = f'{html2md[f]:.1f}'   if f in html2md                   else 'n/a'
    g = f'{go[f]:.1f}'         if f in go                        else 'n/a'
    h = f'{hast[f]:.1f}'       if f in hast                      else 'n/a'
    t = f'{turndown[f]:.1f}'   if turndown.get(f) is not None    else ('ERR' if f in turndown else 'n/a')
    print(f'│ {f:<11} │ {r:>{C-2}} │ {h2:>{C-2}} │ {g:>{C-2}} │ {h:>{C-2}} │ {t:>{C-2}} │')
print(bot)
print()
print('Throughput = input bytes / wall time. Higher is better.')
print('Rust/html2md: Criterion median | Go: go test -bench | JS/turndown: performance.now()')
PYEOF
