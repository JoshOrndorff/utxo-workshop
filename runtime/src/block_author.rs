use sp_core::sr25519::Public;

/// A trait to find the author (miner) of the block.
pub trait BlockAuthor {
	fn block_author() -> Option<Public>;
}

impl BlockAuthor for () {
	fn block_author() -> Option<Public> {
		None
	}
}

//TODO turn this into an actual pallet that provides an inherent for the block author.
