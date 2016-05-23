#!/bin/bash

rustup update nightly
rustup override add nightly
pushd .
cd ../rust
git pull
git submodule update
popd
mkdir -p ./build
rustc --target x86_64-unknown-none-gnu -Z no-landing-pads --out-dir ./build ../rust/src/libcore/lib.rs
