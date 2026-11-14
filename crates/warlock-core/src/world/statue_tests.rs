//! Stiffening and the statues of the stoned (`statue.rs`; the status's `stages` and
//! `kills_at_max` in `status.rs`): three warnings and then death, a creature's statue holding
//! what it carried (laid, solid, seen past, dug, kicked, broken), the scroll of restoration and
//! the Heal skill ending it, and the basilisk's gaze that gives it.

use super::*;
use crate::content::ContentFiles;
use crate::skills::{SkillId, Trained};
use crate::stats::Stat;
use crate::test_support::run;
use crate::test_support::texts;

// A lit hall: the player by a pick-axe and a scroll of restoration, a kobold east of them on a
// dagger, a deer far off, and a basilisk behind the wall (B) for the gaze.
const HALL: &str = "
############
#.@k.......#
#..........#
#..........#
############
";

fn files(scenario: &str) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/stone_hall.toml",
        &format!(
            "name = \"Stone hall\"\nambient = [255, 255, 255, 0]\nmap = '''{HALL}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\", objects = [\"pick_axe\", \
             \"scroll_of_restoration\"] }}\n\
             \"k\" = {{ terrain = \"floor\", creature = \"kobold\", objects = [\"dagger\"] }}\n"
        ),
    );
    files.insert(
        "scenarios/stone_hall.toml",
        &format!("name = \"Stone hall\"\nstart_level = \"stone_hall\"\nhp = 1000\n{scenario}\n"),
    );
    files
}

fn world(scenario: &str) -> World {
    let content = Arc::new(Content::load(&files(scenario)).expect("test content is valid"));
    World::new(content, "stone_hall", None).expect("world")
}

fn stiffening(w: &World) -> StatusId {
    w.content().status_id("stiffening").expect("stiffening")
}

fn severity(w: &World, id: ActorId) -> Option<u16> {
    let s = stiffening(w);
    (w.state.actors.get(&id)).and_then(|a| a.effects.get(s).map(|e| e.severity))
}

fn kobold(w: &World) -> ActorId {
    let kobold = w.content().creature_id("kobold").expect("kobold");
    (w.state.actors.values())
        .find(|a| a.creature == kobold)
        .map(|a| a.id)
        .expect("the kobold")
}

fn terrain_at(w: &World, p: Pos) -> String {
    let (level, _) = w.ground_truth();
    w.content()
        .terrain(level.terrain_at(p).expect("in bounds"))
        .id
        .clone()
}

fn pile_at(w: &World, p: Pos) -> Vec<String> {
    let (level, _) = w.ground_truth();
    (level.objects_at(p).iter())
        .map(|s| w.content().object(s.object).id.clone())
        .collect()
}

/// The kobold, stoned by the player: its cell.
fn stone_the_kobold(w: &mut World) -> Pos {
    let (player, id, s) = (w.state.player, kobold(w), stiffening(w));
    let at = w.state.actors[&id].pos;
    for degree in 1..=3 {
        w.afflict(id, s, 1, Some(player), &mut Vec::new());
        assert_eq!(severity(w, id), Some(degree), "a degree each, alive");
    }
    w.afflict(id, s, 1, Some(player), &mut Vec::new());
    assert!(!w.state.actors.contains_key(&id), "the fourth is stone");
    at
}

#[test]
fn three_warnings_then_stone_and_the_death_screen_says_so() {
    let mut w = world("");
    let player = w.state.player;
    let def = w.content().status(stiffening(&w)).clone();
    assert_eq!((def.max, def.kills_at_max, def.statue), (4, true, true));
    assert_eq!(
        def.start.as_deref(),
        Some("Your limbs stiffen."),
        "the first stage"
    );
    let before = w.messages().len();
    run(&mut w, "afflict:stiffening*3");
    let lines: Vec<String> = texts(&w).split_off(before);
    for line in [
        "Your limbs stiffen.",
        "Your skin greys and hardens.",
        "You can barely move.",
    ] {
        assert!(lines.iter().any(|l| l == line), "{line}: {lines:?}");
    }
    assert_eq!(severity(&w, player), Some(3));
    assert!(w.state.death.is_none());
    // Slow and clumsy from the second degree.
    let me = w.state.player();
    assert!(me.effects.slow(w.content(), 6000) > 6000);
    run(&mut w, "afflict:stiffening");
    let death = w.state.death.as_ref().expect("stone");
    let of = death.of.expect("of a status");
    assert_eq!(of, stiffening(&w));
    assert_eq!(
        w.content().status(of).death.as_deref(),
        Some("Turned to stone")
    );
}

