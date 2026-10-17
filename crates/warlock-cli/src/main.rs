//! `warlock-cli`: drive the simulation without a window (spec §9).
//!
//! ```text
//! warlock-cli dump --scenario showcase --actions "move:N move:N" --layers view,fov
//! warlock-cli replay --create --scenario showcase --actions "move:N wait" -o run.json
//! warlock-cli replay --verify run.json
//! ```

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use warlock_core::action::parse_actions;
use warlock_core::content::{Content, ContentFiles};
use warlock_core::dump;
use warlock_core::replay::Replay;
use warlock_core::{Action, World};

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Read content (`data/`, `levels/`, `scenarios/`) from this directory instead of the
    /// built-in copy.
    #[arg(long, global = true)]
    content_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print the state of a scenario after applying actions.
    Dump(DumpArgs),
    /// Create or verify a replay file.
    Replay(ReplayArgs),
}

/// Where a run starts and which actions it applies.
#[derive(Debug, Args)]
struct RunArgs {
    /// Scenario to start (ignored when `--replay` is given).
    #[arg(long, default_value = "showcase")]
    scenario: String,
    /// Run seed; defaults to the scenario's seed (ignored when `--replay` is given).
    #[arg(long)]
    seed: Option<u64>,
    /// Actions as text, e.g. "move:E move:SE wait open:E close:W descend ascend".
    #[arg(long)]
    actions: Option<String>,
}

#[derive(Debug, Args)]
struct DumpArgs {
    #[command(flatten)]
    run: RunArgs,
    /// Start from the end state of this replay (verified) instead of a fresh scenario;
    /// `--actions` are applied after it.
    #[arg(long)]
    replay: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = Format::Ascii)]
    format: Format,
    /// Comma-separated layers: view, fov, memory, truth (truth is dev-only ground truth).
    #[arg(long, default_value = "view")]
    layers: String,
    /// With `--format json`, include the full world state.
    #[arg(long)]
    state: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Format {
    Ascii,
    Json,
}

#[derive(Debug, Args)]
struct ReplayArgs {
    /// Verify the state hash after every step of this replay file.
    #[arg(long, value_name = "FILE", conflicts_with = "create")]
    verify: Option<PathBuf>,
    /// Record a new replay from `--scenario`, `--seed` and `--actions`.
    #[arg(long, requires = "output")]
    create: bool,
    #[command(flatten)]
    run: RunArgs,
    /// Where `--create` writes the replay.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("warlock-cli: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let content = load_content(cli.content_dir.as_deref())?;
    match cli.command {
        Command::Dump(args) => dump_command(content, args),
        Command::Replay(args) => replay_command(content, args),
    }
}

fn load_content(dir: Option<&Path>) -> Result<Arc<Content>, String> {
    let files = match dir {
        Some(dir) => ContentFiles::from_dir(dir)
            .map_err(|e| format!("cannot read content from {}: {e}", dir.display()))?,
        None => ContentFiles::builtin(),
    };
    Content::load(&files)
        .map(Arc::new)
        .map_err(|e| format!("invalid content: {e}"))
}

fn read_replay(path: &Path) -> Result<Replay, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    Replay::from_json(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn parse_run_actions(args: &RunArgs) -> Result<Vec<Action>, String> {
    parse_actions(args.actions.as_deref().unwrap_or("")).map_err(|e| format!("--actions: {e}"))
}

fn dump_command(content: Arc<Content>, args: DumpArgs) -> Result<(), String> {
    let layers = dump::parse_layers(&args.layers)?;
    let actions = parse_run_actions(&args.run)?;
    let mut world = match &args.replay {
        Some(path) => read_replay(path)?.run(content).map_err(|e| e.to_string())?,
        None => World::new(content, &args.run.scenario, args.run.seed)
            .map_err(|e| format!("cannot start scenario `{}`: {e}", args.run.scenario))?,
    };
    for action in &actions {
        world.apply(action);
    }
    match args.format {
        Format::Ascii => print!("{}", dump::ascii(&world, &layers)),
        Format::Json => {
            let value = dump::json(&world, &layers, args.state);
            let text = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
            println!("{text}");
        }
    }
    Ok(())
}

fn replay_command(content: Arc<Content>, args: ReplayArgs) -> Result<(), String> {
    if let Some(path) = &args.verify {
        let replay = read_replay(path)?;
        replay.run(content).map_err(|e| e.to_string())?;
        println!(
            "{}: ok, {} steps verified",
            path.display(),
            replay.steps.len()
        );
        return Ok(());
    }
    let (true, Some(output)) = (args.create, &args.output) else {
        return Err("expected --verify FILE or --create -o FILE".into());
    };
    let actions = parse_run_actions(&args.run)?;
    let mut world = World::new(content, &args.run.scenario, args.run.seed)
        .map_err(|e| format!("cannot start scenario `{}`: {e}", args.run.scenario))?;
    let mut replay = Replay::new(&world);
    for action in &actions {
        world.apply(action);
        replay.push(action, &world);
    }
    std::fs::write(output, replay.to_json())
        .map_err(|e| format!("cannot write {}: {e}", output.display()))?;
    println!(
        "{}: recorded {} steps",
        output.display(),
        replay.steps.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_definition_is_valid() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
