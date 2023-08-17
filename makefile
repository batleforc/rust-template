WINDOWS_TARGET=x86_64-pc-windows-gnu
LINUX_TARGET=x86_64-unknown-linux-gnu
MACOS_TARGET=x86_64-apple-darwin
REGISTRY=harbor.weebo.fr/batleforc/
IMAGE_NAME=rust_api
TAG=latest

test:
	docker compose up db-test -d
	cargo test
	docker compose down db-test
	docker compose rm -v -f db-test