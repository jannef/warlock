//! Mining (spec §8 "Digging"; #1389, `world/mine.rs`): only the trained break anything out of a
//! wall they dig through, on a dungeon branch's floor; what comes out is drawn by rarity at the
//! floor's danger, mithril and adamantium only deeper than any bounded branch goes; a branch's
//! `ores` leaves out the others; Smelt makes the ore's ingot at an anvil; replays verify.

use super::*;
use crate::action::parse_actions;
use crate::content::{ContentFiles, Mined};
use crate::geom::Dir;
use crate::object::Stack;
use crate::replay::Replay;
use crate::skills::{SkillId, Trained};

/// The built-in content with a one-floor branch `mine` of `generator` at danger `danger`, its
/// `ores` line if given, nothing living there, and every band of `data/mining.toml` certain to be
/// tried (so a test sees the check and the table, not the chance); a dwarf warrior starts on its
/// floor with a pick-axe in the pack.
fn content(generator: &str, danger: u8, ores: &str) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    let branch = format!(
        "\n[[dungeon]]\nid = \"mine\"\nname = \"Mine\"\ngenerator = \"{generator}\"\ndepth = 1\n\
         size = [60, 30]\n{ores}\ndanger = {danger}\npopulation = [\"rat\"]\n\
         population_count = 0\n"
    );
    let dungeons = format!(
        "{}{branch}",
        include_str!("../../../../assets/data/dungeons.toml")
    );
    files.insert("data/dungeons.toml", &dungeons);
    let mining = include_str!("../../../../assets/data/mining.toml")
        .replace("chance = 5\n", "chance = 100\n")
        .replace("chance = 20\n", "chance = 100\n")
        .replace("chance = 40\n", "chance = 100\n");
    files.insert("data/mining.toml", &mining);
    files.insert(
        "scenarios/mine.toml",
        "name = \"Mine\"\nstart_level = \"mine:1\"\ncarry = [\"pick_axe\"]\n\
         character = { race = \"dwarf\", gender = \"male\", class = \"warrior\" }\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn skill(w: &World, id: &str) -> SkillId {
    w.content.skill_id(id).expect(id)
}

/// The player trained in `id` at `value` (`None`: untrained).
fn train(w: &mut World, id: &str, value: Option<u16>) {
    let id = skill(w, id);
    let skills = &mut w.player_mut().skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != id).collect();
    if let Some(value) = value {
        all.push((id, Trained { value, marks: 0 }));
    }
    *skills = all.into_iter().collect();
}

fn wall(w: &World) -> crate::content::TerrainId {
    w.content.terrain_id("wall").expect("wall")
}

/// What `tries` walls dug through beside the player give, the stacks on that cell.
fn mine_many(w: &mut World, tries: usize) -> Vec<Stack> {
    let at = w.player_pos().step(Dir::N);
    let wall = wall(w);
    for _ in 0..tries {
        w.mine(at, wall);
    }
    w.ground_truth().0.objects_at(at).to_vec()
}

/// The metal of an ore stack.
fn metal(w: &World, s: &Stack) -> String {
    let rules = w.content.mining().expect("mining.toml");
    assert_eq!(s.object, rules.ore);
    let m = s.parts(&w.content)[0].material;
    w.content.material(m).id.clone()
}

#[test]
fn only_the_trained_break_anything_out() {
    let content = content("rooms", 5, "");
    let mut w = World::new(content.clone(), "mine", Some(1)).expect("world");
    train(&mut w, "mining", None);
    let said = w.messages().len();
    assert!(mine_many(&mut w, 200).is_empty());
    assert_eq!(w.messages().len(), said, "nothing is said");

    let mut w = World::new(content, "mine", Some(1)).expect("world");
    train(&mut w, "mining", Some(10));
    let found = mine_many(&mut w, 200);
    assert!(!found.is_empty());
    let line = w.messages().iter().last().expect("a line").text.clone();
    assert!(line.starts_with("You break out "), "{line}");
    // Shallow: no mithril, no adamantium.
    let ore = w.content.mining().expect("mining").ore;
    for s in found.iter().filter(|s| s.object == ore) {
        let m = metal(&w, s);
        assert!(m != "mithril" && m != "adamantium", "{m}");
        assert!(s.count >= 1);
    }
}

#[test]
fn a_hand_made_level_gives_nothing() {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/cellar.toml",
        "name = \"Cellar\"\nmap = '''\n#####\n#.@.#\n#####\n'''\n[legend]\n\
         \"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/cellar.toml",
        "name = \"Cellar\"\nstart_level = \"cellar\"\n\
         character = { race = \"dwarf\", gender = \"male\", class = \"warrior\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("content"));
    let mut w = World::new(content, "cellar", None).expect("world");
    train(&mut w, "mining", Some(200));
    assert!(mine_many(&mut w, 300).is_empty());
}

