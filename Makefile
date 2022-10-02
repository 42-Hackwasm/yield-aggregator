build-opt:
	docker run --rm -v "$(CURDIR)":/code \
	 --mount type=volume,source="$$(basename "$(CURDIR)")_cache",target=/code/target \
	 --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	 cosmwasm/workspace-optimizer:0.12.8

build:
	@RUSTFLAGS='-C link-arg=-s' cargo +stable build --all --target wasm32-unknown-unknown --release
	@mkdir -p artifacts-local
	@cp target/wasm32-unknown-unknown/release/*.wasm ./artifacts-local
