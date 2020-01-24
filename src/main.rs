//! UTXO Workshop CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;

pub use sc_cli::{VersionInfo, IntoExit, error};

fn main() -> Result<(), cli::error::Error> {
    let version = VersionInfo {
        name: "UTXO Workshop",
        commit: env!("VERGEN_SHA_SHORT"),
        version: env!("CARGO_PKG_VERSION"),
        executable_name: "utxo",
        author: "Parity Technologies <admin@parity.io>",
        description: "UTXO Workshop",
        support_url: "https://github.com/substrate-developer-hub/utxo-workshop",
    };

    cli::run(std::env::args(), cli::Exit, version)
}
