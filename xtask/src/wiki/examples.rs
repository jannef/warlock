//! The worked examples on the wiki's pages (`> **Example: ...**` boxes on Combat, Stats, Burden,
//! Skills, Mana, and Enchanting; #1578, #1669) are hand-written. These tests rebuild each
//! example's character through `warlock-core`, as `warlock-cli dump --character ... --layers
//! stats,attacks` does, or its hypothetical one through the same rules, and check that every
//! number the box states is the number the game computes: each check is a phrase the box must
//! contain with the computed number in it. Editing an example, or the rules or data under it,
//! without the other fails here.

use std::sync::Arc;

use warlock_core::burden::Burden;
use warlock_core::character::CharacterSpec;
use warlock_core::check;
use warlock_core::combat;
use warlock_core::content::Content;
use warlock_core::mana;
use warlock_core::object::format_weight;
use warlock_core::skills::Effect;
use warlock_core::stats::{self, Stat};
use warlock_core::world::World;

use super::util::signed;
use crate::workspace_root;

/// The box on `page` whose title starts with `title`, as one line: the `> ` markers dropped,
/// the lines joined with spaces, wiki links as their text, and runs of spaces made one.
fn example(page: &str, title: &str) -> String {
    let path = workspace_root().join("wiki/src").join(page);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{page}: {e}"));
    let head = format!("> **Example: {title}");
    let lines: Vec<&str> = text.lines().collect();
    let start = (lines.iter().position(|l| l.starts_with(&head)))
        .unwrap_or_else(|| panic!("{page} has no example \"{title}\""));
    let body: Vec<&str> = (lines[start..].iter())
        .take_while(|l| l.starts_with('>'))
        .map(|l| l.trim_start_matches('>').trim())
        .collect();
    let joined = super::tests::unlink(&body.join(" "));
    joined.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Fails, showing the box, unless `block` says `phrase`.
#[track_caller]
fn says(block: &str, phrase: &str) {
    assert!(
        block.contains(phrase),
        "the example should say `{phrase}`; it reads:\n{block}"
    );
}

/// A chance of `part` in 144 as the examples write it: a tenth, `.0` left out (`85.4%`, `75%`).
fn pct(part: u32) -> String {
    let text = format!("{:.1}", f64::from(part) * 100.0 / 144.0);
    format!("{}%", text.trim_end_matches(".0"))
}

/// A small count in words, as prose writes it.
fn word(n: i32) -> &'static str {
    const WORDS: [&str; 11] = [
        "no", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    ];
    usize::try_from(n)
        .ok()
        .and_then(|n| WORDS.get(n))
        .unwrap_or_else(|| panic!("no word for {n}"))
}

fn content() -> Arc<Content> {
    Arc::new(warlock_assets::content().expect("content"))
}

/// A fresh run of the first campaign's start as `character` with its starting kit, as
/// `warlock-cli dump --scenario ascmearc --character CHARACTER` starts it.
fn run(content: &Arc<Content>, character: &str) -> World {
    let spec: CharacterSpec = character.parse().expect("character");
    World::start_run(content.clone(), "ascmearc", None, Some(&spec), None, true)
        .unwrap_or_else(|e| panic!("cannot start as {character}: {e}"))
}

/// What skill `skill` gives of `effect` at value `value`.
fn skill_gives(content: &Content, skill: &str, effect: Effect, value: u16) -> i32 {
    let id = content.skill_id(skill).expect("skill");
    (content.skill(id).benefits.iter())
        .filter(|b| b.effect == effect)
        .map(|b| b.at(value))
        .sum()
}

fn race_class(content: &Content, world: &World) -> (i32, i32, i32, i32) {
    let spec = &world.character().expect("a character").spec;
    let race = content.race(&spec.race).expect("race");
    let class = content.class(&spec.class).expect("class");
    (race.hp, class.hp, race.mana, class.mana)
}

