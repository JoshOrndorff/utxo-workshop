// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use crate::service;
use crate::chain_spec;
use crate::cli::Cli;
use sc_cli::{SubstrateCli, RuntimeVersion, Role, ChainSpec};
use sc_service::PartialComponents;
use crate::service::new_partial;

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Substrate Node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn executable_name() -> String {
		env!("CARGO_PKG_NAME").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn support_url() -> String {
		"support.anonymous.an".into()
	}

	fn copyright_start_year() -> i32 {
		2017
	}

	fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
		Ok(match id {
			"dev" => Box::new(chain_spec::development_config()),
			"" | "local" => Box::new(chain_spec::local_testnet_config()),
			path => Box::new(chain_spec::ChainSpec::from_json_file(
				std::path::PathBuf::from(path),
			)?),
		})
	}

	fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		&utxo_runtime::VERSION
	}
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();
	let default_sr25519_public_key = sp_core::sr25519::Public::from_raw([0; 32]);

	match &cli.subcommand {
		Some(subcommand) => {
			let runner = cli.create_runner(subcommand)?;
			runner.run_subcommand(subcommand, |config| {
				let PartialComponents { client, backend, task_manager, import_queue, .. }
					= new_partial(&config, default_sr25519_public_key)?;
				Ok((client, backend, import_queue, task_manager))
			})
		},
		None => {
			let sr25519_public_key = cli.run.sr25519_public_key.unwrap_or(default_sr25519_public_key);
			let runner = cli.create_runner(&cli.run.base)?;
			runner.run_node_until_exit(|config| match config.role {
				Role::Light => service::new_light(config, sr25519_public_key),
				_ => service::new_full(config, sr25519_public_key),
			})
		},
	}
}
