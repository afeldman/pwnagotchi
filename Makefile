# Rust-based Pwnagotchi Makefile

.PHONY: build test clean run install fmt clippy check release tools backup restore help

help:
	@echo "Pwnagotchi Rust Makefile"
	@echo ""
	@echo "Available targets:"
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

# Legacy Python targets (deprecated in Rust version)
update_langs:
	@echo "Language update is deprecated in Rust version"

compile_langs:
	@echo "Language compilation is deprecated in Rust version"

