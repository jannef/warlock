//! The body gifts (docs/design/profane-gifts.md §6.4, §13 P5; `world/body_slots.rs`): a slot
//! barred (Horned's helm falls, its equip refused with no time), a slot opened (Many-fingered's
//! ring, and its fall when the gift goes), claws that make bleeding, kin by tag (Rat-king's court),
//! another race's eyes (Deep-eyed), and more mana (Fell-minded).

use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, Stance};
use crate::object::Stack;
use std::sync::LazyLock;

/// A lit corridor: the player at its west end, a rat (vermin) beside them, a kobold farther.
const DEN: &str = r#"
name = "Den"
ambient = [255, 255, 255, 0]
map = '''
############
#@r......k##
############
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat" }
"k" = { terrain = "floor", creature = "kobold" }
"#;

static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.insert("levels/body_den.toml", DEN);
    files.insert(
        "scenarios/body_den.toml",
        "name = \"Den\"\nstart_level = \"body_den\"\nhp = 20\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
});

/// A run in the den as `character` (`human,male,warrior`).
fn world(character: &str) -> World {
    let spec: CharacterSpec = character.parse().expect("a character");
    World::start_run(
        CONTENT.clone(),
        "body_den",
        Some(1),
        Some(&spec),
        None,
        false,
    )
    .expect("world")
}

fn run(w: &mut World, text: &str) {
    for action in parse_actions(text).expect("actions") {
        w.apply(&action);
    }
}

/// Gift `id` given as the dev action gives it, which takes no time.
fn give(w: &mut World, id: &str) {
    let turn = w.turn();
    w.apply(&crate::action::Action::Profane(id.into()));
    assert_eq!(w.turn(), turn, "a dev action takes no time");
    assert!(w.state.player().talents.has(id), "{id} held");
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

/// Object `id` carried, its inventory position.
fn carry(w: &mut World, id: &str) -> usize {
    let object = w.content.object_id(id).expect(id);
    let me = w.player_mut();
    me.inventory.push(Stack::new(object, 1));
    me.inventory.len() - 1
}

fn slot(w: &World, id: &str) -> crate::content::SlotId {
    w.content.slot_id(id).expect(id)
}

fn on_floor(w: &World, id: &str) -> bool {
    let object = w.content.object_id(id).expect(id);
    let here = w.player_pos();
    (w.state.current().objects_at(here).iter()).any(|s| s.object == object)
}

#[test]
fn horned_drops_the_helm_and_refuses_another_with_no_time() {
    let mut w = world("human,male,warrior");
    let helm = carry(&mut w, "iron_helm");
    run(&mut w, &format!("equip:head:{helm}"));
    assert_eq!(w.state.player().equipment.get(slot(&w, "head")), Some(helm));
    let carried = w.state.player().inventory.len();
    give(&mut w, "horned");
    assert!(said(
        &w,
        "Your iron helm no longer fits, and falls to the floor."
    ));
    assert_eq!(w.state.player().equipment.get(slot(&w, "head")), None);
    assert_eq!(
        w.state.player().inventory.len(),
        carried - 1,
        "the helm left the pack"
    );
    assert!(on_floor(&w, "iron_helm"), "it lies on the player's cell");
    // Put on again: refused with the gift's words, no time.
    let helm = carry(&mut w, "iron_helm");
    let turn = w.turn();
    run(&mut w, &format!("equip:head:{helm}"));
    assert_eq!(w.turn(), turn, "refused, no time");
    assert!(said(&w, "Your horns are in the way."));
    assert_eq!(w.state.player().equipment.get(slot(&w, "head")), None);
    // What the player sees: the row and its note.
    let view = w.perception();
    assert_eq!(view.barred(slot(&w, "head")).as_deref(), Some("horns"));
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Equipment).join("\n");
    assert!(rows.contains("a Head: - (horns)"), "{rows}");
    // Rescinded, the slot is free again (nothing is put back).
    assert!(w.rescind(w.content.talents().id("horned").expect("horned")));
    run(&mut w, &format!("equip:head:{helm}"));
    assert_eq!(w.state.player().equipment.get(slot(&w, "head")), Some(helm));
}

#[test]
fn many_fingered_opens_a_ring_slot_and_its_ring_falls_when_it_goes() {
    let mut w = world("human,male,warrior");
    let ring = carry(&mut w, "silver_ring");
    let extra = slot(&w, "extra_ring_1");
    let owned = |w: &World| (w.perception().own_slots().iter()).any(|&(s, _)| s == extra);
    assert!(!owned(&w), "a warrior has no extra ring slot");
    let turn = w.turn();
    run(&mut w, &format!("equip:extra_ring_1:{ring}"));
    assert_eq!(w.turn(), turn, "no such slot: refused");
    give(&mut w, "many_fingered");
    assert!(owned(&w), "the gift opens it");
    run(&mut w, &format!("equip:extra_ring_1:{ring}"));
    assert_eq!(w.state.player().equipment.get(extra), Some(ring));
    assert!(w.rescind(w.content.talents().id("many_fingered").expect("gift")));
    assert!(said(
        &w,
        "Your silver ring slips from a finger that is no longer there."
    ));
    assert!(!owned(&w));
    assert_eq!(w.state.player().equipment.get(extra), None);
    assert!(on_floor(&w, "silver_ring"));
}

#[test]
fn an_aristocrat_keeps_the_ring_when_many_fingered_goes() {
    let mut w = world("human,female,aristocrat");
    let ring = carry(&mut w, "silver_ring");
    let extra = slot(&w, "extra_ring_1");
    give(&mut w, "many_fingered");
    run(&mut w, &format!("equip:extra_ring_1:{ring}"));
    assert!(w.rescind(w.content.talents().id("many_fingered").expect("gift")));
    assert_eq!(
        w.state.player().equipment.get(extra),
        Some(ring),
        "her own slot"
    );
}

#[test]
fn clawed_bare_hands_make_bleeding() {
    let mut w = world("human,male,warrior");
    let gloves = carry(&mut w, "leather_gloves");
    run(&mut w, &format!("equip:hands:{gloves}"));
    give(&mut w, "clawed");
    assert!(said(&w, "Your nails thicken into claws."));
    assert!(on_floor(&w, "leather_gloves"), "the gloves fell");
    let bleeding = w.content.status_id("bleeding").expect("bleeding");
    let profile = w.state.player().melee(&w.content);
    assert!(
        profile.on_hit.iter().any(|h| h.chance == 25),
        "the claws' rider rides bare hands"
    );
    let (player, rat) = (w.player_id(), creature(&w, "rat"));
    let mut bled = false;
    for _ in 0..200 {
        w.state.actors.get_mut(&rat).expect("rat").hp = 1000;
        let profile = w.state.player().melee(&w.content);
        w.attack_with(
            player,
            rat,
            AttackKind::Melee,
            Some(profile),
            &mut Vec::new(),
        );
        if w.state.actors[&rat].effects.has(bleeding) {
            bled = true;
            break;
        }
    }
    assert!(bled, "a clawed blow made the rat bleed");
    // With a weapon in hand, no claws.
    let sword = carry(&mut w, "short_sword");
    run(&mut w, &format!("equip:main_hand:{sword}"));
    assert!(w.state.player().melee(&w.content).on_hit.is_empty());
}

/// The first creature of type `id` on the player's level.
fn creature(w: &World, id: &str) -> ActorId {
    let kind = w.content.creature_id(id).expect(id);
    (w.state.actors.values())
        .find(|a| a.creature == kind && a.id != w.state.player)
        .map(|a| a.id)
        .expect("the creature")
}

#[test]
fn rat_kings_court_calms_vermin_until_one_of_the_type_is_struck() {
    let mut w = world("human,male,warrior");
    let (player, rat, kobold) = (w.player_id(), creature(&w, "rat"), creature(&w, "kobold"));
    assert_eq!(w.stance(player, rat), Stance::Hostile);
    give(&mut w, "rat_kings_court");
    assert_eq!(w.stance(player, rat), Stance::Neutral, "a vermin is kin");
    assert_eq!(w.stance(rat, player), Stance::Neutral);
    assert_eq!(w.stance(player, kobold), Stance::Hostile, "no vermin");
    w.perceive(&mut Vec::new());
    let view = w.perception();
    let seen = (view.actors().iter())
        .find(|a| a.id == rat)
        .expect("the rat in sight");
    assert_eq!(view.stance(seen), Stance::Neutral);
    // Struck: its type is hostile from now on.
    w.attack_with(player, rat, AttackKind::Melee, None, &mut Vec::new());
    assert_eq!(w.stance(player, rat), Stance::Hostile);
    assert!(w.state.progress.empathy_broken.contains("rat"));
}

#[test]
fn deep_eyed_gives_a_human_the_darkvision_eyes() {
    let mut w = world("human,male,warrior");
    assert_eq!(w.eyes().id, "normal");
    give(&mut w, "barrow_eyed");
    let darkvision = |w: &World| w.eyes().darkvision;
    assert_eq!(darkvision(&w), 3, "Barrow-eyed: three cells from none");
    give(&mut w, "deep_eyed");
    assert!(said(
        &w,
        "The thin dark you saw through closes over, and opens wider."
    ));
    assert!(!w.state.player().talents.has("barrow_eyed"), "replaced");
    let profile = w.content.vision("darkvision").expect("darkvision");
    assert_eq!(w.eyes().id, "darkvision");
    assert_eq!(w.eyes().threshold, profile.threshold);
    assert_eq!(darkvision(&w), profile.darkvision + 6);
    // Dim-eyed may not be drawn beside it.
    let dim = w.content.talents().id("dim_eyed").expect("dim_eyed");
    assert!(!w.may_take_gift(w.content.profane().gift(dim).expect("gift")));
    // The dev override still wins.
    run(&mut w, "vision:normal");
    assert_eq!(w.eyes().id, "normal");
}

#[test]
fn fell_minded_raises_maximum_mana_by_a_quarter() {
    let mut w = world("human,male,wizard");
    let before = w.state.player().max_mana(&w.content);
    // The pool with the Intelligence the gift gives but not its quarter.
    let mut bare = w.state.player().clone();
    bare.stats.gift(crate::stats::Stat::Intelligence, 3);
    let pool = bare.max_mana(&w.content);
    give(&mut w, "fell_minded");
    let me = w.state.player();
    assert_eq!(me.max_mana(&w.content), pool * 125 / 100);
    assert!(me.max_mana(&w.content) > pool, "{pool}");
    // Rescinded, mana is kept within the maximum.
    w.player_mut().mana = w.state.player().max_mana(&w.content);
    assert!(w.rescind(w.content.talents().id("fell_minded").expect("gift")));
    let me = w.state.player();
    assert_eq!(me.max_mana(&w.content), before);
    assert!(me.mana <= before);
}
