use codec::{Decode, Encode};
use frame_support::{
	decl_event, decl_module, decl_storage,
	dispatch::{DispatchResult, Vec},
	ensure,
};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{
	crypto::Public as _,
	H256,
	H512,
	sr25519::{Public, Signature},
};
use sp_std::collections::btree_map::BTreeMap;
use sp_runtime::{
	traits::{BlakeTwo256, Hash, SaturatedConversion},
	transaction_validity::{TransactionLongevity, ValidTransaction},
};
use super::{block_author::BlockAuthor, issuance::Issuance};

pub trait Trait: frame_system::Trait {
	/// The ubiquitous Event type
	type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;

	/// A source to determine the block author
	type BlockAuthor: BlockAuthor;

	/// A source to determine the issuance portion of the block reward
	type Issuance: Issuance<<Self as frame_system::Trait>::BlockNumber, Value>;
}

pub type Value = u128;

/// Single transaction to be dispatched
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug)]
pub struct Transaction {
	/// UTXOs to be used as inputs for current transaction
	pub inputs: Vec<TransactionInput>,

	/// UTXOs to be created as a result of current transaction dispatch
	pub outputs: Vec<TransactionOutput>,
}

/// Single transaction input that refers to one UTXO
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug)]
pub struct TransactionInput {
	/// Reference to an UTXO to be spent
	pub outpoint: H256,

	/// Proof that transaction owner is authorized to spend referred UTXO &
	/// that the entire transaction is untampered
	pub sigscript: H512,
}

/// Single transaction output to create upon transaction dispatch
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug)]
pub struct TransactionOutput {
	/// Value associated with this output
	pub value: Value,

	/// Public key associated with this output. In order to spend this output
	/// owner must provide a proof by hashing the whole `Transaction` and
	/// signing it with a corresponding private key.
	pub pubkey: H256,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug)]
pub struct ArchivedTransaction {
	/// UTXOs spent/used as input
	pub inputs: Vec<TransactionOutput>,

	/// output hashes
	pub outputs: Vec<H256>
}

decl_storage! {
	trait Store for Module<T: Trait> as Utxo {
		/// All valid unspent transaction outputs are stored in this map.
		/// Initial set of UTXO is populated from the list stored in genesis.
		/// We use the identity hasher here because the cryptographic hashing is
		/// done explicitly. TODO In the future we should remove the explicit hashing,
		/// and use blake2_128_concat here. I'm deferring that so as not to break
		/// the workshop inputs.
		UtxoStore build(|config: &GenesisConfig| {
			config.genesis_utxos
				.iter()
				.cloned()
				.map(|u| (BlakeTwo256::hash_of(&u), u))
				.collect::<Vec<_>>()
		}): map hasher(identity) H256 => Option<TransactionOutput>;

		/// Map spent UTXOs to the transaction (hash) that spent them.
		UtxoArchive: map hasher(identity) H256 => (H256, u64);
		/// Store successful transactions.
		TransactionArchive: map hasher(identity) H256 => ArchivedTransaction;

		/// Total reward value to be redistributed among authorities.
		/// It is accumulated from transactions during block execution
		/// and then dispersed to validators on block finalization.
		pub RewardTotal get(fn reward_total): Value;
	}

	add_extra_genesis {
		config(genesis_utxos): Vec<TransactionOutput>;
	}
}

// External functions: callable by the end user
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		/// Dispatch a single transaction and update UTXO set accordingly
		#[weight = 1_000_000] //TODO weight should be proportional to number of inputs + outputs
		pub fn spend(_origin, transaction: Transaction) -> DispatchResult {
									// TransactionValidity{}
			let transaction_validity = Self::validate_transaction(&transaction)?;
			ensure!(transaction_validity.requires.is_empty(), "missing inputs");

			Self::update_storage(&transaction, transaction_validity.priority as Value)?;

			Self::deposit_event(Event::TransactionSuccess(transaction));

			Ok(())
		}

		/// Handler called by the system on block finalization
		fn on_finalize() {
			match T::BlockAuthor::block_author() {
				// Block author did not provide key to claim reward
				None => Self::deposit_event(Event::RewardsWasted),
				// Block author did provide key, so issue thir reward
				Some(author) => Self::disperse_reward(&author),
			}
		}
	}
}

decl_event!(
	pub enum Event {
		/// Transaction was executed successfully
		TransactionSuccess(Transaction),
		/// Rewards were issued. Amount, UTXO hash.
		RewardsIssued(Value, H256),
		/// Rewards were wasted
		RewardsWasted,
	}
);

