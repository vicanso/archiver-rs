lint:
	cargo clippy

fmt:
	cargo fmt

gztar:
	cargo run -- ~/tmp/fonts ~/tmp/fonts.gz.tar
targz:
	cargo run -- ~/tmp/fonts ~/tmp/fonts.tar.gz
