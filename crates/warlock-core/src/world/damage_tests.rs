//! Damage types in the world (spec §8 "Damage types"): a blow does the kind that is better
//! against its target and is said with its verb, Protection stands by kind and never against an
//! element, resistances add up across what a creature is and wears (capped short of immunity),
//! added damage rides along unsoaked, only cuts and stabs bleed, poison meets its resistance as a
//! dose, the plunge is cold, spells and traps are typed, and fights with resistances replay.

use super::*;
use crate::action::{AttackKind, Event};
use crate::combat::{Blow, Strike};
use crate::content::{ContentFiles, DamageTypeId};
use crate::damage::{Damage, IMMUNE, MAX_RESISTANCE, ResistanceWord};
use crate::magic::{Affix, Magic, Rarity};
use crate::object::Stack;
use std::sync::LazyLock;

/// A lit room beside the built-in content: the player (`@`) between a salamander (`s`, immune
/// to fire and poison, with a fiery lick) and a frostling (`f`, half resistant to cold, poison,
/// and slashing, and weak to fire), both hostile and too tough to die in a test.
const MAP: &str = "\
##########
#.@s.f...#
#........#
##########
";

const CREATURES: &str = r##"
[[creature]]
id = "salamander"
level = 1
xp = 1
name = "salamander"
glyph = "s"
fg = "#ff8040"
faction = "vermin"
vision = "normal"
sight = 6
hearing = 20
weight = 9000
base_hp = 200
stats = { strength = 1 }
resist = { fire = 100, poison = 100 }
attack = { damage = "1d2", damage_type = "fire", name = "lick", verb = "scorches" }

[[creature]]
id = "frostling"
level = 1
xp = 1
name = "frostling"
glyph = "f"
fg = "#a0d0ff"
faction = "vermin"
vision = "normal"
sight = 6
hearing = 20
weight = 9000
base_hp = 200
stats = { strength = 1 }
resist = { cold = 50, poison = 50, slashing = 50, fire = -50 }
attack = { damage = "1d2", damage_type = "cold", name = "touch", verb = "chills" }
"##;

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("data/creatures/salamander.toml", CREATURES);
    files.insert(
        "levels/dmgtest.toml",
        &format!(
            "name = \"Kinds\"\nambient = [255, 255, 255, 0]\nmap = '''\n{MAP}'''\n\n\
             [legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"s\" = {{ terrain = \"floor\", creature = \"salamander\" }}\n\
             \"f\" = {{ terrain = \"floor\", creature = \"frostling\" }}\n"
        ),
    );
    files.insert(
        "scenarios/dmgtest.toml",
        "name = \"Kinds\"\nstart_level = \"dmgtest\"\nhp = 60\n\
         carry = [\"scroll_of_fireball\", \"long_sword\", \"club\"]\n",
    );
    files
}

static CONTENT: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

fn world() -> World {
    World::new(CONTENT.clone(), "dmgtest", None).expect("world")
}

fn kind(id: &str) -> DamageTypeId {
    CONTENT.damage_type_id(id).expect(id)
}

fn actor_of(w: &World, id: &str) -> ActorId {
    let creature = w.content().creature_id(id).expect(id);
    (w.state().actors.values())
        .find(|a| a.creature == creature)
        .map(|a| a.id)
        .expect("on the level")
}

/// Puts `items` (stack, slot id) on the player, replacing what they carry.
fn wear(w: &mut World, items: &[(Stack, &str)]) {
    let content = w.content().clone();
    let player = w.state.player;
    let me = w.state.actors.get_mut(&player).expect("the player");
    me.inventory = items.iter().map(|(s, _)| s.clone()).collect();
    me.equipment = Default::default();
    for (i, (_, slot)) in items.iter().enumerate() {
        me.equipment.put(&[content.slot_id(slot).expect(slot)], i);
    }
}

fn one(id: &str) -> Stack {
    Stack::one(CONTENT.object_id(id).expect(id))
}

