.PHONY: test build clean build-release

test:
	cargo test --verbose

build:
	cargo build --verbose

build-release:
	cargo build --release --verbose

clean:
	cargo clean
