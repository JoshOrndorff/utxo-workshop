# UTXO on Substrate

A UTXO chain implementation on Substrate

<<<<<<< HEAD
**Reference**: This repo is an updated reimplementation of the original [Substrate UXTO](https://github.com/0x7CFE/substrate-node-template/tree/utxo) by [Dmitriy Kashitsyn](https://github.com/0x7CFE)


## Workshop Format

Checkout the `workshop` branch to get started on this workshop. The following steps will take you through a full implementation of UTXO on Substrate.

> Note: `Master` branch contains all the answers. Where's the fun in that?
=======
**Reference**: This repo is an updated implementation of the original [Substrate UXTO](https://github.com/0x7CFE/substrate-node-template/tree/utxo) by [Dmitriy Kashitsyn](https://github.com/0x7CFE)


## Workshop

**Getting started**: Checkout the `workshop` branch to get started on this workshop. The following steps will take you through a full implementation of UTXO on Substrate.

> Note: This `Master` branch contains all the answers. Try not to peek!
>>>>>>> 600efd8fb907e315e53c7d6988b5034add12a8b4

**Estimated time**: 2 hours

### You will learn
- How to implement the UTXO model on Substrate
- How to secure UTXO transactions against attacks
- How to customize transaction pool logic on Substrate
- Good coding patterns for working with Substrate & Rust
<<<<<<< HEAD

## Getting started
=======
>>>>>>> 600efd8fb907e315e53c7d6988b5034add12a8b4

## Installation

### To install Rust
```zsh
curl https://sh.rustup.rs -sSf | sh

# On Windows, download and run rustup-init.exe
# from https://rustup.rs instead

rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
rustup update stable
cargo install --git https://github.com/alexcrichton/wasm-gc
```
### Clone the boilerplate
<<<<<<< HEAD
```
=======
```zsh
>>>>>>> 600efd8fb907e315e53c7d6988b5034add12a8b4
git clone https://github.com/nczhu/utxo-workshop.git
git checkout -b workshop

# Double check that it builds correctly
./build.sh
cargo build --release
```

## Exercise 1: Security
UTXO validates transactions as follows: 
- Check signatures
- Check all inputs are unspent 
- Check input == output value
- Set Input to “spent”
- Save the new unspent outputs

Similarly in our UTXO implementation, we need to prevent malicious users from sending bad transactions. `utxo.rs` contains some tests that simulate these malicious attacks. 

Your challenge is to extend the implementation such that only secure transactions will go through.

*Hint: Remember the check-before-state-change pattern!*

### Directions
*Make sure you are on the `workshop` branch*

1. Run cargo test: `cargo test -p utxo-runtime`

2. Notice that 7/8 tests are failing!
<<<<<<< HEAD
```
=======
```zsh
>>>>>>> 600efd8fb907e315e53c7d6988b5034add12a8b4
failures:
    utxo::tests::attack_by_double_counting_input
    utxo::tests::attack_by_double_generating_output
    utxo::tests::attack_by_over_spending
    utxo::tests::attack_by_overflowing
    utxo::tests::attack_by_permanently_sinking_outputs
    utxo::tests::attack_with_empty_transactions
    utxo::tests::attack_with_invalid_signature
```

3. In `utxo.rs`, extend `verify_transaction()` to make the following tests pass. 
<<<<<<< HEAD

*Hint: You may want to make them pass in this order!*
=======
>>>>>>> 600efd8fb907e315e53c7d6988b5034add12a8b4

*Hint: You may want to make them pass in this order!*

```zsh
[0] test utxo::tests::attack_with_empty_transactions ... ok
[1] test utxo::tests::attack_by_double_counting_input ... ok
[2] test utxo::tests::attack_by_double_generating_output ... ok
[3] test utxo::tests::attack_with_invalid_signature ... ok
[4] test utxo::tests::attack_by_permanently_sinking_outputs ... ok
[5] test utxo::tests::attack_by_overflowing ... ok
[6] test utxo::tests::attack_by_over_spending ... ok
```

## Exercise 2: Transaction Ordering

**Scenario**: Imagine a situation where Alice pays Bob via **transaction A**, then Bob uses his new utxo to pay Charlie, via **transaction B**. In short, B depends on the success of A. 

However, depending on network latency, the runtime might deliver **transaction B** to a node's transaction pool before delivering **transaction A**!

Your challenge is to overcome this race condition.

A naive solution is to simply drop transaction B. But Substrate lets you implement transaction ordering logic. 

In Substrate, you can specify the requirements for dispatching a transaction, e.g. wait for transaction A to arrive before dispatching B.

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

## Exercise 3: Extensions
You can try building the following extensions:
- Give transactions in the pool a smarter longevity lifetime
- Implement coinbase transactions, by letting users add value through work

## Helpful Resources
- [Substrate documentation](http://crates.parity.io)
- [bytes to Vec<u8> converter](https://cryptii.com/pipes/integer-encoder)
- [Polkadot UI](https://polkadot.js.org/)

<<<<<<< HEAD
#### Launching the UI
=======
### Using a UI boilerplate
Get the UI boilerplate [here](https://github.com/paritytech/substrate-ui)

```zsh
# In the Runtime repo
./target/release/utxo-runtime purge-chain -—dev // If you need to purge your db
./target/release/utxo-runtime —-dev

# In the UI repo
yarn install
yarn run dev
>>>>>>> 600efd8fb907e315e53c7d6988b5034add12a8b4
```

Visit `localhost:8000`

### Using the Polkadot UI

Visit [Polkadot JS](https://substrate-ui.parity.io/#/settings)

```zsh
# In the Runtime repo
./target/release/utxo-runtime purge-chain -—dev // If you need to purge your db
./target/release/utxo-runtime —-dev
```