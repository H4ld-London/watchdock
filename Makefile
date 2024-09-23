.PHONY: build/dev
build/dev:
	cargo build

.PHONY: build/release
build/release:
	cargo build --release

.PHONY: docker/release
docker/release:
	docker build --build-arg BUILD_FLAGS=--release .
