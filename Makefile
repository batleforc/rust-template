WINDOWS_TARGET=x86_64-pc-windows-gnu
LINUX_TARGET=x86_64-unknown-linux-gnu
MACOS_TARGET=x86_64-apple-darwin

run_api:
	@cargo run

run_docker:
	@docker-compose up -d

run: run_docker run_api


stop_docker:
	@docker-compose down

stop: stop_docker

build_api_windows:
	@rustup target add $(WINDOWS_TARGET)
	@cargo build --target $(WINDOWS_TARGET)

build_api_linux:
	@rustup target add $(LINUX_TARGET)
	@cargo build --target $(LINUX_TARGET)

build_api_macos:
	@rustup target add $(MACOS_TARGET)
	@cargo build --target $(MACOS_TARGET)

build_api: build_api_windows build_api_linux build_api_macos