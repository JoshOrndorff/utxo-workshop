# UTXO on Substrate

A UTXO chain implementation on Substrate. This repo is an updated implementation of the original [Substrate UXTO](https://github.com/0x7CFE/substrate-node-template/tree/utxo) by [Dmitriy Kashitsyn](https://github.com/0x7CFE).

For a high-level overview of the code, see the [official blog post](https://www.parity.io/utxo-on-substrate/) by Dmitriy. For a live demo, check out how to set up this repo with [Polkadot UI here](#Demo-Polkadot-UI).

*Caveat: this implementation is being security reviewed and has outstanding issues. Do not use this implementation in production. It is for educational purposes only!*

## Project Structure
- `master` branch contains the full solution (cheats).
- `workshop` branch contains a UTXO boilerplate for the following workshop. The following tutorials will walk you through an implementation of UTXO on Substrate in workshop format. Feel free to host your own workshops in your local communities using this boilerplate!

## Workshop

**Estimated time**: 2 hours

Over this course of this workshop, you will learn:
- How to implement the UTXO model on Substrate
- How to secure UTXO transactions against attacks
- How to seed genesis block with UTXOs
- How to reward block validators in this environment
- How to customize transaction pool logic on Substrate
- Good coding patterns for working with Substrate & Rust

> Note: This `Master` branch contains all the answers. Try not to peek!

### 1. Install | Upgrade Rust
```zsh
curl https://sh.rustup.rs -sSf | sh

# On Windows, download and run rustup-init.exe
# from https://rustup.rs instead

rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
rustup update stable
cargo install --git https://github.com/alexcrichton/wasm-gc
```
### Fork the workshop boilerplate

Fork the workshop to create your own copy of it in your GitHub repository.
Go https://github.com/nczhu/utxo-workshop/fork and choose your GitHub repository username.

### Clone your fork of the workshop

Clone your copy of the workshop codebase and switch to the workshop branch

```zsh
git clone https://github.com/<INSERT-YOUR-GITHUB-USERNAME>/utxo-workshop.git
git fetch origin workshop:workshop
git checkout workshop

# Double check that it builds correctly
./build.sh
cargo build --release
```

## Challenge 1: UTXO Transaction Security
UTXO validates transactions as follows: 
- Check signatures
- Check all inputs are unspent 
- Check input == output value
- Set Input to "spent"

Similarly in our UTXO implementation, we need to prevent malicious users from sending bad transactions. `utxo.rs` contains some tests that simulate these malicious attacks. 

Your challenge is to extend the implementation such that only secure transactions will go through.

*Hint: Remember the check-before-state-change pattern!*

### Directions
*Make sure you have created a local copy of the remote `workshop` branch*

1. Run cargo test: `cargo test -p utxo-runtime`

2. Notice that 7/8 tests are failing!
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

3. In `utxo.rs`, extend `check_transaction()` to make the following tests pass. 

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

## Challenge 2: Transaction Ordering

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

## Challenge 3: Extensions
You can try building the following extensions:
- Give transactions in the pool a smarter longevity lifetime
- Implement coinbase transactions, by letting users add value through work

## Demo Polkadot UI

```zsh
# In the Runtime repo
./target/release/utxo purge-chain --dev // If you need to purge your db
./target/release/utxo --dev
```

1. Visit [Polkadot JS](https://substrate-ui.parity.io/#/settings)

2. Load your type definitions in Settings > Developer
```json
{
  "Value": "u128",
  "LockStatus": "u32",
  "TransactionInput": {
    "parent_output": "Hash",
    "signature": "Signature"
  },
  "TransactionOutput": {
    "value": "Value",
    "pubkey": "Hash",
    "salt": "u64"
  },
  "Transaction": {
    "inputs": "Vec<TransactionInput>",
    "outputs": "Vec<TransactionOutput>"
  }
}
```

3. Go to "Accounts > Create Account".  Create an account `Alice1` from this seed using `sr25519`:
`0x9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60`

4. Check that the genesis block contains 1 pre-configured UTXO for Alice as follows:
```rust
TransactionOutput {
  value: Value::max_value(),
  pubkey: H256::from_slice(&ALICE_KEY),
  salt: 0,
}
```

Hint: UTXO Hash
`0xf414d3dfaf46a7f547f8c3572e5831228fe3795a5f26dd10a1f6ae323993b234`

5. Send a new UTXO transaction from Alice as follows: 
```rust
TransactionOutput {
  value: 100,
  pubkey: H256::from_slice(&ALICE_KEY),
  salt: 2,
}],
```

Hint: Encoded Transaction
`0x04f414d3dfaf46a7f547f8c3572e5831228fe3795a5f26dd10a1f6ae323993b234dc6dda5055768c30c1134dc83ce55b3c463636899a33c9fc62dbac39018b562fa21532b3c487a71dab55036f2e6e0a19ef98b05272c07db6f013c055e3659400046400000000000000000000000000000044a996beb1eef7bdcab976ab6d2ca26104834164ecf28fb375600576fcc6eb0f0200000000000000`

6. Check that the new utxo was generated and the extrinsic succeeded in the block.

Hint: new UTXO hash
`0xd25d4a5cade9f8219cfffffd8474d323a5ba0b2deb5db4a490e1d3b9feb79278`

## Helpful Resources
- [Substrate documentation](http://crates.parity.io)
- [bytes to Vec<u8> converter](https://cryptii.com/pipes/integer-encoder)
- [Polkadot UI](https://polkadot.js.org/)