#[test]
fn combat_a_dwarf_warrior_against_a_kobold() {
    let content = content();
    let block = example("combat.md", "a dwarf warrior against a kobold");
    let world = run(&content, "dwarf,female,warrior");
    let view = world.perception();
    let name = &world.character().expect("character").name;
    says(&block, &format!("{name}, a level 1 dwarf warrior"));

    let strength = view.own_scores()[Stat::Strength];
    let st_mod = stats::modifier(i32::from(strength));
    let skills = content.weapon_skills();
    let blades = skills.id("long_blades").expect("long blades");
    let rank = view.own_weapon_skills().rank(blades);
    let benefit = skills.benefit(blades, rank);
    let sword = content.object(content.object_id("long_sword").expect("long sword"));
    let weapon = sword.attack.as_ref().expect("a weapon");
    says(
        &block,
        &format!("has Strength {strength} and rank {rank} in Long blades"),
    );
    says(
        &block,
        &format!("swings a long sword ({}, no bonus", weapon.damage),
    );
    assert_eq!(
        weapon.to_hit, 0,
        "the long sword has a bonus to hit of its own"
    );

    // Her bonus, as the character screen shows it.
    let attacks = view.own_attacks();
    let blow = (attacks.iter())
        .find(|a| a.name == "long sword")
        .expect("a long sword in hand");
    assert_eq!(
        blow.to_hit,
        st_mod + benefit.to_hit,
        "the blow's bonus is not explained"
    );
    says(
        &block,
        &format!("{} for Strength {strength}", signed(st_mod)),
    );
    says(
        &block,
        &format!("{} for rank {rank} in Long blades", signed(benefit.to_hit)),
    );
    says(&block, &format!("**{}**", signed(blow.to_hit)));
    says(&block, &format!("\"{}\"", blow.summary(&content)));

    // The kobold: no gear, no armour.
    let kobold_id = content.creature_id("kobold").expect("kobold");
    let kobold = super::creatures::bare(&content, kobold_id);
    let def = content.creature(kobold_id);
    assert!(
        def.natural.as_ref().is_none_or(|h| h.protection == 0),
        "the kobold has armour now"
    );
    let dexterity = kobold.stats.score(Stat::Dexterity);
    let dex_mod = stats::modifier(i32::from(dexterity));
    let defence = kobold.defence(&content, Burden::Unburdened, None).total();
    assert_eq!(
        defence,
        10 + dex_mod,
        "the kobold's Defence is not 10 and its Dexterity"
    );
    says(
        &block,
        &format!(
            "Defence is {defence} (10, and {} for its Dexterity {dexterity})",
            signed(dex_mod)
        ),
    );
    let need = defence - blow.to_hit;
    says(
        &block,
        &format!("{defence} \u{2212} {} = **{need}**", blow.to_hit),
    );
    let hits = combat::hit_ways(blow.to_hit, defence);
    says(
        &block,
        &format!(
            "{hits} of the 144 rolls make {need} or more: **{}**",
            pct(hits)
        ),
    );

    // Her damage.
    let damage = &blow.damage;
    assert_eq!(damage.groups().len(), 1, "one die");
    let die = damage.groups()[0];
    let bonus = damage.bonus();
    assert_eq!(
        bonus,
        st_mod + benefit.damage,
        "the damage bonus is not explained"
    );
    says(
        &block,
        &format!(
            "{weapon_dice}, {} for Strength, {} for her rank: **{} to {}**",
            signed(st_mod),
            signed(benefit.damage),
            damage.min(),
            damage.max(),
            weapon_dice = weapon.damage,
        ),
    );
    let hp = kobold.max_hp(&content);
    says(&block, &format!("kobold has {hp} hit points"));
    let sides = i32::try_from(die.sides()).expect("sides");
    let killing = (1..=sides).filter(|r| r + bonus >= hp).count() as i32;
    says(
        &block,
        &format!("{} hits in {} kill it outright", word(killing), word(sides)),
    );

    // A critical.
    let threat = weapon.threat.saturating_sub(benefit.threat);
    let threats: u32 = (i32::from(threat)..=combat::HIGHEST)
        .map(combat::ways)
        .sum();
    says(
        &block,
        &format!(
            "threatens on {threat} to {}, {threats} rolls in 144 ({})",
            combat::HIGHEST,
            pct(threats)
        ),
    );
    says(&block, &format!("{} of those threats confirm", pct(hits)));
    let crit = i32::from(weapon.crit + benefit.crit);
    let times = match crit {
        2 => "twice",
        3 => "three times",
        _ => panic!("say how often a ×{crit} critical rolls"),
    };
    says(
        &block,
        &format!(
            "rolled {times} and added: {}d{}+{}, {} to {}",
            crit,
            die.sides(),
            crit * bonus,
            crit * damage.min(),
            crit * damage.max()
        ),
    );
}