// "Internal" functions, callable by code.
impl<T: Trait> Module<T> {

	/// Check transaction for validity, errors, & race conditions
	/// Called by both transaction pool and runtime execution
	///
	/// Ensures that:
	/// - inputs and outputs are not empty
	/// - all inputs match to existing, unspent and unlocked outputs
	/// - each input is used exactly once
	/// - each output is defined exactly once and has nonzero value
	/// - total output value must not exceed total input value
	/// - new outputs do not collide with existing ones
	/// - sum of input and output values does not overflow
	/// - provided signatures are valid
	/// - transaction outputs cannot be modified by malicious nodes
	pub fn validate_transaction(transaction: &Transaction) -> Result<ValidTransaction, &'static str> {
		// Check basic requirements
		ensure!(!transaction.inputs.is_empty(), "no inputs");
		ensure!(!transaction.outputs.is_empty(), "no outputs");

		{
			let input_set: BTreeMap<_, ()> =transaction.inputs.iter().map(|input| (input, ())).collect();
			ensure!(input_set.len() == transaction.inputs.len(), "each input must only be used once");
		}
		{
			let output_set: BTreeMap<_, ()> = transaction.outputs.iter().map(|output| (output, ())).collect();
			ensure!(output_set.len() == transaction.outputs.len(), "each output must be defined only once");
		}

		let mut total_input: Value = 0;
		let mut total_output: Value = 0;
		let mut output_index: u64 = 0;
		let simple_transaction = Self::get_simple_transaction(transaction);

		// Variables sent to transaction pool
		let mut missing_utxos = Vec::new();
		let mut new_utxos = Vec::new();
		let mut reward = 0;

		// Check that inputs are valid
		for input in transaction.inputs.iter() {
			if let Some(input_utxo) = <UtxoStore>::get(&input.outpoint) {
				ensure!(sp_io::crypto::sr25519_verify(
					&Signature::from_raw(*input.sigscript.as_fixed_bytes()),
					&simple_transaction,
					&Public::from_h256(input_utxo.pubkey)
				), "signature must be valid" );
				total_input = total_input.checked_add(input_utxo.value).ok_or("input value overflow")?;
			} else {
				missing_utxos.push(input.outpoint.clone().as_fixed_bytes().to_vec());
			}
		}

		// Check that outputs are valid
		for output in transaction.outputs.iter() {
			ensure!(output.value > 0, "output value must be nonzero");
			let hash = BlakeTwo256::hash_of(&(&transaction.encode(), output_index));
			output_index = output_index.checked_add(1).ok_or("output index overflow")?;
			ensure!(!<UtxoStore>::contains_key(hash), "output already exists");
			total_output = total_output.checked_add(output.value).ok_or("output value overflow")?;
			new_utxos.push(hash.as_fixed_bytes().to_vec());
		}

		// If no race condition, check the math
		if missing_utxos.is_empty() {
			ensure!( total_input >= total_output, "output value must not exceed input value");
			reward = total_input.checked_sub(total_output).ok_or("reward underflow")?;
		}

