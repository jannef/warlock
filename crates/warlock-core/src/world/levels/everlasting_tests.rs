//! Everlasting corpses (#1624; docs/design/profane-gifts.md §3): every keeper of Grundlēas
//! leaves a corpse that never rots (fresh at any age, never dropped from a pile or a pack, judged
//! fresh by a trained eye), and leaves it even when stoned. The Soulglass case is in
//! `soulglass_tests.rs`; the loader's refusals in `layer_boss_tests.rs`.

use crate::content::ContentFiles;
use crate::object::Stack;
use crate::rot::Stage;
use crate::time::{DAY, Duration, GameTime};
use crate::world::classes::class_active_tests::beside;
use crate::world::*;

/// The ten keepers of Grundlēas, by depth.
const KEEPERS: [&str; 10] = [
    "gnaegan",
    "attorcoppe",
    "hrimwyrm",
    "wealaf",
    "grundwyrgen",
    "ligetdry",
    "staneoten",
    "nihtscinna",
    "waelcyrge",
    "nithdraca",
];

/// A lit hall: the player, and a peaceful kobold far off (the template `beside` copies).
const HALL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
############
#.@.......k#
#..........#
############
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold", faction = "peaceful" }
"##;

fn world() -> World {
    let mut files = ContentFiles::builtin();
    files.insert("levels/keeper_hall.toml", HALL);
    files.insert(
        "scenarios/keeper_hall.toml",
        "name = \"Keeper hall\"\nstart_level = \"keeper_hall\"\nhp = 1000\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    let mut w = World::new(content, "keeper_hall", Some(3)).expect("world");
    // Far enough into the calendar that a corpse can be a month old.
    w.state.time = GameTime::from_seconds(400 * DAY);
    w
}

fn corpse_of(w: &World, creature: &str) -> crate::content::ObjectId {
    let c = w.content().creature_id(creature).expect(creature);
    w.content().corpse_of(c).expect("a corpse")
}

#[test]
fn every_keeper_leaves_a_corpse_that_keeps_and_lasts_for_ever() {
    let w = world();
    for id in KEEPERS {
        let c = w.content().creature_id(id).expect(id);
        let def = w.content().creature(c);
        assert!(def.unique && def.corpse && def.everlasting, "{id}");
        let food = (w.content().object(corpse_of(&w, id)).food).expect("food");
        assert_eq!((food.keeps, food.lasts), (None, None), "{id}");
        assert!(w.content().is_layer_boss(c), "{id}");
    }
    // An ordinary corpse still rots.
    let kobold = w.content().object(corpse_of(&w, "kobold"));
    assert!(kobold.food.expect("food").lasts.is_some());
}

#[test]
fn a_keepers_corpse_a_month_old_is_fresh_stays_put_and_is_judged_fresh() {
    let mut w = world();
    let now = w.state.time;
    let died = GameTime::from_seconds(now.seconds() - 30 * DAY);
    let corpse = Stack::corpse(corpse_of(&w, "gnaegan"), died);
    let rat = Stack::corpse(corpse_of(&w, "kobold"), died);
    assert_eq!(w.stage(&corpse), Some(Stage::Fresh));
    assert_eq!(
        w.stage(&rat),
        Some(Stage::Gone),
        "an ordinary one is long gone"
    );
    assert_eq!(crate::rot::gone_at(w.content(), &corpse), None);
    // On the floor: perception's clean-up leaves it, and takes the kobold's.
    let at = w.player_pos();
    w.state
        .current_mut()
        .set_pile(at, vec![corpse.clone(), rat.clone()]);
    w.rot_away();
    let pile: Vec<_> = w.state.current().objects_at(at).to_vec();
    assert_eq!(pile, vec![corpse.clone()]);
    // In a pack: it never rots away, and no step of time ends at it.
    let player = w.state.player;
    w.player_mut().inventory = vec![corpse.clone()];
    let later = now + Duration::seconds(365 * DAY);
    w.rot_pack(player, later);
    assert_eq!(w.state.player().inventory, vec![corpse.clone()]);
    assert_eq!(
        crate::world::health::rot::until_rots(&w.state.player().inventory, w.content(), later),
        None
    );
    // A trained eye judges it fresh whenever its check passes, never anything else.
    let appraise = w.content().skill_id("appraise").expect("appraise");
    w.player_mut().skills.add(appraise, 50);
    let content = w.content().clone();
    let seed = w.state.seed;
    let told: Vec<Option<Stage>> = (0..200u64)
        .map(|h| {
            let at = GameTime::from_seconds(now.seconds() + h * 3600);
            crate::world::health::freshness::judged(&content, w.state.player(), seed, &corpse, at)
        })
        .collect();
    assert!(told.iter().all(|t| matches!(t, None | Some(Stage::Fresh))));
    assert!(told.contains(&Some(Stage::Fresh)), "told at least once");
}

#[test]
fn a_keeper_stoned_to_death_leaves_its_corpse_not_a_statue() {
    let mut w = world();
    let player = w.state.player;
    let id = beside(&mut w, "gnaegan", 10);
    let at = w.state.actors[&id].pos;
    let stiffening = w.content().status_id("stiffening").expect("stiffening");
    for _ in 0..4 {
        if w.state.actors.contains_key(&id) {
            w.afflict(id, stiffening, 1, Some(player), &mut Vec::new());
        }
    }
    assert!(!w.state.actors.contains_key(&id), "stoned to death");
    let (level, _) = w.ground_truth();
    assert!(level.statues().get(&at).is_none(), "no statue");
    let corpse = corpse_of(&w, "gnaegan");
    assert!(
        level.objects_at(at).iter().any(|s| s.object == corpse),
        "its corpse where it fell"
    );
}
