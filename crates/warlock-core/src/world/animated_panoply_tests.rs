//! Hollow armour and panoplies (spec §8 "Animated objects", #486): the composites. Their
//! families and numbers by the hollow and standard roles; what they are made of (slot lines tried
//! from their level, a line tried twice for the rings, the shield passed over beside a two-handed
//! weapon, ranks in the weapon's skill and in Sword and board); the powers of what they wear and
//! wield, which work for them as for anyone; and Appraiser's eye, which tells a magic one.

use super::*;
use crate::action::AttackKind;
use crate::content::ContentFiles;
use crate::damage::Damage;
use crate::magic::{Affix, Magic, Rarity};
use crate::object::Stack;

// A lit hall: the player between hollow armour of level 12 (west) and a panoply of level 40
// (east), a panoply of level 70 further on.
const HALL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#########
#.......#
#.a@b.c.#
#.......#
#########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"a" = { terrain = "floor", creature = "animated_armour_12" }
"b" = { terrain = "floor", creature = "animated_panoply_40" }
"c" = { terrain = "floor", creature = "animated_panoply_70" }
"##;

fn content() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert("levels/hall.toml", HALL);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn hall() -> World {
    World::new(content(), "hall", None).expect("run")
}

fn at(w: &World, p: Pos) -> ActorId {
    let level = &w.state.current_level;
    w.state.actor_at(level, p).expect("someone there").id
}

fn id(content: &Content, creature: &str) -> crate::content::CreatureId {
    content.creature_id(creature).expect(creature)
}

/// Object `object` with the affixes `affixes` (id, tier, value), unidentified.
fn magic(content: &Content, object: &str, affixes: &[(&str, u8, i32)]) -> Stack {
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 40,
        affixes: (affixes.iter())
            .map(|&(id, tier, value)| Affix {
                id: id.into(),
                tier,
                value,
            })
            .collect(),
        name: None,
        identified: false,
        carried: 0,
        unique: None,
    };
    let object = content.object_id(object).expect(object);
    (magic.check(content.object(object), content)).expect("a valid item");
    Stack::with_magic(object, 1, magic)
}

fn plain(content: &Content, object: &str) -> Stack {
    Stack::one(content.object_id(object).expect(object))
}

/// Actor `who` wears and wields `kit` (each in its slot) and nothing else.
fn outfit(w: &mut World, who: ActorId, kit: Vec<(Stack, &str)>) {
    let content = w.content().clone();
    let actor = w.state.actors.get_mut(&who).expect("actor");
    actor.inventory.clear();
    actor.equipment = Default::default();
    for (i, (stack, slot)) in kit.into_iter().enumerate() {
        actor.inventory.push(stack);
        let slot = content.slot_id(slot).expect(slot);
        let slots = crate::equipment::slots_taken(&content, actor.inventory[i].object, slot);
        actor.equipment.put(&slots, i);
    }
}

#[test]
fn two_families_after_the_blades_before_every_campaigns() {
    let content = content();
    let blades = id(&content, "animated_blade_20");
    let armour = id(&content, "animated_armour_12");
    let panoply = id(&content, "animated_panoply_20");
    assert_eq!(armour.0, blades.0 + 1);
    assert_eq!(panoply.0, armour.0 + 24);
    for level in 12..=35u8 {
        let c = id(&content, &format!("animated_armour_{level}"));
        assert_eq!(content.family(c), armour);
        assert_eq!(content.family_key(c), "animated_armour");
        assert_eq!(content.creature(c).name, "animated armour");
    }
    for level in 20..=70u8 {
        let c = id(&content, &format!("animated_panoply_{level}"));
        assert_eq!(c.0, panoply.0 + u16::from(level - 20), "in a row");
        assert_eq!(content.family_key(c), "animated_panoply");
        assert_eq!(content.creature(c).name, "animated panoply");
    }
    assert_eq!(content.creature_id("animated_armour"), Some(armour));
    assert_eq!(content.creature_id("animated_panoply"), Some(panoply));
    let defs = content.creature_defs();
    assert!(defs[usize::from(panoply.0) + 51].campaign.is_some());
}

