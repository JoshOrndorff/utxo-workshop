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

# Polkadot UI
```
./build.sh              // build wasm
cargo build —release    // build binary
./target/release/utxo-runtime purge-chain -—dev
./target/release/utxo-runtime —-dev
```

https://polkadot.js.org/ 


## Exercise 1
1. Make the tests pass
`cargo test -p utxo-runtime`

## Exercise 2
2. Build the following extensions


## Helpful Resources
- [bytes to Vec<u8> converter](https://cryptii.com/pipes/integer-encoder)
- [Substrate documentation](http://crates.parity.io)
