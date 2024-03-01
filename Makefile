lint:
	cargo clippy

fmt:
	cargo fmt

dev:
	cargo run -- --source=~/tmp/fonts --target=~/tmp/fonts.gz.tar
ls:
	cargo run -- ~/tmp/fonts.gz.tar

release:
	cargo build --release
