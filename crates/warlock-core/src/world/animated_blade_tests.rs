//! Animated blades (spec §8 "Animated objects", #485): a family of one type per level, its
//! numbers by the quick role, what a blade is made of (a melee weapon of its level, made magic
//! or rare by the tier's odds, wielded with a rank by its level), how it shows, and the family
//! read as one kind by first kills, kill quests, and tag objectives.

use super::fight::Harm;
use super::*;
use crate::content::ContentFiles;
use crate::magic::Rarity;

// A lit hall: the player between a blade of level 5 (west) and one of 12 (east), one of 20
// further on.
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
"a" = { terrain = "floor", creature = "animated_blade_5" }
"b" = { terrain = "floor", creature = "animated_blade_12" }
"c" = { terrain = "floor", creature = "animated_blade_20" }
"##;

const QUESTS: &str = r#"
[[quest]]
id = "blades"
name = "Blades"
giver = "shopkeeper"
type = "kill"
target = "animated_blade"
count = 5
offer = "Blades."
accept = "Good."
reminder = "Blades."
done = "Thanks."

[[quest]]
id = "things"
name = "Things"
giver = "shopkeeper"
type = "kill"
target = "tag:animated"
count = 50
offer = "Things."
accept = "Good."
reminder = "Things."
done = "Thanks."
"#;

fn content() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert("levels/hall.toml", HALL);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\n",
    );
    files.insert("data/quests/blade_test.toml", QUESTS);
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn hall() -> World {
    let mut w = World::new(content(), "hall", None).expect("run");
    w.state.quests = serde_json::from_str(
        r#"{"records": {"blades": {"stage": "active"}, "things": {"stage": "active"}}}"#,
    )
    .expect("records");
    w
}

fn at(w: &World, p: Pos) -> ActorId {
    let level = &w.state.current_level;
    w.state.actor_at(level, p).expect("someone there").id
}

fn id(content: &Content, creature: &str) -> crate::content::CreatureId {
    content.creature_id(creature).expect(creature)
}

#[test]
fn one_type_per_level_in_a_row_one_family() {
    let content = content();
    let first = id(&content, "animated_blade_5");
    for level in 5..=20u8 {
        let c = id(&content, &format!("animated_blade_{level}"));
        assert_eq!(c.0, first.0 + u16::from(level - 5), "in a row");
        let def = content.creature(c);
        assert_eq!((def.name.as_str(), def.level), ("animated blade", level));
        assert_eq!(content.family(c), first);
        assert_eq!(content.family_key(c), "animated_blade");
        assert_eq!(content.family_levels(c), Some((5, 20)));
    }
    // The family's own id names its first member; a household type is its own family.
    assert_eq!(content.creature_id("animated_blade"), Some(first));
    assert!(content.creature_id("animated_blade_4").is_none());
    assert!(content.creature_id("animated_blade_21").is_none());
    let teapot = id(&content, "animated_teapot");
    assert_eq!(content.family(teapot), teapot);
    assert_eq!(content.family_key(teapot), "animated_teapot");
    assert_eq!(content.family_levels(teapot), None);
    // After the household types, before every campaign's.
    let defs = content.creature_defs();
    let last = (content.animated().bodies.last()).map(|b| format!("animated_{}", b.spec.id));
    assert_eq!(Some(&defs[usize::from(first.0) - 1].id), last.as_ref());
    assert!(defs[usize::from(first.0) + 16].campaign.is_some());
}

#[test]
fn the_numbers_follow_the_quick_role() {
    let content = content();
    let def = |c: &str| content.creature(id(&content, c));
    let scores = |c: &str| crate::stats::Stat::ALL.map(|s| def(c).stats.score(s));
    // St Dx To In Wi Pr: the base (11 + 0.4 L, 10 + 0.25 L, 12 + 0.4 L, 2, 16 + 0.35 L, 2), then
    // the quick role's +5 Dx, −2 To.
    assert_eq!(scores("animated_blade_5"), [13, 16, 12, 2, 17, 2]);
    assert_eq!(scores("animated_blade_20"), [19, 20, 18, 2, 23, 2]);
    // Hit points 2.5 × L × 0.6; speed 1200 + 5 L; Defence 2 + L/10 (its thin edge); no attack
    // of its own: it wields.
    let five = def("animated_blade_5");
    assert_eq!((five.base_hp, five.speed, five.xp), (7, 1225, 35));
    let hide = five.natural.as_ref().expect("an edge");
    assert_eq!((hide.defence, hide.protection), (2, 0));
    assert!(five.attack.is_none());
    let twenty = def("animated_blade_20");
    assert_eq!((twenty.base_hp, twenty.speed), (30, 1300));
    assert_eq!(twenty.natural.as_ref().map(|h| h.defence), Some(4));
    // Piercing 50, bludgeoning −25; it floats, falls in pieces, and is uncommon.
    let resist = |kind: &str| {
        let kind = content.damage_type_id(kind).expect(kind);
        (five.resist.iter())
            .find(|(k, _)| *k == kind)
            .map_or(0, |&(_, v)| v)
    };
    assert_eq!((resist("piercing"), resist("bludgeoning")), (50, -25));
    assert_eq!((five.footsteps, five.death_verb()), (0, "falls in pieces"));
    assert_eq!(content.creature_rarity(five.rarity).id, "uncommon");
}

