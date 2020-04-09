use super::Aura;
use codec::{Decode, Encode};
use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::{DispatchResult, Vec},
    ensure,
};
use primitive_types::{H256, H512};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::sr25519::{Public, Signature};
use sp_runtime::traits::{BlakeTwo256, Hash, SaturatedConversion};
use sp_std::collections::btree_map::BTreeMap;

pub trait Trait: system::Trait {
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;
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

decl_storage! {
    trait Store for Module<T: Trait> as Utxo {
        /// All valid unspent transaction outputs are stored in this map.
        /// Initial set of UTXO is populated from the list stored in genesis.
        UtxoStore build(|config: &GenesisConfig| {
            config.genesis_utxo
                .iter()
                .cloned()
                .map(|u| (BlakeTwo256::hash_of(&u), u))
                .collect::<Vec<_>>()
        }): map H256 => Option<TransactionOutput>;

        /// Total reward value to be redistributed among authorities.
        /// It is accumulated from transactions during block execution
        /// and then dispersed to validators on block finalization.
        pub RewardTotal get(reward_total): Value;
    }

    add_extra_genesis {
        config(genesis_utxo): Vec<TransactionOutput>;
    }
}

// External functions: callable by the end user
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Dispatch a single transaction and update UTXO set accordingly
        pub fn spend(_origin, transaction: Transaction) -> DispatchResult {

            let reward = Self::check_transaction(&transaction)?;

            Self::update_storage(&transaction, reward)?;

            Self::deposit_event(Event::TransactionSuccess(transaction));

            Ok(())
        }

        /// Handler called by the system on block finalization
        fn on_finalize() {
            let auth:Vec<_> = Aura::authorities().iter().map(|x| {
                let r: &Public = x.as_ref();
                r.0.into()
            }).collect();
            Self::disperse_reward(&auth);
        }
    }
}

decl_event!(
    pub enum Event {
        /// Transaction was executed successfully
        TransactionSuccess(Transaction),
    }
);

// "Internal" functions, callable by code.
impl<T: Trait> Module<T> {

    /// Check transaction for validity.
    /// Returns: Remaining reward value for validators
    /// If any errors, runtime execution will halt
    pub fn check_transaction(transaction: &Transaction) -> Result<Value, &'static str> {
        ensure!(!_transaction.inputs.is_empty(), "no inputs");
        ensure!(!_transaction.outputs.is_empty(), "no outputs");

        { // nested scope so btreemaps resource gets freed
            let input_set: BTreeMap<_, ()> =_transaction.inputs.iter().map(|input| (input, ())).collect();
            ensure!(input_set.len() == _transaction.inputs.len(), "each input must only be used once");
        }
        {
            let output_set: BTreeMap<_, ()> = _transaction.outputs.iter().map(|output| (output, ())).collect();
            ensure!(output_set.len() == _transaction.outputs.len(), "each output must be defined only once");
        }

        let mut total_input: Value = 0;
        let mut total_output: Value = 0;
        let simple_transaction = Self::get_simple_transaction(_transaction);

        for input in _transaction.inputs.iter() {
            let utxo = <UtxoStore>::get(&input.outpoint).ok_or("missing input utxo")?;

            ensure!(sp_io::crypto::sr25519_verify(
                        &Signature::from_raw(*input.sigscript.as_fixed_bytes()),
                        &simple_transaction,
                        &Public::from_h256(utxo.pubkey)
                    ), "signature must be valid"
            );
            
            total_input = total_input.checked_add(utxo.value).ok_or("input value overflow")?;
        }

        let index: u64 = 0;
        for output in _transaction.outputs.iter() {
            ensure!(output.value != 0, "output value must be nonzero");
            
            let hash = BlakeTwo256::hash_of(&(&_transaction.encode(), index));// BlakeTwo256::hash_of(output);
            index.checked_add(1).ok_or("output index overflow")?;
            ensure!(!<UtxoStore>::exists(hash), "output already exists");
            
            total_output = total_output.checked_add(output.value).ok_or("output value overflow")?;
        }

        ensure!( total_input >= total_output, "output value must not exceed input value");

