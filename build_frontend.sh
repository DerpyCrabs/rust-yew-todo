#!/bin/sh

pushd frontend
cargo web build --release --target wasm32-unknown-unknown
popd
