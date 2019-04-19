/// A reimplementation of Dima's UTXO chain
use support::{
    decl_module, 
    decl_storage, 
    decl_event, 
    StorageValue,
    StorageMap,
    ensure,
    dispatch::Result
};

use system::ensure_inherent;
use primitives::{H256, H512};
use rstd::collections::btree_map::BTreeMap;
// use primitives::ed25519::{Public, Pair};
use runtime_primitives::traits::{As, Hash, BlakeTwo256};
use runtime_primitives::{Serialize, Deserialize}; //update
use runtime_io::{ed25519_verify};
// use serde::{de, Serializer, Deserializer}; //not sure about this
use parity_codec::{Codec, Encode, Decode}; //update

pub trait Trait: system::Trait {
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash)]
pub struct Transaction {
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>
}

type Signature = H512; 

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash)]
pub struct TransactionInput {
    // Referen  ce to the input value
    pub parent_output: H256,  // referred UTXO
    pub signature: Signature, // proof that owner is authorized to spend referred UTXO
    // omitted traits ord, partialord bc its not implemented for signature yet
}

pub type Value = u128; // Alias u128 to Value

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash)]
pub struct TransactionOutput {
    pub value: Value,
    pub pubkey: H256, // pub key of the output, owner has to have private key
    pub salt: u64,    // distinguishes outputs of same value/pubkey apart
}

decl_storage! {
	trait Store for Module<T: Trait> as Utxo {

        // pub UnspentOutputs get(unspent_outputs): map H256 => Option<TransactionOutput>;

        UnspentOutputs build(|config: &GenesisConfig<T>| {
			config.initial_utxo
				.iter()
				.cloned()
				.map(|u| (BlakeTwo256::hash_of(&u), u))
				.collect::<Vec<_>>()
		}): map H256 => Option<TransactionOutput>;

        pub LeftOverTotal get(leftover_total): Value;

        // TODO lockedoutputs
	}

    add_extra_genesis {
		config(initial_utxo): Vec<TransactionOutput>;
	}

}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        // custom function for minting tokens (instead of doing in genesis config)
        fn mint(origin, value: Value, pubkey: H256) -> Result {
            let sender = ensure_inherent(origin)?;
            let salt:u64 = <system::Module<T>>::block_number().as_();
            let trx = TransactionOutput { value, pubkey, salt };
            let hash = BlakeTwo256::hash_of(&trx); 
            <UnspentOutputs<T>>::insert(hash, trx);
            
            Ok(())
        }

        fn execute(origin, transaction: Transaction) {
            let sender = ensure_inherent(origin)?;

            // Verify the transaction
            Self::_verify_transaction(&transaction);

            // Update unspent outputs

        }

        // TODO delete this
        pub fn do_something(something: u32) -> Result {
			Self::deposit_event(Event::SomethingStored(something));
			Ok(())
	    }
	}
}

decl_event!(
	pub enum Event {
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit funtion, from our runtime funtions
		SomethingStored(u32),
	}
);
// nice coding pattern, everytime you return a value, 1. wrap enum in resultType 2. use enum to represent different outcomes

pub enum CheckInfo<'a> {
    Totals { input: Value, output: Value },   // struct
    MissingInputs(Vec<&'a H256>),     //Q: why is there a lifetime/reference here?
}

pub type CheckResult<'a> = std::result::Result<CheckInfo<'a>, &'static str>; // errors are already defined

impl<T: Trait> Module<T> {
    /// Verifies the transaction validity, returns the outcome
    fn _verify_transaction(transaction: &Transaction) -> CheckResult<'_> {
        // 1. Verify that inputs and outputs are not empty
        ensure!(transaction.inputs.is_empty(), "no inputs");
        ensure!(transaction.outputs.is_empty(), "no outputs");

        {
            let input_set: BTreeMap<_, ()> = transaction
                .inputs
                .iter()
                .map(|input| (input, ()))
                .collect();

            ensure!(
                input_set.len() == transaction.inputs.len(),
                "each input must only be used once"
            );
        }

        {
            let output_set: BTreeMap<_, ()> = transaction
                .outputs
                .iter()
                .map(|output| (output, ()))
                .collect();

            ensure!(
                output_set.len() == transaction.outputs.len(),
                "each output must be defined only once"
            );
        }

        let mut total_input: Value = 0;
        let mut missing_utxo = Vec::new();
        for input in transaction.inputs.iter() {
            if let Some(output) = <UnspentOutputs<T>>::get(&input.parent_output) {
                // ensure!(!<lockedoutputs<T>>::exists(&input.parent_output), "utxo is locked");

                // Check uxto authorization
                ensure!(
                    ed25519_verify(
                        input.signature.as_fixed_bytes(), // impl s.t. returns [u8; 64]
                        input.parent_output.as_fixed_bytes(),
                        &output.pubkey
                    ),
                    "signature must be valid"
                );

                // Add the value to the input total
                total_input = total_input.checked_add(output.value).ok_or("input value overflow")?;
            } else {
                missing_utxo.push(&input.parent_output);
            }
        }

        let mut total_output: Value = 0;
        for output in transaction.outputs.iter() {
            ensure!(output.value != 0, "output value must be nonzero");

            let hash = BlakeTwo256::hash_of(output);
            ensure!(!<UnspentOutputs<T>>::exists(hash), "output already exists");

            total_output = total_output.checked_add(output.value).ok_or("output value overflow")?;
        }

        if missing_utxo.is_empty() {
            ensure!(total_input >= total_output, "output value must not exceed input value");
            Ok(CheckInfo::Totals { input: total_input, output: total_input })
        } else {
            Ok(CheckInfo::MissingInputs(missing_utxo))
        }
    }

}

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok};
	use runtime_primitives::{
		BuildStorage,
		traits::{BlakeTwo256, IdentityLookup},
		testing::{Digest, DigestItem, Header}
	};

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

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			assert!(true);
		});
	}

    fn can_mint_utxos() {
        with_externalities(&mut new_test_ext(), || {
            let pubkey = H256::random();      //some randome h256
            assert_ok!(Utxo::mint(Origin::INHERENT, 5, pubkey));
        });
    }

}
