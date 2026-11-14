//! The bombers (the potions wave 2 design §5, §7; `hurl.rs`): a creature whose throw is a flask
//! hurls it, and it shatters and splashes as the player's does, its splash the thrower's; the
//! player who sees it is told and learns the flask; up close it fights in melee; nothing of the
//! throw lands; the three shipped bombers throw and carry flasks; the content check; replays.

use crate::action::parse_actions;
use crate::content::{ContentFiles, ZoneKind};
use crate::replay::Replay;
use crate::test_support::{builtin, run, said as texts};
use crate::world::*;

/// A lit hall of `map` (`@` the player, `k` a kobold alchemist, `g` a goblin firebomber, `c` a
/// cult chymist), the player with plenty of hit points.
fn hall(map: &str) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"k\" = {{ terrain = \"floor\", creature = \"kobold_alchemist\" }}\n\
             \"g\" = {{ terrain = \"floor\", creature = \"goblin_firebomber\" }}\n\
             \"c\" = {{ terrain = \"floor\", creature = \"cult_chymist\" }}\n"
        ),
    );
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 1000\n",
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "hall", None).expect("world")
}

fn first_of(w: &World, creature: &str) -> ActorId {
    let kind = w.content.creature_id(creature).expect("a type");
    (w.state.actors.values())
        .find(|a| a.creature == kind)
        .map(|a| a.id)
        .expect("one stands there")
}

/// Whether anything of `object` lies anywhere on the level.
fn lying(w: &World, object: &str) -> bool {
    let id = w.content.object_id(object).expect("object");
    let level = w.state.current();
    let (width, height) = (level.width(), level.height());
    (0..height).any(|y| {
        (0..width).any(|x| (level.objects_at(Pos::new(x, y)).iter()).any(|s| s.object == id))
    })
}

#[test]
fn the_kobold_alchemist_hurls_oil_and_the_floor_goes_slick_by_its_hand() {
    let mut w = hall("#########\n#@.....k#\n#########\n");
    let kobold = first_of(&w, "kobold_alchemist");
    let oil = w.content.object_id("flask_of_oil").expect("oil");
    run(&mut w, "wait*4");
    let said = texts(&w);
    assert!(
        said.contains(&"The kobold alchemist hurls a flask.".to_string()),
        "{said:?}"
    );
    assert!(
        said.contains(&"It was a flask of oil.".to_string()),
        "{said:?}"
    );
    assert!(w.state.known_scrolls.contains(&oil), "seen, so learned");
    let zones = w.zones();
    assert!(!zones.is_empty(), "the oil spread");
    assert!(zones.iter().all(|z| z.kind() == ZoneKind::Slick));
    assert!(
        zones.iter().all(|z| z.by() == kobold),
        "the zone is the thrower's"
    );
    assert!(!lying(&w, "flask_of_oil"), "nothing of the throw lands");
}

#[test]
fn the_firebombers_blast_hurts_and_is_no_attack_roll() {
    let mut w = hall("#########\n#@.....g#\n#########\n");
    let before = w.state.player().hp;
    let outs = run(&mut w, "wait*4");
    assert!(w.state.player().hp < before, "the blast struck");
    let thrown_attack = outs.iter().flat_map(|o| &o.events).any(|e| {
        matches!(
            e,
            Event::Attacked {
                kind: crate::action::AttackKind::Thrown,
                ..
            }
        )
    });
    assert!(!thrown_attack, "a flask has no to-hit roll");
    assert!(
        texts(&w).contains(&"It was a flask of fire.".to_string()),
        "{:?}",
        texts(&w)
    );
    assert!(!lying(&w, "flask_of_fire"));
}

#[test]
fn a_bomber_beside_its_foe_fights_in_melee() {
    let mut w = hall("#########\n#@k.....#\n#########\n");
    run(&mut w, "wait*4");
    assert!(
        !texts(&w).iter().any(|l| l.contains("hurls")),
        "{:?}",
        texts(&w)
    );
    assert!(w.zones().is_empty());
}

#[test]
fn the_three_bombers_throw_flasks_and_carry_them() {
    let content = builtin();
    for (bomber, flask) in [
        ("kobold_alchemist", "flask_of_oil"),
        ("goblin_firebomber", "flask_of_fire"),
        ("cult_chymist", "stinking_flask"),
    ] {
        let def = content.creature(content.creature_id(bomber).expect(bomber));
        let flask = content.object_id(flask).expect(flask);
        let throw = def.throw.as_ref().expect("a throw");
        assert_eq!(throw.object, flask, "{bomber}");
        assert!(content.object(flask).flask.is_some(), "{bomber}: a flask");
        let carries = (def.gear.lines.iter())
            .flat_map(|l| &l.one_of)
            .flat_map(|(stacks, _)| stacks)
            .any(|s| s.object == flask);
        assert!(carries, "{bomber} drops its kind");
    }
}

#[test]
fn a_throw_without_damage_must_be_of_a_flask() {
    let creature = |object: &str| {
        format!(
            "[[creature]]\nid = \"odd\"\nname = \"odd\"\nglyph = \"o\"\nfg = \"#808080\"\n\
             weight = 30000\n[creature.throw]\nobject = \"{object}\"\nrange = 6\nverb = \"hurls\"\n"
        )
    };
    let mut files = ContentFiles::builtin();
    files.insert("data/creatures/odd.toml", &creature("rock"));
    let err = Content::load(&files).expect_err("no damage").to_string();
    assert!(err.contains("no `damage`"), "{err}");
    files.insert("data/creatures/odd.toml", &creature("flask_of_tar"));
    Content::load(&files).expect("a flask needs none");
}

#[test]
fn a_bombers_throws_replay() {
    let mut w = hall("##########\n#@......c#\n##########\n");
    let content = w.content.clone();
    let mut replay = Replay::new(&w);
    for action in parse_actions("wait*5").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(
        texts(&w)
            .iter()
            .any(|l| l == "The cult chymist hurls a flask."),
        "{:?}",
        texts(&w)
    );
    let played = replay.run(content).expect("replays");
    assert_eq!(played.state_hash(), w.state_hash());
}
