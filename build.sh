#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo +stable build --all --target wasm32-unknown-unknown --release

mkdir res
cp target/wasm32-unknown-unknown/release/*.wasm ./res