/// `id` with one affix of `affix` at `tier` rolling `value`, identified or not.
fn magic(id: &str, affix: &str, tier: u8, value: i32, identified: bool) -> Stack {
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 8,
        affixes: vec![Affix {
            id: affix.into(),
            tier,
            value,
        }],
        name: None,
        identified,
        carried: 0,
        unique: None,
    };
    let object = CONTENT.object_id(id).expect(id);
    (magic.check(CONTENT.object(object), &CONTENT)).expect("a valid item");
    Stack::with_magic(object, 1, magic)
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

#[test]
fn the_built_in_content_types_every_source_of_damage() {
    let c = Content::builtin().expect("content");
    assert_eq!(c.unarmed_damage(), c.damage_type_id("bludgeoning"));
    for o in c.object_defs() {
        if let Some(attack) = &o.attack {
            assert!(!attack.kinds.is_empty(), "{} names no damage type", o.id);
        }
        if let Some(bash) = &o.bash {
            assert_eq!(bash.kinds.main(), c.unarmed_damage(), "{}", o.id);
        }
        if let Some(spell) = o.read.as_ref().and_then(|r| r.spell()) {
            let kind = match &spell.cast {
                crate::content::Cast::Burst(b) => b.kind,
                crate::content::Cast::Bolt(b) => b.kind,
            };
            assert!(kind.is_some(), "{} casts untyped damage", o.id);
        }
    }
    for creature in c.creature_defs() {
        if let Some(attack) = &creature.attack {
            assert!(
                !attack.attack.kinds.is_empty(),
                "{} bites untyped",
                creature.id
            );
        }
    }
    for trap in c.trap_defs() {
        if trap.effect.damage().is_some() {
            assert!(
                trap.effect.damage_kind().is_some(),
                "{} is untyped",
                trap.id
            );
        }
    }
    for terrain in c.terrain_defs() {
        if let Some(freeze) = &terrain.freeze {
            assert_eq!(freeze.kind, c.damage_type_id("cold"), "{}", terrain.id);
        }
    }
    // Bleeding comes of cuts and stabs; poison is poison.
    let bleeding = c.status(c.status_id("bleeding").expect("bleeding"));
    let wound = bleeding.wound.as_ref().expect("a wound");
    let cuts = [c.damage_type_id("slashing"), c.damage_type_id("piercing")];
    assert_eq!(
        wound.kinds.iter().map(|k| Some(*k)).collect::<Vec<_>>(),
        cuts
    );
    let poisoned = c.status(c.status_id("poisoned").expect("poisoned"));
    assert_eq!(poisoned.damage_type, c.damage_type_id("poison"));
    // The flaming arrow adds fire; swords cut or stab, axes cleave.
    let arrow = c.object(c.object_id("flaming_arrow").expect("flaming arrow"));
    assert_eq!(arrow.added.map(|r| r.kind), c.damage_type_id("fire"));
    let sword = c.object(c.object_id("long_sword").expect("long sword"));
    let kinds: Vec<_> = sword.attack.expect("a weapon").kinds.iter().collect();
    assert_eq!(kinds, [kind("slashing"), kind("piercing")]);
    let axe = c.object(c.object_id("battle_axe").expect("battle axe"));
    assert_eq!(axe.verb.as_deref(), Some("cleaves"));
    // The damage types join the content fingerprint.
    let mut without = files();
    without.remove("data/damage_types.toml");
    without.edit_table("objects", |text| {
        text.lines()
            .filter(|l| !l.starts_with("damage_type") && !l.starts_with("added"))
            .map(|l| {
                l.split_once(", damage_type")
                    .map_or(l.to_string(), |(a, _)| format!("{a} }}"))
            })
            .collect::<Vec<_>>()
            .join("\n")
    });
    assert!(
        Content::load(&without).is_err(),
        "the rest of the data still names kinds"
    );
}

