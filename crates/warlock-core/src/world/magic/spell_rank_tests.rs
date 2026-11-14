//! Spell ranks (spec §8 "Spells": "Ranks"; #1264): a known spell's grimoire read again deepens
//! it, rank by rank, in rank times the learning time and against a check harder per rank; a
//! failure costs the time alone; the last rank is refused; a class's spell is deepened like any
//! other; what a rank gives joins an own cast's hooks and no scroll's or wand's; the rank is
//! saved, hashed, shown by the dumps, the read list, and the grimoire's description.

use super::caster_tests::{kobold_at, sp};
use super::learn_tests::{
    HALL, count, given, identified, kitted, seeded, set_decipher, world_with,
};
use crate::character::CharacterSpec;
use crate::content::MAX_SPELL_RANK;
use crate::replay::Replay;
use crate::test_support::{last, run};
use crate::time::Duration;
use crate::world::*;

/// A hall with the learning pack, Decipher script at `decipher`, Frost known at `rank`.
fn knowing_frost(decipher: u16, rank: u8) -> World {
    let mut w = identified(world_with(HALL, 10));
    set_decipher(&mut w, decipher);
    w.player_mut().spells.insert(sp("frost"), rank);
    w
}

#[test]
fn a_deepening_is_a_check_harder_per_rank_and_never_certain() {
    let w = knowing_frost(40, 1);
    let odds = |rank: u8| {
        let mut w = w.clone();
        w.player_mut().spells.insert(sp("frost"), rank);
        let me = w.state.player();
        study_odds(me, &w.content, None, sp("frost")).expect("odds")
    };
    // Learning 6 reaches Frost (level 6): a learning would be certain, a deepening is a check.
    let unknown = {
        let mut w = w.clone();
        w.player_mut().spells.remove(&sp("frost"));
        let me = w.state.player();
        study_odds(me, &w.content, None, sp("frost")).expect("odds")
    };
    assert_eq!((unknown.rank, unknown.check), (None, None));
    let second = odds(1);
    assert_eq!(second.rank, Some(2));
    let (bonus, difficulty) = second.check.expect("a check");
    // Frost's 6 known levels: 6 / 20 = 0 points; `per_rank` 2 for the one rank held.
    assert_eq!(difficulty, 12 + 2);
    assert_eq!(odds(4).check, Some((bonus, 12 + 2 * 4)));
    assert_eq!(odds(9).check, Some((bonus, 12 + 2 * 9)));
    assert_eq!(odds(9).rank, Some(10));
    // At the last rank there is nothing to read for.
    let mut w = w.clone();
    w.player_mut().spells.insert(sp("frost"), MAX_SPELL_RANK);
    let me = w.state.player();
    assert_eq!(study_odds(me, &w.content, None, sp("frost")), None);
    assert!(
        w.perception()
            .studies()
            .iter()
            .all(|s| s.spell != sp("frost"))
    );
}

#[test]
fn a_known_spells_grimoire_deepens_it_in_rank_times_the_time() {
    // Decipher script 200: `study` +20, so the check is lost on a natural 2 alone.
    let mut deepened = 0;
    for seed in 0..6 {
        let mut w = identified(seeded(HALL, 10, Some(seed)));
        set_decipher(&mut w, 200);
        w.player_mut().spells.insert(sp("frost"), 1);
        let xp = w.state.experience;
        let out = run(&mut w, "read:c").remove(0);
        // Frost's learning takes 22 minutes; rank 2 twice that.
        assert_eq!(out.elapsed, Duration::minutes(44));
        assert_eq!(count(&w, "grimoire_of_frost"), 2, "kept");
        assert_eq!(w.state.experience, xp, "no experience for a rank");
        if w.state.player().spell_rank(&w.content, sp("frost")) == Some(2) {
            deepened += 1;
            assert_eq!(
                last(&w),
                "The words settle deeper: you know Frost at rank 2 of 10."
            );
        }
    }
    assert!(deepened >= 5, "{deepened}");
    // Rank 9's reading takes nine times the learning, for the last rank.
    let mut w = knowing_frost(200, 9);
    let out = run(&mut w, "read:c").remove(0);
    assert_eq!(out.elapsed, Duration::minutes(22 * 10));
}

#[test]
fn a_failed_deepening_costs_the_time_and_nothing_else() {
    // Learning 1 against Frost's level 6, at rank 9: difficulty 12 + 5 + 18 = 35, past the dice.
    let mut w = knowing_frost(0, 9);
    let (hp, mana) = (w.state.player().hp, w.state.player().mana);
    let before = w.state.player().clone();
    let out = run(&mut w, "read:c").remove(0);
    assert!(out.took_turn);
    assert_eq!(last(&w), "The words add nothing new.");
    let me = w.state.player();
    assert_eq!(me.spell_rank(&w.content, sp("frost")), Some(9));
    assert_eq!((me.hp, me.mana), (hp, mana), "no mishap");
    assert_eq!(me.pos, before.pos, "no teleport");
    assert_eq!(count(&w, "grimoire_of_frost"), 2, "kept");
    // No mishap's words were said.
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        (w.content.mishaps().iter()).all(|m| !said.contains(&m.message)),
        "{said:?}"
    );
    // It may be read again at once: no limit but the time.
    assert!(run(&mut w, "read:c")[0].took_turn);
}

