#![cfg_attr(not(feature = "std"), no_std)]

/// A trait for types that can provide the amount of issuance to award to the block
/// author for the given block number.
pub trait Issuance<BlockNumber, Balance> {
	fn issuance(block: BlockNumber) -> Balance;
}

// A minimal implementation for when you don't actually want any issuance
impl Issuance<u32, u128> for () {
	fn issuance(_block: u32) -> u128 {
		0
	}
}
