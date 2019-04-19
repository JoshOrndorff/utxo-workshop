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
use system::ensure_signed;
use primitives::H256;
use primitives::ed25519::{Public, Signature};
use runtime_primitives::traits::{Hash, BlakeTwo256};
use runtime_primitives::{Serialize, Deserialize}; //update
use serde::{de, Serializer, Deserializer}; //not sure about this
use parity_codec::{Encode, Decode}; //update

pub trait Trait: system::Trait {
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(PartialEq, Eq, Default, Clone, Encode, Decode, Hash)]
pub struct Transaction {
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>
}

impl<'de> Deserialize<'de> for Signature {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> 
        where D: Deserializer<'de> 
    {
		Signature::from_ss58check(&String::deserialize(deserializer)?)
			.map_err(|e| de::Error::custom(format!("{:?}", e)))
	}
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer 
    {
		serializer.serialize_str(&self.to_ss58check())
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(PartialEq, Eq, Default, Clone, Encode, Decode, Hash)]
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
    pub salt: u32,    // distinguishes outputs of same value/pubkey apart
}

decl_storage! {
	trait Store for Module<T: Trait> as Utxo {
        /// Mocks the UTXO state
		pub UnspentOutputs get(unspent_outputs) build(|config: &GenesisConfig<T>| {
            config.initial_utxo
                .iter()
                .cloned()      //clones underlying iterator
                .map(|u| (BlakeTwo256::hash_of(&u), u))
                .collect::<Vec<_>>()
        }): map H256 => Option<TransactionOutput>;

	}

    add_extra_genesis {
        config(initial_utxo): Vec<(TransactionOutput)>;
    }

}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event<T>() = default;

        pub fn do_something(origin, something: u32) -> Result {
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
}
