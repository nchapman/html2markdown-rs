# UniFFI Binding Tests

Foreign-language test suites for the `html2markdown-uniffi` crate. Each language uses its native test framework and exercises the same API surface.

## Prerequisites

| Language | Requirements |
|----------|-------------|
| Python   | Python 3.8+, pip |
| Swift    | Swift 5.9+ (macOS only) |
| Kotlin   | JDK 17+ (Gradle wrapper included) |

All languages require a Rust toolchain (`cargo`).

## Running tests

```sh
# Individual languages
make test-python
make test-swift
make test-kotlin

# All three
make test-bindings
```

First run installs dependencies automatically (Python venv, Gradle downloads).

## How it works

1. `cargo build --release` compiles the `html2markdown-uniffi` cdylib/staticlib
2. **Python**: `maturin develop` builds and installs the wheel into a local venv, then `pytest` runs
3. **Swift/Kotlin**: `uniffi-bindgen generate` produces language bindings from the compiled library, then `swift test` / `gradlew test` runs

Generated bindings are build artifacts — they live in `uniffi/generated/` (gitignored) and are never checked in.

## Adding a fixture to binding tests

The fixture lists are in:
- `python/test_fixtures.py` → `FIXTURE_NAMES`
- `swift/.../ConvertTests.swift` → individual `testFixture*()` methods
- `kotlin/.../ConvertTest.kt` → `@ValueSource(strings = [...])`

Add the fixture name to all three files. The fixture must exist in `test-fixtures/` with `index.html`, `index.md`, and `index.json` (with `"fragment": true`).

## Adding a new API test

Write the test in all three languages to keep coverage consistent:
- `python/test_convert.py` or `python/test_options.py`
- `swift/.../ConvertTests.swift`
- `kotlin/.../ConvertTest.kt`
