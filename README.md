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

The following demo takes you through a scenario where `Alice sends Bob a UTXO with value 50` from her original UTXO (value 100) that she already had during genesis: 


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

3. In the console, notice the following helper printouts. In particular, notice the default account `Alice` was already has `100 UTXO` upon the genesis block.

```zsh

```

4. Open [Polkadot JS](https://polkadot.js.org/apps/#/settings), making sure the client is connected to your local node by going to Settings > General, and selecting `Local Node` in the `remote node` dropdown.

5. **Declare the custom datatypes in PolkadotJS**, since the JS client cannot authomatically infer this from the UTXO module. Go to Settings > Developer tab and paste in the following JSON:

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

6. **Check that Alice already has 100 UTXO at genesis**. In `Chain State` > `Storage`, select `utxoModule`. Input the hash `0x76584168d10a20084082ed80ec71e2a783abbb8dd6eb9d4893b089228498e9ff`. Click the `+` notation to query blockchain state.

Verify that: 
 - This UTXO has a value of `100`
 - This UTXO belongs to Alice's pubkey. You use the [subkey](https://substrate.dev/docs/en/next/development/tools/subkey#well-known-keys) tool to confirm that the pubkey indeed belongs to Alice

7. **Spend Alice's UTXO, giving 50 to Bob.** In the `Extrinsics` tab, invoke the `spend` function from the `utxoModule`, using Alice as the transaction sender. Use the following input parameters:

- outpoint: `0x76584168d10a20084082ed80ec71e2a783abbb8dd6eb9d4893b089228498e9ff`
- sigscript: `0x6ceab99702c60b111c12c2867679c5555c00dcd4d6ab40efa01e3a65083bfb6c6f5c1ed3356d7141ec61894153b8ba7fb413bf1e990ed99ff6dee5da1b24fd83`
- value: `50`
- pubkey: `0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48`
```

Send this as an unsigned transaction, since the proof is already in the `sigscript` input.

8. **Verify that your transaction succeeded**. In `Chain State`, look up the UTXO hash: `0xdbc75ab8ee9b83dcbcea4695f9c42754d94e92c3c397d63b1bc627c2a2ef94e6` to verify that a new UTXO of 50, belonging to Bob, now exists! Also you can verify that Alice's original UTXO has been spent and no longer exists in UtxoStore.

*Coming soon: A video walkthrough of the above demo.*

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
