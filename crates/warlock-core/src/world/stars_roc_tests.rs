//! The birth sign's gifts from the Roc to the Barghest (spec §8 "Birth sign"): each changes the
//! one rule it names, and the Phoenix rises from its ashes once in a life.

use super::*;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, Sign, TalentEffect};
use crate::dump::{self, Layer};
use crate::magic::loot::Luck;
use crate::object::Stack;
use crate::perception::Freshness;
use crate::rng::Rng;
use crate::stats::{PerStat, Stats};
use crate::time::Duration;
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

/// A human warrior in `scenario` born under the `element` `animal` (`None`: under no sign), the
/// run restored with that birth, so the gifts are granted as a load grants them.
fn born_in(content: Arc<Content>, scenario: &str, sign: Option<(&str, &str)>) -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let w = World::with_character(content.clone(), scenario, Some(7), Some(&spec)).expect("world");
    let mut state = w.state().clone();
    let character = state.character.as_mut().expect("a character");
    character.sign = sign.map(|(animal, element)| Sign {
        animal: animal.into(),
        element: element.into(),
    });
    // Never a lucky day: the 2nd is none's.
    character.born.day = 2;
    World::from_state(content, state).expect("restores")
}

fn born_under(animal: &str, element: &str) -> World {
    born_in(BUILTIN.clone(), "showcase", Some((animal, element)))
}

fn unborn() -> World {
    born_in(BUILTIN.clone(), "showcase", None)
}

