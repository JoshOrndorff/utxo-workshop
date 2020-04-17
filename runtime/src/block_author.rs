#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::sr25519;
use sp_std::vec::Vec;
use sp_runtime::RuntimeString;
use frame_support::{
	decl_module, decl_storage, decl_error, ensure,
	weights::SimpleDispatchInfo,
};
use system::ensure_none;
use sp_inherents::{InherentIdentifier, InherentData, ProvideInherent, IsFatalError};
#[cfg(feature = "std")]
use sp_inherents::ProvideInherentData;
use codec::{Encode, Decode};

/// The pallet's configuration trait. Nothing to configure.
pub trait Trait: system::Trait {}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Author already set in block.
		AuthorAlreadySet,
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Rewards {
		Author: Option<sr25519::Public>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		#[weight = SimpleDispatchInfo::FixedOperational(10_000)]
		fn set_author(origin, author: sr25519::Public) {
			ensure_none(origin)?;
			ensure!(Author::get().is_none(), Error::<T>::AuthorAlreadySet);

			<Self as Store>::Author::put(author);
		}

		fn on_initialize() {
			// Reset the author to None at the beginning of the block
			<Self as Store>::Author::kill();
		}
	}
}

//TODO maybe make the trait generic over the "account" type
/// A trait to find the author (miner) of the block.
pub trait BlockAuthor {
	fn block_author() -> Option<sr25519::Public>;
}

impl BlockAuthor for () {
	fn block_author() -> Option<sr25519::Public> {
		None
	}
}

impl<T: Trait> BlockAuthor for Module<T> {
	fn block_author() -> Option<sr25519::Public> {
		Author::get()
	}
}

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"author__";

#[derive(Encode)]
#[cfg_attr(feature = "std", derive(Debug, Decode))]
pub enum InherentError {
	Other(RuntimeString),
}

impl IsFatalError for InherentError {
	fn is_fatal_error(&self) -> bool {
		match *self {
			InherentError::Other(_) => true,
		}
	}
}

impl InherentError {
	/// Try to create an instance ouf of the given identifier and data.
	#[cfg(feature = "std")]
	pub fn try_from(id: &InherentIdentifier, data: &[u8]) -> Option<Self> {
		if id == &INHERENT_IDENTIFIER {
			<InherentError as codec::Decode>::decode(&mut &data[..]).ok()
		} else {
			None
		}
	}
}

/// The type of data that the inherent will contain.
/// Just a byte array. It will be decoded to an actual pubkey later
pub type InherentType = Vec<u8>;

#[cfg(feature = "std")]
pub struct InherentDataProvider(pub InherentType);

#[cfg(feature = "std")]
impl ProvideInherentData for InherentDataProvider {
	fn inherent_identifier(&self) -> &'static InherentIdentifier {
		&INHERENT_IDENTIFIER
	}

	fn provide_inherent_data(&self, inherent_data: &mut InherentData) -> Result<(), sp_inherents::Error> {
		inherent_data.put_data(INHERENT_IDENTIFIER, &self.0)
	}

	fn error_to_string(&self, error: &[u8]) -> Option<String> {
		InherentError::try_from(&INHERENT_IDENTIFIER, error).map(|e| format!("{:?}", e))
	}
}

impl<T: Trait> ProvideInherent for Module<T> {
	type Call = Call<T>;
	type Error = InherentError;
	const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

	fn create_inherent(data: &InherentData) -> Option<Self::Call> {
		// Grab the Vec<u8> labelled with "author_" from the map of all inherent data
		let author_raw = data.get_data::<InherentType>(&INHERENT_IDENTIFIER)
			.expect("Gets and decodes authorship inherent data")?;

		// Decode the Vec<u8> into an actual author
		let author = sr25519::Public::decode(&mut &author_raw[..])
			.expect("Decodes author raw inherent data");

		Some(Call::set_author(author))
	}
}
