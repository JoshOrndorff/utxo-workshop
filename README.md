# UTXO on Substrate

A UTXO chain implementation on Substrate. This repo is an updated implementation of the original [Substrate UXTO](https://github.com/0x7CFE/substrate-node-template/tree/utxo) by [Dmitriy Kashitsyn](https://github.com/0x7CFE).

For a high-level overview of the code, see the [official blog post](https://www.parity.io/utxo-on-substrate/) by Dmitriy. For a live demo, check out how to set up this repo with [Polkadot UI here](#Demo-Polkadot-UI).

*Caveat: this implementation is being security reviewed and has outstanding issues. Do not use this implementation in production. It is for educational purposes only!*

Please note that the code in this repository was upgraded to Substrate pre-v2.0 in January, 2020.

## Project Structure
- `master` branch contains the full solution (cheats).
- `workshop` branch contains a UTXO boilerplate for the following workshop. The following tutorials will walks you through an implementation of UTXO on Substrate in workshop format. Feel free to host your own workshops in your local communities using this boilerplate!

## Workshop

**Estimated time**: 2 hours

Over this course of this workshop, you will learn:
- How to implement the UTXO model on Substrate
- How to secure UTXO transactions against attacks
- How to seed genesis block with UTXOs
- How to reward block validators in this environment
- How to customize transaction pool logic on Substrate
- Good coding patterns for working with Substrate & Rust

> Note: This branch contains all the answers. Try not to peek!

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
Go https://github.com/substrate-developer-hub/utxo-workshop/fork and choose your GitHub repository username.

### Clone your fork of the workshop

Clone your copy of the workshop codebase and switch to the `workshop` branch

```zsh
git clone https://github.com/<INSERT-YOUR-GITHUB-USERNAME>/utxo-workshop.git
git fetch origin workshop:workshop
git checkout workshop

# Initialize your Wasm Build environment:
./scripts/init.sh

# Build Wasm and native code:
cargo build --release
```

## Demo Polkadot UI

You can use the [Polkadot UI](https://substrate.dev/docs/en/development/front-end/polkadot-js) to exercise the capabilities of your UTXO blockchain to ensure that everything is working as expected. First, you will need ensure that any existing UTXO blockchain data has been removed and then build the UTXO blockchain:

```zsh
./target/release/utxo-workshop purge-chain --dev
./target/release/utxo-workshop --dev
```

You will notice that when you start the UTXO blockchain there is some output that is provided to help you with the following steps:
```
Initial UTXO Hash: 0x02f8ef6ff423559b1d07a31075d83c00883fbf02e054af102ab4a0ed8c50d2c5
Transaction #1 TransactionOutput {
	value: 100,
	pubkey: 0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48,
	salt: 0 },
	Hash: 0x6cef4a2aa20b106a010cc0cdeebd7f48767372e9b68e37704d595c6abed6b2be
Transaction #2 TransactionOutput {
	value: 340282366920938463463374607431768211355,
	pubkey: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d,
	salt: 0 },
	Hash: 0x4478cdec8efc46049b9b5ef6f615255455c39b67e9a932314dc81a2d63bd9681
```

1. Visit [Polkadot JS](https://polkadot.js.org/apps/#/settings)
  - Make sure you are using a browser that is compatible with local development or [consider running the UI locally](https://github.com/polkadot-js/apps)

2. Load the UTXO type definitions in Settings > Developer
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

3. Check that the genesis block contains 1 pre-configured UTXO for Alice as follows:
```rust
utxo::TransactionOutput {
	value: utxo::Value::max_value(),
	pubkey: H256::from_slice(get_from_seed::<sr25519::Public>("Alice").as_slice()),
	salt: 0,
}
```
  - Use the Polkadot JS Chain State app to query the `unspentOutputs` storage map from the `utxoModule`
  - Query the storage map with the value `0x02f8ef6ff423559b1d07a31075d83c00883fbf02e054af102ab4a0ed8c50d2c5`, which is the hash of the above transaction
  - You can [use the `subkey` tool to confirm that the public key associated with the initial UTXO belongs to Alice](https://substrate.dev/docs/en/next/development/tools/subkey#well-known-keys)

4. Send a new UTXO transaction from Alice as follows: 
```rust
utxo::Transaction {
	inputs: [ utxo::TransactionInput {
		parent_output: 0x02f8ef6ff423559b1d07a31075d83c00883fbf02e054af102ab4a0ed8c50d2c5,
		signature: sig_alice(parent_output)
	}],
	outputs: [ utxo::TransactionOutput {
		value: 100,
		pubkey: H256::from_slice(get_from_seed::<sr25519::Public>("Bob").as_slice()),
		salt: 0,
	}, utxo::TransactionOutput {
		value: utxo::Value::max_value() - 100,
		pubkey: H256::from_slice(get_from_seed::<sr25519::Public>("Alice").as_slice()),
		salt: 0,
	}]
}
```
  - Use the Polkadot JS Extrinsics app to invoke the `execute` function from the `utxoModule`
  - You will notice that the transaction's input is the pre-configured UTXO from the genesis block
  - Use the Polkadot JS Toolbox app to generate the input's `signature` (make sure you are using Alice's account to generate this value)
  - The public keys are provided for you via console output, which you can easily verify using the `subkey` tool

5. Use the Chain State app to verify the following:
  - The pre-configured UTXO from the genesis block (`0x02f8ef6ff423559b1d07a31075d83c00883fbf02e054af102ab4a0ed8c50d2c5`) no longer appears in the `unspentOutput` map
  - The `unspentOutput` map has references to both of the outputs from the above transaction (the transaction hashes are provided for you via console output)

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
1. Read about a [transaction lifecycle](https://substrate.dev/docs/en/1.0/overview/transaction-lifecycle) in Substrate

2. Read about [TaggedTransactionQueue](https://crates.parity.io/sp_transaction_pool/runtime_api/trait.TaggedTransactionQueue.html) and [TransactionValidity](https://crates.parity.io/sp_runtime/transaction_validity/type.TransactionValidity.html)

3. Implement the correct transaction ordering logic such that transactions with pending dependencies can wait in the transaction pool until its requirements are satisfied.

Hint: You can filter for specific types of transactions using the following syntax: 

```rust
if let Some(&utxo::Call::execute(ref transaction)) = IsSubType::<utxo::Module<Runtime>, Runtime>::is_sub_type(&tx.function) {
    // Your implementation here
}
```

## Challenge 3: Extensions
You can try building the following extensions:
- Give transactions in the pool a smarter longevity lifetime
- Implement coinbase transactions, by letting users add value through work

## Helpful Resources
- [Substrate documentation](http://crates.parity.io)
- [bytes to Vec<u8> converter](https://cryptii.com/pipes/integer-encoder)
- [Polkadot UI](https://polkadot.js.org/)