        Ok( total_input.checked_sub(total_output).ok_or("reward underflow")? )
    }
    
    /// Update storage to reflect changes made by transaction
    /// Where each utxo key is a hash of the entire transaction and its order in the TransactionOutputs vector
    fn update_storage(transaction: &Transaction, reward: Value) -> DispatchResult {
        // Calculate new reward total
        let new_total = <RewardTotal>::get()
            .checked_add(reward)
            .ok_or("Reward overflow")?;
        <RewardTotal>::put(new_total);

        // Storing updated reward value
        for input in &transaction.inputs {
            <UtxoStore>::remove(input.outpoint);
        }

        // Add new UTXO to be used by future transactions
        let index: u64 = 0;
        for output in &transaction.outputs {
            let hash = BlakeTwo256::hash_of(&(&transaction.encode(), index));
            <UtxoStore>::insert(hash, output);
            index.checked_add(1).ok_or("output index overflow")?;
        }

        Ok(())
    }

    /// Redistribute combined reward value evenly among chain authorities
    fn disperse_reward(authorities: &[H256]) {
        let reward = <RewardTotal>::take();
        let share_value: Value = reward
            .checked_div(authorities.len() as Value)
            .ok_or("No authorities")
            .unwrap();
        if share_value == 0 { return }

        let remainder = reward
            .checked_sub(share_value * authorities.len() as Value)
            .ok_or("Sub underflow")
            .unwrap();
        <RewardTotal>::put(remainder as Value);

        for authority in authorities {
            let utxo = TransactionOutput {
                value: share_value,
                pubkey: *authority,
            };

            let hash = BlakeTwo256::hash_of(&(&utxo, 
                        <system::Module<T>>::block_number().saturated_into::<u64>()));

            if !<UtxoStore>::exists(hash) {
                <UtxoStore>::insert(hash, utxo);
                sp_runtime::print("transaction reward sent to");
                sp_runtime::print(hash.as_fixed_bytes() as &[u8]);
            } else {
                sp_runtime::print("transaction reward wasted due to hash collision");
            }
        }
    }
        
    /// Strips a transaction of its signatures by replacing the sigscript field of each input with ZERO-initialized fixed hash.
    pub fn get_simple_transaction(_transaction: &Transaction) -> Vec<u8> {//&'a [u8] {
        let mut trx = _transaction.clone();
        for input in trx.inputs.iter_mut() {
            input.sigscript = H512::zero();
        }

        trx.encode()
    }

    /// Helper fn for Transaction Pool
    /// Checks for race condition, if a certain trx is missing input_utxos in UtxoStore
    /// If None missing inputs: no race condition, gtg
    /// if Some(missing inputs): there are missing variables
    pub fn has_race_condition(_transaction: &Transaction) -> Option<Vec<&H256>> {
        let mut missing_utxo = Vec::new();
        for input in _transaction.inputs.iter() {
            if <UtxoStore>::get(&input.outpoint).is_none() {
                missing_utxo.push(&input.outpoint);
            }
        }
        if ! missing_utxo.is_empty() { return Some(missing_utxo) };
        None
    }
}

/// Tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use frame_support::{assert_ok, assert_err, impl_outer_origin, parameter_types, weights::Weight};
    use primitive_types::H256;
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
    impl system::Trait for Test {
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
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type ModuleToIndex = ();
    }
    impl Trait for Test {
        type Event = ();
    }

    type Utxo = Module<Test>;

    // need to manually import this crate since its no include by default
    use hex_literal::hex;

    const ALICE_PHRASE: &str = "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
    const GENESIS_UTXO: [u8; 32] = hex!("79eabcbd5ef6e958c6a7851b36da07691c19bda1835a08f875aa286911800999");
    
    // This function basically just builds a genesis storage key/value store according to our desired mockup.
    // We start each test by giving Alice 100 utxo to start with.
    fn new_test_ext() -> sp_io::TestExternalities {
    
        let keystore = KeyStore::new(); // a key storage to store new key pairs during testing
        let alice_pub_key = keystore.write().sr25519_generate_new(SR25519, Some(ALICE_PHRASE)).unwrap();

        let mut t = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        t.top.extend(
            GenesisConfig {
                genesis_utxo: vec![
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
            assert!(!UtxoStore::exists(H256::from(GENESIS_UTXO)));
            assert!(UtxoStore::exists(new_utxo_hash));
            assert_eq!(50, UtxoStore::get(new_utxo_hash).unwrap().value);
        });
    }

    #[test]
    fn test_race_condition() {
        new_test_ext().execute_with(|| {
            let alice_pub_key = sp_io::crypto::sr25519_public_keys(SR25519)[0];
            let nonexistent_utxo = H256::random();

            let mut transaction = Transaction {
                inputs: vec![TransactionInput {
                    outpoint: nonexistent_utxo,
                    sigscript: H512::zero(),
                }],
                outputs: vec![TransactionOutput {
                    value: 50,
                    pubkey: H256::from(alice_pub_key),
                }],
            };

            let alice_signature = sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode()).unwrap();
            transaction.inputs[0].sigscript = H512::from(alice_signature);
            let missing_utxo_hash = Utxo::has_race_condition(&transaction).unwrap()[0];
            
            assert_eq!(missing_utxo_hash.as_fixed_bytes(), nonexistent_utxo.as_fixed_bytes());
        });
    }

    // Exercise 1: Fortify transactions against attacks
    // ================================================
    // The following tests simulate malicious UTXO transactions
    // Implement the check_transaction() function to thwart such attacks

    #[test]
    fn attack_with_empty_transactions() {
        new_test_ext().execute_with(|| {
            assert_err!(
                Utxo::spend(Origin::signed(0), Transaction::default()), // an empty trx
                "no inputs"
            );

            assert_err!(
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

            assert_err!(
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

            assert_err!(
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

            assert_err!(
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

            assert_err!(
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

            assert_err!(
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

            assert_err!(
                Utxo::spend(Origin::signed(0), transaction),
                "output value must not exceed input value"
            );
        });
    }
}