#[test]
fn a_blow_does_the_kind_that_is_better_and_is_said_with_its_verb() {
    let mut w = world();
    let frostling = actor_of(&w, "frostling");
    let salamander = actor_of(&w, "salamander");
    wear(&mut w, &[(one("long_sword"), "main_hand")]);
    // The frostling resists slashing: the sword stabs it. The salamander resists neither: the
    // sword's first kind, slashing.
    let hits = |w: &mut World, target| {
        let mut kinds = Vec::new();
        for _ in 0..30 {
            let mut events = Vec::new();
            w.state.messages = Default::default();
            w.player_attacks(target, true, &mut events);
            for e in &events {
                if let Event::Attacked {
                    strike: Some(strike),
                    damage,
                    ..
                } = e
                    && strike.is_hit()
                {
                    kinds.push((*damage, said(w).first().cloned().unwrap_or_default()));
                }
            }
        }
        kinds
    };
    let stabs = hits(&mut w, frostling);
    assert!(!stabs.is_empty());
    for (kind_done, text) in &stabs {
        assert_eq!(*kind_done, Some(kind("piercing")));
        assert!(text.starts_with("You stab the frostling"), "{text}");
    }
    let slashes = hits(&mut w, salamander);
    assert!(!slashes.is_empty());
    for (kind_done, text) in &slashes {
        assert_eq!(*kind_done, Some(kind("slashing")));
        assert!(text.starts_with("You slash the salamander"), "{text}");
    }
    // An axe says its own verb; bare hands still "hit".
    wear(&mut w, &[(one("battle_axe"), "main_hand")]);
    let cleaves = hits(&mut w, salamander);
    assert!(cleaves.iter().all(|(_, t)| t.starts_with("You cleave the")));
    wear(&mut w, &[]);
    let fists = hits(&mut w, salamander);
    assert!(fists.iter().all(|(k, t)| {
        *k == CONTENT.unarmed_damage() && t.starts_with("You hit the salamander")
    }));
}

#[test]
fn protection_stands_by_kind_and_never_against_an_element() {
    let mut w = world();
    let player = w.state.player;
    wear(
        &mut w,
        &[(one("chain_mail"), "body"), (one("iron_helm"), "head")],
    );
    let pv = w.protection_of(player);
    assert_eq!(pv.base, 4);
    assert_eq!(pv.against(Some(kind("slashing"))), 5);
    assert_eq!(pv.against(Some(kind("piercing"))), 3);
    assert_eq!(pv.against(Some(kind("bludgeoning"))), 4);
    assert_eq!(pv.against(Some(kind("fire"))), 0);
    assert_eq!(pv.against(None), 4, "untyped harm is soaked in full");
    w.perceive(&mut Vec::new());
    assert_eq!(w.perception().own_protection(), 4, "the total only");
    // The salamander's fiery lick is never soaked: the mail turns nothing of it.
    let salamander = actor_of(&w, "salamander");
    let mut soaked = 0;
    let mut burnt = 0;
    for _ in 0..60 {
        let mut events = Vec::new();
        w.attack(salamander, player, AttackKind::Melee, &mut events);
        match events.iter().find_map(|e| match e {
            Event::Attacked { strike, .. } => *strike,
            _ => None,
        }) {
            Some(Strike::Soaked) => soaked += 1,
            Some(Strike::Hit { .. }) => burnt += 1,
            _ => {}
        }
        w.state.actors.get_mut(&player).expect("player").hp = 60;
    }
    assert!(burnt > 0 && soaked == 0, "{burnt} burnt, {soaked} soaked");
}

#[test]
fn resistances_add_up_across_what_one_is_and_wears_and_cap_short_of_immunity() {
    let mut w = world();
    let player = w.state.player;
    let cold = kind("cold");
    assert_eq!(w.resistances_of(player).get(Some(cold)), 0);
    wear(
        &mut w,
        &[
            (one("fur_cloak"), "cloak"),
            (one("padded_coat"), "body"),
            (magic("gold_amulet", "of_warmth", 9, 30, false), "neck"),
        ],
    );
    assert_eq!(w.resistances_of(player).get(Some(cold)), 60);
    assert_eq!(w.resisted(player, Damage::new(10, Some(cold))), 4);
    assert_eq!(w.resisted(player, Damage::untyped(10)), 10);
    wear(
        &mut w,
        &[
            (one("fur_cloak"), "cloak"),
            (one("padded_coat"), "body"),
            (magic("gold_amulet", "of_warmth", 12, 45, false), "neck"),
            (
                magic("silver_ring", "of_warmth", 12, 45, false),
                "left_ring",
            ),
        ],
    );
    assert_eq!(w.resistances_of(player).get(Some(cold)), MAX_RESISTANCE);
    // Only what a creature is makes it immune: the salamander, whatever it wears or lacks.
    let salamander = actor_of(&w, "salamander");
    assert_eq!(w.resistances_of(salamander).get(Some(kind("fire"))), IMMUNE);
    assert_eq!(
        w.resisted(salamander, Damage::new(18, Some(kind("fire")))),
        0
    );
    let frostling = actor_of(&w, "frostling");
    assert_eq!(w.resisted(frostling, Damage::new(6, Some(kind("fire")))), 9);
    assert_eq!(w.resisted(frostling, Damage::new(6, Some(cold))), 3);
    // The player sees words only, in damage type order; never a creature's.
    w.perceive(&mut Vec::new());
    let view = w.perception();
    assert_eq!(view.own_resistances(), [(cold, ResistanceWord::Resistant)]);
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Stats);
    assert!(rows.contains(&"Resistant to cold".to_string()), "{rows:?}");
    assert!(!rows.iter().any(|r| r.contains('%')), "{rows:?}");
    let json = crate::dump::json(&w, &[], false);
    assert_eq!(json["resistances"]["cold"], "resistant");
}

