.PHONY: test build clean build-release run

INSTALL_PREFIX ?= /usr/local

test:
	cargo test --verbose

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
