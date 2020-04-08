# UTXO on Substrate

A UTXO chain implementation on Substrate, with two self guided workshops.Original [UXTO inspiration](https://github.com/0x7CFE/substrate-node-template/tree/utxo) by [Dmitriy Kashitsyn](https://github.com/0x7CFE).

Substrate Version: `pre-v2.0`. For educational purposes only. 

## Table of Contents
- [Installation](#Installation): Setting up Rust & Substrate dependencies

- [UI Demo](#UI-Demo): Demoing this UTXO implementation in a simple UI

- [Beginner Workshop](#Beginner-Workshop): A self guided, 1 hour workshop that familiarizes you with Substrate basics

- [Advanced Workshop](#Advanced-Workshop): A self guided, 2 hour video tutorial, that teaches you how to build this UTXO blockchain from scratch

- [Helpful Resources](#Helpful-Resources): Documentation and references if you get stuck in the workshops


## Installation

### 1. Install or update Rust
```zsh
curl https://sh.rustup.rs -sSf | sh

# On Windows, download and run rustup-init.exe
# from https://rustup.rs instead

rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
rustup update stable
cargo install --git https://github.com/alexcrichton/wasm-gc
```

### 2. Clone this workshop

Clone your copy of the workshop codebase 

```zsh
git clone https://github.com/substrate-developer-hub/utxo-workshop.git
```

## UI Demo

In this UI demo, you will interact with the UTXO blockchain via the [Polkadot UI](https://substrate.dev/docs/en/development/front-end/polkadot-js).


1. Compile and build a release in dev mode
```
# Initialize your Wasm Build environment:
./scripts/init.sh

# Build Wasm and native code:
cargo build --release
```

2. Start your node & start producing blocks:
```zsh
./target/release/utxo-workshop --dev

# If you already modified state, run this to purge the chain
./target/release/utxo-workshop purge-chain --dev
```

3. In the console, notice the following helper printouts. In particular, notice a seed account `Alice` was already seeded with `100 UTXO` upon the genesis block. 

Notice we've printed out a few more helper hashes for you, including the transaction encoding for if `Alice were to send Bob 50 UTXO`.

```zsh
TODO put printouts here
```

4. Open [Polkadot JS](https://polkadot.js.org/apps/#/settings). Make sure the client is connected to your local node by going to Settings > General, and selecting `Local Node` in the `remote node` dropdown.

5. This UTXO project defines custom datatypes that the JS client cannot infer. Define the custom types in PolkadotJS by going to Settings > Developer tab and paste in the following JSON:

```json
{
  "Value": "u128",
  "TransactionInput": {
    "outpoint": "Hash",
    "sigscript": "Hash"
  },
  "TransactionOutput": {
    "value": "Value",
    "pubkey": "Hash"
  },
  "Transaction": {
    "inputs": "Vec<TransactionInput>",
    "outputs": "Vec<TransactionOutput>"
  }
}
```

6. **Check that Alice already has 100 UTXO at genesis**. In `Chain State` > `Storage`, select `utxoModule`. Input the hash `xxxx`. Click the `+` notation to query blockchain state.

Verify that: 
 - UTXO value is `100`
 - The pubkey indeed belongs to Alice (which is a default, hardcoded account). You use the [subkey](https://substrate.dev/docs/en/next/development/tools/subkey#well-known-keys) tool to confirm that the pubkey indeed belongs to Alice

7. **Spend Alice's UTXO, giving 50 to Bob.** In the `Extrinsics` tab, invoke the `spend` function from the `utxoModule`, using Alice as the transaction sender. Submit the following transaction hash `TODO` in which Alice sends Bob 50 utxo, burning the remaining amout. 

8. **Verify that your transaction succeeded**. In `Chain State`, look up the following UTXO hash: `TODO`.
Also verify that the genesis utxo has been spent and no longer exists. 

Coming soon: A video walkthrough of the above. 

## Beginner Workshop
**Estimated time**: 2 hours

In this workshop, you will: 
- Get familiar with basic Rust and Substrate functionality
- Prevent malicious users from sending bad UTXO transactions

Your challenge is to fix the code such that: 
1. The Rust compiler compiles without errors
2. All tests in `utxo.rs` pass, ensuring secure transactions

### Directions
1. Checkout the `workshop` branch. The `Master` branch has the solutions, so don't peek!

```zsh
git fetch origin workshop:workshop
git checkout workshop
```

2. Cd into the base directory. Try running the test with: `cargo test -p utxo-runtime`. Notice all the compiler errors!
```zsh
ADD all the errors
```

3. Once your code compiles, that `X/9` tests are failing!
```zsh
failures:
    utxo::tests::attack_by_double_counting_input
    utxo::tests::attack_by_double_generating_output
    utxo::tests::attack_by_over_spending
    utxo::tests::attack_by_overflowing
    utxo::tests::attack_by_permanently_sinking_outputs
    utxo::tests::attack_with_empty_transactions
    utxo::tests::attack_with_invalid_signature
```

4. Go to `utxo.rs` and read the workshop comments. Your goal is to edit the `check_transaction()` function & make all tests pass.

*Hint: You may want to make them pass in the following order!*

```zsh
[0] test utxo::tests::attack_with_empty_transactions ... ok
[1] test utxo::tests::attack_by_double_counting_input ... ok
[2] test utxo::tests::attack_by_double_generating_output ... ok
[3] test utxo::tests::attack_with_invalid_signature ... ok
[4] test utxo::tests::attack_by_permanently_sinking_outputs ... ok
[5] test utxo::tests::attack_by_overflowing ... ok
[6] test utxo::tests::attack_by_over_spending ... ok
```

## Advanced Workshop
**VIDEO TUTORIALS COMING SOON**

**Estimated time**: 1 hour

In this workshop, you will implement this UTXO project from scratch and learn:
- How to implement the UTXO model on Substrate
- How to secure UTXO transactions against attacks
- How to seed genesis block with UTXOs
- How to reward block validators in this environment
- How to customize transaction pool logic on Substrate
- Good coding patterns for working with Substrate & Rust


## Helpful Resources
- [Substrate documentation](http://crates.parity.io)
- [bytes to Vec<u8> converter](https://cryptii.com/pipes/integer-encoder)
- [Polkadot UI](https://polkadot.js.org/)
