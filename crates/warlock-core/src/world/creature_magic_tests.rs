//! Creatures honour the mana affixes of what they wear and wield (#525, spec §8 "Mana"): a
//! larger pool, a faster pace, and a higher caster level, read by the same hooks as the
//! player's (`Mods::worn`: `Actor::max_mana`, `World::mana_rate`, `World::caster_level`), and a
//! summoner spends from the larger pool. A creature without them is unchanged.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::equipment::Equipment;

/// The shipped content with a test affix that adds to caster level (none ships yet).
fn content() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/affixes/zz_creature_magic.toml",
        "[[prefix]]\nid = \"zz_creature_magic\"\nname = \"Testing\"\n\
         effect = \"caster_level\"\nbases = [\"ring\"]\n\
         tiers = [{ level = 1, min = 1, max = 2 }]\n",
    );
    Arc::new(Content::load(&files).expect("content"))
}

/// The showcase's summoning hall, its goblin wizard stripped of whatever gear it rolled.
fn hall(content: &Arc<Content>) -> (World, ActorId) {
    let mut w = World::new(content.clone(), "summoners", None).expect("the summoning hall");
    let kind = content.creature_id("goblin_wizard").expect("a type");
    let wizard = (w.state.actors.values())
        .find(|a| a.creature == kind)
        .expect("the wizard")
        .id;
    let me = w.state.actors.get_mut(&wizard).expect("the wizard");
    me.inventory.clear();
    me.equipment = Equipment::default();
    me.mana = me.max_mana(content);
    (w, wizard)
}

/// `actor` puts on one `object` with affix `id` at `value` in `slot`, as gear would give it.
fn wear(w: &mut World, actor: ActorId, (object, slot): (&str, &str), (id, value): (&str, i32)) {
    let content = w.content().clone();
    let magic = crate::magic::Magic {
        rarity: crate::magic::Rarity::Magic,
        level: 1,
        affixes: vec![crate::magic::Affix {
            id: id.into(),
            tier: 1,
            value,
        }],
        name: None,
        identified: false,
        carried: 0,
        unique: None,
    };
    let object = content.object_id(object).expect("an object");
    let slot = content.slot_id(slot).expect("a slot");
    let me = w.state.actors.get_mut(&actor).expect("the actor");
    me.inventory.push(Stack::with_magic(object, 1, magic));
    me.equipment.put(&[slot], me.inventory.len() - 1);
}

/// The wizard's maximum mana, mana pace, and caster level.
fn numbers(w: &World, id: ActorId) -> (i32, u32, i32) {
    let me = &w.state.actors[&id];
    (
        me.max_mana(w.content()),
        w.mana_rate(id),
        w.caster_level(id),
    )
}

/// Mana it gets back over `seconds` from none.
fn regained(w: &mut World, id: ActorId, seconds: u64) -> i32 {
    w.state.actors.get_mut(&id).expect("the actor").mana = 0;
    let start = w.time();
    let mut events = Vec::new();
    w.pass_time(std::iter::once(id), start, seconds, &mut events);
    w.state.actors[&id].mana
}

#[test]
fn a_creature_wearing_mana_affixes_has_more_mana_sooner_and_a_higher_caster_level() {
    let content = content();
    let (mut plain, id) = hall(&content);
    let (mut worn, _) = hall(&content);
    wear(&mut worn, id, ("gold_ring", "left_ring"), ("mystic", 5));
    wear(&mut worn, id, ("jade_amulet", "neck"), ("of_clarity", 50));
    wear(
        &mut worn,
        id,
        ("gold_ring", "right_ring"),
        ("zz_creature_magic", 2),
    );

    // Without them it is as its type makes it: its mana and mind, its level, the normal pace.
    let def = content.creature(content.creature_id("goblin_wizard").expect("type"));
    let me = &plain.state.actors[&id];
    let stats = me.effective_stats(&content);
    let base = crate::mana::max_mana(&stats, def.base_mana);
    assert_eq!(numbers(&plain, id), (base, 100, i32::from(def.level)));
    // With them: five more, half again as fast, two levels higher.
    let level = i32::from(def.level) + 2;
    assert_eq!(numbers(&worn, id), (base + 5, 150, level));

    // Half an hour from nothing: a point every `regen_seconds` of its Willpower at its pace.
    let willpower = stats.score(crate::stats::Stat::Willpower);
    let every = u64::from(crate::mana::regen_seconds(willpower));
    let half_hour = 30 * 60;
    let slow = regained(&mut plain, id, half_hour);
    let fast = regained(&mut worn, id, half_hour);
    assert_eq!(
        i64::from(slow),
        i64::try_from(half_hour / every).expect("small")
    );
    assert_eq!(
        i64::from(fast),
        i64::try_from(half_hour * 3 / 2 / every).expect("small")
    );
    assert!(fast > slow);
}

#[test]
fn a_summoner_short_of_mana_summons_with_a_mystic_ring() {
    let content = content();
    let summon = content
        .creature(content.creature_id("goblin_wizard").expect("type"))
        .summon
        .as_ref()
        .expect("a summoner")
        .mana;
    // Its own pool one short of a summoning's cost.
    let short = |w: &mut World, id: ActorId| {
        let me = w.state.actors.get_mut(&id).expect("the wizard");
        me.mana_base -= me.max_mana(&content) - (summon - 1);
        me.mana = me.max_mana(&content);
    };
    let (mut plain, id) = hall(&content);
    short(&mut plain, id);
    let (mut ringed, _) = hall(&content);
    short(&mut ringed, id);
    wear(&mut ringed, id, ("gold_ring", "left_ring"), ("mystic", 5));
    // Made with it on, as its gear would be: it starts with all of the larger pool.
    let me = ringed.state.actors.get_mut(&id).expect("the wizard");
    me.mana = me.max_mana(&content);
    let full = me.mana;

    let walk = "move:W*13 move:NW open:N wait";
    for w in [&mut plain, &mut ringed] {
        for action in parse_actions(walk).expect("actions") {
            w.apply(&action);
        }
    }
    let called = |w: &World| (w.state.actors.values()).any(|a| a.summoner == Some(id));
    assert!(!called(&plain), "one short, it calls nobody");
    assert!(called(&ringed), "the ring's mana pays for it");
    let left = ringed.state.actors[&id].mana;
    assert!(left <= full - summon, "it paid from the larger pool");
}
