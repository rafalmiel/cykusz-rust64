#!/bin/bash

multirust update nightly
multirust override nightly
pushd .
cd ../rust
git stash
git pull
git stash pop
popd
mkdir -p ~/.multirust/toolchains/nightly/lib/rustlib/x86_64-unknown-none-gnu/lib
rustc --target x86_64-unknown-none-gnu -Z no-landing-pads --out-dir ~/.multirust/toolchains/nightly/lib/rustlib/x86_64-unknown-none-gnu/lib ../rust/src/libcore/lib.rs