#[test]
fn a_stoned_creature_is_a_statue_holding_what_it_carried() {
    let mut w = world("");
    let at = stone_the_kobold(&mut w);
    assert_eq!(terrain_at(&w, at), "petrified");
    let (level, _) = w.ground_truth();
    let statue = &level.statues()[&at];
    assert_eq!(w.content().creature(statue.of).id, "kobold");
    let held: Vec<&str> = (statue.holds.iter())
        .map(|s| w.content().object(s.object).id.as_str())
        .collect();
    assert!(held.contains(&"dagger"), "the pile it stood on: {held:?}");
    assert!(
        pile_at(&w, at).is_empty(),
        "no corpse, no pile: {:?}",
        pile_at(&w, at)
    );
    // Solid but seen past, and seen for what it was, never what it holds.
    assert!(!level.is_walkable(w.content(), at));
    let view = w.perception();
    assert_eq!(view.statue_at(at), Some(statue.of));
    assert!(view.is_visible(at.step(Dir::E)), "seen past");
    let truth = crate::dump::layer_rows(&w, crate::dump::Layer::Truth).join("\n");
    assert!(
        truth.contains(&format!(
            "statue (kobold) at {at}: holds {}",
            match statue.holds.len() {
                1 => "1 stack".to_string(),
                n => format!("{n} stacks"),
            }
        )),
        "{truth}"
    );
    // A step into it is refused.
    let from = w.player_pos();
    run(&mut w, "move:E");
    assert_eq!(w.player_pos(), from);
}

#[test]
fn a_statue_is_dug_through_and_falls_to_rubble_and_what_it_held() {
    let mut w = world("stats = { strength = 18 }");
    w.learn_mining();
    let at = stone_the_kobold(&mut w);
    run(&mut w, "pickup:a equip:tool:a dig:E");
    assert_eq!(terrain_at(&w, at), "rubble", "{:?}", texts(&w).last());
    assert!(pile_at(&w, at).contains(&"dagger".to_string()));
    let (level, _) = w.ground_truth();
    assert!(level.statues().is_empty());
}

#[test]
fn kicks_mostly_bruise_but_earn_strength_marks_and_one_breaks_it() {
    let mut w = world("stats = { strength = 18 }");
    let at = stone_the_kobold(&mut w);
    let marks = w.state.player().stat_marks[Stat::Strength];
    let mut kicks = 0;
    while terrain_at(&w, at) == "petrified" {
        kicks += 1;
        assert!(kicks < 500, "a kick breaks it at last");
        run(&mut w, "kick:E");
    }
    assert!(kicks > 1, "hard to break: {kicks}");
    assert!(
        w.state.player().stat_marks[Stat::Strength] > marks,
        "Strength marks"
    );
    assert_eq!(terrain_at(&w, at), "rubble");
    assert!(pile_at(&w, at).contains(&"dagger".to_string()));
    assert!(
        texts(&w)
            .iter()
            .any(|l| l == "The statue cracks and falls to pieces!")
    );
}

#[test]
fn the_scroll_of_restoration_ends_it() {
    let mut w = world("");
    let player = w.state.player;
    run(&mut w, "pickup afflict:stiffening*3");
    let scroll = w
        .content()
        .object_id("scroll_of_restoration")
        .expect("scroll");
    let at = (w.state.player().inventory.iter())
        .position(|s| s.object == scroll)
        .expect("carried");
    let before = w.messages().len();
    run(&mut w, &format!("read:{at}"));
    // A read may fail to be made out; read again until it is.
    for _ in 0..20 {
        if severity(&w, player).is_none() {
            break;
        }
        run(&mut w, &format!("read:{at}"));
    }
    assert_eq!(severity(&w, player), None);
    assert!(
        texts(&w)[before..]
            .iter()
            .any(|l| l == "Your limbs move freely again."),
        "{:?}",
        &texts(&w)[before..]
    );
}

