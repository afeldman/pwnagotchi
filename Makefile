# Rust-based Pwnagotchi Makefile

.PHONY: build test clean run install fmt clippy check release tools backup restore help \
        py-install py-check py-lint py-format py-test py-type py-all

help:
	@echo "Pwnagotchi Rust Makefile"
	@echo ""
	@echo "Rust targets:"
	@echo "  build       - Build debug version"
	@echo "  release     - Build optimized release version"
	@echo "  test        - Run all tests"
	@echo "  check       - Fast check without building"
	@echo "  run         - Run the CLI with debug logging"
	@echo "  install     - Install binary to ~/.cargo/bin"
	@echo "  fmt         - Format all code with rustfmt"
	@echo "  clippy      - Run clippy linter"
	@echo "  clean       - Clean build artifacts"
	@echo "  tools       - Build utility tools (backup/restore)"
	@echo "  backup      - Backup from device (requires HOST=x.x.x.x)"
	@echo "  restore     - Restore to device (requires HOST=x.x.x.x BACKUP=file.tgz)"
	@echo ""
	@echo "Python plugin targets:"
	@echo "  py-install  - Install Python development dependencies"
	@echo "  py-check    - Run all Python checks (lint + type + test)"
	@echo "  py-lint     - Run ruff linter on Python plugins"
	@echo "  py-format   - Format Python code with black and ruff"
	@echo "  py-test     - Run Python plugin tests with pytest"
	@echo "  py-type     - Run mypy type checker"
	@echo "  py-all      - Format, lint, type-check, and test Python code"
	@echo ""

build:
	cargo build

release:
	cargo build --release

test:
	cargo test --all

check:
	cargo check --all

run:
	cargo run -- -d start

install:
	cargo install --path pwnagotchi-cli

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all -- -D warnings

clean:
	cargo clean

tools:
	cargo build --release -p pwnagotchi-tools

backup: tools
	@test -n "$(HOST)" || (echo "Usage: make backup HOST=10.0.0.2 [USER=pi] [OUTPUT=backup.tgz]"; exit 1)
	@./target/release/pwn-backup backup -n $(HOST) $(if $(USER),-u $(USER)) $(if $(OUTPUT),-o $(OUTPUT))

restore: tools
	@test -n "$(HOST)" || (echo "Usage: make restore HOST=10.0.0.2 [USER=pi] [BACKUP=file.tgz]"; exit 1)
	@./target/release/pwn-backup restore -n $(HOST) $(if $(USER),-u $(USER)) $(if $(BACKUP),-b $(BACKUP))

# Python plugin development targets
py-install:
	@echo "Installing Python development dependencies..."
	pip install -r requirements-dev.txt

py-lint:
	@echo "Running ruff linter..."
	ruff check plugins/ pwnagotchi_plugin.py

py-format:
	@echo "Formatting Python code..."
	black plugins/ pwnagotchi_plugin.py
	ruff check --fix plugins/ pwnagotchi_plugin.py

py-type:
	@echo "Running mypy type checker..."
	mypy --strict plugins/ pwnagotchi_plugin.py

py-test:
	@echo "Running pytest..."
	pytest tests/ -v

py-check: py-lint py-type
	@echo "All Python checks passed!"

py-all: py-format py-lint py-type py-test
	@echo "Python code formatted, linted, type-checked, and tested!"

# Legacy Python targets (deprecated in Rust version)
update_langs:
	@echo "Language update is deprecated in Rust version"

compile_langs:
	@echo "Language compilation is deprecated in Rust version"

