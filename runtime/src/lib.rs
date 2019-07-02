#![allow(clippy::drop_ref)]

//! The Substrate Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

use client::{
    block_builder::api::{self as block_builder_api, CheckInherentsResult, InherentData},
    impl_runtime_apis, runtime_api,
};
use parity_codec::{Decode, Encode};
#[cfg(feature = "std")]
use primitives::bytes;
use primitives::{ed25519, sr25519, OpaqueMetadata};
use rstd::prelude::*;
use runtime_primitives::{
    create_runtime_str, generic,
    traits::{self, BlakeTwo256, Block as BlockT, NumberFor, StaticLookup, Verify},
    transaction_validity::TransactionValidity,
    ApplyResult,
};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use version::NativeVersion;
use version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
pub use balances::Call as BalancesCall;
#[cfg(any(feature = "std", test))]
pub use runtime_primitives::BuildStorage;
pub use runtime_primitives::{Perbill, Permill};
pub use support::{construct_runtime, parameter_types, StorageValue};
pub use timestamp::BlockPeriod;
pub use timestamp::Call as TimestampCall;

/// Alias to the signature scheme used for Aura authority signatures.
pub type AuraSignature = ed25519::Signature;

/// The Ed25519 pub key of an session that belongs to an Aura authority of the chain.
pub type AuraId = ed25519::Public;

/// Alias to pubkey that identifies an account on the chain.
pub type AccountId = <AccountSignature as Verify>::Signer;

/// The type used by authorities to prove their ID.
pub type AccountSignature = sr25519::Signature;

/// A hash of some data used by the chain.
pub type Hash = primitives::H256;

/// Index of a block number in the chain.
pub type BlockNumber = u64;

/// Index of an account's extrinsic in the chain.
pub type Nonce = u64;

pub mod utxo;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datastructures.
pub mod opaque {
    use super::*;

    /// Opaque, encoded, unchecked extrinsic.
    #[derive(PartialEq, Eq, Clone, Default, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub struct UncheckedExtrinsic(#[cfg_attr(feature = "std", serde(with = "bytes"))] pub Vec<u8>);
    #[cfg(feature = "std")]
    impl std::fmt::Debug for UncheckedExtrinsic {
        fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(fmt, "{}", primitives::hexdisplay::HexDisplay::from(&self.0))
        }
    }
    impl traits::Extrinsic for UncheckedExtrinsic {
        fn is_signed(&self) -> Option<bool> {
            None
        }
    }
    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
    /// Opaque session key type.
    pub type SessionKey = AuraId;
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("utxo"),
    impl_name: create_runtime_str!("utxo"),
    authoring_version: 3,
    spec_version: 3,
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
};

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

impl system::Trait for Runtime {
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = Indices;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Nonce;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous origin type.
    type Origin = Origin;
}

impl aura::Trait for Runtime {
    type HandleReport = ();
    type AuthorityId = AuraId;
}

impl indices::Trait for Runtime {
    /// The type for recording indexing into the account enumeration. If this ever overflows, there
    /// will be problems!
    type AccountIndex = u32;
    /// Use the standard means of resolving an index hint from an id.
    type ResolveHint = indices::SimpleResolveHint<Self::AccountId, Self::AccountIndex>;
    /// Determine whether an account is dead.
    type IsDeadAccount = Balances;
    /// The uniquitous event type.
    type Event = Event;
}

impl timestamp::Trait for Runtime {
    /// A timestamp: seconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Aura;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
    pub const TransferFee: u128 = 0;
    pub const CreationFee: u128 = 0;
    pub const TransactionBaseFee: u128 = 1;
    pub const TransactionByteFee: u128 = 0;
}

impl balances::Trait for Runtime {
    /// The type for recording an account's balance.
    type Balance = u128;
    /// What to do if an account's free balance gets zeroed.
    type OnFreeBalanceZero = ();
    /// What to do if a new account is created.
    type OnNewAccount = Indices;
    /// The uniquitous event type.
    type Event = Event;

    type TransactionPayment = ();
    type DustRemoval = ();
    type TransferPayment = ();

