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

use sc_cli::{SubstrateCli};
use crate::service;
use crate::chain_spec;
use crate::cli::Cli;

impl SubstrateCli for Cli {
	fn impl_name() -> &'static str {
		"Substrate Node"
	}

	fn impl_version() -> &'static str {
		env!("SUBSTRATE_CLI_IMPL_VERSION")
	}

	fn executable_name() -> &'static str {
		env!("CARGO_PKG_NAME")
	}

	fn author() -> &'static str {
		env!("CARGO_PKG_AUTHORS")
	}

	fn description() -> &'static str {
		env!("CARGO_PKG_DESCRIPTION")
	}

	fn support_url() -> &'static str {
		"support.anonymous.an"
	}

	fn copyright_start_year() -> i32 {
		2017
	}

	fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
		Ok(match chain_spec::Alternative::from(id) {
			Some(spec) => Box::new(spec.load()?),
			None => Box::new(chain_spec::ChainSpec::from_json_file(std::path::PathBuf::from(id))?),
		})
	}
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();
	let default_sr25519_public_key = sp_core::sr25519::Public::from_raw([0; 32]);

	match &cli.subcommand {
		Some(subcommand) => {
			let runner = cli.create_runner(subcommand)?;

			runner.run_subcommand(
				subcommand,
				|config| Ok(new_full_start!(config, default_sr25519_public_key).0),
			)
		},
		None => {
			let sr25519_public_key = cli.run.sr25519_public_key.unwrap_or(default_sr25519_public_key);
			let runner = cli.create_runner(&cli.run.base)?;

			runner.run_node(
				|config| service::new_light(config, sr25519_public_key),
				|config| service::new_full(config, sr25519_public_key),
				utxo_runtime::VERSION,
			)
		},
	}
}