#[test]
fn stats_a_dwarf_warriors_stats() {
    let content = content();
    let block = example("stats.md", "a dwarf warrior's stats");
    let world = run(&content, "dwarf,female,warrior");
    let view = world.perception();
    let name = &world.character().expect("character").name;
    says(&block, &format!("{name}, a level 1 dwarf warrior"));
    let scores = view.own_scores();
    let cell = |stat: Stat| {
        let score = scores[stat];
        format!("{score} ({})", signed(stats::modifier(i32::from(score))))
    };
    let row: Vec<String> = Stat::ALL.iter().map(|&s| cell(s)).collect();
    says(&block, &format!("| {} |", row.join(" | ")));

    let st = scores[Stat::Strength];
    let to = scores[Stat::Toughness];
    let st_mod = stats::modifier(i32::from(st));
    says(
        &block,
        &format!(
            "Strength {}: {} to hit and {} damage",
            signed(st_mod),
            signed(st_mod),
            signed(st_mod)
        ),
    );
    let dx_mod = stats::modifier(i32::from(scores[Stat::Dexterity]));
    says(&block, &format!("Dexterity {}:", signed(dx_mod)));
    let (race_hp, class_hp, ..) = race_class(&content, &world);
    let (_, max_hp) = view.own_hp();
    assert_eq!(max_hp, i32::from(to) + race_hp + class_hp);
    says(
        &block,
        &format!(
            "Toughness {to}: {to} + {race_hp} (dwarf) + {class_hp} (warrior) = {max_hp} hit points"
        ),
    );
    says(
        &block,
        &format!(
            "every 360 \u{2212} 12 × {to} = {} seconds",
            combat::regen_seconds(to)
        ),
    );
    let load = view.own_load();
    says(
        &block,
        &format!("1.5 kg × (2 × {st} + {to}) = {}", format_weight(load.limit)),
    );
    let pr_mod = stats::modifier(i32::from(scores[Stat::Presence]));
    let per_point = i32::from(content.shops().expect("shops").presence);
    let against = -pr_mod * per_point;
    assert!(against > 0, "her Presence no longer works against her");
    says(
        &block,
        &format!(
            "Presence {}: shop prices are {against}% against her",
            signed(pr_mod)
        ),
    );
}

#[test]
fn burden_a_dwarf_warriors_starting_load() {
    let content = content();
    let block = example("burden.md", "a dwarf warrior's starting load");
    let world = run(&content, "dwarf,female,warrior");
    let view = world.perception();
    let scores = view.own_scores();
    let (st, to) = (scores[Stat::Strength], scores[Stat::Toughness]);
    says(&block, &format!("has Strength {st} and Toughness {to}"));
    let spec = &world.character().expect("character").spec;
    let race = content.race(&spec.race).expect("race");
    let carry = content.size(race.size).carry;
    assert_eq!(carry % 100, 0, "a size that carries {carry}%");
    let load = view.own_load();
    let [(_, burdened), (_, strained), (_, overloaded)] = load.thresholds();
    says(
        &block,
        &format!(
            "1.5 kg × (2 × {st} + {to}) × {} = **{}**",
            carry / 100,
            format_weight(load.limit)
        ),
    );
    says(
        &block,
        &format!(
            "unburdened up to a third of it, {}, burdened up to {}, and strained up to {}",
            format_weight(burdened),
            format_weight(strained),
            format_weight(overloaded)
        ),
    );
    assert_eq!(
        load.burden(),
        Burden::Burdened,
        "she no longer starts burdened"
    );
    says(
        &block,
        &format!(
            "weighs {}, so she starts just burdened",
            format_weight(load.carried)
        ),
    );
    assert!(load.ignores_burdened, "a dwarf feels Burdened now");
    assert_eq!(load.penalty(), Burden::Unburdened);
    says(
        &block,
        "Being a dwarf, that costs her nothing until she is strained",
    );
}

