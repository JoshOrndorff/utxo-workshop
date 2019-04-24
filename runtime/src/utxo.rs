// Original Author: @0x7CFE
use support::{
    decl_event, decl_module, decl_storage,
    dispatch::{Result, Vec},
    ensure, StorageMap, StorageValue,
};
use primitives::{H256, H512};
use rstd::collections::btree_map::BTreeMap;
use runtime_primitives::traits::{As, BlakeTwo256, Hash};
use system::{ensure_inherent, ensure_signed};
use super::Consensus;
use parity_codec::{Decode, Encode};
use runtime_io::sr25519_verify;
#[cfg(feature = "std")]
use serde_derive::{Deserialize, Serialize};

pub trait Trait: system::Trait {
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;
}

/// Representation of UTXO value
pub type Value = u128;

/// Representation of UTXO value
type Signature = H512;

/// Single transaction to be dispatched
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash)]
pub struct Transaction {
    /// UTXOs to be used as inputs for current transaction
    pub inputs: Vec<TransactionInput>,
    
    /// UTXOs to be created as a result of current transaction dispatch
    pub outputs: Vec<TransactionOutput>,
}

/// Single transaction input that refers to one UTXO
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash)]
pub struct TransactionInput {
    /// Reference to an UTXO to be spent
    pub parent_output: H256,
    
    /// Proof that transaction owner is authorized to spend referred UTXO
    pub signature: Signature,
}

/// Single transaction output to create upon transaction dispatch
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash)]
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

/// A UTXO can be locked indefinitely or until a certain block height
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash)]
pub enum LockStatus<BlockNumber> {
    Locked,
    LockedUntil(BlockNumber),
}

decl_storage! {
    trait Store for Module<T: Trait> as Utxo {
        /// All valid unspent transaction outputs are stored in this map.
        /// Initial set of UTXO is populated from the list stored in genesis.
        UnspentOutputs build(|config: &GenesisConfig<T>| {
            config.initial_utxo
                .iter()
                .cloned()
                .map(|u| (BlakeTwo256::hash_of(&u), u))
                .collect::<Vec<_>>()
        }): map H256 => Option<TransactionOutput>;


        /// Total leftover value to be redistributed among authorities.
        /// It is accumulated during block execution and then drained
        /// on block finalization.
        pub LeftoverTotal get(leftover_total): Value;

        /// All UTXO that are locked
        LockedOutputs: map H256 => Option<LockStatus<T::BlockNumber>>;
    }

    add_extra_genesis {
        config(initial_utxo): Vec<TransactionOutput>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Dispatch a single transaction and update UTXO set accordingly
        pub fn execute(origin, transaction: Transaction) -> Result {
            ensure_inherent(origin)?;

            // Verify the transaction
            let leftover = match Self::check_transaction(&transaction)? {
                CheckInfo::Totals{input, output} => input - output,
                CheckInfo::MissingInputs(_) => return Err("Invalid transaction inputs")
            };

            // Update unspent outputs
            Self::update_storage(&transaction, leftover)?;

            // Emit event
            Self::deposit_event(Event::TransactionExecuted(transaction));

            Ok(())
        }

        /// DANGEROUS! Adds specified output to the storage potentially overwriting existing one.
        /// Does not perform enough checks. Must only be used for testing purposes.
        pub fn mint(origin, value: Value, pubkey: H256) -> Result {
            ensure_signed(origin)?;
            let salt:u64 = <system::Module<T>>::block_number().as_();
            let utxo = TransactionOutput { value, pubkey, salt };
            let hash = BlakeTwo256::hash_of(&utxo);

            if !<UnspentOutputs<T>>::exists(hash) {
                <UnspentOutputs<T>>::insert(hash, utxo);
            } else {
                runtime_io::print("cannot mint due to hash collision");
            }

            Ok(())
        }

        /// Handler called by the system on block finalization
        fn on_finalize() {
            let auth:Vec<_> = Consensus::authorities().iter().map(|x| x.0.into() ).collect();
            Self::spend_leftover(&auth);
        }
    }
}

decl_event!(
    pub enum Event {
        /// Transaction was executed successfully
        TransactionExecuted(Transaction),
    }
);

