# utxo

An updated, re-implementation of [UXTO](https://github.com/0x7CFE/substrate-node-template/tree/utxo) on Substrate, by [Dmitriy Kashitsyn](https://github.com/0x7CFE). With [Amar Singh](https://github.com/AmarRSingh).

Reformatted for workshops & developer onboarding onto Substrate.

## Getting started

## Installation
```
# To install Rust:
curl https://sh.rustup.rs -sSf | sh

# On Windows, download and run rustup-init.exe
# from https://rustup.rs instead

rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
rustup update stable
cargo install --git https://github.com/alexcrichton/wasm-gc

# Clone the repo
./build.sh
```

## Exercise 1
1. Make the tests pass
`cargo test -p utxo-runtime`

## Exercise 2
2. Build the following extensions
