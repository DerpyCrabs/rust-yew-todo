#!/bin/sh

pushd frontend
cargo web build --release --target wasm32-unknown-unknown
popd

cp frontend/target/wasm32-unknown-unknown/release/frontend.js backend/static/frontend.js
cp frontend/target/wasm32-unknown-unknown/release/frontend.wasm backend/static/frontend.wasm

pushd backend
cargo build --release
popd