/// Information collected during transaction verification
pub enum CheckInfo<'a> {
    /// Combined value of all inputs and outputs
    Totals { input: Value, output: Value },

    /// Some referred UTXOs were missing
    MissingInputs(Vec<&'a H256>),
}

/// Result of transaction verification
pub type CheckResult<'a> = rstd::result::Result<CheckInfo<'a>, &'static str>;

impl<T: Trait> Module<T> {
    /// Check transaction for validity.
    pub fn check_transaction(transaction: &Transaction) -> CheckResult<'_> {
        
        // TODO

        Ok(CheckInfo::Totals { input: 0, output: 0 })
        
    }
	
    /// Redistribute combined leftover value evenly among chain authorities
    fn spend_leftover(authorities: &[H256]) {
        let leftover = <LeftoverTotal<T>>::take();
        let share_value: Value = leftover
            .checked_div(authorities.len() as Value)
            .ok_or("No authorities")
            .unwrap();
        if share_value == 0 { return }

        let remainder = leftover
            .checked_sub(share_value * authorities.len() as Value)
            .ok_or("Sub underflow")
            .unwrap();
        <LeftoverTotal<T>>::put(remainder as Value);

        for authority in authorities {
            let utxo = TransactionOutput {
                value: share_value,
                pubkey: *authority,
                salt: <system::Module<T>>::block_number().as_(),
            };

            let hash = BlakeTwo256::hash_of(&utxo);

            if !<UnspentOutputs<T>>::exists(hash) {
                <UnspentOutputs<T>>::insert(hash, utxo);
                runtime_io::print("leftover share sent to");
                runtime_io::print(hash.as_fixed_bytes() as &[u8]);
            } else {
                runtime_io::print("leftover share wasted due to hash collision");
            }
        }
    }

    /// Update storage to reflect changes made by transaction
    fn update_storage(transaction: &Transaction, leftover: Value) -> Result {
        // Calculate new leftover total
        let new_total = <LeftoverTotal<T>>::get()
            .checked_add(leftover)
            .ok_or("Leftover overflow")?;
        <LeftoverTotal<T>>::put(new_total);

        // Storing updated leftover value
        for input in &transaction.inputs {
            <UnspentOutputs<T>>::remove(input.parent_output);
        }

        // Add new UTXO to be used by future transactions
        for output in &transaction.outputs {
            let hash = BlakeTwo256::hash_of(output);
            <UnspentOutputs<T>>::insert(hash, output);
        }

        Ok(())
    }

    pub fn lock_utxo(hash: &H256, until: Option<T::BlockNumber>) -> Result {
        ensure!(!<LockedOutputs<T>>::exists(hash), "utxo is already locked");
        ensure!(<UnspentOutputs<T>>::exists(hash), "utxo does not exist");

        if let Some(until) = until {
            ensure!(
                until > <system::Module<T>>::block_number(),
                "block number is in the past"
            );
            <LockedOutputs<T>>::insert(hash, LockStatus::LockedUntil(until));
        } else {
            <LockedOutputs<T>>::insert(hash, LockStatus::Locked);
        }

        Ok(())
    }

    pub fn unlock_utxo(hash: &H256) -> Result {
        ensure!(!<LockedOutputs<T>>::exists(hash), "utxo is not locked");
        <LockedOutputs<T>>::remove(hash);
        Ok(())
    }
}

/// Tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use support::{assert_err, assert_ok, impl_outer_origin};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type Log = DigestItem;
    }
    impl Trait for Test {
        type Event = ();
    }

    type Utxo = Module<Test>;

    // Test set up
    // Alice's Public Key: Pair::from_seed(*b"12345678901234567890123456789012");
    const ALICE_KEY: [u8; 32] = [68, 169, 150, 190, 177, 238, 247, 189, 202, 185, 118, 171, 109, 44, 162, 97, 4, 131, 65, 100, 236, 242, 143, 179, 117, 96, 5, 118, 252, 198, 235, 15];

    // Alice's Signature to spend alice_utxo(): signs a token she owns Pair::sign(&message[..])
    const ALICE_SIG: [u8; 64] = [220, 109, 218, 80, 85, 118, 140, 48, 193, 19, 77, 200, 60, 229, 91, 60, 70, 54, 54, 137, 154, 51, 201, 252, 98, 219, 172, 57, 1, 139, 86, 47, 162, 21, 50, 179, 196, 135, 167, 29, 171, 85, 3, 111, 46, 110, 10, 25, 239, 152, 176, 82, 114, 192, 125, 182, 240, 19, 192, 85, 227, 101, 148, 0]; //[148, 250, 180, 5, 112, 29, 240, 241, 122, 26, 249, 125, 87, 102, 180, 179, 127, 79, 120, 72, 253, 21, 26, 215, 157, 35, 208, 126, 54, 181, 150, 12, 117, 177, 134, 104, 124, 16, 70, 249, 31, 4, 131, 192, 247, 143, 73, 123, 24, 66, 144, 189, 64, 90, 65, 79, 185, 36, 107, 135, 195, 212, 219, 10];

    // Alice's Signature to spend alice_utxo_100(): signs a token she owns Pair::sign(&message[..])
    const ALICE_SIG100: [u8; 64] = [212, 108, 199, 137, 228, 149, 233, 230, 129, 251, 80, 16, 160, 95, 191, 199, 207, 176, 151, 234, 5, 157, 245, 136, 62, 169, 87, 203, 188, 11, 47, 76, 230, 159, 10, 125, 35, 244, 76, 89, 174, 52, 41, 78, 32, 102, 200, 231, 31, 22, 35, 42, 143, 85, 255, 235, 31, 58, 236, 95, 52, 205, 224, 2]; // [228, 33, 239, 151, 136, 93, 241, 82, 205, 248, 154, 139, 52, 157, 231, 222, 66, 242, 86, 120, 92, 170, 98, 214, 78, 226, 93, 229, 130, 174, 168, 26, 7, 151, 88, 13, 185, 161, 15, 247, 222, 85, 235, 107, 246, 135, 23, 47, 162, 71, 81, 29, 227, 230, 210, 112, 0, 157, 86, 218, 130, 11, 8, 0];

    // Creates a max value UTXO for Alice
    fn alice_utxo() -> (H256, TransactionOutput) {
        let transaction = TransactionOutput {
            value: Value::max_value(),
            pubkey: H256::from_slice(&ALICE_KEY),
            salt: 0,
        };

        (BlakeTwo256::hash_of(&transaction), transaction)
    }

    // Creates a 100 value UTXO for Alice
    fn alice_utxo_100() -> (H256, TransactionOutput) {
        let transaction = TransactionOutput {
            value: 100,
            pubkey: H256::from_slice(&ALICE_KEY),
            salt: 0,
        };

        (BlakeTwo256::hash_of(&transaction), transaction)
    }

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .0;
        t.extend(
            GenesisConfig::<Test> {
                initial_utxo: vec![alice_utxo().1, alice_utxo_100().1],
                ..Default::default()
            }
            .build_storage()
            .unwrap()
            .0,
        );
        t.into()
    }

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
        with_externalities(&mut new_test_ext(), || {
            assert_err!(
                Utxo::execute(Origin::INHERENT, Transaction::default()), // an empty trx
                "no inputs"
            );

            assert_err!(
                Utxo::execute(
                    Origin::INHERENT,
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
        with_externalities(&mut new_test_ext(), || {
            let (parent_hash, _) = alice_utxo();

            println!("PARENT HASH: {:x?}: ", parent_hash);
            let transaction = Transaction {
                inputs: vec![
                    TransactionInput {
                        parent_output: parent_hash,
                        signature: Signature::from_slice(&ALICE_SIG),
                    },
                    TransactionInput {
                        parent_output: parent_hash, // Double spending input!
                        signature: Signature::from_slice(&ALICE_SIG),
                    },
                ],
                outputs: vec![TransactionOutput {
                    value: 100,
                    pubkey: H256::from_slice(&ALICE_KEY),
                    salt: 0,
                }],
            };

            assert_err!(
                Utxo::execute(Origin::INHERENT, transaction),
                "each input must only be used once"
            );
        });
    }

    #[test]
    fn attack_by_double_generating_output() {
        with_externalities(&mut new_test_ext(), || {
            let (parent_hash, _) = alice_utxo();

            let transaction = Transaction {
                inputs: vec![TransactionInput {
                    parent_output: parent_hash,
                    signature: Signature::from_slice(&ALICE_SIG),
                }],
                outputs: vec![
                    TransactionOutput {
                        value: 100,
                        pubkey: H256::from_slice(&ALICE_KEY),
                        salt: 0,
                    },
                    TransactionOutput {
                        // Same output defined here!
                        value: 100,
                        pubkey: H256::from_slice(&ALICE_KEY),
                        salt: 0,
                    },
                ],
            };

            assert_err!(
                Utxo::execute(Origin::INHERENT, transaction),
                "each output must be defined only once"
            );
        });
    }

    #[test]
    fn attack_with_invalid_signature() {
        with_externalities(&mut new_test_ext(), || {
            let (parent_hash, _) = alice_utxo();

            let transaction = Transaction {
                inputs: vec![TransactionInput {
                    parent_output: parent_hash,
                    signature: H512::random(), // Just a random signature!
                }],
                outputs: vec![TransactionOutput {
                    value: 100,
                    pubkey: H256::from_slice(&ALICE_KEY),
                    salt: 0,
                }],
            };

            assert_err!(
                Utxo::execute(Origin::INHERENT, transaction),
                "signature must be valid"
            );
        });
    }

    #[test]
    fn attack_by_permanently_sinking_outputs() {
        with_externalities(&mut new_test_ext(), || {
            let (parent_hash, _) = alice_utxo();

            let transaction = Transaction {
                inputs: vec![TransactionInput {
                    parent_output: parent_hash,
                    signature: Signature::from_slice(&ALICE_SIG),
                }],
                outputs: vec![TransactionOutput {
                    value: 0, // A 0 value output burns this output forever!
                    pubkey: H256::from_slice(&ALICE_KEY),
                    salt: 0,
                }],
            };

            assert_err!(
                Utxo::execute(Origin::INHERENT, transaction),
                "output value must be nonzero"
            );
        });
    }

    #[test]
    fn attack_by_overflowing() {
        with_externalities(&mut new_test_ext(), || {
            let (parent_hash, _) = alice_utxo();

            let transaction = Transaction {
                inputs: vec![TransactionInput {
                    parent_output: parent_hash,
                    signature: Signature::from_slice(&ALICE_SIG),
                }],
                outputs: vec![
                    TransactionOutput {
                        value: Value::max_value(),
                        pubkey: H256::from_slice(&ALICE_KEY),
                        salt: 1,
                    },
                    TransactionOutput {
                        value: 10 as Value, // Attempts to do overflow total output value
                        pubkey: H256::from_slice(&ALICE_KEY),
                        salt: 1,
                    },
                ],
            };

            assert_err!(
                Utxo::execute(Origin::INHERENT, transaction),
                "output value overflow"
            );
        });
    }

    #[test]
    fn attack_by_over_spending() {
        with_externalities(&mut new_test_ext(), || {
            let (parent_hash, _) = alice_utxo_100();

            let transaction = Transaction {
                inputs: vec![TransactionInput {
                    parent_output: parent_hash,
                    signature: Signature::from_slice(&ALICE_SIG100),
                }],
                outputs: vec![
                    TransactionOutput {
                        value: 100 as Value,
                        pubkey: H256::from_slice(&ALICE_KEY),
                        salt: 1,
                    },
                    TransactionOutput {
                        value: 1 as Value, // Creates 1 new utxo out of thin air!
                        pubkey: H256::from_slice(&ALICE_KEY),
                        salt: 1,
                    },
                ],
            };

            assert_err!(
                Utxo::execute(Origin::INHERENT, transaction),
                "output value must not exceed input value"
            );
        });
    }
    
    #[test]
    fn valid_transaction() {
        with_externalities(&mut new_test_ext(), || {
            let (parent_hash, _) = alice_utxo();

            let transaction = Transaction {
                inputs: vec![TransactionInput {
                    parent_output: parent_hash,
                    signature: Signature::from_slice(&ALICE_SIG),
                }],
                outputs: vec![TransactionOutput {
                    value: 100,
                    pubkey: H256::from_slice(&ALICE_KEY),
                    salt: 2,
                }],
            };
            
            let output_hash = BlakeTwo256::hash_of(&transaction.outputs[0]);

            assert_ok!(Utxo::execute(Origin::INHERENT, transaction));
            assert!(!<UnspentOutputs<Test>>::exists(parent_hash));
            assert!(<UnspentOutputs<Test>>::exists(output_hash));
        });
    }
}
