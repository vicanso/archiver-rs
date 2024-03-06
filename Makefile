lint:
	cargo clippy

fmt:
	cargo fmt

dev:
	LOG_LEVEL=debug cargo run -- --source=~/tmp/fonts --target=~/tmp/fonts.gz.tar
ls:
	cargo run -- ~/tmp/fonts.gz.tar
unarchive:
	cargo run -- ~/tmp/fonts.gz.tar --output=~/tmp/fonts-new

release:
	cargo build --release