#[test]
fn the_numbers_follow_the_hollow_and_standard_roles() {
    let content = content();
    let def = |c: &str| content.creature(id(&content, c));
    let scores = |c: &str| crate::stats::Stat::ALL.map(|s| def(c).stats.score(s));
    // St Dx To In Wi Pr: the base, then hollow's +5 St, −2 Dx, +5 To; standard's +2 To (at most
    // 40).
    assert_eq!(scores("animated_armour_12"), [20, 11, 21, 2, 20, 2]);
    assert_eq!(scores("animated_armour_35"), [30, 16, 31, 2, 28, 2]);
    assert_eq!(scores("animated_panoply_20"), [19, 15, 22, 2, 23, 2]);
    assert_eq!(scores("animated_panoply_70"), [39, 27, 40, 2, 40, 2]);
    // Hit points 2.5 × L × 1.2; speed 900 + 3 L (hollow), 1000 + 4 a level past 20 to 1200.
    let low = def("animated_armour_12");
    assert_eq!((low.base_hp, low.speed, low.xp), (36, 936, 70));
    let high = def("animated_armour_35");
    assert_eq!((high.base_hp, high.speed), (105, 1005));
    assert_eq!(
        (
            def("animated_panoply_20").base_hp,
            def("animated_panoply_20").speed
        ),
        (60, 1000)
    );
    assert_eq!(
        (
            def("animated_panoply_70").base_hp,
            def("animated_panoply_70").speed
        ),
        (210, 1200)
    );
    // Hollow armour slams with its gauntlets by the curve; a panoply has no blow of its own.
    let slam = low.attack.as_ref().expect("gauntlets");
    assert_eq!(slam.attack.damage, crate::dice::Dice::of(2, 8));
    let slam = high.attack.as_ref().expect("gauntlets");
    assert_eq!(slam.attack.damage, crate::dice::Dice::of(3, 10));
    assert!(def("animated_panoply_40").attack.is_none());
    // Both open doors, step loudly, are shown as `A`, and are medium; piercing passes through
    // the hollow.
    for c in ["animated_armour_20", "animated_panoply_40"] {
        let d = def(c);
        assert!(d.opens_doors, "{c}");
        assert_eq!(
            (d.glyph, d.footsteps, d.death_verb()),
            ('A', 60, "collapses with a clang")
        );
        assert!(d.natural.is_none(), "{c}");
    }
    let piercing = content.damage_type_id("piercing").expect("piercing");
    assert_eq!(low.resist, vec![(piercing, 25)]);
    assert!(def("animated_panoply_40").resist.is_empty());
    assert_eq!(content.creature_rarity(low.rarity).id, "uncommon");
    assert_eq!(
        content
            .creature_rarity(def("animated_panoply_40").rarity)
            .id,
        "rare"
    );
}

/// What a creature of `creature` is made of, rolled `n` times on streams of their own.
fn rolls(content: &Content, creature: &str, n: u32) -> Vec<crate::kit::Pack> {
    let c = id(content, creature);
    (0..n)
        .map(|i| {
            let mut rng = crate::animated::stream(7, &i.to_string());
            crate::animated::compose_slots(content, &mut rng, c)
        })
        .collect()
}

/// The ids of the classes of what `pack` wears and wields, and of what it carries in no slot.
fn worn(content: &Content, pack: &crate::kit::Pack) -> (Vec<String>, usize) {
    let mut classes: Vec<String> = (pack.equipment.iter())
        .map(|(_, i)| {
            content
                .item_class(content.object(pack.items[i].object).class)
                .id
                .clone()
        })
        .collect();
    classes.dedup();
    let loose = (0..pack.items.len())
        .filter(|&i| !pack.equipment.holds(i))
        .count();
    (classes, loose)
}