    type ExistentialDeposit = ExistentialDeposit;
    type TransferFee = TransferFee;
    type CreationFee = CreationFee;
    type TransactionBaseFee = TransactionBaseFee;
    type TransactionByteFee = TransactionByteFee;
}

impl sudo::Trait for Runtime {
    /// The uniquitous event type.
    type Event = Event;
    type Proposal = Call;
}

impl utxo::Trait for Runtime {
    type Event = Event;
}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: system::{Module, Call, Storage, Config, Event},
        Timestamp: timestamp::{Module, Call, Storage, Config<T>, Inherent},
        Aura: aura::{Module, Config<T>, Inherent(Timestamp)},
        Indices: indices::{default, Config<T>},
        Balances: balances,
        Sudo: sudo,
        Utxo: utxo::{Module, Call, Storage, Config, Event},
    }
);

/// The type used as a helper for interpreting the sender of transactions.
type Context = system::ChainContext<Runtime>;
/// The address format for describing accounts.
type Address = <Indices as StaticLookup>::Source;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedMortalCompactExtrinsic<Address, Nonce, Call, AccountSignature>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Nonce, Call>;
/// Executive: handles dispatch to the various modules.
pub type Executive = executive::Executive<Runtime, Block, Context, Balances, Runtime, AllModules>;

// Implement our runtime API endpoints. This is just a bunch of proxying.
impl_runtime_apis! {
    impl runtime_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }

    }

    impl runtime_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            Runtime::metadata().into()
        }
    }

    impl block_builder_api::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(block: Block, data: InherentData) -> CheckInherentsResult {
            data.check_extrinsics(&block)
        }

        fn random_seed() -> <Block as BlockT>::Hash {
            System::random_seed()
        }
    }

    impl runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
            use support::IsSubType;
            use runtime_primitives::{
                traits::Hash,
                transaction_validity::{TransactionLongevity, TransactionPriority, TransactionValidity},
            };

            if let Some(&utxo::Call::execute(ref transaction)) = IsSubType::<utxo::Module<Runtime>, Runtime>::is_aux_sub_type(&tx.function) {
                // List of tags to require
                let requires;

                // Transaction priority to assign
                let priority : TransactionPriority;

                const INVALID_UTXO: i8 = -99;

                match <utxo::Module<Runtime>>::check_transaction(&transaction) {
                    // Transaction verification failed
                    Err(e) => {
                        runtime_io::print(e);
                        return TransactionValidity::Invalid(INVALID_UTXO);
                    }

                    // Transaction is valid and verified
                    Ok(utxo::CheckInfo::Totals {input, output}) => {
                        // All input UTXOs were found, so we consider input conditions to be met
                        requires = Vec::new();

                        // Priority is based on a transaction fee that is equal to the leftover value
                        let max_priority = utxo::Value::from(TransactionPriority::max_value());
                        priority = max_priority.min(input - output) as TransactionPriority;
                    }

                    // Transaction is missing inputs
                    Ok(utxo::CheckInfo::MissingInputs(missing)) => {
                        // Since some referred UTXOs were not found in the storage yet,
                        // we tag current transaction as requiring those particular UTXOs
                        requires = missing
                            .iter()         // copies itself into a new vec
                            .map(|hash| hash.as_fixed_bytes().to_vec())
                            .collect();

                        // Transaction could not be validated at this point,
                        // so we have no sane way to calculate the priority
                        priority = 0;
                    }
                }

                // Output tags this transaction provides
                let provides = transaction.outputs
                    .iter()
                    .map(|output| BlakeTwo256::hash_of(output).as_fixed_bytes().to_vec())
                    .collect();

                return TransactionValidity::Valid {
                    requires,
                    provides,
                    priority,
                    longevity: TransactionLongevity::max_value(),
                    propagate: true
                };
            }

            // Fall back to default logic for non UTXO::execute extrinsics
            Executive::validate_transaction(tx)
        }
    }

    impl consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> u64 {
            Aura::slot_duration()
        }

        fn authorities() -> Vec<AuraId> {
            Aura::authorities()
        }
    }

    impl offchain_primitives::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(n: NumberFor<Block>) {
            Executive::offchain_worker(n)
        }
    }

}
