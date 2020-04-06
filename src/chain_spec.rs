use sp_core::{Pair, Public, sr25519};
use utxo_runtime::{
    AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig,
    SudoConfig, IndicesConfig, SystemConfig, WASM_BINARY, Signature
};
use sp_consensus_aura::sr25519::{AuthorityId as AuraId};
use grandpa_primitives::{AuthorityId as GrandpaId};
use sc_service;
use sp_runtime::traits::{Verify, IdentifyAccount, BlakeTwo256, Hash};

use primitive_types::H256;
use utxo_runtime::utxo;

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::ChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
    /// Whatever the current runtime is, with just Alice as an auth.
    Development,
    /// Whatever the current runtime is, with simple Alice/Bob auths.
    LocalTestnet,
}

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

/// Helper function to generate an authority key for Aura
pub fn get_authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (
      get_from_seed::<AuraId>(s),
      get_from_seed::<GrandpaId>(s),
    )
}

impl Alternative {
    /// Get an actual chain config from one of the alternatives.
    pub(crate) fn load(self) -> Result<ChainSpec, String> {
        Ok(match self {
            Alternative::Development => ChainSpec::from_genesis(
                "Development",
                "dev",
                || testnet_genesis(vec![
                    get_authority_keys_from_seed("Alice"),
                ],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                true),
                vec![],
                None,
                None,
                None,
                None,
            ),
            Alternative::LocalTestnet => ChainSpec::from_genesis(
                "Local Testnet",
                "local_testnet",
                || testnet_genesis(vec![
                    get_authority_keys_from_seed("Alice"),
                    get_authority_keys_from_seed("Bob"),
                ],
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
                true),
                vec![],
                None,
                None,
                None,
                None,
            ),
        })
    }

    pub(crate) fn from(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(Alternative::Development),
            "" | "local" => Some(Alternative::LocalTestnet),
            _ => None,
        }
    }
}

// Dev mode genesis setup
fn testnet_genesis(initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool) -> GenesisConfig 
{
    let genesis_utxo = utxo::TransactionOutput {
      value: utxo::Value::max_value(),
      pubkey: H256::from_slice(get_from_seed::<sr25519::Public>("Alice").as_slice()),
    };

    GenesisConfig {
      system: Some(SystemConfig {
        code: WASM_BINARY.to_vec(),
        changes_trie_config: Default::default(),
      }),
      indices: Some(IndicesConfig {
        ids: endowed_accounts.clone(),
      }),
      balances: Some(BalancesConfig {
        balances: endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
        vesting: vec![],
      }),
      sudo: Some(SudoConfig {
        key: root_key,
      }),
      aura: Some(AuraConfig {
        authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
      }),
      grandpa: Some(GrandpaConfig {
        authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
      }),
      utxo: Some(utxo::GenesisConfig {
        genesis_utxo: vec![genesis_utxo],
      }),
    }

    // ----------------------
    // HELPER PRINT OUTS FOR DEMO PURPOSES
    println!("Genesis UTXO Hash: {:?}", BlakeTwo256::hash_of(&genesis_utxo));

    let txn1 = utxo::TransactionOutput {
      value: 100,
      pubkey: H256::from_slice(get_from_seed::<sr25519::Public>("Bob").as_slice()),
    };

    // TODO update this per latest sigscript scheme
    let txn2 = utxo::TransactionOutput {
      value: utxo::Value::max_value() - 100,
      pubkey: H256::from_slice(get_from_seed::<sr25519::Public>("Alice").as_slice()),
    };

    println!("Transaction #1 {:?}, Hash: {:?}", txn1, BlakeTwo256::hash_of(&txn1));
    println!("Transaction #2 {:?}, Hash: {:?}", txn2, BlakeTwo256::hash_of(&txn2));
    // ----------------------
}