#[test]
fn added_damage_rides_unsoaked_and_is_resisted_on_its_own() {
    let w = world();
    let content = w.content().clone();
    // A Flaming dagger's blows carry a fire die after the dagger's own.
    let mut me = w.state.player().clone();
    me.inventory = vec![magic("dagger", "flaming", 4, 4, false)];
    me.equipment = Default::default();
    me.equipment
        .put(&[content.slot_id("main_hand").expect("main hand")], 0);
    let blow = me.melee(&content);
    assert_eq!(blow.kinds.main(), Some(kind("piercing")));
    let riders: Vec<_> = blow.riders.iter().map(|r| (r.kind, r.dice)).collect();
    assert_eq!(riders, [(kind("fire"), crate::dice::Dice::of(1, 4))]);
    // A shot of a flaming arrow: the arrow's own fire, before any affix's.
    me.inventory = vec![one("short_bow"), one("flaming_arrow")];
    me.equipment = Default::default();
    me.equipment
        .put(&[content.slot_id("missile_weapon").expect("slot")], 0);
    me.equipment
        .put(&[content.slot_id("missiles").expect("slot")], 1);
    let shot = me.ranged(&content).expect("a shot");
    assert_eq!(shot.kinds.main(), Some(kind("piercing")));
    assert_eq!(shot.riders.len(), 1);
    assert_eq!(shot.riders[0].kind, kind("fire"));
    // Against the salamander the fire does nothing and the point still stabs; against a
    // target in mail the fire is not soaked.
    let mut w = world();
    let salamander = actor_of(&w, "salamander");
    wear(
        &mut w,
        &[(magic("dagger", "flaming", 4, 4, false), "main_hand")],
    );
    let mut events = Vec::new();
    for _ in 0..30 {
        w.player_attacks(salamander, true, &mut events);
    }
    assert!(events.iter().any(|e| matches!(
        e,
        Event::Attacked {
            strike: Some(Strike::Hit { .. }),
            ..
        }
    )));
    assert!(!events.iter().any(|e| matches!(
        e,
        Event::Attacked {
            strike: Some(Strike::Resisted),
            ..
        }
    )));
}

/// Every shipped elemental prefix (#638: Flaming, Freezing, Venomous, Shocking, Corrosive,
/// Withering) at its top tier rides a dagger's blow as one die of its own kind, and lands on
/// every hit against armour that soaks the dagger's point whole: an element is never soaked.
#[test]
fn every_elemental_rider_lands_on_a_hit_and_is_never_soaked() {
    let w = world();
    let content = w.content().clone();
    let elemental: Vec<_> = (content.affixes().all().iter())
        .filter(|a| a.effect == crate::content::Effect::Elemental)
        .collect();
    assert_eq!(elemental.len(), 6, "the six elemental prefixes ship");
    for a in elemental {
        let top = a.tiers.len() - 1;
        let sides = a.tiers[top].max;
        let element = a.damage_type.expect("an elemental affix names its kind");
        let mut me = w.state.player().clone();
        me.inventory = vec![magic("dagger", &a.id, a.tiers.len() as u8, sides, false)];
        me.equipment = Default::default();
        me.equipment
            .put(&[content.slot_id("main_hand").expect("main hand")], 0);
        let mut blow = me.melee(&content);
        let riders: Vec<_> = blow.riders.iter().map(|r| (r.kind, r.dice)).collect();
        assert_eq!(
            riders,
            [(element, crate::dice::Dice::of(1, sides as u32))],
            "{}",
            a.id
        );
        blow.to_hit = 100;
        let mail = crate::combat::Target::protected(crate::combat::Defence::default(), 1000);
        let mut rng = crate::rng::Rng::stream(7, "elemental");
        for _ in 0..50 {
            let hit = crate::combat::strike(&blow, &mail, &mut rng);
            assert!(
                matches!(hit.strike, Strike::Hit { .. }) && (1..=8 * 5).contains(&hit.damage),
                "{}: {hit:?}",
                a.id
            );
        }
    }
}

