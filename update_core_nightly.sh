#!/bin/bash

multirust update nightly
pushd .
cd ../rust
git pull
popd
rustc --target x86_64-unknown-none-gnu -Z no-landing-pads --out-dir ~/.multirust/toolchains/nightly/lib/rustlib/x86_64-unknown-none-gnu/lib ../rust/src/libcore/lib.rs
