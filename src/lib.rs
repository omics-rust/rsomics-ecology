#![forbid(unsafe_code)]

mod alpha;
mod cli;
mod counts;
mod phylogeny;

/// Runs the executable's command-line interface.
pub fn run_cli() -> std::process::ExitCode {
    cli::run()
}
