#!/bin/sh

# A couple of steps are necessary to get this build working which makes it slightly
# nonstandard compared to most other builds.
#
# * First, the Rust standard library needs to be recompiled with atomics
#   enabled. to do that we use Cargo's unstable `-Zbuild-std` feature.
#
# * Next we need to compile everything with the `atomics` and `bulk-memory`
#   features enabled, ensuring that LLVM will generate atomic instructions,
#   shared memory, passive segments, etc.

RUSTFLAGS='-C target-feature=+atomics,+bulk-memory' \
  cargo build -p client --target wasm32-unknown-unknown --release -Z build-std=std,panic_abort || exit 1

# Note the usage of `--target no-modules` here which is required for passing
# the memory import to each wasm module.
wasm-bindgen target/wasm32-unknown-unknown/release/client.wasm \
  --out-dir . \
  --target no-modules || exit 1

echo Serving HTTP on http://localhost:`[[ $1 ]] && echo $1 || echo 8000`/
python3 server.py `[[ $1 ]] && echo $1 || echo 8000` |> /dev/null
