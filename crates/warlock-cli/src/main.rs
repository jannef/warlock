//! `warlock-cli`: drive the simulation without a window (spec §9).
//!
//! ```text
//! warlock-cli dump --scenario showcase --actions "move:N move:N" --layers view,fov
//! warlock-cli dump --scenario showcase --character dwarf,female,warrior --name Brakka --layers stats
//! warlock-cli dump --character dwarf,male,warrior --talents healthy,lean --layers talents
//! warlock-cli replay --create --scenario showcase --actions "move:N wait" -o run.json
//! warlock-cli replay --verify run.json
//! warlock-cli gen --level depths:3 --seed warlock
//! ```

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use warlock_core::action::parse_actions;
use warlock_core::character::CharacterSpec;
use warlock_core::content::{Content, ContentFiles};
use warlock_core::dump;
use warlock_core::mapgen;
use warlock_core::replay::Replay;
use warlock_core::rng::parse_seed;
use warlock_core::{Action, World};

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Read content (`data/`, `levels/`, `scenarios/`, `vaults/`) from this directory instead of
    /// the built-in copy.
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
    /// Print a generated level (dev-only ground truth) without playing to it.
    Gen(GenArgs),
}

/// Where a run starts and which actions it applies.
#[derive(Debug, Args)]
struct RunArgs {
    /// Scenario to start (ignored when `--replay` is given).
    #[arg(long, default_value = "showcase")]
    scenario: String,
    /// Run seed: a number, or a word that is hashed; defaults to the scenario's seed (ignored
    /// when `--replay` is given).
    #[arg(long, value_parser = parse_seed)]
    seed: Option<u64>,
    /// The character to start as, RACE,GENDER,CLASS (e.g. `dwarf,female,warrior`); defaults to
    /// the scenario's (ignored when `--replay` is given).
    #[arg(long)]
    character: Option<CharacterSpec>,
    /// The character's name (up to 20 characters); defaults to the scenario's for its own
    /// character, else one of the race's names picked from the seed (ignored when `--replay` is
    /// given).
    #[arg(long)]
    name: Option<String>,
    /// The talents picked at creation, by id, separated by commas (`hardy,weapon_focus:long_blades`,
    /// a choice after a colon); defaults to the character's Random picks. Like `--character`, it
    /// starts the run with the character's kit (ignored when `--replay` is given).
    #[arg(long)]
    talents: Option<String>,
    /// Actions as text, e.g. "move:E move:SE wait open:E close:W kick:N descend ascend pickup:ab
    /// drop:a bash:E chat:E ask:E:rooms".
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
    /// Comma-separated layers: view, fov, memory, light (brightness digit 0-9 per visible cell),
    /// sound (heard sounds over the view: f footsteps, o/c a door opening/closing; upper case
    /// when loud), inventory (what the player carries and weighs), equipment (what they wear
    /// and wield), stats (the player's race, gender, class, and scores), skills (the player's
    /// trained skills and their values), weapon_skills (the player's weapon skills: rank and
    /// progress toward the next), schools (the schools of magic and the caster styles: rank and
    /// progress), talents (the player's talents with their ranks and choices),
    /// abilities (the player's abilities in the abilities menu's
    /// order: what each asks for, or why it cannot be used now), quests (the player's quest
    /// log: what each asks and how far it
    /// has got), truth (dev-only ground truth), actors (dev-only: every actor on the level with
    /// its weight, what it carries, its weapon skills with marks, its stats as score/potential,
    /// and its trained skills with their marks), progress (dev-only: the level-up choices due,
    /// the talents the player may take now, and the first kills that paid), rooms (dev-only: the level's special rooms,
    /// their plain ground marked with their effect's letter, then one line per room), ice
    /// (dev-only: the temperature and the ice on the level's water, its thickness in cm: 0-9,
    /// a-z for 10-35, # beyond).
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
    /// Record a new replay from `--scenario`, `--seed`, `--character`, and `--actions`.
    #[arg(long, requires = "output")]
    create: bool,
    #[command(flatten)]
    run: RunArgs,
    /// Where `--create` writes the replay.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct GenArgs {
    /// A level of a dungeon branch, e.g. `depths:3`, or a local area of the overworld, e.g.
    /// `local:49,23`.
    #[arg(long)]
    level: String,
    /// Run seed: a number, or a word that is hashed.
    #[arg(long, value_parser = parse_seed, default_value = "1")]
    seed: u64,
    /// The campaign whose overworld a local area is of (spec §3).
    #[arg(long, default_value = "showcase")]
    campaign: String,
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
        Command::Gen(args) => gen_command(&content, &args),
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

/// A fresh run of `--scenario` with `--seed`, `--character`, and `--name`: a character chosen
/// with `--character` or `--name` starts with its kit, the scenario's own with the scenario's
/// pack (spec §8 "Character creation").
fn start(content: Arc<Content>, args: &RunArgs) -> Result<World, String> {
    let kit = args.character.is_some() || args.name.is_some() || args.talents.is_some();
    let talents = args
        .talents
        .as_deref()
        .map(warlock_core::world::parse_talents);
    let character = match &talents {
        Some(talents) => warlock_core::world::chosen_with_talents(
            &content,
            &args.scenario,
            args.character.as_ref(),
            talents,
        ),
        None => args.character.clone(),
    };
    World::start_run(
        content,
        &args.scenario,
        args.seed,
        character.as_ref(),
        args.name.as_deref(),
        kit,
    )
    .map_err(|e| format!("cannot start scenario `{}`: {e}", args.scenario))
}

fn dump_command(content: Arc<Content>, args: DumpArgs) -> Result<(), String> {
    let layers = dump::parse_layers(&args.layers)?;
    let actions = parse_run_actions(&args.run)?;
    let mut world = match &args.replay {
        Some(path) => read_replay(path)?.run(content).map_err(|e| e.to_string())?,
        None => start(content, &args.run)?,
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
    let mut world = start(content, &args.run)?;
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

/// Prints the level's name, stairs, special rooms, traps (hidden ones too), secret doors, and map
/// (`*` marks a light placed on floor, `&` a locked door, `S` a secret door, `^` a trap; the
/// walkable cells of a special room show its effect's letter). On a branch's first level the up staircase leads nowhere here: it
/// depends on where the player came from.
fn gen_command(content: &Content, args: &GenArgs) -> Result<(), String> {
    let level = mapgen::generate(content, (&args.campaign, args.seed), &args.level, None)
        .map_err(|e| format!("cannot generate `{}`: {e}", args.level))?;
    println!(
        "{} ({}, {}x{}, seed {})",
        level.name,
        level.id,
        level.terrain.width(),
        level.terrain.height(),
        args.seed
    );
    for stairs in &level.stairs {
        let to = stairs.to.as_deref().unwrap_or("nowhere");
        println!("stairs {:?} at {} -> {to}", stairs.dir, stairs.pos);
    }
    if let Some(exit) = &level.exit {
        println!("edges -> {} at {}", exit.to, exit.pos);
    }
    if let Some(id) = level.pervading {
        println!("pervading {}", content.room_effect(id).name);
    }
    for room in &level.rooms {
        let effect = content.room_effect(room.effect);
        let breach = (room.breach).map_or(String::new(), |b| format!(", breach {b}"));
        println!(
            "special room {} ({}) at {}-{}{breach}",
            effect.name, effect.mark, room.min, room.max
        );
    }
    for (p, trap) in &level.traps {
        println!("{} at {p}", content.trap(trap.kind).name);
    }
    for (p, secret) in &level.secrets {
        println!("secret door at {p} (difficulty {})", secret.difficulty);
    }
    print!("{}", mapgen::ascii_with_rooms(&level, content));
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

    #[test]
    fn seeds_are_numbers_or_words() {
        let seed = |text: &str| {
            let cli = Cli::try_parse_from(["warlock-cli", "dump", "--seed", text]).expect("parses");
            let Command::Dump(args) = cli.command else {
                panic!("expected dump");
            };
            args.run.seed
        };
        assert_eq!(seed("42"), Some(42));
        assert_eq!(seed("warlock"), parse_seed("warlock").ok());
        assert_ne!(seed("warlock"), seed("Warlock"));
        assert!(Cli::try_parse_from(["warlock-cli", "dump", "--seed", ""]).is_err());
    }

    #[test]
    fn characters_are_race_gender_class() {
        let run = |args: &[&str]| {
            let cli = Cli::try_parse_from([&["warlock-cli", "dump"], args].concat())?;
            let Command::Dump(args) = cli.command else {
                panic!("expected dump");
            };
            Ok::<_, clap::Error>(args.run)
        };
        let args = run(&["--character", "dwarf,female,warrior"]).expect("parses");
        let content = load_content(None).expect("content");
        let world = start(content.clone(), &args).expect("starts");
        assert_eq!(
            world.character().map(|c| c.spec.to_string()).as_deref(),
            Some("dwarf,female,warrior")
        );
        assert!(run(&["--character", "dwarf,woman,warrior"]).is_err());
        let orc = run(&["--character", "orc,male,warrior"]).expect("parses");
        let err = start(content.clone(), &orc).expect_err("refused");
        assert!(err.contains("unknown race `orc`"), "{err}");
        // `--talents` names the creation picks, of the scenario's own character without one.
        let picks = &[
            "--character",
            "dwarf,male,warrior",
            "--talents",
            "healthy,lean",
        ];
        let world = start(content.clone(), &run(picks).expect("parses")).expect("starts");
        let spec = &world.character().expect("a character").spec;
        assert_eq!(spec.talents, ["healthy", "lean"]);
        // The showcase's Tamsin is human: three (Versatile's one more).
        let own = run(&["--talents", "healthy,lean,pious"]).expect("parses");
        let world = start(content.clone(), &own).expect("starts");
        assert!(world.started_with_kit());
        let bad = run(&["--character", "dwarf,male,warrior", "--talents", "healthy"]);
        let err = start(content, &bad.expect("parses")).expect_err("too few");
        assert!(err.contains("takes 2 at creation"), "{err}");
    }

    #[test]
    fn gen_prints_a_generated_level() {
        let content = load_content(None).expect("content");
        let args = GenArgs {
            level: "depths:2".into(),
            seed: 1,
            campaign: "showcase".into(),
        };
        assert!(gen_command(&content, &args).is_ok());
        let local = GenArgs {
            level: "local:49,23".into(),
            ..args
        };
        assert!(gen_command(&content, &local).is_ok());
        let bad = GenArgs {
            level: "hub".into(),
            ..local
        };
        assert!(gen_command(&content, &bad).is_err());
    }
}