#[test]
fn a_branch_s_ores_leave_out_the_others() {
    let content = content("rooms", 5, "ores = [\"copper\"]");
    let mut w = World::new(content, "mine", Some(1)).expect("world");
    train(&mut w, "mining", Some(100));
    let found = mine_many(&mut w, 400);
    let ore = w.content.mining().expect("mining").ore;
    let ores: Vec<String> = (found.iter().filter(|s| s.object == ore))
        .map(|s| metal(&w, s))
        .collect();
    assert!(!ores.is_empty());
    assert!(ores.iter().all(|m| m == "copper"), "{ores:?}");
    // Stone still comes out: the list is of ores only.
    assert!(found.iter().any(|s| s.object != ore));
}

#[test]
fn deep_rock_gives_mithril_and_bigger_margins_more_lumps() {
    let content = content("rooms", 90, "ores = [\"mithril\", \"adamantium\"]");
    let mut w = World::new(content, "mine", Some(3)).expect("world");
    train(&mut w, "mining", Some(200));
    let found = mine_many(&mut w, 400);
    let ore = w.content.mining().expect("mining").ore;
    let metals: Vec<String> = (found.iter().filter(|s| s.object == ore))
        .map(|s| metal(&w, s))
        .collect();
    assert!(metals.iter().any(|m| m == "mithril"), "{metals:?}");
    assert!(metals.iter().any(|m| m == "adamantium"), "{metals:?}");
    let named = Stack::new(ore, 1)
        .with_parts(vec![crate::item_parts::ItemPart::material(
            &w.content,
            None,
            w.content.material_id("mithril").expect("mithril"),
        )])
        .describe(&w.content);
    assert_eq!(named, "a lump of mithril ore");
}

#[test]
fn mithril_and_adamantium_lie_deeper_than_any_bounded_branch() {
    let content = Content::builtin().expect("content");
    let rules = content.mining().expect("mining.toml");
    let first = |name: &str| {
        let m = content.material_id(name).expect(name);
        rules
            .finds
            .iter()
            .find(|f| f.what == Mined::Ore(m))
            .expect(name)
            .danger
    };
    let (mithril, adamantium) = (first("mithril"), first("adamantium"));
    assert_eq!((mithril, adamantium), (60, 80));
    for def in content.dungeons() {
        if def.endless.is_some() {
            continue;
        }
        for depth in 1..=def.depth {
            let id = match def.portal {
                Some(_) => format!("{}@1:{depth}", def.id),
                None => format!("{}:{depth}", def.id),
            };
            let danger = content.floor_danger(&id, crate::content::MAX_LEVEL);
            assert!(danger < mithril, "{id}: {danger}");
        }
    }
    // Grundlēas reaches them.
    assert!(content.floor_danger("grundleas:80", 1) >= adamantium);
    // Every branch's ores are ores of the table.
    let names: Vec<_> = rules
        .ores()
        .map(|m| content.material(m).id.clone())
        .collect();
    for def in content.dungeons() {
        for m in def.ores.iter().flatten() {
            assert!(names.contains(m), "{}: {m}", def.id);
        }
    }
}

#[test]
fn bad_files_are_refused() {
    let text = include_str!("../../../../assets/data/mining.toml");
    for (from, to, says) in [
        ("skill = \"mining\"", "skill = \"digging\"", "unknown skill"),
        ("stat = \"strength\"", "stat = \"luck\"", "unknown stat"),
        ("ore = \"iron\"", "ore = \"oak\"", "unknown material"),
        ("ore = \"iron\"", "ore = \"oak\"", "unknown material"),
        ("ore = \"iron\"", "ore = \"ruby\"", "no ore"),
        ("ore = \"copper\"", "ore = \"iron\"", "twice"),
        ("rarity = 40", "rarity = 0", "`rarity`"),
        ("more_every = 6", "more_every = 0", "`more_every`"),
        ("most = 3", "most = 0", "`most`"),
        ("hardness = 400", "hardness = 0", "rise"),
        ("chance = 20", "chance = 101", "above 100"),
        ("gem = true", "gem = true\nore = \"tin\"", "exactly one"),
    ] {
        let bad = text.replacen(from, to, 1);
        assert_ne!(bad, text, "{from}");
        let mut files = ContentFiles::builtin();
        files.insert("data/mining.toml", &bad);
        let error = Content::load(&files).expect_err(from).to_string();
        assert!(error.contains(says), "{from}: {error}");
    }
    // A branch naming an ore the table has no row for.
    let mut files = ContentFiles::builtin();
    let dungeons = format!(
        "{}\n[[dungeon]]\nid = \"bad\"\ngenerator = \"rooms\"\ndepth = 1\nsize = [40, 20]\n\
         ores = [\"steel\"]\n",
        include_str!("../../../../assets/data/dungeons.toml")
    );
    files.insert("data/dungeons.toml", &dungeons);
    let error = Content::load(&files).expect_err("steel").to_string();
    assert!(error.contains("no ore row"), "{error}");
}

