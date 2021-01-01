use sp_core::{Pair, Public, sr25519, H256};
use utxo_runtime::{
	AccountId, BalancesConfig, GenesisConfig, DifficultyAdjustmentConfig,
	SudoConfig, SystemConfig, WASM_BINARY, Signature, UtxoConfig,
};
use sc_service;
use sp_runtime::traits::{Verify, IdentifyAccount};
use utxo_runtime::utxo;

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn development_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Development",
		"dev",
		sc_service::ChainType::Development,
		|| testnet_genesis(
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			],
			// Genesis set of pubkeys that own UTXOs
			vec![
				get_from_seed::<sr25519::Public>("Alice"),
				get_from_seed::<sr25519::Public>("Bob"),
			],
			true,
		),
		vec![],
		None,
		None,
		None,
		None
	)
}

pub fn local_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		sc_service::ChainType::Local,
		|| testnet_genesis(
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Dave"),
				get_account_id_from_seed::<sr25519::Public>("Eve"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
				get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
				get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
			],
			// Genesis set of pubkeys that own UTXOs
			vec![
				get_from_seed::<sr25519::Public>("Alice"),
				get_from_seed::<sr25519::Public>("Bob"),
			],
			true,
		),
		vec![],
		None,
		None,
		None,
		None
	)
}

fn testnet_genesis(
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	endowed_utxos: Vec<sr25519::Public>,
	_enable_println: bool
) -> GenesisConfig {
	// This prints upon creation of the genesis block
	println!("============ HELPER INPUTS FOR THE UI DEMO ============");
	println!("OUTPOINT (Alice's UTXO Hash): 0x76584168d10a20084082ed80ec71e2a783abbb8dd6eb9d4893b089228498e9ff\n");
	println!("SIGSCRIPT (Alice Signature on a transaction where she spends 50 utxo on Bob): 0x6ceab99702c60b111c12c2867679c5555c00dcd4d6ab40efa01e3a65083bfb6c6f5c1ed3356d7141ec61894153b8ba7fb413bf1e990ed99ff6dee5da1b24fd83\n");
	println!("PUBKEY (Bob's public key hash): 0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48\n");
	println!("NEW UTXO HASH in UTXOStore onchain: 0xdbc75ab8ee9b83dcbcea4695f9c42754d94e92c3c397d63b1bc627c2a2ef94e6\n");

	GenesisConfig {
		frame_system: Some(SystemConfig {
			code: WASM_BINARY.to_vec(),
			changes_trie_config: Default::default(),
		}),
		balances: Some(BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
		}),
		sudo: Some(SudoConfig {
			key: root_key,
		}),
		difficulty: Some(DifficultyAdjustmentConfig {
			initial_difficulty: 4_000_000.into(),
		}),
		utxo: Some(UtxoConfig {
		  genesis_utxos: endowed_utxos
			.iter()
			.map(|x|
				utxo::TransactionOutput {
					value: 100 as utxo::Value,
					pubkey: H256::from_slice(x.as_slice()),
				}
			)
			.collect()
		}),
	}
}