#[test]
fn a_blade_wields_a_melee_weapon_of_its_level_with_a_rank() {
    let w = hall();
    let content = w.content();
    let main = content.slot_id("main_hand").expect("main hand");
    let melee = content.item_class_id("melee_weapons").expect("melee");
    for (p, level) in [(2, 5u8), (4, 12), (6, 20)] {
        let blade = &w.state.actors[&at(&w, Pos::new(p, 2))];
        assert_eq!(blade.inventory.len(), 1, "one weapon");
        let weapon = &blade.inventory[0];
        let def = content.object(weapon.object);
        assert_eq!(blade.equipment.get(main), Some(0), "wielded");
        assert_eq!(def.class, melee);
        assert!(def.level <= level && def.frequency > 0, "{}", def.id);
        // 1 + L/10 ranks in its skill.
        let skill = def.skill.expect("a weapon skill");
        assert_eq!(
            blade.weapon_skills.rank(skill),
            1 + level / 10,
            "{}",
            def.id
        );
        // Shown by its weapon: its glyph, in its material's colour.
        let view = w.perception();
        let seen = view.actor_at(Pos::new(p, 2)).expect("seen");
        assert_eq!(seen.glyph(content), def.glyph);
        assert_eq!(seen.fg(content), weapon.fg(content));
    }
}

#[test]
fn a_blade_is_composed_from_the_actors_own_stream() {
    let (a, b) = (hall(), hall());
    let blade = at(&a, Pos::new(4, 2));
    let actor = &a.state.actors[&blade];
    assert_eq!(actor.inventory, b.state.actors[&blade].inventory);
    let content = a.content();
    let mut rng = crate::animated::stream(a.state.seed, &blade.0.to_string());
    let pack = crate::animated::compose_slots(content, &mut rng, actor.creature);
    assert_eq!(
        (pack.items, pack.equipment),
        (actor.inventory.clone(), actor.equipment.clone())
    );
}

#[test]
fn magic_and_rare_pieces_by_the_tiers_odds() {
    let content = content();
    let count = |creature: &str| {
        let c = id(&content, creature);
        let (mut magic, mut rare) = (0, 0);
        for n in 0..2000 {
            let mut rng = crate::animated::stream(9, &n.to_string());
            let pack = crate::animated::compose_slots(&content, &mut rng, c);
            match pack.items[0].rarity() {
                Rarity::Rare => rare += 1,
                Rarity::Magic => magic += 1,
                _ => {}
            }
        }
        (magic, rare)
    };
    // Level 5: at least magic 10 % (a piece made magic may come out rare by the rarities'
    // weights, as loot's "magic or better" does).
    let (magic, rare) = count("animated_blade_5");
    assert!((140..=260).contains(&(magic + rare)), "{magic} {rare}");
    // Level 20: at least magic 40 %, rare made so 5 % of them all, and more by the weights.
    let (magic, rare) = count("animated_blade_20");
    assert!((700..=900).contains(&(magic + rare)), "{magic} {rare}");
    assert!(rare >= 100 && rare > count("animated_blade_5").1, "{rare}");
}

/// Kills actor `id` outright by the player's hand.
fn kill(w: &mut World, id: ActorId) {
    let player = w.player_id();
    assert!(
        w.hurt(id, 10_000, Harm::Actor(player), &mut Vec::new()),
        "it died"
    );
}

#[test]
fn a_family_is_learnt_of_and_counted_as_one_kind() {
    let mut w = hall();
    let (a, b) = (at(&w, Pos::new(2, 2)), at(&w, Pos::new(4, 2)));
    kill(&mut w, a);
    kill(&mut w, b);
    let learnt = (w.news.iter())
        .filter(|m| *m == "You learn something of animated blades.")
        .count();
    assert_eq!(learnt, 1, "once for the family");
    assert!(w.state.progress.discovered.kills.contains("animated_blade"));
    // Both count toward the family's quest and the tag's; the tally is the family's.
    let quests = w.perception().quests();
    let status = |id: &str| (quests.iter().find(|q| q.id == id)).map(|q| q.status.clone());
    assert_eq!(status("blades").as_deref(), Some("2 / 5 killed."));
    assert_eq!(status("things").as_deref(), Some("2 / 50 killed."));
    assert_eq!(w.state.quests.killed("animated_blade"), 2);
    assert_eq!(w.state.quests.prey(), Some("animated_blade"));
    // The tag's objective names the family once.
    let objective = (quests.iter().find(|q| q.id == "things")).map(|q| q.objective.clone());
    let objective = objective.expect("logged");
    assert_eq!(
        objective.matches("animated blades").count(),
        1,
        "{objective}"
    );
    assert!(objective.contains("animated teapots"), "{objective}");
}

#[test]
fn a_blade_fights_with_its_weapon() {
    let mut w = hall();
    for _ in 0..10 {
        w.apply(&Action::Wait);
    }
    let said: Vec<String> = (w.messages().iter()).map(|m| m.text.clone()).collect();
    assert!(
        said.iter()
            .any(|m| m.starts_with("The animated blade") && m.contains("you")),
        "{said:?}"
    );
}

#[test]
fn a_blade_survives_a_save() {
    let w = hall();
    let loaded = crate::save::decode(&crate::save::encode(&w), content()).expect("decodes");
    assert_eq!(loaded.state_hash(), w.state_hash());
    let blade = at(&w, Pos::new(6, 2));
    assert_eq!(
        loaded.state.actors[&blade].weapon_skills,
        w.state.actors[&blade].weapon_skills
    );
}