		// Returns transaction details
		Ok(ValidTransaction {
			requires: missing_utxos,
			provides: new_utxos,
			priority: reward as u64,
			longevity: TransactionLongevity::max_value(),
			propagate: true,
		})
	}

	/// Update storage to reflect changes made by transaction
	/// Where each utxo key is a hash of the entire transaction and its order in the TransactionOutputs vector
	fn update_storage(transaction: &Transaction, reward: Value) -> DispatchResult {
		// Calculate new reward total
		let new_total = <RewardTotal>::get()
			.checked_add(reward)
			.ok_or("Reward overflow")?;
		<RewardTotal>::put(new_total);

		// Removing spent UTXOs
		let mut inputs = Vec::new();
		let mut to_archive = Vec::new();
		let mut archive_idx = 0;
		for input in &transaction.inputs {
			if let Some(input_utxo) = <UtxoStore>::take(input.outpoint) {
				inputs.push(input_utxo);
				to_archive.push((input.outpoint, archive_idx));
				archive_idx += 1;
			}
		}

		let mut index: u64 = 0;
		let mut outputs = Vec::new();
		for output in &transaction.outputs {
			let hash = BlakeTwo256::hash_of(&(&transaction.encode(), index));
			index = index.checked_add(1).ok_or("output index overflow")?;
			<UtxoStore>::insert(hash, output);
			outputs.push(hash);
		}

		// archive the transaction for tracing
		let archive_tx = ArchivedTransaction { inputs, outputs };
		let archive_tx_hash = BlakeTwo256::hash_of(&archive_tx.encode());
		<TransactionArchive>::insert(archive_tx_hash, archive_tx);
		for (hash, idx) in to_archive {
			<UtxoArchive>::insert(hash, (archive_tx_hash, idx));
		}

		Ok(())
	}

	/// Redistribute combined reward value to block Author
	fn disperse_reward(author: &Public) {
		let reward = RewardTotal::take() + T::Issuance::issuance(frame_system::Module::<T>::block_number());

		let utxo = TransactionOutput {
			value: reward,
			pubkey: H256::from_slice(author.as_slice()),
		};

		let hash = BlakeTwo256::hash_of(&(&utxo,
					<frame_system::Module<T>>::block_number().saturated_into::<u64>()));

		<UtxoStore>::insert(hash, utxo);
		Self::deposit_event(Event::RewardsIssued(reward, hash));
	}

	// Strips a transaction of its Signature fields by replacing value with ZERO-initialized fixed hash.
	pub fn get_simple_transaction(transaction: &Transaction) -> Vec<u8> {//&'a [u8] {
		let mut trx = transaction.clone();
		for input in trx.inputs.iter_mut() {
			input.sigscript = H512::zero();
		}

		trx.encode()
	}

	/// Helper fn for Transaction Pool
	/// Checks for race condition, if a certain trx is missing input_utxos in UtxoStore
	/// If None missing inputs: no race condition, gtg
	/// if Some(missing inputs): there are missing variables
	pub fn get_missing_utxos(transaction: &Transaction) -> Vec<&H256> {
		let mut missing_utxos = Vec::new();
		for input in transaction.inputs.iter() {
			if <UtxoStore>::get(&input.outpoint).is_none() {
				missing_utxos.push(&input.outpoint);
			}
		}
		missing_utxos
	}
}

/// Tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use frame_support::{assert_ok, assert_noop, impl_outer_origin, parameter_types, weights::Weight};
	use sp_runtime::{testing::Header, traits::IdentityLookup, Perbill};
	use sp_core::testing::{KeyStore, SR25519};
	use sp_core::traits::KeystoreExt;

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
			pub const BlockHashCount: u64 = 250;
			pub const MaximumBlockWeight: Weight = 1024;
			pub const MaximumBlockLength: u32 = 2 * 1024;
			pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl frame_system::Trait for Test {
		type BaseCallFilter = ();
		type Origin = Origin;
		type Call = ();
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type DbWeight = ();
		type BlockExecutionWeight = ();
		type ExtrinsicBaseWeight = ();
		type MaximumExtrinsicWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
		type ModuleToIndex = ();
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
	}

	impl Trait for Test {
		type Event = ();
		type BlockAuthor = ();
		type Issuance = ();
	}

	type Utxo = Module<Test>;

	// need to manually import this crate since its no include by default
	use hex_literal::hex;

	const ALICE_PHRASE: &str = "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
	// other random account generated with subkey
	const KARL_PHRASE: &str = "monitor exhibit resource stumble subject nut valid furnace obscure misery satoshi assume";
	const GENESIS_UTXO: [u8; 32] = hex!("79eabcbd5ef6e958c6a7851b36da07691c19bda1835a08f875aa286911800999");

	// This function basically just builds a genesis storage key/value store according to our desired mockup.
	// We start each test by giving Alice 100 utxo to start with.
	fn new_test_ext() -> sp_io::TestExternalities {

		let keystore = KeyStore::new(); // a key storage to store new key pairs during testing
		let alice_pub_key = keystore.write().sr25519_generate_new(SR25519, Some(ALICE_PHRASE)).unwrap();

		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		t.top.extend(
			GenesisConfig {
				genesis_utxos: vec![
					TransactionOutput {
						value: 100,
						pubkey: H256::from(alice_pub_key),
					}
				],
				..Default::default()
			}
			.build_storage()
			.unwrap()
			.top,
		);

		// Print the values to get GENESIS_UTXO
		let mut ext = sp_io::TestExternalities::from(t);
		ext.register_extension(KeystoreExt(keystore));
		ext
	}

	fn new_test_ext_and_keys() -> (sp_io::TestExternalities, Public, Public) {

		let keystore = KeyStore::new(); // a key storage to store new key pairs during testing
		let alice_pub_key = keystore.write().sr25519_generate_new(SR25519, Some(ALICE_PHRASE)).unwrap();
		let karl_pub_key = keystore.write().sr25519_generate_new(SR25519, Some(KARL_PHRASE)).unwrap();

		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		t.top.extend(
			GenesisConfig {
				genesis_utxos: vec![
					TransactionOutput {
						value: 100,
						pubkey: H256::from(alice_pub_key),
					}
				],
				..Default::default()
			}
			.build_storage()
			.unwrap()
			.top,
		);

		// Print the values to get GENESIS_UTXO
		let mut ext = sp_io::TestExternalities::from(t);
		ext.register_extension(KeystoreExt(keystore));
		(ext, alice_pub_key, karl_pub_key)
	}

	#[test]
	fn test_simple_transaction() {
		new_test_ext().execute_with(|| {
			let alice_pub_key = sp_io::crypto::sr25519_public_keys(SR25519)[0];

			// Alice wants to send herself a new utxo of value 50.
			let mut transaction = Transaction {
				inputs: vec![TransactionInput {
					outpoint: H256::from(GENESIS_UTXO),
					sigscript: H512::zero(),
				}],
				outputs: vec![TransactionOutput {
					value: 50,
					pubkey: H256::from(alice_pub_key),
				}],
			};

			let alice_signature = sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode()).unwrap();
			transaction.inputs[0].sigscript = H512::from(alice_signature);
			let new_utxo_hash = BlakeTwo256::hash_of(&(&transaction.encode(), 0 as u64));

			assert_ok!(Utxo::spend(Origin::signed(0), transaction));
			assert!(!UtxoStore::contains_key(H256::from(GENESIS_UTXO)));
			assert!(UtxoStore::contains_key(new_utxo_hash));
			assert_eq!(50, UtxoStore::get(new_utxo_hash).unwrap().value);
		});
	}

	#[test]
	fn attack_with_missing_account() {
		let (mut test_ext, alice_pub_key, karl_pub_key) = new_test_ext_and_keys();
		test_ext.execute_with(|| {
			// Construct a transaction that consumes a bogus input, and sends 50 tokens to Alice.
			let mut transaction = Transaction {
				inputs: vec![TransactionInput {
					// @apopiak outpoint is supposed to be a utxo hash, not a pubkey.
					// Karl's key works because it happens to be the same number of bits.
					outpoint: H256::from(karl_pub_key),
					sigscript: H512::zero(),
				}],
				outputs: vec![TransactionOutput {
					value: 50,
					pubkey: H256::from(alice_pub_key),
				}],
			};

			// Alice signs the transaction
			let alice_signature = sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode()).unwrap();
			transaction.inputs[0].sigscript = H512::from(alice_signature);


			assert_noop!(Utxo::spend(Origin::signed(0), transaction), "missing inputs");
		});
	}

	#[test]
	fn attack_with_sending_to_own_account() {
		let (mut test_ext, _alice, karl_pub_key) = new_test_ext_and_keys();
		test_ext.execute_with(|| {
			// Karl wants to send himself a new utxo of value 50 out of thin air.
			let mut transaction = Transaction {
				inputs: vec![TransactionInput {
					outpoint: H256::from(karl_pub_key),
					sigscript: H512::zero(),
				}],
				outputs: vec![TransactionOutput {
					value: 50,
					pubkey: H256::from(karl_pub_key),
				}],
			};

			let karl_signature = sp_io::crypto::sr25519_sign(SR25519, &karl_pub_key, &transaction.encode()).unwrap();
			transaction.inputs[0].sigscript = H512::from(karl_signature);

			assert_noop!(Utxo::spend(Origin::signed(0), transaction), "missing inputs");
		});
	}

	#[test]
	fn attack_with_empty_transactions() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				Utxo::spend(Origin::signed(0), Transaction::default()), // an empty trx
				"no inputs"
			);

			assert_noop!(
				Utxo::spend(
					Origin::signed(0),
					Transaction {
						inputs: vec![TransactionInput::default()], // an empty trx
						outputs: vec![],
					}
				),
				"no outputs"
			);
		});
	}

	#[test]
	fn attack_by_double_counting_input() {
		new_test_ext().execute_with(|| {
			let alice_pub_key = sp_io::crypto::sr25519_public_keys(SR25519)[0];

			let mut transaction = Transaction {
				inputs: vec![
					TransactionInput {
						outpoint: H256::from(GENESIS_UTXO.clone()),
						sigscript: H512::zero(),
					},
					// A double spend of the same UTXO!
					TransactionInput {
						outpoint: H256::from(GENESIS_UTXO),
						sigscript: H512::zero(),
					},
				],
				outputs: vec![TransactionOutput {
					value: 100,
					pubkey: H256::from(alice_pub_key),
				}],
			};

			let alice_signature = sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode()).unwrap();
			transaction.inputs[0].sigscript = H512::from(alice_signature.clone());
			transaction.inputs[1].sigscript = H512::from(alice_signature);

			assert_noop!(
				Utxo::spend(Origin::signed(0), transaction),
				"each input must only be used once"
			);
		});
	}

	#[test]
	fn attack_by_double_generating_output() {
		new_test_ext().execute_with(|| {

			let alice_pub_key = sp_io::crypto::sr25519_public_keys(SR25519)[0];

			let mut transaction = Transaction {
				inputs: vec![TransactionInput {
					outpoint: H256::from(GENESIS_UTXO),
					sigscript: H512::zero(),
				}],
				outputs: vec![
					TransactionOutput {
						value: 100,
						pubkey: H256::from(alice_pub_key),
					},
					// Same output defined here!
					TransactionOutput {
						value: 100,
						pubkey: H256::from(alice_pub_key),
					},
				],
			};

			let alice_signature = sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode()).unwrap();
			transaction.inputs[0].sigscript = H512::from(alice_signature);

			assert_noop!(
				Utxo::spend(Origin::signed(0), transaction),
				"each output must be defined only once"
			);
		});
	}

	#[test]
	fn attack_with_invalid_signature() {
		new_test_ext().execute_with(|| {
			let alice_pub_key = sp_io::crypto::sr25519_public_keys(SR25519)[0];

			let transaction = Transaction {
				inputs: vec![TransactionInput {
					outpoint: H256::from(GENESIS_UTXO),
					// Just a random signature!
					sigscript: H512::random(),
				}],
				outputs: vec![TransactionOutput {
					value: 100,
					pubkey: H256::from(alice_pub_key),
				}],
			};

			assert_noop!(
				Utxo::spend(Origin::signed(0), transaction),
				"signature must be valid"
			);
		});
	}

	#[test]
	fn attack_by_permanently_sinking_outputs() {
		new_test_ext().execute_with(|| {
			let alice_pub_key = sp_io::crypto::sr25519_public_keys(SR25519)[0];

			let mut transaction = Transaction {
				inputs: vec![TransactionInput {
					outpoint: H256::from(GENESIS_UTXO),
					sigscript: H512::zero(),
				}],
				// A 0 value output burns this output forever!
				outputs: vec![TransactionOutput {
					value: 0,
					pubkey: H256::from(alice_pub_key),
				}],
			};

			let alice_signature = sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode()).unwrap();
			transaction.inputs[0].sigscript = H512::from(alice_signature);

			assert_noop!(
				Utxo::spend(Origin::signed(0), transaction),
				"output value must be nonzero"
			);
		});
	}

	#[test]
	fn attack_by_overflowing_value() {
		new_test_ext().execute_with(|| {
			let alice_pub_key = sp_io::crypto::sr25519_public_keys(SR25519)[0];

			let mut transaction = Transaction {
				inputs: vec![TransactionInput {
					outpoint: H256::from(GENESIS_UTXO),
					sigscript: H512::zero(),
				}],
				outputs: vec![
					TransactionOutput {
						value: Value::max_value(),
						pubkey:  H256::from(alice_pub_key),
					},
					// Attempts to do overflow total output value
					TransactionOutput {
						value: 10 as Value,
						pubkey: H256::from(alice_pub_key),
					},
				],
			};

			let alice_signature = sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode()).unwrap();
			transaction.inputs[0].sigscript = H512::from(alice_signature);

			assert_noop!(
				Utxo::spend(Origin::signed(0), transaction),
				"output value overflow"
			);
		});
	}

	#[test]
	fn attack_by_over_spending() {
		new_test_ext().execute_with(|| {
			let alice_pub_key = sp_io::crypto::sr25519_public_keys(SR25519)[0];

			let mut transaction = Transaction {
				inputs: vec![TransactionInput {
					outpoint: H256::from(GENESIS_UTXO),
					sigscript: H512::zero(),
				}],
				outputs: vec![
					TransactionOutput {
						value: 100 as Value,
						pubkey: H256::from(alice_pub_key),
					},
					// Creates 2 new utxo out of thin air!
					TransactionOutput {
						value: 2 as Value,
						pubkey: H256::from(alice_pub_key),
					},
				],
			};

			let alice_signature = sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode()).unwrap();
			transaction.inputs[0].sigscript = H512::from(alice_signature);

			assert_noop!(
				Utxo::spend(Origin::signed(0), transaction),
				"output value must not exceed input value"
			);
		});
	}
}