#[test]
fn only_cuts_and_stabs_bleed() {
    let mut w = world();
    let salamander = actor_of(&w, "salamander");
    let bleeding = CONTENT.status_id("bleeding").expect("bleeding");
    let big = |kind| Blow {
        strike: Strike::Hit { critical: true },
        damage: 100,
        kind,
    };
    let mut events = Vec::new();
    w.wounded(
        salamander,
        &big(Some(kind("bludgeoning"))),
        None,
        &mut events,
    );
    assert!(w.state.actors[&salamander].effects.is_empty(), "a club");
    w.wounded(salamander, &big(Some(kind("fire"))), None, &mut events);
    assert!(w.state.actors[&salamander].effects.is_empty(), "a fireball");
    w.wounded(salamander, &big(Some(kind("piercing"))), None, &mut events);
    assert!(w.state.actors[&salamander].effects.get(bleeding).is_some());
    let frostling = actor_of(&w, "frostling");
    w.wounded(frostling, &big(None), None, &mut events);
    assert!(
        w.state.actors[&frostling].effects.get(bleeding).is_some(),
        "an untyped blow wounds as before"
    );
}

#[test]
fn poison_meets_its_resistance_as_a_dose() {
    let mut w = world();
    let poisoned = CONTENT.status_id("poisoned").expect("poisoned");
    let (salamander, frostling) = (actor_of(&w, "salamander"), actor_of(&w, "frostling"));
    let mut events = Vec::new();
    w.afflict(salamander, poisoned, 5, None, &mut events);
    assert!(w.state.actors[&salamander].effects.is_empty(), "immune");
    w.afflict(frostling, poisoned, 5, None, &mut events);
    let dose = w.state.actors[&frostling].effects.get(poisoned);
    assert_eq!(dose.map(|e| e.severity), Some(3), "5 at 50 %: 2.5, half up");
    // A single degree at 50 % still lands; at 75 % it does not.
    let mut w = world();
    let frostling = actor_of(&w, "frostling");
    w.afflict(frostling, poisoned, 1, None, &mut events);
    assert_eq!(
        w.state.actors[&frostling]
            .effects
            .get(poisoned)
            .map(|e| e.severity),
        Some(1)
    );
    // The player is not immune to anything by nature, so bad food still sickens them.
    let player = w.state.player;
    w.afflict(player, poisoned, 2, None, &mut events);
    assert_eq!(
        w.state.actors[&player]
            .effects
            .get(poisoned)
            .map(|e| e.severity),
        Some(2)
    );
}

#[test]
fn spells_and_traps_are_typed_and_say_when_they_do_no_harm() {
    let mut w = world();
    let salamander = actor_of(&w, "salamander");
    let frostling = actor_of(&w, "frostling");
    let hp = |w: &World, id| w.state.actors[&id].hp;
    let (s0, f0) = (hp(&w, salamander), hp(&w, frostling));
    // The scroll of fireball read at the salamander: it bursts on it, in the player's face.
    let out = w.apply(&"read:a:3,1".parse().expect("read"));
    assert!(out.took_turn);
    let texts = said(&w);
    assert!(
        texts
            .iter()
            .any(|t| t == "The fireball does the salamander no harm."),
        "{texts:?}"
    );
    assert_eq!(hp(&w, salamander), s0, "immune");
    assert!(hp(&w, frostling) < f0, "weak to fire, at the edge");
    let resisted = out.events.iter().any(|e| {
        matches!(
            e,
            Event::Attacked {
                target,
                strike: Some(Strike::Resisted),
                damage,
                ..
            } if *target == salamander && *damage == Some(kind("fire"))
        )
    });
    assert!(resisted, "{:?}", out.events);
    // A burst that the target resists in full bleeds nothing and wounds nothing.
    assert!(w.state.actors[&salamander].effects.is_empty());
}

