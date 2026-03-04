CARGO       ?= cargo
UNIFFI_DIR  := uniffi
TARGET_DIR  := $(UNIFFI_DIR)/target
RELEASE_DIR := $(TARGET_DIR)/release

# Detect library extension
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
  CDYLIB_EXT  := dylib
  STATICLIB_EXT := a
else
  CDYLIB_EXT  := so
  STATICLIB_EXT := a
endif

CDYLIB  := $(RELEASE_DIR)/libhtml2markdown_uniffi.$(CDYLIB_EXT)
STATICLIB := $(RELEASE_DIR)/libhtml2markdown_uniffi.$(STATICLIB_EXT)

# --- Prerequisite checks ---

define require
  $(if $(shell which $(1) 2>/dev/null),,$(error "$(1)" not found — install it first))
endef

# --- Cargo build ---

.PHONY: cargo-build
cargo-build:
	$(CARGO) build --manifest-path $(UNIFFI_DIR)/Cargo.toml --release

# --- uniffi-bindgen ---

BINDGEN := $(CARGO) run --manifest-path $(UNIFFI_DIR)/Cargo.toml --features cli --bin uniffi-bindgen --
DART_BINDGEN := $(CARGO) run --manifest-path $(UNIFFI_DIR)/Cargo.toml --features dart-cli --bin uniffi-bindgen-dart --

GENERATED_DIR := $(UNIFFI_DIR)/generated

$(GENERATED_DIR)/python: cargo-build
	$(BINDGEN) generate --library $(CDYLIB) --language python --out-dir $@

$(GENERATED_DIR)/swift: cargo-build
	$(BINDGEN) generate --library $(CDYLIB) --language swift --out-dir $@

$(GENERATED_DIR)/kotlin: cargo-build
	$(BINDGEN) generate --library $(CDYLIB) --language kotlin --out-dir $@

$(GENERATED_DIR)/ruby: cargo-build
	$(BINDGEN) generate --library $(CDYLIB) --language ruby --out-dir $@

$(GENERATED_DIR)/dart: cargo-build
	$(DART_BINDGEN) generate --library $(CDYLIB) --out-dir $@

# --- Python ---

VENV := .venv
PYTHON := $(VENV)/bin/python
PIP := $(VENV)/bin/pip

$(VENV):
	$(call require,python3)
	python3 -m venv $(VENV)
	$(PIP) install --upgrade pip

.PHONY: setup-python
setup-python: $(VENV)
	$(PIP) install -r requirements-dev.txt

.PHONY: build-python
build-python: $(VENV)
	$(call require,python3)
	$(VENV)/bin/maturin develop --manifest-path $(UNIFFI_DIR)/Cargo.toml --release

.PHONY: test-python
test-python: build-python
	$(VENV)/bin/pytest tests/bindings/python/ -v

.PHONY: lint-python
lint-python: $(VENV)
	$(VENV)/bin/mypy tests/bindings/python/

# --- Swift ---

SWIFT_TEST_DIR := tests/bindings/swift
SWIFT_SRC_DIR  := $(SWIFT_TEST_DIR)/Sources/html2markdown_uniffiFFI

.PHONY: build-swift
build-swift: $(GENERATED_DIR)/swift cargo-build
	$(call require,swift)
	cp $(GENERATED_DIR)/swift/html2markdown_uniffiFFI.h $(SWIFT_SRC_DIR)/
	mkdir -p $(SWIFT_TEST_DIR)/Sources/Html2Markdown
	cp $(GENERATED_DIR)/swift/html2markdown_uniffi.swift \
		$(SWIFT_TEST_DIR)/Sources/Html2Markdown/html2markdown_uniffi.swift

.PHONY: test-swift
test-swift: build-swift
	cd $(SWIFT_TEST_DIR) && \
		swift test \
			-Xlinker -L../../../$(RELEASE_DIR) \
			-Xlinker -lhtml2markdown_uniffi

# --- Kotlin ---

KOTLIN_TEST_DIR := tests/bindings/kotlin
KOTLIN_GEN_DIR  := $(KOTLIN_TEST_DIR)/src/main/kotlin

.PHONY: build-kotlin
build-kotlin: $(GENERATED_DIR)/kotlin cargo-build
	$(call require,java)
	mkdir -p $(KOTLIN_GEN_DIR)
	cp -r $(GENERATED_DIR)/kotlin/uniffi $(KOTLIN_GEN_DIR)/

.PHONY: test-kotlin
test-kotlin: build-kotlin
	cd $(KOTLIN_TEST_DIR) && ./gradlew test

# --- Ruby ---

RUBY_TEST_DIR := tests/bindings/ruby

.PHONY: build-ruby
build-ruby: $(GENERATED_DIR)/ruby cargo-build
	$(call require,ruby)
	$(call require,bundle)
	mkdir -p $(RUBY_TEST_DIR)/lib
	cp $(GENERATED_DIR)/ruby/html2markdown_uniffi.rb $(RUBY_TEST_DIR)/lib/
	cd $(RUBY_TEST_DIR) && bundle install

.PHONY: test-ruby
test-ruby: build-ruby
	cd $(RUBY_TEST_DIR) && bundle exec rake test

# --- Dart ---

DART_TEST_DIR := tests/bindings/dart

.PHONY: build-dart
build-dart: $(GENERATED_DIR)/dart cargo-build
	$(call require,dart)
	mkdir -p $(DART_TEST_DIR)/lib
	cp $(GENERATED_DIR)/dart/html2markdown_uniffi.dart $(DART_TEST_DIR)/lib/
	cd $(DART_TEST_DIR) && dart pub get

.PHONY: test-dart
test-dart: build-dart
	cd $(DART_TEST_DIR) && dart test -r expanded

# --- Aggregate ---

.PHONY: test-bindings
test-bindings: test-python test-swift test-kotlin test-ruby test-dart

.PHONY: clean
clean:
	rm -rf $(GENERATED_DIR)
	rm -rf $(VENV)
	rm -rf $(SWIFT_TEST_DIR)/.build
	rm -rf $(KOTLIN_TEST_DIR)/build $(KOTLIN_TEST_DIR)/.gradle
	rm -rf $(RUBY_TEST_DIR)/vendor $(RUBY_TEST_DIR)/.bundle $(RUBY_TEST_DIR)/Gemfile.lock
	rm -rf $(DART_TEST_DIR)/.dart_tool $(DART_TEST_DIR)/pubspec.lock
	cd $(UNIFFI_DIR) && $(CARGO) clean