#[test]
fn slot_lines_are_tried_from_their_level_and_rings_twice() {
    let content = content();
    let has = |packs: &[crate::kit::Pack], class: &str| {
        (packs.iter())
            .filter(|p| worn(&content, p).0.iter().any(|c| c == class))
            .count()
    };
    // Every piece is worn or wielded; armour always wears body armour and never wields.
    let armour = rolls(&content, "animated_armour_25", 300);
    for pack in &armour {
        let (classes, loose) = worn(&content, pack);
        assert_eq!(loose, 0);
        assert!(classes.contains(&"body_armour".to_string()), "{classes:?}");
        assert!(
            !classes.contains(&"melee_weapons".to_string()),
            "{classes:?}"
        );
    }
    assert!(has(&armour, "shields") > 60 && has(&armour, "gloves") > 40);
    // Below 30 no gloves, boots, or cloak; below 50 no amulet or ring.
    let low = rolls(&content, "animated_panoply_29", 300);
    for class in ["gloves", "boots", "cloaks", "amulets", "rings"] {
        assert_eq!(has(&low, class), 0, "{class}");
    }
    let mid = rolls(&content, "animated_panoply_49", 300);
    assert!(has(&mid, "gloves") > 100 && has(&mid, "cloaks") > 100);
    assert_eq!(has(&mid, "amulets") + has(&mid, "rings"), 0);
    // From 50, an amulet and a ring on each hand, each tried on its own.
    let high = rolls(&content, "animated_panoply_70", 600);
    assert!(has(&high, "amulets") > 120, "{}", has(&high, "amulets"));
    let (left, right) = (
        content.slot_id("left_ring").expect("slot"),
        content.slot_id("right_ring").expect("slot"),
    );
    let rings = |p: &crate::kit::Pack| [left, right].map(|s| p.equipment.get(s).is_some());
    let both = high.iter().filter(|p| rings(p) == [true, true]).count();
    let one = high.iter().filter(|p| rings(p) == [true, false]).count();
    assert!((30..=80).contains(&both), "{both}");
    assert!(one > 200, "{one}");
    // A right ring only beside a left one: each further ring goes to the next free slot.
    assert!(!high.iter().any(|p| rings(p) == [false, true]));
}

#[test]
fn no_shield_beside_a_two_handed_weapon() {
    let content = content();
    let (main, off) = (
        content.slot_id("main_hand").expect("slot"),
        content.slot_id("off_hand").expect("slot"),
    );
    let (mut two_handed, mut shielded) = (0, 0);
    for pack in rolls(&content, "animated_panoply_40", 600) {
        let weapon = pack.equipment.get(main).expect("it wields");
        let (classes, loose) = worn(&content, &pack);
        assert_eq!(loose, 0, "nothing carried in no slot");
        if content.object(pack.items[weapon].object).two_handed {
            two_handed += 1;
            assert_eq!(pack.equipment.get(off), Some(weapon), "both hands");
            assert!(!classes.contains(&"shields".to_string()));
        } else if classes.contains(&"shields".to_string()) {
            shielded += 1;
        }
    }
    assert!(two_handed > 30 && shielded > 150, "{two_handed} {shielded}");
}

#[test]
fn ranks_in_the_weapons_skill_and_in_sword_and_board_beside_a_shield() {
    let content = content();
    let board = (content.weapon_skills().id("sword_and_board")).expect("style");
    let shields = content.item_class_id("shields").expect("shields");
    let c = id(&content, "animated_panoply_60");
    let (mut with, mut without) = (0, 0);
    for pack in rolls(&content, "animated_panoply_60", 200) {
        let skills = crate::animated::weapon_skills(&content, c, &pack.items, &pack.equipment)
            .expect("ranks");
        let weapon = (pack.equipment.iter())
            .find_map(|(_, i)| content.object(pack.items[i].object).skill)
            .expect("a weapon");
        // 1 + L/10, at most 8; L/15 in the style beside a shield.
        assert_eq!(skills.rank(weapon), 7);
        let shield = (pack.equipment.iter())
            .any(|(_, i)| content.object(pack.items[i].object).class == shields);
        if shield {
            with += 1;
            assert_eq!(skills.rank(board), 4);
        } else {
            without += 1;
            assert_eq!(skills.rank(board), 0);
        }
    }
    assert!(with > 0 && without > 0);
    // Hollow armour wields nothing and holds no rank.
    let armour = id(&content, "animated_armour_30");
    for pack in rolls(&content, "animated_armour_30", 50) {
        assert!(
            crate::animated::weapon_skills(&content, armour, &pack.items, &pack.equipment)
                .is_none()
        );
    }
}