#[test]
fn skills_a_thief_disarms_an_arrow_trap() {
    let content = content();
    let block = example("skills.md", "a thief disarms an arrow trap");
    let (dexterity, value, best) = (14, 25, 100);
    let trap = content.trap(content.trap_id("arrow").expect("arrow trap"));
    let difficulty = i32::from(trap.difficulty);
    says(
        &block,
        &format!("Dexterity {dexterity} and Disable device {value}"),
    );
    says(&block, &format!("arrow trap of difficulty {difficulty}"));

    let dx_mod = stats::modifier(dexterity);
    let gives = |v| skill_gives(&content, "disable_device", Effect::Disarm, v);
    let (top, here) = (gives(best), gives(value));
    // Even growth: the page works it out as a fraction of the top.
    assert_eq!(gives(0), 0);
    assert_eq!(here, top * i32::from(value) / i32::from(best));
    says(
        &block,
        &format!("{} for Dexterity {dexterity}", signed(dx_mod)),
    );
    says(
        &block,
        &format!(
            "from +0 at 0 to {} at {best}, so {value} gives {top} × {value} ÷ {best} = {here}",
            signed(top)
        ),
    );
    let modifier = dx_mod + here;
    says(&block, &format!("In all, **{}**", signed(modifier)));
    let need = difficulty - modifier;
    let made = check::ways(modifier, difficulty);
    says(
        &block,
        &format!(
            "{difficulty} \u{2212} {modifier} = **{need}**: {made} of the 144 rolls, **{}**",
            pct(made)
        ),
    );
    let short = content.trap_disarm().springs_short_by;
    let springs = check::ways_short_by(modifier, difficulty, short);
    says(
        &block,
        &format!(
            "falling short by {short} or more (a total of {} or less, so dice of {} or less)",
            difficulty - short,
            need - short
        ),
    );
    says(
        &block,
        &format!(
            "sets the trap off at her: {springs} rolls in 144, **{}**",
            pct(springs)
        ),
    );

    let modifier = dx_mod + top;
    let fails = 144 - check::ways(modifier, difficulty);
    assert_eq!(check::ways_short_by(modifier, difficulty, short), 0);
    says(
        &block,
        &format!(
            "At Disable device {best} she adds {} and the dice need {}: only {} rolls in 144 fail ({}), and none sets the trap off",
            signed(modifier),
            difficulty - modifier,
            fails,
            pct(fails)
        ),
    );
}

