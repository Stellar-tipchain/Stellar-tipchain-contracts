WASM_TARGET := wasm32v1-none
WASM := target/$(WASM_TARGET)/release/tipjar.wasm

.PHONY: all build test fmt fmt-check clippy check clean

all: build test

build:
	cargo build -p tipjar --target $(WASM_TARGET) --release

test:
	cargo test -p tipjar

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --all-targets -- -D warnings

check: fmt-check clippy test

clean:
	cargo clean
