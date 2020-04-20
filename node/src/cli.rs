pub use sc_cli::Subcommand;
use std::convert::TryInto;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Cli {
	#[structopt(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[structopt(flatten)]
	pub run: RunCmd,
}

#[derive(Debug, StructOpt)]
pub struct RunCmd {
	#[structopt(flatten)]
	pub base: sc_cli::RunCmd,

	/// SR25519 public key
	#[structopt(long, parse(try_from_str = parse_sr25519_public_key))]
	pub sr25519_public_key: Option<sp_core::sr25519::Public>,
}

fn parse_sr25519_public_key(i: &str) -> Result<sp_core::sr25519::Public, String> {
	hex::decode(i)
		.map_err(|e| e.to_string())?
		.as_slice()
		.try_into()
		.or(Err("invalid length for SR25519 public key".to_string()))
}