#[test]
fn a_composite_is_shown_as_a_in_its_armours_colour() {
    let w = hall();
    let content = w.content();
    let body = content.slot_id("body").expect("slot");
    for p in [2, 4, 6] {
        let actor = &w.state.actors[&at(&w, Pos::new(p, 2))];
        let armour = &actor.inventory[actor.equipment.get(body).expect("worn")];
        let view = w.perception();
        let seen = view.actor_at(Pos::new(p, 2)).expect("seen");
        assert_eq!(seen.glyph(content), 'A');
        assert_eq!(seen.fg(content), armour.fg(content));
    }
}

#[test]
fn a_panoply_is_composed_from_the_actors_own_stream() {
    let (a, b) = (hall(), hall());
    let panoply = at(&a, Pos::new(6, 2));
    let actor = &a.state.actors[&panoply];
    assert_eq!(actor.inventory, b.state.actors[&panoply].inventory);
    let content = a.content();
    let mut rng = crate::animated::stream(a.state.seed, &panoply.0.to_string());
    let pack = crate::animated::compose_slots(content, &mut rng, actor.creature);
    assert_eq!(
        (pack.items, pack.equipment),
        (actor.inventory.clone(), actor.equipment.clone())
    );
}

/// The hit points actor `target` loses to `n` melee attacks by `attacker`, kept alive throughout.
fn harm(w: &mut World, attacker: ActorId, target: ActorId, n: usize) -> i32 {
    let mut lost = 0;
    for _ in 0..n {
        w.state.actors.get_mut(&target).expect("target").hp = 10_000;
        w.attack(attacker, target, AttackKind::Melee, &mut Vec::new());
        lost += 10_000 - w.state.actors[&target].hp;
    }
    lost
}

#[test]
fn a_flaming_sword_burns_for_a_panoply() {
    let mut w = hall();
    let content = w.content().clone();
    let (panoply, player) = (at(&w, Pos::new(4, 2)), w.player_id());
    let fire = content.damage_type_id("fire").expect("fire");
    outfit(
        &mut w,
        panoply,
        vec![(plain(&content, "long_sword"), "main_hand")],
    );
    assert!(w.state.actors[&panoply].melee(&content).riders.is_empty());
    let cold = harm(&mut w, panoply, player, 300);
    outfit(
        &mut w,
        panoply,
        vec![(
            magic(&content, "long_sword", &[("flaming", 4, 8)]),
            "main_hand",
        )],
    );
    let profile = w.state.actors[&panoply].melee(&content);
    assert_eq!(profile.riders.len(), 1);
    assert_eq!(profile.riders[0].kind, fire);
    // Its blows carry the fire: the same blows hurt more.
    let burning = harm(&mut w, panoply, player, 300);
    assert!(burning > cold + 300, "{burning} {cold}");
}

#[test]
fn of_the_bear_hits_harder_and_of_the_ox_holds_more_life() {
    let mut w = hall();
    let content = w.content().clone();
    let panoply = at(&w, Pos::new(4, 2));
    outfit(
        &mut w,
        panoply,
        vec![(plain(&content, "long_sword"), "main_hand")],
    );
    let me = |w: &World| w.state.actors[&panoply].clone();
    let (before, life) = (me(&w).melee(&content), me(&w).max_hp(&content));
    outfit(
        &mut w,
        panoply,
        vec![
            (plain(&content, "long_sword"), "main_hand"),
            (
                magic(&content, "iron_ring", &[("of_the_bear", 4, 4)]),
                "left_ring",
            ),
            (
                magic(&content, "gold_amulet", &[("of_the_ox", 4, 4)]),
                "neck",
            ),
        ],
    );
    let after = me(&w).melee(&content);
    assert!(
        after.bonus > before.bonus && after.to_hit >= before.to_hit,
        "{after:?} {before:?}"
    );
    assert!(me(&w).max_hp(&content) > life);
}

#[test]
fn a_resistance_affix_takes_less_for_a_panoply() {
    let mut w = hall();
    let content = w.content().clone();
    let panoply = at(&w, Pos::new(4, 2));
    let fire = content.damage_type_id("fire").expect("fire");
    outfit(
        &mut w,
        panoply,
        vec![(plain(&content, "chain_mail"), "body")],
    );
    assert_eq!(w.resisted(panoply, Damage::new(20, Some(fire))), 20);
    outfit(
        &mut w,
        panoply,
        vec![(
            magic(&content, "chain_mail", &[("of_the_salamander", 4, 40)]),
            "body",
        )],
    );
    assert_eq!(w.resisted(panoply, Damage::new(20, Some(fire))), 12);
}

