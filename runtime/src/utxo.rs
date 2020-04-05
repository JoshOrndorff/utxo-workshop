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
use system::ensure_signed;

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
	/// owner must provide a proof by hashing whole `TransactionOutput` and
	/// signing it with a corresponding private key.
    pub pubkey: H256,

    /// Unique (potentially random) value used to distinguish this
	/// particular output from others addressed to the same public
	/// key with the same value. Prevents potential replay attacks.
    pub salt: u64,
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
        pub fn execute(origin, transaction: Transaction) -> DispatchResult {
            ensure_signed(origin)?; //TODO remove this check.

            let reward = Self::check_transaction(&transaction)?;

            Self::update_storage(&transaction, reward)?;

            Self::deposit_event(Event::TransactionExecuted(transaction));

            Ok(())
        }

        /// DANGEROUS! Adds specified output to the storage potentially overwriting existing one.
        /// Only be used for testing & demo purposes.
        pub fn mint(origin, value: Value, pubkey: H256) -> DispatchResult {
            ensure_signed(origin)?;
            let salt:u64 = <system::Module<T>>::block_number().saturated_into::<u64>();
            let utxo = TransactionOutput { value, pubkey, salt };
            let hash = BlakeTwo256::hash_of(&utxo);

            if !<UtxoStore>::exists(hash) {
                <UtxoStore>::insert(hash, utxo);
            } else {
                sp_runtime::print("cannot mint due to hash collision");
            }

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
        TransactionExecuted(Transaction),
    }
);

// "Internal" functions, callable by code.
impl<T: Trait> Module<T> {
    
    // Strips a transaction of its Signature fields by replacing value with ZERO-initialized fixed hash.
    pub fn get_simple_transaction(_transaction: &Transaction) -> Vec<u8> {//&'a [u8] {
        let mut trx = _transaction.clone();
        for input in trx.inputs.iter_mut() {
            input.sigscript = H512::zero();
        }
        trx.encode()
    }

    /// Check transaction for validity.
    /// Returns: Dust value if everything is ok
    /// If any errors, runtime execution will auto stop!
    pub fn check_transaction(_transaction: &Transaction) -> Result<Value, &'static str> {
        ensure!(!_transaction.inputs.is_empty(), "no inputs");
        ensure!(!_transaction.outputs.is_empty(), "no outputs");

        { //TODO check if can take out of fn scope, likely not...
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
            
            // Check that each input-utxo sigscript is
            // 1. Verfied to be the same key as the utxo's pubKeyScript in UtxoStore
            // 2. Untampered transaction fields
            ensure!(sp_io::crypto::sr25519_verify(
                        &Signature::from_raw(*input.sigscript.as_fixed_bytes()),
                        &simple_transaction,//input.outpoint.as_fixed_bytes(), //fixed bytes
                        &Public::from_h256(utxo.pubkey)
                    ), "signature must be valid"
            );
            // Add the value to the input total
            total_input = total_input.checked_add(utxo.value).ok_or("input value overflow")?;
        }

        for output in _transaction.outputs.iter() {
            ensure!(output.value != 0, "output value must be nonzero");
            let hash = BlakeTwo256::hash_of(output);
            ensure!(!<UtxoStore>::exists(hash), "output already exists");
            total_output = total_output.checked_add(output.value).ok_or("output value overflow")?;
        }

        ensure!( total_input >= total_output, "output value must not exceed input value");

