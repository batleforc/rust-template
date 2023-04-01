WINDOWS_TARGET=x86_64-pc-windows-gnu
LINUX_TARGET=x86_64-unknown-linux-gnu
MACOS_TARGET=x86_64-apple-darwin

run_api:
	@cargo run

up_docker:
	@docker-compose up jaeger postgres -d

run: up_docker run_api

test: test_docker test_api test_docker_stop

test_docker:
	@docker-compose stop postgres
	@docker-compose up jaeger test-postgres -d

test_docker_stop:
	@docker-compose rm test-postgres -f -s

test_api:
	@cargo test

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