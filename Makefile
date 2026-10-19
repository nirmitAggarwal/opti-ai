.PHONY: test fmt-check clippy build demo

test:
	cargo test --workspace --offline

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets --all-features --offline -- -D warnings

build:
	cargo build --workspace --release --offline

demo:
	cargo run --offline -p opti-cli -- demo
