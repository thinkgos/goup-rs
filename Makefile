
fmt.check:
	cargo fmt -- --check

clippy: 
	cargo clippy --all-targets --all-features -- -D warnings

check:
	cargo check --all

test:
	cargo test --all-features

udeps:
	cargo +nightly udeps

shear:
	cargo shear

audit:
	cargo audit

typos:
	typos

lint: typos fmt.check clippy check test

coverage:
	cargo tarpaulin --out lcov

publish:
	cargo publish --registry crates-io

.PHONY:  fmt.check clippy check test lint coverage publish