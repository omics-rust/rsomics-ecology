#![forbid(unsafe_code)]

mod alpha;
mod beta;
mod cli;
mod counts;
mod distance;
mod phylogeny;
mod rarefy;
mod rng;

/// Runs the executable's command-line interface.
pub fn run_cli() -> std::process::ExitCode {
    cli::run()
}