#[test]
fn a_glowing_ring_lights_a_panoply() {
    let mut w = hall();
    let content = w.content().clone();
    let panoply = at(&w, Pos::new(4, 2));
    let glow = content.light_id("item_glow_faint").expect("glow");
    let glowing = |w: &World| {
        let pos = w.state.actors[&panoply].pos;
        (w.perception().light_sources().iter())
            .any(|s| s.preset() == Some(glow) && s.pos() == Some(pos))
    };
    for p in [4, 6] {
        let who = at(&w, Pos::new(p, 2));
        outfit(&mut w, who, vec![(plain(&content, "plate_armour"), "body")]);
    }
    w.perceive(&mut Vec::new());
    assert!(!glowing(&w));
    outfit(
        &mut w,
        panoply,
        vec![
            (plain(&content, "plate_armour"), "body"),
            (
                magic(&content, "iron_ring", &[("of_light", 1, 0)]),
                "right_ring",
            ),
        ],
    );
    w.perceive(&mut Vec::new());
    assert!(glowing(&w), "it carries the ring's light");
}

#[test]
fn appraisers_eye_tells_a_magic_composite() {
    let mut w = hall();
    let content = w.content().clone();
    let (plain_one, magic_one) = (at(&w, Pos::new(2, 2)), at(&w, Pos::new(4, 2)));
    outfit(
        &mut w,
        plain_one,
        vec![(plain(&content, "chain_mail"), "body")],
    );
    outfit(
        &mut w,
        magic_one,
        vec![
            (plain(&content, "chain_mail"), "body"),
            (
                magic(&content, "long_sword", &[("flaming", 4, 8)]),
                "main_hand",
            ),
        ],
    );
    let hums = |w: &World, p: i32| (w.perception().actor_at(Pos::new(p, 2))).map(|a| a.hums);
    w.perceive(&mut Vec::new());
    // Without the talent nothing tells; with it, the magic one hums and the plain one does not.
    assert_eq!((hums(&w, 2), hums(&w, 4)), (Some(false), Some(false)));
    w.player_mut().talents.take("appraisers_eye", None);
    w.perceive(&mut Vec::new());
    assert_eq!((hums(&w, 2), hums(&w, 4)), (Some(false), Some(true)));
}

#[test]
fn hollow_armour_is_in_fyrdelf_and_a_panoply_nowhere_yet() {
    let content = Content::builtin().expect("content");
    let armour = id(&content, "animated_armour");
    let panoply = id(&content, "animated_panoply");
    let population = |d: &crate::content::DungeonDef| {
        (d.spawning.as_ref()).map_or(Vec::new(), |s| s.population.clone())
    };
    let fyrdelf = content.dungeon("fyrdelf").expect("fyrdelf");
    assert!(population(fyrdelf).contains(&armour));
    assert!(
        !content.dungeons().any(|d| population(d).contains(&panoply)),
        "a panoply waits for a deeper branch"
    );
}

#[test]
fn a_panoply_survives_a_save() {
    let w = hall();
    let loaded = crate::save::decode(&crate::save::encode(&w), content()).expect("decodes");
    assert_eq!(loaded.state_hash(), w.state_hash());
    let panoply = at(&w, Pos::new(6, 2));
    let (a, b) = (&loaded.state.actors[&panoply], &w.state.actors[&panoply]);
    assert_eq!(
        (&a.inventory, &a.weapon_skills),
        (&b.inventory, &b.weapon_skills)
    );
    // A fight with them (the player falls soon), replayed.
    let mut w = hall();
    let mut replay = crate::replay::Replay::new(&w);
    for action in crate::action::parse_actions("wait*4 move:W wait*4").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let (again, divergence) = replay.play(content()).expect("plays");
    assert!(divergence.is_none(), "the replay verifies");
    assert_eq!(again.state_hash(), w.state_hash());
}
