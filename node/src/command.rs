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

use sc_cli::VersionInfo;
use crate::service;
use crate::chain_spec;
use crate::cli::Cli;

/// Parse and run command line arguments
pub fn run(version: VersionInfo) -> sc_cli::Result<()> {
	let opt = sc_cli::from_args::<Cli>(&version);

	let mut config = sc_service::Configuration::from_version(&version);

	match opt.subcommand {
		Some(subcommand) => {
			let sr25519_public_key = sp_core::sr25519::Public::from_raw([0; 32]);

			subcommand.init(&version)?;
			subcommand.update_config(&mut config, chain_spec::load_spec, &version)?;
			subcommand.run(
				config,
				|config: _| Ok(new_full_start!(config, sr25519_public_key).0),
			)
		},
		None => {
			let sr25519_public_key = opt.run.sr25519_public_key;

			opt.run.base.init(&version)?;
			opt.run.base.update_config(&mut config, chain_spec::load_spec, &version)?;
			opt.run.base.run(
				config,
				|config| service::new_light(config, sr25519_public_key),
				|config| service::new_full(config, sr25519_public_key),
				&version,
			)
		},
	}
}
