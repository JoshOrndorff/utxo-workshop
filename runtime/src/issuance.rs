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

/// A type that provides block issuance according to bitcoin's rules
/// Initial issuance is 50 / block
/// Issuance is cut in half every 210,000 blocks
/// cribbed from github.com/Bitcoin-ABC/bitcoin-abc/blob/9c7b12e6f128a59423f4de3d6d4b5231ebe9aac2/src/validation.cpp#L1007
pub struct BitcoinHalving;

/// The number of blocks between each halvening.
const HALVING_INTERVAL: u32 = 210_000;
/// The per-block issuance before any halvenings. Decimal places should be accounted for here.
const INITIAL_ISSUANCE: u32 = 50;

impl Issuance<u32, u128> for BitcoinHalving {

	fn issuance(block: u32) -> u128 {
		let halvings = block / HALVING_INTERVAL;
		// Force block reward to zero when right shift is undefined.
		if halvings >= 64 {
			return 0;
		}

		// Subsidy is cut in half every 210,000 blocks which will occur
		// approximately every 4 years.
		(INITIAL_ISSUANCE >> halvings).into()
	}
}
