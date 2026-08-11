.PHONY: test test-integration test-unit build clean build-release run install \
	lint check-format format install-llvm-cov coverage

INSTALL_PREFIX ?= /usr/local

test:
	cargo test

test-integration:
	cargo test --test service

test-unit:
	cargo test --lib

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

openapi:
	cargo run --bin greenbone-feed-key-cli openapi

install-llvm-cov:
	cargo install --locked cargo-llvm-cov

coverage: install-llvm-cov
	cargo llvm-cov --locked --all-targets --html --output-dir target/coverage
	cargo llvm-cov report --locked --lcov --output-path target/coverage/lcov.info
