.PHONY: test test-integration test-unit build clean build-release run install lint check-format format

INSTALL_PREFIX ?= /usr/local

test:
	cargo test --verbose

test-integration:
	cargo test --test service --verbose

test-unit:
	cargo test --lib --verbose

build:
	cargo build --verbose

build-release:
	cargo build --release --verbose

clean:
	cargo clean

install:
	cargo install --path . --root $(DESTDIR)$(INSTALL_PREFIX)

lint:
	cargo clippy --all-targets -- -D warnings

check-format:
	cargo fmt --all -- --check

format:
	cargo fmt --all

run:
	cargo run --bin greenbone-feed-key