#[test]
fn the_last_rank_is_refused_with_no_time() {
    let mut w = knowing_frost(200, MAX_SPELL_RANK);
    let out = run(&mut w, "read:c").remove(0);
    assert!(!out.took_turn);
    assert_eq!(last(&w), "You know all there is of Frost.");
}

#[test]
fn a_class_spell_is_deepened_like_any_other_and_kept_in_the_map() {
    // The wizard's Ray of frost comes of her class: known at rank 1 with nothing in the map.
    let mut w = kitted("wizard");
    set_decipher(&mut w, 200);
    let ray = sp("ray_of_frost");
    assert!(w.state.player().knows_spell(&w.content, ray));
    assert!(!w.state.player().spells.contains_key(&ray));
    assert_eq!(w.state.player().spell_rank(&w.content, ray), Some(1));
    let item = given(&mut w, "grimoire_of_ray_of_frost");
    let studies = w.perception().studies();
    let study = studies.iter().find(|s| s.item == item).expect("listed");
    assert_eq!(study.rank, Some(2));
    for _ in 0..8 {
        run(&mut w, &format!("read:{item}"));
        if w.state.player().spells.get(&ray) == Some(&2) {
            break;
        }
    }
    assert_eq!(w.state.player().spell_rank(&w.content, ray), Some(2));
    assert!(w.state.player().knows_spell(&w.content, ray));
    // The dev rank reads it too, and a spell not known has none.
    assert_eq!(
        w.state.player().spell_rank(&w.content, sp("fireball")),
        None
    );
}

/// A level-7 wizard at Dexterity 14 (+2) with Ray of frost at `rank`, and a kobold two cells
/// east.
fn wizard_at(rank: u8) -> (World, ActorId) {
    let mut w = kitted("wizard");
    w.player_mut().spells.insert(sp("ray_of_frost"), rank);
    let kobold = kobold_at(&mut w, (2, 0), 1000);
    (w, kobold)
}

#[test]
fn what_a_rank_gives_joins_an_own_cast_and_no_scrolls_or_wands() {
    let content = kitted("wizard").content().clone();
    let ray = content.spell(sp("ray_of_frost"));
    let (plain, _) = wizard_at(1);
    let (deep, _) = wizard_at(MAX_SPELL_RANK);
    let (me, deeper) = (plain.state.player(), deep.state.player());
    // Rank 1: the table gives nothing.
    assert_eq!(me.rank_mods(&content, ray), Default::default());
    // Rank 10: Ray of frost's table summed (its file): +2 to hit, criticals from 20, 1s rolled
    // again, d8s, a die more, two cells farther, a quarter off the mana.
    let mods = deeper.rank_mods(&content, ray);
    assert_eq!((mods.to_hit, mods.threat, mods.reroll), (2, 2, 1));
    assert_eq!(
        (mods.sides, mods.dice, mods.cells, mods.cost),
        (2, 1, 2, -25)
    );
    // The roll: 2d6 at Intelligence 14 (below its 16 rung), grown to 3d8 by the rank.
    let roll = |w: &World, by: CastBy| {
        (w.state.player().spell_roll(&content, ray, by))
            .expect("rolls")
            .to_string()
    };
    assert_eq!(roll(&plain, CastBy::Own), "2d6");
    assert_eq!(roll(&deep, CastBy::Own), "3d8");
    // A scroll or a wand casts at rank 1: its roll is what it was (the wizard's own scroll
    // rungs, Scroll adept's, and all), whatever the rank.
    for by in [CastBy::Scroll(0), CastBy::Wand(0)] {
        assert_eq!(roll(&deep, by), roll(&plain, by), "{by:?}");
    }
    // The reach: Willpower 14 reaches the ladder's 7; the rank adds two.
    let range = |w: &World| w.state.player().spell_reach(&content, ray).range;
    assert_eq!(range(&plain), Some(7));
    assert_eq!(range(&deep), Some(9));
    // The aim and the threat.
    assert_eq!(
        deeper.spell_to_hit(&content, ray),
        me.spell_to_hit(&content, ray) + 2
    );
    assert_eq!(me.spell_threat(&content, ray), 22);
    assert_eq!(deeper.spell_threat(&content, ray), 20);
    // The mana: a quarter off the floor's 4, rounded down with the talents' percents.
    assert_eq!(me.spell_cost(&content, ray), 4);
    assert_eq!(deeper.spell_cost(&content, ray), 3);
    // The menu's words follow (`own_spell_roll`, the Magic page's row).
    let view = deep.perception();
    assert_eq!(view.own_spell_rank(sp("ray_of_frost")), Some(10));
    assert_eq!(
        view.own_spell_roll(sp("ray_of_frost")).map(|r| r.text),
        Some("3d8 cold, numbs".to_string())
    );
}

