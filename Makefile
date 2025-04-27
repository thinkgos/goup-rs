
fmt.check:
	cargo fmt -- --check

clippy: 
	cargo clippy --all-targets --all-features -- -D warnings

check:
	cargo check --all

test:
	cargo test --all-features

typos:
	typos

lint: typos fmt.check clippy check test

coverage:
	cargo tarpaulin -p ${Package} --out lcov

publish:
	cargo publish --registry crates-io -p goup-version
	cargo publish --registry crates-io -p goup-downloader
	cargo publish --registry crates-io -p goup-rs

.PHONY:  fmt.check clippy check test lint coverage publish