/// A forge room with an anvil north of the player, a dwarf craftsperson carrying `ore`.
fn forge(seed: u64) -> World {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/test_forge.toml",
        "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n#####\n#.A.#\n#.@.#\n\
         #####\n'''\n[legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
         \"A\" = \"anvil\"\n",
    );
    files.insert(
        "scenarios/test_forge.toml",
        "name = \"T\"\nstart_level = \"test_forge\"\n\
         character = { race = \"dwarf\", gender = \"female\", class = \"craftsperson\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("content"));
    let mut w = World::new(content, "test_forge", Some(seed)).expect("world");
    w.player_mut().inventory.clear();
    w
}

fn copper_ore(w: &World, count: u32) -> Stack {
    let rules = w.content.mining().expect("mining");
    let copper = w.content.material_id("copper").expect("copper");
    super::mine::ore(&w.content, rules, copper, count)
}

#[test]
fn smelt_makes_the_ore_s_ingot_at_an_anvil_for_the_trained() {
    let mut made = 0;
    for seed in 1..=20 {
        let mut w = forge(seed);
        train(&mut w, "metalcraft", Some(200));
        let ore = copper_ore(&w, 2);
        assert_eq!(ore.describe(&w.content), "2 lumps of copper ore");
        w.player_mut().inventory.push(ore);
        for action in parse_actions("craft:smelt:a").expect("parse") {
            w.apply(&action);
        }
        let inv = w.state.player().inventory.clone();
        assert_eq!(inv[0].count, 1, "one lump smelted");
        if let Some(ingot) = inv.get(1) {
            assert_eq!(ingot.describe(&w.content), "a copper ingot");
            made += 1;
        }
    }
    assert!(made >= 18, "{made}");
    // Not listed for one untrained in Metalcraft (not `anyone`).
    let mut w = forge(1);
    train(&mut w, "metalcraft", None);
    let smelt = w.content.recipes().id("smelt").expect("smelt");
    let rows = super::craft::recipe_rows(
        &w.content,
        w.lore(),
        w.state.current(),
        w.state.player(),
        w.content.recipe(smelt).skill,
    );
    assert!(rows.iter().all(|r| r.id != smelt));
}

/// The first seed (and way) of a caves floor where the player, running that way from the start,
/// stops by a stone wall that breaks, and digging through it breaks something out: the actions.
fn a_find(content: &Arc<Content>) -> (u64, String) {
    (1..=40)
        .flat_map(|seed| [Dir::N, Dir::E, Dir::S, Dir::W].map(|d| (seed, d)))
        .find_map(|(seed, dir)| {
            let mut w = World::new(content.clone(), "mine", Some(seed)).expect("world");
            for action in parse_actions(&format!("equip:tool:a run:{dir}")).expect("parse") {
                w.apply(&action);
            }
            let at = w.player_pos().step(dir);
            let level = w.state.current();
            let wall = w.content.terrain_id("wall");
            if !level.breaks_at(&w.content, at) || level.terrain_at(at) != wall {
                return None;
            }
            w.apply(&parse_actions(&format!("dig:{dir}")).expect("parse")[0]);
            let found = !w.ground_truth().0.objects_at(at).is_empty();
            found.then(|| (seed, format!("equip:tool:a run:{dir} dig:{dir}")))
        })
        .expect("a wall that gives something")
}

#[test]
fn digging_through_breaks_ore_out_and_replays_verify() {
    let content = content("caves", 5, "");
    let (seed, actions) = a_find(&content);
    let mut recorded = World::new(content.clone(), "mine", Some(seed)).expect("world");
    let mut replay = Replay::new(&recorded);
    for action in parse_actions(&actions).expect("parse") {
        recorded.apply(&action);
        replay.push(&action, &recorded);
    }
    let lines: Vec<&str> = recorded
        .messages()
        .iter()
        .map(|m| m.text.as_str())
        .collect();
    let crumbles = lines
        .iter()
        .position(|l| l.contains("crumbles"))
        .expect("crumbled");
    assert!(
        lines[crumbles + 1].starts_with("You break out "),
        "{lines:?}"
    );
    let replay = Replay::from_json(&replay.to_json()).expect("parses");
    let played = replay.run(content).expect("replay verifies");
    assert_eq!(played.state(), recorded.state());
}