#[test]
fn a_cast_at_rank_ten_strikes_with_the_ranks_aim_and_dice() {
    // Ray of frost at rank 10 against a kobold: 3d8 cold, so a hit takes 3 to 24.
    let (mut w, kobold) = wizard_at(MAX_SPELL_RANK);
    let here = w.player_pos();
    let mut hits = 0;
    for _ in 0..40 {
        let hp = w.state.actors[&kobold].hp;
        w.player_mut().mana = 40;
        w.apply(&Action::Cast(
            "ray_of_frost".into(),
            Some(Pos::new(here.x + 2, here.y)),
        ));
        let lost = hp - w.state.actors[&kobold].hp;
        if lost > 0 {
            hits += 1;
            assert!((3..=48).contains(&lost), "{lost}: 3d8, twice on a critical");
        }
    }
    assert!(hits > 20, "{hits}");
}

#[test]
fn a_ranks_time_off_shortens_the_cast() {
    // Magic missile's rank 10 halves the casting time: 3 seconds of the turn's 6.
    let mut w = kitted("wizard");
    w.player_mut().spells.insert(sp("magic_missile"), 1);
    let kobold = kobold_at(&mut w, (2, 0), 1000);
    let here = w.player_pos();
    let cast = |w: &mut World| {
        w.player_mut().mana = 40;
        let out = w.apply(&Action::Cast(
            "magic_missile".into(),
            Some(Pos::new(here.x + 2, here.y)),
        ));
        out.elapsed
    };
    let full = cast(&mut w);
    assert_eq!(full, Duration::seconds(6));
    w.player_mut()
        .spells
        .insert(sp("magic_missile"), MAX_SPELL_RANK);
    assert_eq!(cast(&mut w), Duration::seconds(3));
    assert!(w.state.actors[&kobold].hp < 1000);
}

#[test]
fn ranks_are_saved_hashed_and_replayed() {
    let mut w = knowing_frost(200, 3);
    let hash = w.state_hash();
    w.player_mut().spells.insert(sp("frost"), 4);
    assert_ne!(w.state_hash(), hash, "the rank is in the hash");
    let json = serde_json::to_string(w.state.player()).expect("json");
    assert!(
        json.contains(&format!("\"spells\":{{\"{}\":4}}", sp("frost").0)),
        "{json}"
    );
    let content = w.content.clone();
    let loaded = crate::save::decode(&crate::save::encode(&w), content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    // A replay with a deepening in it plays back to the same state: a wizard (Ray of frost her
    // class's) with its grimoire in her pack, read twice.
    let mut files = crate::content::ContentFiles::builtin();
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{HALL}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n"
        ),
    );
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\ncarry = [\"grimoire_of_ray_of_frost\"]\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let wizard: CharacterSpec = "human,female,wizard".parse().expect("a character");
    let mut w = World::with_character(content.clone(), "hall", None, Some(&wizard)).expect("world");
    let mut replay = Replay::new(&w);
    for action in crate::action::parse_actions("read:a wait read:a").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let played = replay.run(content).expect("replays");
    assert_eq!(played.state_hash(), w.state_hash());
}

#[test]
fn the_dumps_the_read_list_and_the_description_say_the_rank() {
    use crate::dump::{json, layer_rows, parse_layers};
    let spells = parse_layers("spells").expect("layer")[0];
    let w = knowing_frost(40, 3);
    let rows = layer_rows(&w, spells);
    assert!(rows[0].starts_with("Frost (level 6, rank 3, "), "{rows:?}");
    let value = json(&w, &[spells], false);
    assert_eq!(value["spells"][0]["id"], "frost");
    assert_eq!(value["spells"][0]["rank"], 3);
    // The read list: the grimoire of frost deepens, to rank 4; the others teach.
    let view = w.perception();
    let studies = view.studies();
    let frost = studies
        .iter()
        .find(|s| s.spell == sp("frost"))
        .expect("listed");
    assert_eq!(frost.rank, Some(4));
    assert!(
        studies
            .iter()
            .filter(|s| s.spell != sp("frost"))
            .all(|s| s.rank.is_none())
    );
    // The grimoire's description: what it teaches, and where the reader stands with it.
    let text = |w: &World, item: usize| -> Vec<String> {
        let stack = w.state.player().inventory[item].clone();
        (w.perception().describe_item(&stack).into_iter())
            .map(|l| l.text)
            .collect()
    };
    let lines = text(&w, 2);
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("Teaches Frost (level 6): ")),
        "{lines:?}"
    );
    assert!(
        lines.contains(
            &"You know it at rank 3 of 10: reading it again would deepen it.".to_string()
        ),
        "{lines:?}"
    );
    let lines = text(&w, 0);
    assert!(
        lines.contains(&"You do not know it: reading it is a study.".to_string()),
        "{lines:?}"
    );
    let w = knowing_frost(40, MAX_SPELL_RANK);
    let lines = text(&w, 2);
    assert!(
        lines.contains(&"You know it at rank 10 of 10: it has nothing more to teach.".to_string()),
        "{lines:?}"
    );
}
