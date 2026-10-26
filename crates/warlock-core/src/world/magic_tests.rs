//! Magic items in play (spec §8 "Magic items"): every effect applies through the rules while
//! the item is worn or wielded, identified or not; identifying reveals the affixes and nothing
//! else leaks them; items survive saves and replays; creatures drop loot and generated levels hold
//! it, deterministically.

use super::*;
use crate::action::{AttackKind, Positions, parse_actions};
use crate::combat::{self, Category, Profile};
use crate::content::{ContentFiles, Effect};
use crate::magic::{Affix, Magic, Rarity};
use crate::noise::Act;
use crate::replay::Replay;
use crate::stats::Stat;

// A lit room: the player, and a kobold behind a door.
const LEVEL: &str = r##"
name = "Vault"
ambient = [255, 255, 255, 0]
item_level = 3
map = '''
#########
#@......#
#.......#
#.....k.#
#########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"##;

fn content() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    // At peace: the kobold stands by.
    files.insert(
        "data/factions.toml",
        "[[faction]]\nid = \"player\"\nname = \"you\"\n\
         [[faction]]\nid = \"vermin\"\nname = \"vermin\"\n\
         [[faction]]\nid = \"kobolds\"\nname = \"kobolds\"\n\
         [[faction]]\nid = \"goblins\"\nname = \"goblins\"\n\
         [[faction]]\nid = \"orcs\"\nname = \"orcs\"\n\
         [[faction]]\nid = \"undead\"\nname = \"undead\"\n\
         [[faction]]\nid = \"marsh\"\nname = \"marsh things\"\n\
         [[faction]]\nid = \"cult\"\nname = \"cultists\"\n\
         [[faction]]\nid = \"hags\"\nname = \"hags\"\n\
         [[faction]]\nid = \"duergar\"\nname = \"duergar\"\n\
         [[faction]]\nid = \"giant\"\nname = \"giants\"\n\
         [[faction]]\nid = \"troll\"\nname = \"trolls\"\n\
         [[faction]]\nid = \"fire\"\nname = \"fire things\"\n\
         [[faction]]\nid = \"demon\"\nname = \"demons\"\n\
         [[faction]]\nid = \"elemental\"\nname = \"elementals\"\n\
         [[faction]]\nid = \"seelie\"\nname = \"Seelie court\"\n\
         [[faction]]\nid = \"unseelie\"\nname = \"Unseelie court\"\n\
         [[faction]]\nid = \"fiends\"\nname = \"fiends\"\n\
         [[faction]]\nid = \"devil\"\nname = \"devils\"\n\
         [[faction]]\nid = \"yugoloth\"\nname = \"yugoloths\"\n\
         [[faction]]\nid = \"wildlife\"\nname = \"wildlife\"\n\
         [[faction]]\nid = \"peaceful\"\nname = \"peaceful folk\"\n\
         [[faction]]\nid = \"merchants\"\nname = \"merchants\"\n\
         [[faction]]\nid = \"outraged\"\nname = \"outraged\"\nhostile = [\"player\"]\n",
    );
    files.insert("levels/vault.toml", LEVEL);
    files.insert(
        "scenarios/vault.toml",
        "name = \"Vault\"\nstart_level = \"vault\"\n\
         stats = { strength = 12, dexterity = 14, toughness = 10 }\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn world() -> World {
    World::new(content(), "vault", None).expect("world")
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions parse") {
        w.apply(&action);
    }
}

/// One `object` with the affixes `(id, value)` (each tier 1), magic or (with a name) rare.
fn magic(w: &World, object: &str, affixes: &[(&str, i32)], identified: bool) -> Stack {
    let c = w.content();
    let rare = affixes.len() > 2;
    let magic = Magic {
        rarity: if rare { Rarity::Rare } else { Rarity::Magic },
        level: 1,
        affixes: (affixes.iter())
            .map(|(id, value)| Affix {
                id: (*id).into(),
                tier: 1,
                value: *value,
            })
            .collect(),
        name: rare.then(|| "Grim Bite".into()),
        identified,
        carried: 0,
        unique: None,
    };
    let id = c.object_id(object).expect("object");
    Stack::with_magic(id, 1, magic)
}

fn plain(w: &World, object: &str) -> Stack {
    Stack::one(w.content().object_id(object).expect("object"))
}

/// The player carries `stack` and wears or wields it in `slot`; returns its letter.
fn wear(w: &mut World, slot: &str, stack: Stack) -> usize {
    let content = w.content().clone();
    let player = w.state.player;
    let actor = w.state.actors.get_mut(&player).expect("player");
    actor.inventory.push(stack);
    let item = actor.inventory.len() - 1;
    let slot = content.slot_id(slot).expect("slot");
    let object = actor.inventory[item].object;
    let taken = crate::equipment::slots_taken(&content, object, slot);
    for &s in &taken {
        if let Some(other) = actor.equipment.get(s) {
            actor.equipment.take_off(other);
        }
    }
    actor.equipment.put(&taken, item);
    item
}

/// The player carries `stack` without wearing it.
fn carry(w: &mut World, stack: Stack) {
    let player = w.state.player;
    w.state
        .actors
        .get_mut(&player)
        .expect("player")
        .inventory
        .push(stack);
}

fn me(w: &World) -> &Actor {
    w.state().player()
}

fn defence(w: &World) -> combat::Defence {
    w.defence_of(w.state.player)
}

#[test]
fn attack_effects_on_a_weapon_change_its_blows() {
    let mut plain_w = world();
    let sword = plain(&plain_w, "long_sword");
    wear(&mut plain_w, "main_hand", sword);
    let base = me(&plain_w).melee(plain_w.content());

    let mut w = world();
    let sword = magic(&w, "long_sword", &[("sharp", 1), ("of_precision", 1)], true);
    wear(&mut w, "main_hand", sword);
    let p = me(&w).melee(w.content());
    assert_eq!((p.to_hit, p.bonus), (base.to_hit + 1, base.bonus + 1));

    let mut w = world();
    let sword = magic(&w, "long_sword", &[("heavy", 3), ("keen", 1)], true);
    wear(&mut w, "main_hand", sword);
    let p = me(&w).melee(w.content());
    assert_eq!(p.extra.map(|d| d.to_string()), Some("1d3".into()));
    assert_eq!(
        p.threat,
        base.threat - 1,
        "keen: criticals threaten on one roll more"
    );

    let mut w = world();
    let sword = magic(&w, "long_sword", &[("guarding", 1)], true);
    wear(&mut w, "main_hand", sword);
    assert_eq!(defence(&w).bonus(Category::Parry), 1);
}

#[test]
fn global_attack_effects_change_every_attack() {
    let mut w = world();
    let base = me(&w).melee(w.content());
    let ring = magic(&w, "iron_ring", &[("accurate", 1), ("of_slaying", 1)], true);
    wear(&mut w, "left_ring", ring);
    let gloves = magic(
        &w,
        "leather_gloves",
        &[("vicious", 1), ("of_ruin", 1)],
        true,
    );
    wear(&mut w, "hands", gloves);
    let p = me(&w).melee(w.content());
    assert_eq!(p.to_hit, base.to_hit + 1, "bare hands, accurate");
    assert_eq!(p.bonus, base.bonus + 1);
    assert_eq!(p.threat, base.threat - 1);
    assert_eq!(p.crit, base.crit + 1);
}

/// What a blow with the wielded weapon costs the player.
fn blow_price(w: &World) -> i64 {
    let skills = me(w).attack_skills(w.content(), AttackKind::Melee, None);
    let with = me(w).weapon_item(w.content());
    w.attack_price(w.state.player, ActionKind::Melee, with, skills)
}

/// What a bash with the shield held costs the player.
fn bash_price(w: &World) -> i64 {
    let shield = me(w).shield(w.content()).map(|(s, _)| s);
    let skills = me(w).attack_skills(w.content(), AttackKind::Melee, shield);
    let with = me(w).shield_item(w.content());
    assert!(with.is_some(), "a shield to bash with");
    w.attack_price(w.state.player, ActionKind::Melee, with, skills)
}

#[test]
fn speed_effects_make_attacks_and_steps_quicker() {
    let mut w = world();
    let player = w.state.player;
    let sword = plain(&w, "long_sword");
    wear(&mut w, "main_hand", sword);
    let melee = blow_price(&w);
    let floor = w
        .content()
        .terrain(w.content().terrain_id("floor").expect("floor"))
        .clone();
    let step = w.step_price(player, &floor, false);
    let sword = magic(&w, "long_sword", &[("of_celerity", 8)], true);
    wear(&mut w, "main_hand", sword);
    assert_eq!(blow_price(&w), melee * 92 / 100);
    let boots = magic(&w, "leather_boots", &[("fleet", 8)], true);
    wear(&mut w, "feet", boots);
    assert_eq!(w.step_price(player, &floor, false), step * 92 / 100);
    // A great sword is slower, by its own attack cost.
    let great = plain(&w, "great_sword");
    wear(&mut w, "main_hand", great);
    assert_eq!(blow_price(&w), melee * 12 / 10);
}

#[test]
fn a_shield_bash_costs_a_blow_with_the_shield() {
    let mut w = world();
    let shield = plain(&w, "small_shield");
    wear(&mut w, "off_hand", shield);
    let plain_bash = bash_price(&w);
    // Neither the wielded weapon's own cost nor its affixes change a bash: a rapier's blows
    // are quicker, and so are those of a sword of celerity, but the shield is neither.
    for (object, affixes) in [
        ("rapier", &[][..]),
        ("long_sword", &[("of_celerity", 8)][..]),
    ] {
        let weapon = magic(&w, object, affixes, true);
        wear(&mut w, "main_hand", weapon);
        assert!(blow_price(&w) < plain_bash, "{object}: a quicker blow");
        assert_eq!(
            bash_price(&w),
            plain_bash,
            "{object}: the bash is the shield's"
        );
    }
    // What the player wears quickens every attack, a bash too.
    let gloves = magic(&w, "leather_gloves", &[("of_celerity", 8)], true);
    wear(&mut w, "hands", gloves);
    assert_eq!(bash_price(&w), plain_bash * 92 / 100);
}

#[test]
fn ranged_effects_change_shots_and_throws() {
    let mut w = world();
    let bow = plain(&w, "long_bow");
    wear(&mut w, "missile_weapon", bow);
    let mut arrows = plain(&w, "arrow");
    arrows.count = 10;
    wear(&mut w, "missiles", arrows);
    let base = me(&w).ranged(w.content()).expect("a shot");
    assert_eq!(me(&w).launcher_range(w.content()), Some(15));

    let mut w = world();
    let bow = magic(&w, "long_bow", &[("hunters", 2), ("steady", 2)], true);
    wear(&mut w, "missile_weapon", bow);
    let mut arrows = magic(&w, "arrow", &[("glinting", 1), ("of_durability", 30)], true);
    arrows.count = 10;
    let quiver = wear(&mut w, "missiles", arrows);
    let p = me(&w).ranged(w.content()).expect("a shot");
    assert_eq!(me(&w).launcher_range(w.content()), Some(17));
    // St 12 against the bow's 14: two short, made up by the draw; the arrows' own to-hit.
    assert_eq!(p.to_hit, base.to_hit + 2 + 1);
    assert_eq!(p.bonus, base.bonus + 2);
    let arrow = &me(&w).inventory[quiver];
    assert_eq!(
        super::shoot::break_chance(arrow, w.content()),
        30 * 70 / 100
    );

    let mut w = world();
    let dagger = magic(&w, "dagger", &[("of_reach", 2), ("sharp", 1)], true);
    let item = wear(&mut w, "main_hand", dagger);
    assert_eq!(me(&w).throw_range(item, w.content()), Some(7));
    let plain_throw = Profile::thrown(
        &w.content()
            .object(w.content().object_id("dagger").expect("dagger"))
            .attack
            .expect("a weapon"),
        &me(&w).effective_stats(w.content()),
    );
    assert_eq!(
        me(&w).thrown(item, w.content()).map(|p| p.bonus),
        Some(plain_throw.bonus + 1)
    );
}

#[test]
fn defence_effects_raise_their_categories() {
    let mut w = world();
    let (jerkin, shield) = (plain(&w, "leather_jerkin"), plain(&w, "small_shield"));
    wear(&mut w, "body", jerkin);
    wear(&mut w, "off_hand", shield);
    let base = defence(&w);
    let jerkin = magic(
        &w,
        "leather_jerkin",
        &[("reinforced", 1), ("of_evasion", 1)],
        true,
    );
    wear(&mut w, "body", jerkin);
    let shield = magic(
        &w,
        "small_shield",
        &[("sturdy", 1), ("of_the_tortoise", 1)],
        true,
    );
    wear(&mut w, "off_hand", shield);
    let ring = magic(&w, "iron_ring", &[("shimmering", 1)], true);
    wear(&mut w, "left_ring", ring);
    let d = defence(&w);
    assert_eq!(d.bonus(Category::Armour), base.bonus(Category::Armour) + 1);
    assert_eq!(d.bonus(Category::Shield), base.bonus(Category::Shield) + 1);
    assert_eq!(d.bonus(Category::Dodge), 1);
    assert_eq!(d.bonus(Category::Deflection), 1);
    assert_eq!(d.bonus(Category::Natural), 1);
    assert_eq!(d.total(), base.total() + 5);
    assert_eq!(w.perception().own_defence(), d.total());

    // Supple armour lets Dexterity add more: Dx 14 (+2) under chain mail's +4 is no limit, so a
    // nimble player needs more Dexterity to show it.
    let dexterity = |supple: bool| {
        let mut w = world();
        let ring = magic(&w, "iron_ring", &[("of_the_lynx", 10)], true);
        wear(&mut w, "left_ring", ring);
        let mail = match supple {
            true => magic(&w, "chain_mail", &[("supple", 1)], true),
            false => plain(&w, "chain_mail"),
        };
        wear(&mut w, "body", mail);
        defence(&w).bonus(Category::Dexterity)
    };
    assert_eq!(dexterity(false), 4, "chain mail caps Dexterity at +4");
    assert_eq!(dexterity(true), 5);
}

#[test]
fn life_stats_and_load_effects_apply_to_the_wearer() {
    let mut w = world();
    let content = w.content().clone();
    let (max, scores, load) = (
        me(&w).max_hp(&content),
        me(&w).effective_stats(&content),
        w.load_of(w.state.player),
    );
    let ring = magic(&w, "iron_ring", &[("hale", 4), ("of_the_bear", 1)], true);
    wear(&mut w, "left_ring", ring);
    let amulet = magic(&w, "bone_amulet", &[("of_the_mule", 5)], true);
    wear(&mut w, "neck", amulet);
    assert_eq!(me(&w).max_hp(&content), max + 4);
    let now = me(&w).effective_stats(&content);
    assert_eq!(now.score(Stat::Strength), scores.score(Stat::Strength) + 1);
    assert_eq!(
        now.potential(Stat::Strength),
        scores.potential(Stat::Strength)
    );
    assert_eq!(
        w.perception().own_scores()[Stat::Strength],
        now.score(Stat::Strength)
    );
    // One more Strength is 3 kg more, the mule 5 kg more (kept a multiple of 3 g, so the
    // thirds stay exact).
    let limit = w.load_of(w.state.player).limit;
    assert_eq!(limit, crate::burden::with_carry(load.limit + 3000, 5));
    assert_eq!((limit - load.limit, limit % 3), (7998, 0));

    // Featherweight: lighter to carry.
    let mail = plain(&w, "chain_mail");
    let light = magic(&w, "chain_mail", &[("featherweight", 20)], true);
    assert_eq!(light.weight(&content), mail.weight(&content) * 80 / 100);

    // Taking off what raised the maximum leaves no hit points above it.
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("player").hp = max + 4;
    run(&mut w, "remove:left_ring");
    assert_eq!(me(&w).hp, max);
}

#[test]
fn regeneration_comes_faster() {
    let hurt_and_wait = |ring: Option<Stack>| {
        let mut w = world();
        if let Some(ring) = ring {
            wear(&mut w, "left_ring", ring);
        }
        let player = w.state.player;
        w.state.actors.get_mut(&player).expect("player").hp = 1;
        let now = w.state.time;
        w.pass_time(std::iter::once(player), now, 600, &mut Vec::new());
        me(&w).hp
    };
    let w = world();
    let mending = magic(&w, "iron_ring", &[("of_mending", 50)], true);
    // Toughness 10: one every 240 s, 160 s with half again as fast.
    assert_eq!(hurt_and_wait(None), 1 + 600 / 240);
    assert_eq!(hurt_and_wait(Some(mending)), 1 + 600 / 160);
}

#[test]
fn worn_magic_raises_the_scores_status_effects_lower() {
    let mut w = world();
    let content = w.content().clone();
    let ring = magic(&w, "iron_ring", &[("of_the_bear", 2)], true);
    wear(&mut w, "left_ring", ring);
    let strength = |w: &World| me(w).effective_stats(&content).score(Stat::Strength);
    assert_eq!(strength(&w), 12 + 2);
    let to_hit = me(&w).melee(&content).to_hit;
    // Four degrees of poison: a point of Strength off the raised score, for every rule.
    run(&mut w, "afflict:poisoned*4");
    assert_eq!(strength(&w), 12 + 2 - 1);
    assert_eq!(me(&w).melee(&content).to_hit, to_hit - 1);
    let view = w.perception();
    assert_eq!(view.own_scores()[Stat::Strength], 13);
    assert_eq!(
        view.own_base_scores()[Stat::Strength],
        14,
        "the ring is no penalty"
    );
}

#[test]
fn a_magic_weapon_readied_to_throw_keeps_its_magic_but_parries_nothing() {
    let mut w = world();
    let content = w.content().clone();
    let base = defence(&w).bonus(Category::Parry);
    let sword = plain(&w, "long_sword");
    let item = wear(&mut w, "missile_weapon", sword);
    let plain_throw = me(&w).thrown(item, &content).expect("thrown whole");
    let mut w = world();
    let sword = magic(&w, "long_sword", &[("sharp", 2), ("guarding", 1)], true);
    let item = wear(&mut w, "missile_weapon", sword.clone());
    assert_eq!(
        defence(&w).bonus(Category::Parry),
        base,
        "only what is held parries"
    );
    let range = me(&w).throw_range(item, &content);
    assert_eq!(range, Some(combat::IMPROVISED_RANGE));
    let throw = me(&w).thrown(item, &content).expect("thrown whole");
    assert_eq!(throw.to_hit, plain_throw.to_hit);
    assert_eq!(
        throw.bonus,
        plain_throw.bonus + 2,
        "its own affixes ride along"
    );
    let letter = crate::action::letter(item).expect("a letter");
    run(&mut w, &format!("throw:{letter}:4,1"));
    assert!(me(&w).inventory.iter().all(|s| s.object != sword.object));
    let level = w.state.current();
    let landed: Vec<&Stack> = (1..8)
        .flat_map(|x| (1..4).map(move |y| Pos::new(x, y)))
        .flat_map(|p| level.objects_at(p))
        .collect();
    assert_eq!(landed, [&sword], "it lands with its magic");
}

#[test]
fn stealth_quiets_footsteps_and_light_shines_from_the_wearer() {
    let mut w = world();
    let to = w.player_pos().step(Dir::E);
    let loud = |w: &World| {
        let a = me(w);
        let burden = w.load_of(a.id).penalty();
        crate::noise::noise(w.content(), w.state.current(), a, burden, Act::Step(to))
            .map_or(0, |n| n.loudness)
    };
    let before = loud(&w);
    assert!(before > 0);
    let boots = magic(&w, "leather_boots", &[("of_stealth", 20)], true);
    wear(&mut w, "feet", boots);
    assert_eq!(loud(&w), before * 80 / 100);

    let glow = w.content().light_id("item_glow_faint").expect("glow");
    let glowing = |w: &World| {
        (w.perception().light_sources().iter())
            .any(|s| s.preset() == Some(glow) && s.pos() == Some(w.player_pos()))
    };
    assert!(!glowing(&w));
    let ring = magic(&w, "iron_ring", &[("of_light", 0)], false);
    wear(&mut w, "left_ring", ring);
    run(&mut w, "wait");
    assert!(
        glowing(&w),
        "an unidentified glowing ring lights its wearer too"
    );
    assert_eq!(me(&w).mods(w.content()).light, Some(glow));
}

#[test]
fn effects_apply_only_while_worn_and_unidentified_ones_too() {
    let mut w = world();
    let content = w.content().clone();
    let scores = me(&w).effective_stats(&content);
    let ring = magic(&w, "iron_ring", &[("of_the_bear", 1)], false);
    carry(&mut w, ring.clone());
    assert_eq!(
        me(&w).effective_stats(&content),
        scores,
        "carried, not worn: nothing"
    );
    let item = wear(&mut w, "left_ring", ring);
    let strength = me(&w).effective_stats(&content).score(Stat::Strength);
    assert_eq!(
        strength,
        scores.score(Stat::Strength) + 1,
        "worn: it works, unknown"
    );

    // What the player is shown knows its rarity, never its affixes.
    let view = w.perception();
    let stack = &view.inventory()[item];
    assert_eq!(stack.describe(&content), "an iron ring (magic)");
    assert!(stack.known_magic().is_none());
    let json = crate::dump::json(&w, &[crate::dump::Layer::Inventory], false).to_string();
    assert!(!json.contains("of_the_bear"), "{json}");
    assert!(json.contains("\"rarity\":\"magic\""), "{json}");
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Inventory).join("\n");
    assert!(!rows.contains("Bear"), "{rows}");
    // Dev dumps show the truth.
    let actors = crate::dump::layer_rows(&w, crate::dump::Layer::Actors).join("\n");
    assert!(
        actors.contains("iron ring of the Bear (unidentified)"),
        "{actors}"
    );
    assert!(actors.contains("magic strength +1"), "{actors}");

    // Identifying reveals it, and says so.
    assert_eq!(
        w.identify(item),
        Identified::Now("an iron ring of the Bear".into())
    );
    assert_eq!(w.identify(item), Identified::Known);
    assert_eq!(w.identify(25), Identified::NoSuchItem);
    let said: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    assert!(
        said.iter()
            .any(|m| m == "This is an iron ring of the Bear!"),
        "{said:?}"
    );
    let known = w.perception().inventory()[item].known_magic().cloned();
    assert_eq!(known.map(|m| m.affixes.len()), Some(1));
    let json = crate::dump::json(&w, &[], false).to_string();
    assert!(json.contains("of_the_bear"), "{json}");
}

#[test]
fn identifying_is_a_dev_action_that_takes_no_time() {
    let mut w = world();
    let (ring, dagger) = (
        magic(&w, "iron_ring", &[("of_the_bear", 1)], false),
        plain(&w, "dagger"),
    );
    carry(&mut w, ring);
    carry(&mut w, dagger);
    let turn = w.turn();
    let out = w.apply(&Action::Identify(Positions::default()));
    assert!(!out.took_turn && out.elapsed.is_zero());
    assert_eq!(w.turn(), turn);
    assert!(w.perception().inventory()[0].is_identified());
    w.apply(&Action::Identify(Positions::default()));
    let last = w.messages().iter().last().map(|m| m.display());
    assert_eq!(last.as_deref(), Some("There is nothing to identify (dev)."));
}

#[test]
fn magic_items_survive_saves_and_replays() {
    let content = content();
    let mut w = World::new(content.clone(), "vault", None).expect("world");
    let mut replay = Replay::new(&w);
    let sword = magic(&w, "long_sword", &[("sharp", 1), ("of_the_bear", 1)], false);
    carry(&mut w, sword);
    let mut arrows = magic(&w, "arrow", &[("sharp", 1)], true);
    arrows.count = 7;
    carry(&mut w, arrows);
    let state = w.state().clone();
    for action in parse_actions("equip:main_hand:a identify:a drop:b wait pickup").expect("ok") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let names: Vec<String> = (w.perception().inventory().iter())
        .map(|s| s.describe(&content))
        .collect();
    assert_eq!(names, ["a Sharp long sword of the Bear", "7 Sharp arrows"]);
    // A save keeps every instance as it is.
    let bytes = crate::save::encode(&w);
    let back = crate::save::decode(&bytes, content.clone()).expect("loads");
    assert_eq!(back.state(), w.state());
    assert_eq!(back.state_hash(), w.state_hash());
    // A replay from the same start state reproduces it.
    let mut again = World::from_state(content.clone(), state).expect("world");
    for step in parse_actions("equip:main_hand:a identify:a drop:b wait pickup").expect("ok") {
        again.apply(&step);
    }
    assert_eq!(again.state_hash(), w.state_hash());
    assert_eq!(replay.steps.len(), 5);
    // A save with an affix this content does not have is refused.
    let mut bad = w.state().clone();
    let player = bad.player;
    let ring = magic(&w, "iron_ring", &[("of_nonsense", 1)], false);
    bad.actors
        .get_mut(&player)
        .expect("player")
        .inventory
        .push(ring);
    assert!(World::from_state(content, bad).is_err());
}

#[test]
fn creatures_drop_loot_deterministically() {
    let w = world();
    let kobold = (w.state().actors.values())
        .find(|a| a.kind == ActorKind::Creature)
        .expect("kobold")
        .clone();
    assert_eq!(w.content().creature(kobold.creature).loot, 25);
    // From its own stream: the same actor drops the same thing.
    assert_eq!(w.loot_of(&kobold), w.loot_of(&kobold));
    // Over many actors about a quarter drop something, always magic or rare, at the level's item
    // level (3), or now and then a loose gem (spec §7 "Gems").
    let mut drops = 0;
    for id in 1..400 {
        let mut a = kobold.clone();
        a.id = ActorId(id);
        if let Some(stack) = w.loot_of(&a) {
            drops += 1;
            if stack.gem(w.content()).is_some() {
                continue;
            }
            let magic = stack.magic().expect("magic or rare");
            assert!(magic.rarity >= Rarity::Magic && !magic.identified);
            assert_eq!(magic.level, 3);
        }
    }
    assert!((60..140).contains(&drops), "{drops} of 399");
}

#[test]
fn generated_levels_hold_floor_and_vault_loot() {
    let content = Arc::new(Content::builtin().expect("content"));
    for depth in 1..=8 {
        let id = format!("depths:{depth}");
        let a = crate::mapgen::generate(&content, ("showcase", 42), &id, None).expect("level");
        let b = crate::mapgen::generate(&content, ("showcase", 42), &id, None).expect("level");
        assert_eq!(a, b, "deterministic");
        // Loot: base items and scrolls (spec §8 "Scrolls"), what has a loot frequency.
        let loot: Vec<&Stack> = (a.objects.values().flatten())
            .filter(|s| content.object(s.object).frequency > 0)
            .collect();
        assert!(loot.len() >= 2, "depth {depth}: {} items", loot.len());
        for s in loot.iter().filter_map(|s| s.magic()) {
            assert_eq!(u32::from(s.level), depth);
        }
    }
    // The showcase's magic items load as written.
    let w = World::new(content.clone(), "showcase", None).expect("world");
    let (level, _) = w.ground_truth();
    let names = |p: Pos| -> Vec<String> {
        level
            .objects_at(p)
            .iter()
            .map(|s| s.describe(&content))
            .collect()
    };
    assert_eq!(
        names(Pos::new(19, 14)),
        [
            "Dread Visage, an iron helm",
            "a Sharp emerald-pommelled long sword of the Bear"
        ]
    );
    assert_eq!(
        names(Pos::new(18, 14)),
        ["a silver ring (magic)", "a short bow (rare)"]
    );
    let helm = &level.objects_at(Pos::new(19, 14))[0];
    let mods = crate::magic::Mods::local(helm, &content);
    assert_eq!(mods.get(Effect::MaxHp), 0, "Hale is global");
}

#[test]
fn skill_affixes_raise_what_skills_give() {
    let mut w = world();
    let to = w.player_pos().step(Dir::E);
    let loud = |w: &World| {
        let a = me(w);
        let burden = w.load_of(a.id).penalty();
        crate::noise::noise(w.content(), w.state.current(), a, burden, Act::Step(to))
            .map_or(0, |n| n.loudness)
    };
    let quiet = |w: &World| me(w).skill_effect(w.content(), crate::skills::Effect::Quiet);
    let before = (loud(&w), quiet(&w));
    assert_eq!(before.1, 0, "no stealth trained");
    let boots = magic(&w, "leather_boots", &[("of_shadows", 30)], true);
    wear(&mut w, "feet", boots);
    // Stealth at 30 (untrained, from 0) on its curve to 60 % at 100: 18 % quieter.
    assert_eq!(quiet(&w), 18);
    assert_eq!(loud(&w), before.0 * 82 / 100);
    let stealth = w.content().skill_id("stealth").expect("stealth");
    assert_eq!(me(&w).mods(w.content()).skills(), [(stealth, 30)]);
    let item = me(&w).inventory.len() - 1;
    let text = crate::magic::describe::lines(&me(&w).inventory[item], w.content());
    assert!(
        text.iter().any(|l| l.text == "of Shadows: +30 to Stealth"),
        "{text:?}"
    );
}
