//! `warlock-cli`: drive the simulation without a window.

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
    println!("warlock-cli {}", warlock_core::GAME_VERSION);
}