fn mods(w: &World) -> crate::talent::TalentMods {
    w.state.player().talent_mods(w.content()).into_owned()
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

/// The gifts held, by id, in grant order (a class's abilities left out).
fn gifts(w: &World) -> Vec<String> {
    let book = w.content().talents();
    (w.state.player().talents.iter())
        .filter(|t| book.id(&t.id).is_some_and(|id| book.get(id).granted))
        .filter(|t| {
            book.id(&t.id)
                .is_none_or(|id| !book.tree(book.get(id).tree).class)
        })
        // A race's grant (the humans' Versatile) is no gift of the sign.
        .filter(|t| {
            book.id(&t.id)
                .is_none_or(|id| Some(book.get(id).tree) != book.tree_id("racial"))
        })
        .map(|t| t.id.clone())
        .collect()
}

#[test]
fn the_animals_from_the_roc_to_the_barghest_grant_their_gifts_and_flaws() {
    // The animal's gifts only: a Fire year adds the element's Blaze and Kindled after them.
    let animal = |id: &str| {
        let mut held = gifts(&born_under(id, "fire"));
        assert_eq!(held.split_off(held.len() - 2), ["blaze", "kindled"]);
        held
    };
    assert_eq!(
        animal("roc"),
        ["rocs_strength", "eyes_of_the_heights", "heedless"]
    );
    assert_eq!(
        animal("griffin"),
        ["hoard_sense", "watchful", "set_in_ways"]
    );
    assert_eq!(animal("sphinx"), ["riddlers_mind", "old_lore", "ponderous"]);
    assert_eq!(
        animal("phoenix"),
        ["rise_from_ashes", "inner_light", "frail"]
    );
    assert_eq!(
        animal("kraken"),
        ["grasping_arms", "of_the_deep", "ungainly_ashore"]
    );
    assert_eq!(
        animal("barghest"),
        ["hounds_nose", "gatekeeper", "grim_company"]
    );
    // The flaws are flaws.
    let book = BUILTIN.talents();
    for flaw in ["set_in_ways", "frail", "ungainly_ashore"] {
        assert!(book.get(book.id(flaw).expect(flaw)).flaw, "{flaw}");
    }
}

/// How many cells the player sees.
fn seen(w: &World) -> usize {
    (dump::layer_rows(w, Layer::Fov).iter())
        .map(|row| row.matches('*').count())
        .sum()
}

#[test]
fn eyes_of_the_heights_see_farther_on_the_overworld_by_day_and_by_night() {
    let roc = |sign| born_in(BUILTIN.clone(), "world", sign);
    let (mut high, mut low) = (roc(Some(("roc", "fire"))), roc(None));
    // Clear weather, so the sky is the same and the difference is the gift's.
    for w in [&mut high, &mut low] {
        w.apply(&"weather:clear".parse::<Action>().expect("an action"));
        w.apply(&"skip:4h".parse::<Action>().expect("an action"));
    }
    assert!(daylight_phase(&high) == crate::daylight::DayPhase::Day);
    assert!(seen(&high) > seen(&low), "farther by day");
    for w in [&mut high, &mut low] {
        w.apply(&"skip:12h".parse::<Action>().expect("an action"));
    }
    assert!(daylight_phase(&high) == crate::daylight::DayPhase::Night);
    assert!(seen(&high) > seen(&low), "farther by night");
    let m = mods(&high);
    assert_eq!((m.day_sight, m.night_sight), (2, 1));
}

fn daylight_phase(w: &World) -> crate::daylight::DayPhase {
    crate::daylight::phase(w.state.time)
}

#[test]
fn hoard_sense_finds_magic_and_gems_more_often_from_the_same_draws() {
    let luck = Luck::of(&mods(&born_under("griffin", "fire")));
    assert_eq!(
        luck,
        Luck {
            magic: 50,
            gems: 100
        }
    );
    let content = &*BUILTIN;
    let gems = |luck: Luck| {
        let mut rng = Rng::gameplay(3);
        (0..4000)
            .filter_map(|_| {
                crate::magic::loot::roll_with(
                    content,
                    &mut rng,
                    6,
                    crate::magic::LootGrade::Any,
                    crate::variant::Source::Loot,
                    luck,
                )
            })
            .filter(|s| s.gem(content).is_some())
            .count()
    };
    let (plain, lucky) = (gems(Luck::default()), gems(luck));
    assert!(
        lucky > plain * 3 / 2,
        "{plain} gems, {lucky} with hoard sense"
    );
    // The map is the same: only what the loot rolls changes.
    let id = "depths:2";
    let plain = crate::mapgen::generate(content, ("showcase", 5), id, None).expect("level");
    let lucky =
        crate::mapgen::generate_with(content, ("showcase", 5), id, None, luck).expect("level");
    assert_eq!(plain.terrain, lucky.terrain);
}

#[test]
fn watchful_hears_fainter_and_places_footsteps_exactly() {
    let w = born_under("griffin", "fire");
    let m = mods(&w);
    assert!(m.exact_footsteps);
    let threshold = |w: &World| w.state.player().hearing(w.content()).expect("hears");
    assert_eq!(threshold(&w), threshold(&unborn()) - 3);
}

#[test]
fn those_set_in_their_ways_take_a_moment_to_change_stance() {
    let stance = "stance:aggressive".parse::<Action>().expect("an action");
    let mut free = unborn();
    let before = free.state.time;
    free.apply(&stance);
    assert_eq!(free.state.time, before, "free for others");
    let mut set = born_under("griffin", "fire");
    let before = set.state.time;
    let wait = set.price_of(set.state.player, crate::content::ActionKind::Wait);
    assert!(wait > 0);
    set.apply(&stance);
    assert!(set.state.time > before, "a wait's time");
    assert_eq!(
        set.state.player().stance,
        crate::content::CombatStance::Aggressive
    );
    assert!(
        said(&set)
            .iter()
            .any(|m| m == "You take an aggressive stance.")
    );
}

#[test]
fn old_lore_identifies_what_is_carried_a_day_and_reads_quickly() {
    let m = mods(&born_under("sphinx", "fire"));
    assert_eq!(m.identify_carried, Some(24));
    assert_eq!(m.read_percent, -50);
}

/// A creature of type `creature` with `hp` hit points beside the player, to the east.
fn beside(world: &mut World, creature: &str, hp: i32) -> ActorId {
    let content = world.content().clone();
    let at = world.player_pos().step(crate::geom::Dir::E);
    let template = (world.state.actors.values())
        .find(|a| a.id != world.state.player)
        .cloned()
        .expect("a creature in the showcase");
    let id = ActorId(world.state.next_actor_id);
    world.state.next_actor_id += 1;
    let creature = content.creature_id(creature).expect("a creature type");
    let def = content.creature(creature);
    let actor = Actor {
        id,
        creature,
        pos: at,
        patrol: None,
        npc: None,
        faction_override: None,
        alignment_override: None,
        inventory: Vec::new(),
        equipment: Default::default(),
        stats: def.stats,
        skills: def.skills.clone(),
        weapon_skills: def.weapon_skills.clone(),
        effects: Default::default(),
        hp_base: hp,
        hp,
        level: world.state.current_level.clone(),
        ..template
    };
    world.state.actors.insert(id, actor);
    id
}

fn ashen(w: &World) -> bool {
    let status = w.content().status_id("ashen").expect("ashen");
    w.state.player().effects.has(status)
}

#[test]
fn the_phoenix_rises_from_its_ashes_once_in_a_life() {
    let mut w = born_under("phoenix", "fire");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(18));
    let player = w.state.player;
    let kobold = beside(&mut w, "kobold", 50);
    let mut events = Vec::new();
    // A blow that would kill: the Phoenix rises instead, Ashen, with a third of its hit points.
    assert!(!w.hurt(player, 999, fight::Harm::Actor(kobold), &mut events));
    assert!(w.state.death.is_none());
    assert!(ashen(&w));
    let max = w.state.player().max_hp(w.content());
    assert_eq!(w.state.player().hp, (max * 33 / 100).max(1));
    assert!(w.state.progress.happened(RISEN));
    assert!(said(&w).contains(&"You burst into flame and rise from your ashes!".to_string()));
    // Flames burst from them: the effect over the cells around, and the kobold beside burns.
    assert!(events.iter().any(|e| matches!(
        e,
        Event::Spell { effect, shape: crate::action::SpellShape::Burst { radius: 1, .. }, .. }
            if effect == "phoenix_fire"
    )));
    assert!(w.state.actors[&kobold].hp < 50, "the kobold is burned");
    // The Talents page and the dump say it is spent.
    let rows = dump::layer_rows(&w, Layer::Talents);
    assert!(
        rows.contains(&"Rise from the ashes (sign, spent)".to_string()),
        "{rows:?}"
    );
    // Saved and restored, it stays spent: the next deadly blow kills.
    let restored = World::from_state(w.content().clone(), w.state().clone()).expect("restores");
    assert!(restored.state.progress.happened(RISEN));
    assert!(w.hurt(player, 999, fight::Harm::Actor(kobold), &mut events));
    assert!(w.state.death.is_some());
}

