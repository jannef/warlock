//! The dark face of the priest's prayer and Communion with the dark (`world/classes/dark_face.rs`; the
//! priest design §6.2, §6.3): the plague and the poison passed to the nearest creature seen, a
//! hex turned on its caster (not on one unseen), a hold's holder webbed, the brim; no strain in
//! an unholy place and no Ebbing in a draining one, and nothing for the holy face.

use crate::alignment::Score;
use crate::content::ContentFiles;
use crate::test_support::{builtin, run, said_text as said};
use crate::world::*;

/// A lit hall: the player at (1, 1), a peaceful kobold beside them at `k` (2, 1), another at `j`
/// (8, 1), and a peaceful skeleton (immune to poison) at `s` (5, 3); a wall at x 14 hides the
/// east part, where a third kobold stands at `h` (20, 2).
const HALL: &str = "\
##############################
#@k.....j.....#..............#
#.............#.....h........#
#....s........#..............#
##############################
";

const K: Pos = Pos::new(2, 1);
const J: Pos = Pos::new(8, 1);
const S: Pos = Pos::new(5, 3);
const H: Pos = Pos::new(20, 2);
/// A cell of the east part (a special room with `room`).
const EAST: Pos = Pos::new(18, 1);

/// The fixture: [`HALL`], its east part a special room of effect `room` (if any), the player a
/// priest with every prayer talent up to release, the hunger use, and Communion, Callous (the
/// dark answers), with full hit points and some mana.
fn world(room: Option<&str>) -> World {
    let mut files = ContentFiles::builtin();
    let room = room.map_or_else(String::new, |effect| {
        format!("[[room]]\neffect = \"{effect}\"\narea = [14, 0, 29, 4]\n")
    });
    files.insert(
        "levels/test_dark.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{HALL}'''\n{room}\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"k\" = {{ terrain = \"floor\", creature = \"kobold\", faction = \"peaceful\" }}\n\
             \"j\" = {{ terrain = \"floor\", creature = \"kobold\", faction = \"peaceful\" }}\n\
             \"h\" = {{ terrain = \"floor\", creature = \"kobold\", faction = \"peaceful\" }}\n\
             \"s\" = {{ terrain = \"floor\", creature = \"skeleton\", faction = \"peaceful\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_dark.toml",
        "name = \"T\"\nstart_level = \"test_dark\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content, "test_dark", None).expect("world");
    let content = w.content.clone();
    let me = w.player_mut();
    for talent in [
        "orison",
        "daily_bread",
        "prayer_of_cleansing",
        "prayer_of_release",
        "communion",
    ] {
        me.talents.take(talent, None);
    }
    me.alignment = Some(Score::new(-150));
    me.hp = me.max_hp(&content);
    me.mana_base = 20;
    me.mana = 20;
    w
}

fn at(w: &World, p: Pos) -> ActorId {
    let level = w.state.current_level.clone();
    w.state.actor_at(&level, p).expect("someone there").id
}

fn status(w: &World, id: &str) -> StatusId {
    w.content.status_id(id).expect("a status")
}

/// The degrees of status `id` actor `who` suffers (0: none).
fn degrees(w: &World, who: ActorId, id: &str) -> u16 {
    (w.state.actors[&who].effects.get(status(w, id))).map_or(0, |e| e.severity)
}

/// Actor `who` comes to suffer `degrees` of `id`, caused by `by`.
fn lay(w: &mut World, who: ActorId, id: &str, degrees: u16, by: Option<ActorId>) {
    let status = status(w, id);
    let mut events = Vec::new();
    w.afflict(who, status, degrees, by, &mut events);
}

/// The player catches `id` and it shows at once.
fn sicken(w: &mut World, id: &str) -> crate::content::DiseaseId {
    let disease = w.content.disease_id(id).expect("a disease");
    let player = w.state.player;
    assert!(w.infect(player, disease, None));
    let me = w.player_mut();
    me.diseases.get_mut(disease).expect("caught").shown = true;
    disease
}

#[test]
fn the_plague_passes_to_the_nearest_creature_seen() {
    let mut w = world(None);
    let fever = sicken(&mut w, "filth_fever");
    let shakes = w.content.disease_id("the_shakes").expect("a disease");
    let player = w.state.player;
    assert!(w.infect(player, shakes, None), "a brewing one goes too");
    run(&mut w, "use:orison");
    assert!(w.state.player().diseases.is_empty());
    assert!(said(&w, "You whisper to the dark."));
    assert!(said(&w, "The sickness leaves you, and finds another."));
    let k = &w.state.actors[&at(&w, K)];
    assert!(k.diseases.has(fever) && k.diseases.has(shakes));
    assert!(
        !k.diseases.get(fever).expect("caught").shown,
        "it incubates"
    );
    assert!(
        !w.state.actors[&at(&w, J)].diseases.has(fever),
        "the nearest only"
    );
}

#[test]
fn the_holy_face_passes_nothing_on() {
    let mut w = world(None);
    w.player_mut().alignment = Some(Score::new(0));
    let fever = sicken(&mut w, "filth_fever");
    run(&mut w, "afflict:poisoned*3 use:orison");
    assert!(said(&w, "The sickness leaves you."));
    assert!(!w.state.actors[&at(&w, K)].diseases.has(fever));
    w.state.progress.prayer = None;
    run(&mut w, "use:orison");
    assert_eq!(degrees(&w, at(&w, K), "poisoned"), 0);
}

#[test]
fn the_poison_passes_at_its_degrees_and_an_immune_one_shrugs_it_off() {
    let mut w = world(None);
    let player = w.state.player;
    lay(&mut w, player, "poisoned", 3, None);
    let had = degrees(&w, player, "poisoned");
    assert!(had > 0);
    run(&mut w, "use:orison");
    assert_eq!(degrees(&w, player, "poisoned"), 0);
    assert!(said(&w, "The poison leaves you, and finds another."));
    let k = at(&w, K);
    // The kobold's own turn may already have eased it a little.
    let now = degrees(&w, k, "poisoned");
    assert!(now > 0 && now <= had, "{now} of {had}");
    let by = w.state.actors[&k]
        .effects
        .get(status(&w, "poisoned"))
        .and_then(|e| e.by);
    assert_eq!(by, Some(player), "a kill of it is the player's");
    // With the kobolds gone, the skeleton is the nearest seen: immune, it takes nothing.
    for p in [K, J] {
        let id = at(&w, p);
        w.state.actors.remove(&id);
    }
    w.state.progress.prayer = None;
    lay(&mut w, player, "poisoned", 3, None);
    run(&mut w, "use:orison");
    assert_eq!(degrees(&w, player, "poisoned"), 0);
    assert_eq!(degrees(&w, at(&w, S), "poisoned"), 0);
}

#[test]
fn a_hex_turns_on_its_caster_seen_and_a_holder_is_webbed() {
    let mut w = world(None);
    let player = w.state.player;
    let (j, h, k) = (at(&w, J), at(&w, H), at(&w, K));
    lay(&mut w, player, "ill_wished", 2, Some(j));
    lay(&mut w, player, "weakened", 2, Some(h));
    lay(&mut w, player, "held", 1, Some(k));
    let held = degrees(&w, player, "held");
    assert!(held > 0);
    run(&mut w, "use:orison");
    assert!(said(&w, "You are released, and it is returned."));
    for id in ["ill_wished", "weakened", "held"] {
        assert_eq!(degrees(&w, player, id), 0, "{id} released");
    }
    assert!(degrees(&w, j, "ill_wished") > 0, "the hex on its caster");
    assert_eq!(degrees(&w, h, "weakened"), 0, "unseen behind the wall");
    assert!(degrees(&w, k, "webbed") > 0, "the holder webbed");
    assert_eq!(degrees(&w, k, "held"), 0);
}

#[test]
fn the_dark_feeds_to_the_brim() {
    let mut w = world(None);
    let hunger = w.content.hunger().expect("hunger").states;
    w.player_mut()
        .satiation
        .as_mut()
        .expect("hungers")
        .nutrition = hunger.hungry;
    run(&mut w, "use:orison");
    assert!(said(&w, "You feel glutted."));
    let fed = w.state.player().satiation.expect("hungers").nutrition;
    // One short of Bloated, less what the prayer's own time digested.
    assert!(fed < hunger.bloated && fed > hunger.satiated, "{fed}");
}

#[test]
fn communion_prays_free_in_an_unholy_place() {
    let mut w = world(Some("cursed"));
    let player = w.state.player;
    w.player_mut().hp = 1;
    // Outside the room the strain is added.
    run(&mut w, "use:orison");
    assert_eq!(w.state.progress.prayer.map(|s| s.value), Some(100));
    // Inside, none is.
    w.state.progress.prayer = None;
    w.player_mut().hp = 1;
    w.state.actors.get_mut(&player).expect("player").pos = EAST;
    run(&mut w, "use:orison");
    assert!(said(&w, "Your wounds close, cold to the touch."));
    assert_eq!(w.state.progress.prayer.map(|s| s.value), Some(0));
    // A priest the holy powers answer has the talent, and it does nothing.
    w.state.progress.prayer = None;
    w.player_mut().alignment = Some(Score::new(0));
    w.player_mut().hp = 1;
    run(&mut w, "use:orison");
    assert_eq!(w.state.progress.prayer.map(|s| s.value), Some(100));
}

#[test]
fn communion_keeps_a_draining_room_off() {
    let mut w = world(Some("draining"));
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("player").pos = EAST;
    run(&mut w, "wait*3");
    assert_eq!(degrees(&w, player, "ebbing"), 0, "communes: no Ebbing");
    assert!(
        w.state.actors[&at(&w, H)].effects.has(status(&w, "ebbing")) || {
            // A kobold with no mana takes none: the room still ebbs those it can.
            w.state.actors[&at(&w, H)].max_mana(&w.content) <= 0
        }
    );
    w.player_mut().alignment = Some(Score::new(0));
    run(&mut w, "wait*3");
    assert!(
        degrees(&w, player, "ebbing") > 0,
        "the holy face ebbs as anyone"
    );
}

#[test]
fn communion_is_a_band_gated_passive() {
    let content = builtin();
    let book = content.talents();
    let def = book.get(book.id("communion").expect("communion"));
    assert!(def.alignment.is_some());
    assert!(def.effects.iter().any(|e| e.band_gated()));
}