        Ok( total_input - total_output )  // TODO: check_substract here just to be safe
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
                salt: <system::Module<T>>::block_number().saturated_into::<u64>(),
            };

            let hash = BlakeTwo256::hash_of(&utxo);

            if !<UtxoStore>::exists(hash) {
                <UtxoStore>::insert(hash, utxo);
                sp_runtime::print("transaction reward sent to");
                sp_runtime::print(hash.as_fixed_bytes() as &[u8]);
            } else {
                sp_runtime::print("transaction reward wasted due to hash collision");
            }
        }
    }

    /// Update storage to reflect changes made by transaction
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
        for output in &transaction.outputs {
            let hash = BlakeTwo256::hash_of(output);
            <UtxoStore>::insert(hash, output);
        }

        Ok(())
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

    use frame_support::{assert_ok, impl_outer_origin, parameter_types, weights::Weight};
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

    // Helper function: creates a utxo
    fn create_utxo(value: Value, pubkey: Public) -> (H256, TransactionOutput) {

        let transaction = TransactionOutput {
            value,
            pubkey: H256::from(pubkey), //H256::from_slice(&ALICE_KEY),
            salt: 0,
        };
        
        (BlakeTwo256::hash_of(&transaction), transaction)
    }

    // need to manually import this crate since its no include by default
    use hex_literal::hex;

    const ALICE_PHRASE: &str = "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
    const BOB_PHRASE: &str = "lobster flock few equip connect boost excuse glass machine find wonder tattoo";
    const GENESIS_UTXO: [u8; 32] = hex!("0a746d36b8357640690608da229648a51552b0add7ddbd8803efa1013cadbd4c");
    
    // This function basically just builds a genesis storage key/value store according to our desired mockup.
    // We start each test by giving Alice 100 utxo to start with.
    fn new_test_ext() -> sp_io::TestExternalities {
    
        let keystore = KeyStore::new(); // a key storage to store new key pairs during testing
        let alice_pub_key = keystore.write().sr25519_generate_new(SR25519, Some(ALICE_PHRASE)).unwrap();
        let _bob_pub_key = keystore.write().sr25519_generate_new(SR25519, Some(BOB_PHRASE)).unwrap();

        let mut t = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        t.top.extend(
            GenesisConfig {
                genesis_utxo: vec![create_utxo(100, alice_pub_key).1],
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
            let bob_pub_key = sp_io::crypto::sr25519_public_keys(SR25519)[1];

            // Alice wants to send Bob a utxo of value 50.
            let mut transaction = Transaction {
                inputs: vec![TransactionInput {
                    outpoint: H256::from(GENESIS_UTXO),
                    sigscript: H512::zero(),
                }],
                outputs: vec![TransactionOutput {
                    value: 50,
                    pubkey: H256::from(bob_pub_key),
                    salt: 1,
                }],
            };
            
            // TODO: this test randomly fails about 1/3 runs, with the error "Signature must be valid"
            // Figure out what randomness exists in the following line that causes this.
            let alice_signature = sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode()).unwrap();
            transaction.inputs[0].sigscript = H512::from(alice_signature);
            let transaction_hash = BlakeTwo256::hash_of(&transaction.outputs[0]);
            assert_ok!(Utxo::execute(Origin::signed(0), transaction));
            
            // Check that Bob indeed owns utxo of value 50
            assert!(!UtxoStore::exists(H256::from(GENESIS_UTXO)));
            assert!(UtxoStore::exists(transaction_hash));
            assert_eq!(H256::from(bob_pub_key), UtxoStore::get(transaction_hash).unwrap().pubkey);
            assert_eq!(50, UtxoStore::get(transaction_hash).unwrap().value);
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
                    salt: 1,
                }],
            };

            let alice_signature = sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode()).unwrap();
            transaction.inputs[0].sigscript = H512::from(alice_signature);

            let missing_utxo_hash = Utxo::has_race_condition(&transaction).unwrap()[0];
            assert_eq!(missing_utxo_hash.as_fixed_bytes(), nonexistent_utxo.as_fixed_bytes());
        });
    }

    // expected `&sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::sp_application_crypto::sp_core::H256`, 
    // found struct `sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::sp_application_crypto::sp_core::H256`


    // Exercise 1: Fortify transactions against attacks
    // ================================================
    //
    // The following tests simulate malicious UTXO transactions
    // Implement the check_transaction() function to thwart such attacks
    //
    // Hint: Examine types CheckResult, CheckInfo for the expected behaviors of this function
    // Hint: Make this function public, as it will be later used outside of this module

    #[test]
    fn attack_with_empty_transactions() {
        new_test_ext().execute_with(|| {
            assert!(
                Utxo::execute(Origin::signed(0), Transaction::default()).is_err(), // an empty trx
                "no inputs"
            );

            assert!(
                Utxo::execute(
                    Origin::signed(0),
                    Transaction {
                        inputs: vec![TransactionInput::default()], // an empty trx
                        outputs: vec![],
                    }
                )
                .is_err(),
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
                    salt: 0,
                }],
            };

            let alice_signature = sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode()).unwrap();
            transaction.inputs[0].sigscript = H512::from(alice_signature.clone());
            transaction.inputs[1].sigscript = H512::from(alice_signature);

            assert!(
                Utxo::execute(Origin::signed(0), transaction).is_err(),
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
                        salt: 0,
                    },
                    TransactionOutput {
                        // Same output defined here!
                        value: 100,
                        pubkey: H256::from(alice_pub_key),
                        salt: 0, // TODO check this
                    },
                ],
            };

            let alice_signature = sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode()).unwrap();
            transaction.inputs[0].sigscript = H512::from(alice_signature);

            assert!(
                Utxo::execute(Origin::signed(0), transaction).is_err(),
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
                    salt: 0,
                }],
            };

            assert!(
                Utxo::execute(Origin::signed(0), transaction).is_err(),
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
                outputs: vec![TransactionOutput {
                    value: 0, // A 0 value output burns this output forever!
                    pubkey: H256::from(alice_pub_key),
                    salt: 0,
                }],
            };

            let alice_signature = sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode()).unwrap();
            transaction.inputs[0].sigscript = H512::from(alice_signature);

            assert!(
                Utxo::execute(Origin::signed(0), transaction).is_err(),
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
                        salt: 1,
                    },
                    TransactionOutput {
                        value: 10 as Value, // Attempts to do overflow total output value
                        pubkey: H256::from(alice_pub_key),
                        salt: 1,
                    },
                ],
            };

            let alice_signature = sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode()).unwrap();
            transaction.inputs[0].sigscript = H512::from(alice_signature);

            assert!(
                Utxo::execute(Origin::signed(0), transaction).is_err(),
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
                        salt: 1,
                    },
                    TransactionOutput {
                        value: 1 as Value, // Creates 1 new utxo out of thin air!
                        pubkey: H256::from(alice_pub_key),
                        salt: 1,
                    },
                ],
            };

            let alice_signature = sp_io::crypto::sr25519_sign(SR25519, &alice_pub_key, &transaction.encode()).unwrap();
            transaction.inputs[0].sigscript = H512::from(alice_signature);

            assert!(
                Utxo::execute(Origin::signed(0), transaction).is_err(),
                "output value must not exceed input value"
            );
        });
    }
}
