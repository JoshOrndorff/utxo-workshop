# UTXO on Substrate

This workshop takes you through a full implementaion of UTXO on Substrate. This repo is structured in workshop format.

**Estimated time**: 2 hours

**You will learn:**
- How to implement the UTXO model on Substrate
- How to secure UTXO transactions against attacks
- How to customize transaction pool logic on Substrate

**Reference**: This repo is an updated reimplementation of the original [Substrate UXTO](https://github.com/0x7CFE/substrate-node-template/tree/utxo) by [Dmitriy Kashitsyn](https://github.com/0x7CFE). 

*Note: for the full implementation, check out the `solution` branch. The `master` branch is a starting boilerplate for workshops.*

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

## Exercise 1: Security
UTXO validates transactions as follows: 
- Check signatures
- Check all inputs are unspent 
- Check input == output value
- Set Input to “spent”
- Save the new unspent outputs

Similarly in our implementation, we need to prevent malicious users from sending bad transactions.

The following tests simulate transaction attacks to our utxo implementation. The goal is to perform security checks to ensure that only valid transactions will go through.

*Hint: Remember the check-before-state-change pattern!*

### Directions
1. Run cargo test: `cargo test -p utxo-runtime`

2. Extend `verify_transaction()` to make the following tests pass. 

Hint: You may want to make them pass in this order!

```
[0] test utxo::tests::attack_with_empty_transactions ... ok
[1] test utxo::tests::attack_by_double_counting_input ... ok
[2] test utxo::tests::attack_by_double_generating_output ... ok
[3] test utxo::tests::attack_with_invalid_signature ... ok
[4] test utxo::tests::attack_by_permanently_sinking_outputs ... ok
[5] test utxo::tests::attack_by_overflowing ... ok
[6] test utxo::tests::attack_by_over_spending ... ok
```

### Answers
Answers are available by pulling the branch: `security-answers`. But where's the fun in that?

## Exercise 2: Transaction Ordering

**Scenario**: Imagine a situation where Alice pays Bob via **transaction A**, then Bob uses his new utxo to pay Charlie, via **transaction B**. In short, B depends on the success of A. 

However, depending on network latency, the runtime might deliver transaction B to a node's transaction pool before delivering transaction A!

A naive solution is to simply drop transaction B.

But Substrate lets you implement transaction ordering logic, such that you can wait for transaction A befoere dispatching B.

Directions: 
1. Read about a [transaction lifecycle](https://docs.substrate.dev/docs/transaction-lifecycle-in-substrate) in Substrate

2. Read about [TaggedTransactionQueue](https://crates.parity.io/substrate_client/runtime_api/trait.TaggedTransactionQueue.html?search=) and [TransactionValidity](https://crates.parity.io/sr_primitives/transaction_validity/enum.TransactionValidity.html)

3. Implement the correct transaction ordering logic such that transactions with pending dependencies can wait in the transaction pool until its requirements are satisfied.

Hint: You can filter for specific types of transactions using the following syntax: 

```rust
if let Some(&utxo::Call::execute(ref transaction)) = IsSubType::<utxo::Module<Runtime>>::is_aux_sub_type(&tx.function) {
    // Your implementation here
}
```


### Answers
Answers are available by pulling the branch: `ordering-answers`.


## Exercise 3: Extensions
You can try building the following extensions:
- 
- 

## Helpful Resources
- [Substrate documentation](http://crates.parity.io)
- [bytes to Vec<u8> converter](https://cryptii.com/pipes/integer-encoder)
- [Polkadot UI](https://polkadot.js.org/)

### Launching the UI
```
./build.sh              // build wasm
cargo build —release    // build binary
./target/release/utxo-runtime purge-chain -—dev // or
./target/release/utxo-runtime —-dev
```