#[test]
fn the_plunge_is_cold_and_the_words_say_what_a_creature_did() {
    let mut w = world();
    let frostling = actor_of(&w, "frostling");
    let player = w.state.player;
    let cold = CONTENT.damage_type_id("cold");
    // What a fall through the ice does, the frostling takes half of.
    let water = (CONTENT.terrain_defs().iter())
        .find(|t| t.freeze.is_some())
        .expect("water that freezes");
    assert_eq!(water.freeze.expect("freeze").kind, cold);
    assert_eq!(w.resisted(frostling, Damage::new(3, cold)), 2);
    assert_eq!(w.resisted(player, Damage::new(3, cold)), 3);
    // A creature's natural attack is said with its own verb, and shows the kind it did.
    let mut events = Vec::new();
    let mut chilled = false;
    for _ in 0..40 {
        events.clear();
        w.attack(frostling, player, AttackKind::Melee, &mut events);
        if let Some(Event::Attacked {
            strike: Some(Strike::Hit { .. }),
            damage,
            ..
        }) = events.first()
        {
            assert_eq!(*damage, cold);
            chilled = true;
        }
        w.state.actors.get_mut(&player).expect("player").hp = 60;
    }
    assert!(chilled);
    let reports = w.perceive_events(&mut events);
    let texts: Vec<&str> = reports.iter().map(|(t, _)| t.as_str()).collect();
    assert!(
        texts.iter().all(|t| {
            t.starts_with("The frostling chills you")
                || t.starts_with("The frostling misses you")
                || t.starts_with("You dodge the frostling's touch")
        }),
        "{texts:?}"
    );
}

#[test]
fn descriptions_say_kinds_added_damage_and_resistances() {
    let text = |s: &Stack| -> Vec<String> {
        (crate::magic::describe::lines(s, &CONTENT).into_iter())
            .map(|l| l.text)
            .collect()
    };
    let mail = text(&one("chain_mail"));
    assert!(
        mail.contains(&"Protection 3 (+1 against slashing, \u{2212}1 against piercing)".into()),
        "{mail:?}"
    );
    let cloak = text(&one("fur_cloak"));
    assert!(cloak.contains(&"Resists cold 20%".into()), "{cloak:?}");
    let arrow = text(&one("flaming_arrow"));
    assert!(arrow.contains(&"Adds 1d4 fire damage".into()), "{arrow:?}");
    let sword = text(&one("long_sword"));
    assert!(
        sword.contains(&"Slashing or piercing damage".into()),
        "{sword:?}"
    );
    let mace = text(&one("mace"));
    assert!(mace.contains(&"Bludgeoning damage".into()), "{mace:?}");
    // Affixes, once identified.
    let flaming = text(&magic("dagger", "flaming", 4, 4, true));
    assert!(
        flaming.contains(&"Flaming: adds 1d4 fire damage".into()),
        "{flaming:?}"
    );
    let warm = text(&magic("gold_amulet", "of_warmth", 3, 20, true));
    assert!(
        warm.contains(&"of Warmth: +20% cold resistance".into()),
        "{warm:?}"
    );
    let hidden = text(&magic("gold_amulet", "of_warmth", 3, 20, false));
    assert!(!hidden.iter().any(|l| l.contains("cold")), "{hidden:?}");
}

