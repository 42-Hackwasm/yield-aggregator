#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo +stable build --all --target wasm32-unknown-unknown --release

mkdir -p res
cp target/wasm32-unknown-unknown/release/*.wasm ./res
