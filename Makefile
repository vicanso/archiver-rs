lint:
	cargo clippy

fmt:
	cargo fmt

dev:
	cargo run -- --source=~/tmp/fonts --target=~/tmp/fonts.gz.tar
ls:
	cargo run -- ~/tmp/fonts.gz.tar
unarchive:
	cargo run -- ~/tmp/fonts.gz.tar --mode=unarchive --output=~/tmp/fonts-new

release:
	cargo build --release