#[test]
fn the_phoenix_still_starves_and_others_never_rise() {
    let mut w = born_under("phoenix", "fire");
    let player = w.state.player;
    let when = w.state.time;
    assert!(w.hurt(player, 999, fight::Harm::Starving(when), &mut Vec::new()));
    assert!(w.state.death.is_some(), "starving kills");
    assert!(!w.state.progress.happened(RISEN));
    let mut other = unborn();
    let player = other.state.player;
    let kobold = beside(&mut other, "kobold", 5);
    assert!(other.hurt(player, 999, fight::Harm::Actor(kobold), &mut Vec::new()));
    // And a run without the gift writes nothing of it.
    assert!(other.state.progress.once.is_empty());
}

#[test]
fn the_phoenix_is_frail() {
    let w = born_under("phoenix", "fire");
    let mut whole = w.state.player().clone();
    whole.talents = Default::default();
    let content = w.content();
    let full = whole.max_hp(content);
    assert_eq!(w.state.player().max_hp(content), (full * 90 / 100).max(1));
}

/// A dark room (no light at all) with the player in it.
fn dark_room() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/cellar.toml",
        "name = \"Cellar\"\nambient = [0, 0, 0, 0]\nmap = '''\n#######\n#.....#\n#..@..#\n\
         #.....#\n#######\n'''\n[legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/cellar.toml",
        "name = \"Cellar\"\nstart_level = \"cellar\"\n",
    );
    Arc::new(Content::load(&files).expect("test content loads"))
}