#[test]
fn mana_an_elf_wizards_mana() {
    let content = content();
    let block = example("mana.md", "an elf wizard's mana");
    let world = run(&content, "elf,female,wizard");
    let view = world.perception();
    let name = &world.character().expect("character").name;
    says(&block, &format!("{name}, a level 1 elf wizard"));
    let scores = view.own_scores();
    let modifier = |s: Stat| stats::modifier(i32::from(scores[s]));
    let (int, wil, pre) = (
        modifier(Stat::Intelligence),
        modifier(Stat::Willpower),
        modifier(Stat::Presence),
    );
    says(
        &block,
        &format!(
            "Intelligence {} ({}), Willpower {} ({}), and Presence {} ({})",
            scores[Stat::Intelligence],
            signed(int),
            scores[Stat::Willpower],
            signed(wil),
            scores[Stat::Presence],
            signed(pre)
        ),
    );
    let (.., race_mana, class_mana) = race_class(&content, &world);
    let (_, max) = view.own_mana();
    assert_eq!(max, race_mana + class_mana + int + wil + pre);
    says(
        &block,
        &format!(
            "{race_mana} (elf) + {class_mana} (wizard) + {} + {} + {} = **{max} mana**",
            int.abs(),
            wil.abs(),
            pre.abs()
        ),
    );
    assert!(
        int >= 0 && wil >= 0 && pre >= 0,
        "write a negative modifier as one"
    );
    let burning = content.spell_id("burning_hands").expect("burning hands");
    let cost = view
        .own_spell_cost(burning)
        .expect("she knows Burning hands");
    says(
        &block,
        &format!(
            "{} casts of Burning hands at {cost} mana each",
            word(max / cost)
        ),
    );

    let level = mana::level_modifier(&stats::Stats::new(scores, scores));
    let more = class_mana / 2 + level;
    says(
        &block,
        &format!(
            "half the wizard's {class_mana}, plus the better of her Intelligence and Willpower modifiers: {} + {level} = **{more} more**",
            class_mana / 2
        ),
    );
    let willpower = scores[Stat::Willpower];
    let seconds = mana::regen_seconds(willpower);
    says(
        &block,
        &format!(
            "with Willpower {willpower}, a point every 360 \u{2212} 12 × {willpower} = {seconds} seconds ({} minutes and {} seconds)",
            seconds / 60,
            seconds % 60
        ),
    );
    let fill = u32::try_from(max).expect("mana") * seconds;
    assert_eq!(
        (fill + 1800) / 3600,
        1,
        "an empty pool no longer fills in about an hour"
    );
    says(&block, "an empty pool fills in about an hour");
}

#[test]
fn enchanting_kindling_a_long_sword() {
    let content = content();
    let block = example("enchanting.md", "kindling a long sword");
    let (willpower, value) = (14, 20);
    let rules = content.enchanting().expect("enchanting rules");
    assert_eq!(rules.stat, Stat::Willpower);
    let skill = content.skill(rules.skill).id.clone();
    says(
        &block,
        &format!("Willpower {willpower} and Enchanting {value}"),
    );
    let reach = skill_gives(&content, &skill, Effect::EnchantLevel, value).clamp(1, 12);
    let craft = skill_gives(&content, &skill, Effect::Craft, value);
    let modifier = stats::modifier(willpower) + craft;
    let recipe = |id: &str| content.recipe(content.recipes().id(id).expect("recipe"));
    let step = i32::from(rules.level_step.max(1));

    // Kindle rolls at the reach: a plain item has no item level of its own.
    let kindle = recipe("kindle");
    let difficulty = kindle.difficulty + reach / step;
    says(
        &block,
        &format!("her reach at Enchanting {value}: **{reach}**"),
    );
    says(
        &block,
        &format!(
            "Kindle's {}, plus {reach} ÷ {step} = {}: **{difficulty}**",
            kindle.difficulty,
            reach / step
        ),
    );
    says(
        &block,
        &format!(
            "{} for Willpower {willpower} (modifiers), {} craft bonus from Enchanting {value}: **{}**",
            signed(stats::modifier(willpower)),
            signed(craft),
            signed(modifier)
        ),
    );
    let made = check::ways(modifier, difficulty);
    says(
        &block,
        &format!(
            "The dice must reach {}: **{}**",
            difficulty - modifier,
            pct(made)
        ),
    );
    says(
        &block,
        &format!("A natural 2** (1 roll in 144, {}) warps the sword", pct(1)),
    );

    // Quicken on a magic sword of a higher item level rolls at the sword's.
    let quicken = recipe("quicken");
    assert!(
        quicken.from <= value,
        "she cannot quicken at Enchanting {value}"
    );
    let level = 6;
    assert!(level > reach);
    let difficulty = quicken.difficulty + level / step;
    let made = check::ways(modifier, difficulty);
    says(
        &block,
        &format!(
            "of item level {level}, rolls at its own {level}: Quicken's {} plus {}, difficulty {difficulty}, and the dice need {}, {}",
            quicken.difficulty,
            level / step,
            difficulty - modifier,
            pct(made)
        ),
    );
}
