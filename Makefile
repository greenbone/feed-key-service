.PHONY: test test-integration test-unit build clean build-release run

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

run:
	cargo run --bin greenbone-feed-key