#[test]
fn inner_light_shows_the_cells_beside_in_the_dark() {
    let content = dark_room();
    let lit = born_in(content.clone(), "cellar", Some(("phoenix", "fire")));
    let dark = born_in(content, "cellar", None);
    let next = lit.player_pos().step(crate::geom::Dir::E);
    assert!(
        lit.perception().light(next).is_some(),
        "the ember lights it"
    );
    assert!(dark.perception().light(next).is_none(), "dark without it");
    // Two cells away it is dark again: the ember reaches one.
    let far = next.step(crate::geom::Dir::E);
    assert!(lit.perception().light(far).is_none());
}

#[test]
fn the_kraken_fights_with_many_arms_and_is_of_the_deep() {
    let w = born_under("kraken", "fire");
    let content = w.content();
    let m = mods(&w);
    assert!(m.light_on_ice && m.plunge_proof);
    assert_eq!((m.climb_percent, m.travel_percent), (-50, 10));
    let twin = content
        .weapon_skills()
        .id("twin_blades")
        .expect("twin blades");
    let base = unborn().state.player().benefit(content, Some(twin));
    let kraken = w.state.player().benefit(content, Some(twin));
    assert_eq!(kraken.ease_main, base.ease_main + 3);
    assert_eq!(kraken.ease_off, base.ease_off + 3);
}

#[test]
fn hounds_nose_smells_blood_and_rot() {
    let mut w = born_under("barghest", "fire");
    let content = w.content().clone();
    let bleeding = content.status_id("bleeding").expect("bleeding");
    assert_eq!(mods(&w).scent, [(bleeding, 6)]);
    assert!(mods(&w).free_blows.gatekeeper);
    let bread = content.object_id("bread").expect("bread");
    let now = w.state.time;
    let made = |days: u64| {
        let mut stack = Stack::one(bread);
        stack.died = Some(crate::time::GameTime::from_seconds(
            now.seconds() - Duration::hours(24 * days).as_seconds(),
        ));
        stack
    };
    w.player_mut().inventory.clear();
    let view = w.perception();
    assert_eq!(view.smell(&made(1)), Some(Freshness::Fresh));
    assert_eq!(view.smell(&made(3)), Some(Freshness::Turning));
    assert_eq!(view.smell(&made(6)), Some(Freshness::Rotten));
    // A dagger smells of nothing, and those without the nose smell nothing.
    let dagger = Stack::one(content.object_id("dagger").expect("dagger"));
    assert_eq!(view.smell(&dagger), None);
    assert_eq!(unborn().perception().smell(&made(6)), None);
}

#[test]
fn the_rules_of_the_new_gifts_are_in_their_talents() {
    // Every gift of this wave rides an effect kind of its own or one that was there.
    let book = BUILTIN.talents();
    let effects = |id: &str| -> Vec<TalentEffect> {
        let def = book.get(book.id(id).expect(id));
        def.effects_at(1).collect()
    };
    assert!(matches!(
        effects("rise_from_ashes").as_slice(),
        [TalentEffect::RiseFromAshes {
            hp_percent: 33,
            radius: 1,
            ..
        }]
    ));
    assert!(matches!(
        effects("gatekeeper").as_slice(),
        [TalentEffect::FreeBlow {
            trigger: crate::content::FreeBlowTrigger::Gatekeeper
        }]
    ));
}