#[test]
fn the_heal_skill_treats_it_a_degree_a_try() {
    let mut w = world("stats = { intelligence = 18 }");
    let player = w.state.player;
    let heal: SkillId = w.content().skill_id("heal").expect("Heal");
    let skills = &mut w.state.actors.get_mut(&player).expect("player").skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != heal).collect();
    all.push((
        heal,
        Trained {
            value: 100,
            marks: 0,
        },
    ));
    *skills = all.into_iter().collect();
    run(&mut w, "afflict:stiffening*2");
    for _ in 0..40 {
        if severity(&w, player) == Some(1) {
            break;
        }
        run(&mut w, "heal:stiffening");
    }
    assert_eq!(severity(&w, player), Some(1), "eased, not ended, by a try");
}

#[test]
fn the_basilisk_gazes_stiffening_resisted_by_toughness() {
    let w = world("");
    let content = w.content();
    let basilisk = content.creature(content.creature_id("basilisk").expect("basilisk"));
    let gaze = basilisk.gaze.as_ref().expect("a gaze");
    assert!(basilisk.dread && gaze.averted);
    assert_eq!((gaze.range, gaze.glance), (6, 25));
    assert_eq!(basilisk.speed, 600, "slow: walking away works");
    let stone = gaze.riders.iter().find_map(|r| match r.what {
        crate::content::Affliction::Status {
            status,
            severity,
            check,
        } if status == stiffening(&w) => Some((severity, check)),
        _ => None,
    });
    assert_eq!(stone, Some((1, Some((Stat::Toughness, 15)))));
}

/// The kobold, made a basilisk, stares at the player `n` times, its cooldown cleared each time.
fn stared(w: &mut World, n: usize) {
    let player = w.state.player;
    let kobold = kobold(w);
    let basilisk = w.content().creature_id("basilisk").expect("basilisk");
    w.state.actors.get_mut(&kobold).expect("kobold").creature = basilisk;
    let cooldown = w.content().status_id("gaze").expect("gaze");
    for _ in 0..n {
        w.state
            .actors
            .get_mut(&kobold)
            .expect("it")
            .effects
            .remove(cooldown);
        w.perceive(&mut Vec::new());
        w.creature_gazes(kobold, player, &mut Vec::new());
    }
}

#[test]
fn a_feeble_player_seen_by_the_basilisk_stiffens_a_hardy_one_seldom() {
    let mut feeble = world("stats = { toughness = 3 }");
    stared(&mut feeble, 30);
    let player = feeble.state.player;
    assert!(
        severity(&feeble, player).is_some() || feeble.state.death.is_some(),
        "a glance in thirty, and a check failed"
    );
    // Averted: the basilisk in sight forces it.
    let mut hardy = world("stats = { toughness = 30 }");
    stared(&mut hardy, 30);
    let player = hardy.state.player;
    assert!(hardy.averted(player), "the basilisk in sight: averted");
    assert!(
        severity(&hardy, player).is_none_or(|d| d < 4) && hardy.state.death.is_none(),
        "the check holds it off"
    );
}

/// The kobold, made a gorgon, breathes at the player `n` times, its cooldown cleared each time.
fn breathed(w: &mut World, n: usize) {
    let player = w.state.player;
    let kobold = kobold(w);
    let gorgon = w.content().creature_id("gorgon").expect("gorgon");
    w.state.actors.get_mut(&kobold).expect("kobold").creature = gorgon;
    let cooldown = w.content().status_id("breath").expect("breath");
    for _ in 0..n {
        w.state
            .actors
            .get_mut(&kobold)
            .expect("it")
            .effects
            .remove(cooldown);
        w.perceive(&mut Vec::new());
        let at = w.state.actors[&player].pos;
        w.creature_breathes(kobold, at, &mut Vec::new());
        if w.state.death.is_some() {
            break;
        }
    }
}

#[test]
fn the_gorgons_breath_stiffens_a_feeble_player_and_seldom_a_hardy_one() {
    // #1138: a breath's riders fall on those it harmed, behind their Toughness check.
    let mut feeble = world("stats = { toughness = 3 }");
    breathed(&mut feeble, 2);
    let player = feeble.state.player;
    assert!(
        severity(&feeble, player).is_some() || feeble.state.death.is_some(),
        "two breaths, and a check failed"
    );
    assert!(
        texts(&feeble)
            .iter()
            .any(|l| l == "The gorgon breathes a grey vapour!"),
        "{:?}",
        texts(&feeble)
    );
    let mut hardy = world("stats = { toughness = 30 }");
    breathed(&mut hardy, 3);
    let player = hardy.state.player;
    assert!(
        severity(&hardy, player).is_none_or(|d| d < 3) && hardy.state.death.is_none(),
        "the check holds it off"
    );
}
