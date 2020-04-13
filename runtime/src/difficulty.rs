//! A difficuty adjustment algorithm (DAA) to keep the block time close to a particular goal
//! Cribbed from Kulupu https://github.com/kulupu/kulupu/blob/master/runtime/src/difficulty.rs
//!
//! It is possible to implement other DAAs such as that of BTC and BCH. This would be an interesting
//! and worth-while experiment. The DAAs should be abstracted away with a trait.

use core::cmp::{min, max};
use sp_runtime::traits::UniqueSaturatedInto;
use frame_support::{decl_storage, decl_module, traits::Get};
use codec::{Encode, Decode};
use sp_core::U256;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, Debug)]
pub struct DifficultyAndTimestamp<M> {
	pub difficulty: Difficulty,
	pub timestamp: M,
}

/// Move value linearly toward a goal
pub fn damp(actual: u128, goal: u128, damp_factor: u128) -> u128 {
	(actual + (damp_factor - 1) * goal) / damp_factor
}

/// limit value to be within some factor from a goal
pub fn clamp(actual: u128, goal: u128, clamp_factor: u128) -> u128 {
	max(goal / clamp_factor, min(actual, goal * clamp_factor))
}

/// Pallet's configuration trait.
/// Tightly coupled to the timestamp trait because we need it's timestamp information
pub trait Trait: timestamp::Trait {
	/// The block time that the DAA will attempt to maintain
	type TargetBlockTime: Get<u128>;
	/// Dampening factor to use for difficulty adjustment
	type DampFactor: Get<u128>;
	/// Clamp factor to use for difficulty adjustment
	/// Limit value to within this factor of goal. Recommended value: 2
	type ClampFactor: Get<u128>;
	/// The maximum difficulty allowed. Recommended to use u128::max_value()
	type MaxDifficulty: Get<u128>;
	/// Minimum difficulty, enforced in difficulty retargetting
	/// avoids getting stuck when trying to increase difficulty subject to dampening
	/// Recommended to use same value as DampFactor
	type MinDifficulty: Get<u128>;
}

const DIFFICULTY_ADJUST_WINDOW: u128 = 60;
type Difficulty = U256;

decl_storage! {
	trait Store for Module<T: Trait> as Difficulty {
		/// Past difficulties and timestamps, from earliest to latest.
		PastDifficultiesAndTimestamps:
		[Option<DifficultyAndTimestamp<T::Moment>>; 60]
			= [None; DIFFICULTY_ADJUST_WINDOW as usize];
		/// Current difficulty.
		pub CurrentDifficulty get(difficulty) build(|config: &GenesisConfig| {
			config.initial_difficulty
		}): Difficulty;
		/// Initial difficulty.
		pub InitialDifficulty config(initial_difficulty): Difficulty;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn on_finalize(_n: T::BlockNumber) {
			let mut data = PastDifficultiesAndTimestamps::<T>::get();

			for i in 1..data.len() {
				data[i - 1] = data[i];
			}

			data[data.len() - 1] = Some(DifficultyAndTimestamp {
				timestamp: <timestamp::Module<T>>::get(),
				difficulty: Self::difficulty(),
			});

			let mut ts_delta = 0;
			for i in 1..(DIFFICULTY_ADJUST_WINDOW as usize) {
				let prev: Option<u128> = data[i - 1].map(|d| d.timestamp.unique_saturated_into());
				let cur: Option<u128> = data[i].map(|d| d.timestamp.unique_saturated_into());

				let delta = match (prev, cur) {
					(Some(prev), Some(cur)) => cur.saturating_sub(prev),
					_ => T::TargetBlockTime::get(),
				};
				ts_delta += delta;
			}

			if ts_delta == 0 {
				ts_delta = 1;
			}

			let mut diff_sum = U256::zero();
			for i in 0..(DIFFICULTY_ADJUST_WINDOW as usize) {
				let diff = match data[i].map(|d| d.difficulty) {
					Some(diff) => diff,
					None => InitialDifficulty::get(),
				};
				diff_sum += diff;
			}

			if diff_sum < U256::from(T::MinDifficulty::get()) {
				diff_sum = U256::from(T::MinDifficulty::get());
			}

			// Calculate the average length of the adjustment window
			let adjustment_window = DIFFICULTY_ADJUST_WINDOW * T::TargetBlockTime::get();

			// adjust time delta toward goal subject to dampening and clamping
			let adj_ts = clamp(
				damp(ts_delta, adjustment_window, T::DampFactor::get()),
				adjustment_window,
				T::ClampFactor::get(),
			);

			// minimum difficulty avoids getting stuck due to dampening
			let difficulty = min(U256::from(T::MaxDifficulty::get()),
								 max(U256::from(T::MinDifficulty::get()),
									 diff_sum * U256::from(T::TargetBlockTime::get()) / U256::from(adj_ts)));

			<PastDifficultiesAndTimestamps<T>>::put(data);
			<CurrentDifficulty>::put(difficulty);
		}
	}
}
