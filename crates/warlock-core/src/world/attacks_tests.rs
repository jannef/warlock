//! The player's attacks as the character screen shows them (`attacks.rs`): the very profiles a
//! strike rolls, in the style in use, an unknown affix moving nothing, and their text.

use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::character::CharacterSpec;
use crate::magic::{Affix, Magic, Rarity};
use crate::object::Stack;
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

fn start(character: &str, actions: &str) -> World {
    let spec: CharacterSpec = character.parse().expect("a character");
    let mut w = World::start_run(
        BUILTIN.clone(),
        "showcase",
        Some(7),
        Some(&spec),
        None,
        true,
    )
    .expect("world");
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
    w
}

#[test]
fn a_sword_and_shield_shows_the_blow_a_strike_rolls() {
    let w = start("human,male,warrior", "wait");
    let content = w.content();
    let me = w.state().player();
    let attacks = w.perception().own_attacks();
    assert_eq!(attacks.len(), 1, "{attacks:#?}");
    let main = &attacks[0];
    let profile = me.melee(content);
    assert_eq!(main.source, AttackSource::MainHand);
    assert_eq!(main.kind, AttackKind::Melee);
    assert_eq!(main.name, "long sword");
    assert_eq!(main.to_hit, profile.to_hit);
    assert_eq!(main.damage, profile.damage.plus(profile.bonus));
    assert_eq!(main.range, None);
}

#[test]
fn two_weapons_show_both_hands_at_their_penalties_and_the_bow_its_range() {
    let w = start("human,male,ranger", "move:SW*2 pickup equip:off_hand:k");
    let content = w.content();
    let me = w.state().player();
    let attacks = w.perception().own_attacks();
    let sources: Vec<AttackSource> = attacks.iter().map(|a| a.source).collect();
    assert_eq!(
        sources,
        [
            AttackSource::MainHand,
            AttackSource::OffHand,
            AttackSource::Missile
        ]
    );
    let (_, off) = me.off_hand_blow(content).expect("an off hand");
    assert_eq!(attacks[1].name, "dagger");
    assert_eq!(attacks[1].to_hit, off.to_hit);
    assert!(attacks[1].to_hit < attacks[0].to_hit, "{attacks:#?}");
    let shot = me.ranged(content).expect("a bow");
    assert_eq!(attacks[2].kind, AttackKind::Ranged);
    assert_eq!(attacks[2].name, "short bow");
    assert_eq!(attacks[2].to_hit, shot.to_hit);
    assert_eq!(attacks[2].range, me.launcher_range(content));
    // The off hand's weapon put away: the main hand fights alone, without its penalty.
    let mut w = w;
    w.apply(&parse_actions("remove:off_hand").expect("action")[0]);
    let alone = w.perception().own_attacks();
    assert_eq!(alone.len(), 2);
    assert!(alone[0].to_hit > attacks[0].to_hit, "{alone:#?}");
}

#[test]
fn a_ready_throwing_weapon_shows_its_throw() {
    let w = start(
        "human,male,ranger",
        "move:SW*2 pickup remove:missile_weapon equip:missile_weapon:k",
    );
    let content = w.content();
    let me = w.state().player();
    let attacks = w.perception().own_attacks();
    let throw = attacks.last().expect("a throw");
    let item = me.ready_to_throw(content).expect("a dagger ready");
    assert_eq!(throw.source, AttackSource::Missile);
    assert_eq!(throw.kind, AttackKind::Thrown);
    assert_eq!(throw.name, "dagger");
    assert_eq!(
        throw.to_hit,
        me.thrown(item, content).expect("throws").to_hit
    );
    assert_eq!(throw.range, me.throw_range(item, content));
}

#[test]
fn bare_hands_show_when_nothing_is_wielded() {
    let w = start("human,male,warrior", "remove:main_hand remove:off_hand");
    let attacks = w.perception().own_attacks();
    assert_eq!(attacks.len(), 1);
    assert_eq!(attacks[0].name, "bare hands");
    assert_eq!(
        attacks[0].damage_text(w.content()).split(' ').nth(1),
        Some("bludgeoning")
    );
}

#[test]
fn an_unknown_affix_moves_nothing_until_identified() {
    let mut w = start("human,male,warrior", "wait");
    let plain = w.perception().own_attacks()[0].clone();
    let content = w.content().clone();
    let me = w.player_mut();
    let sword = me.weapon_item(&content).expect("a sword");
    let object = me.inventory[sword].object;
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 1,
        affixes: vec![
            Affix {
                id: "of_precision".into(),
                tier: 1,
                value: 1,
            },
            Affix {
                id: "flaming".into(),
                tier: 1,
                value: 2,
            },
        ],
        name: None,
        identified: false,
        carried: 0,
        unique: None,
    };
    magic
        .check(content.object(object), &content)
        .expect("valid");
    me.inventory[sword] = Stack::with_magic(object, 1, magic);
    let unknown = w.perception().own_attacks()[0].clone();
    assert_eq!(unknown.to_hit, plain.to_hit);
    assert_eq!(unknown.damage_text(&content), plain.damage_text(&content));
    // The rules know better all along.
    assert_eq!(w.state().player().melee(&content).to_hit, plain.to_hit + 1);
    w.player_mut().inventory[sword].identify();
    let known = w.perception().own_attacks()[0].clone();
    assert_eq!(known.to_hit, plain.to_hit + 1);
    assert!(
        known.damage_text(&content).ends_with(" fire"),
        "{}",
        known.damage_text(&content)
    );
}

#[test]
fn damage_reads_dice_bonus_kinds_and_riders() {
    let content = &**BUILTIN;
    let slashing = content.damage_type_id("slashing").expect("slashing");
    let piercing = content.damage_type_id("piercing").expect("piercing");
    let fire = content.damage_type_id("fire").expect("fire");
    let mut attack = OwnAttack {
        source: AttackSource::MainHand,
        kind: AttackKind::Melee,
        name: "long sword".into(),
        to_hit: 4,
        damage: crate::dice::Dice::of(1, 8).plus(2),
        extra: None,
        kinds: crate::damage::Kinds::of(&[slashing, piercing]).expect("two"),
        riders: Vec::new(),
        range: None,
    };
    assert_eq!(
        attack.summary(content),
        "+4 to hit, 1d8+2 slashing or piercing"
    );
    attack.extra = Some(crate::dice::Dice::of(1, 4));
    attack.riders.push(crate::damage::Rider {
        kind: fire,
        dice: crate::dice::Dice::of(1, 6),
    });
    attack.to_hit = -1;
    attack.range = Some(3);
    assert_eq!(
        attack.summary(content),
        "-1 to hit, 1d8+1d4+2 slashing or piercing +1d6 fire, range 3"
    );
}