#[test]
fn bad_typed_data_is_refused() {
    let bad = |path: &str, text: &str| {
        let mut files = ContentFiles::builtin();
        files.insert(path, text);
        Content::load(&files).expect_err("refused").to_string()
    };
    let object = |fields: &str| {
        bad(
            "data/objects/zz_test.toml",
            &format!(
                "[[object]]\nid = \"zz_test\"\nname = \"test\"\nglyph = \"x\"\nfg = \"#ffffff\"\n\
                 class = \"melee_weapons\"\nmaterials = [\"iron\"]\nweight = 100\nvalue = 1\n\
                 {fields}\n"
            ),
        )
    };
    for (fields, why) in [
        ("damage_type = \"fire\"", "`damage_type` is for weapons"),
        (
            "slot = \"main_hand\"\ndamage = \"1d4\"\ncategory = \"daggers\"\ndamage_type = \"sonic\"",
            "unknown damage type `sonic`",
        ),
        (
            "slot = \"main_hand\"\ndamage = \"1d4\"\ncategory = \"daggers\"\n\
             damage_type = [\"slashing\", \"piercing\", \"bludgeoning\"]",
            "at most 2",
        ),
        (
            "slot = \"main_hand\"\ndamage = \"1d4\"\ncategory = \"daggers\"\n\
             damage_type = \"piercing\"\nadded = { type = \"slashing\", damage = \"1d4\" }",
            "is physical",
        ),
        (
            "added = { type = \"fire\", damage = \"1d4\" }",
            "`added` is for weapons",
        ),
        (
            "resist = { fire = 20 }",
            "`resist` is for things worn or held",
        ),
        (
            "slot = \"body\"\nprotection = 2\nresist = { fire = 80 }",
            "`fire` must be -100 to 75",
        ),
        (
            "slot = \"body\"\nprotection = 2\nprotects = { fire = 1 }",
            "no physical damage type",
        ),
        (
            "slot = \"body\"\nprotects = { slashing = 1 }",
            "`protects` is for armour",
        ),
        ("verb = \"cleaves\"", "`verb` is for weapons"),
    ] {
        let err = object(fields);
        assert!(err.contains(why), "{fields}: {err}");
    }
    let err = bad(
        "data/affixes/zz_burning.toml",
        "[[prefix]]\nid = \"zz_burning\"\nname = \"Burning\"\neffect = \"elemental\"\ncategory = \"elemental\"\n\
         bases = [\"weapon\"]\ntiers = [{ level = 1, min = 2, max = 3 }]\n",
    );
    assert!(err.contains("names its `damage_type`"), "{err}");
    let err = bad(
        "data/affixes/zz_sharp.toml",
        "[[prefix]]\nid = \"zz_sharp\"\nname = \"Sharper\"\neffect = \"damage\"\ncategory = \"damage\"\n\
         damage_type = \"fire\"\nbases = [\"weapon\"]\ntiers = [{ level = 1, min = 1, max = 2 }]\n",
    );
    assert!(err.contains("`damage_type` is for affixes"), "{err}");
    let err = bad(
        "data/creatures/zz_x.toml",
        "[[creature]]\nid = \"zz_x\"\nname = \"x\"\nglyph = \"x\"\nfg = \"#ffffff\"\n\
         weight = 1000\nresist = { fire = 101 }\n",
    );
    assert!(err.contains("`fire` must be -100 to 100"), "{err}");
    let mut files = ContentFiles::builtin();
    files.edit_table("status", |t| {
        t.replace("from = [\"slashing\", \"piercing\"]", "from = [\"sonic\"]")
    });
    let err = Content::load(&files).expect_err("refused").to_string();
    assert!(err.contains("wound: from: unknown damage type"), "{err}");
}

#[test]
fn fights_with_resistances_replay_exactly() {
    // The same fight twice: the kinds chosen, the riders rolled, and the resistances taken
    // off leave the two runs identical.
    let play = || {
        let mut w = world();
        wear(
            &mut w,
            &[(magic("dagger", "flaming", 4, 4, false), "main_hand")],
        );
        for action in crate::action::parse_actions("move:E*10 wait*4").expect("actions") {
            w.apply(&action);
        }
        w
    };
    let (a, b) = (play(), play());
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(a.messages(), b.messages());
    // And a replay from the scenario's start verifies.
    let mut w = world();
    let mut replay = crate::replay::Replay::new(&w);
    for action in crate::action::parse_actions("move:E*10 wait*4").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let verified = (crate::replay::Replay::from_json(&replay.to_json()))
        .expect("json")
        .run(CONTENT.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}
