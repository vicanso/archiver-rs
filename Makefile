lint:
	cargo clippy

fmt:
	cargo fmt

dev:
	cargo run -- --source=~/tmp/fonts --target=~/tmp/fonts.gz.tar
ls:
	cargo run -- --mode=ls --target=~/tmp/fonts.gz.tar

release:
	cargo build --release
