use std::io::{self, Read as _};

use anyhow::Context as _;
use tracing_subscriber::prelude::*;
use wyz_aoc::Puzzle as _;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, clap::Parser)]
struct Args {}

fn main() -> anyhow::Result<()> {
    let trace_fmt = tracing_subscriber::fmt::layer()
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::rfc_3339())
        .json();
    let trace_filt = tracing_subscriber::EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(trace_fmt)
        .with(trace_filt)
        .try_init()
        .map_err(|err| anyhow::anyhow!("{err}"))
        .context("failed to install a trace sink")?;
    // tracing::error!("begin");
    // tracing::warn!("begin");
    // tracing::info!("begin");
    // tracing::debug!("begin");
    // tracing::trace!("begin");

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    Ok(())
}
