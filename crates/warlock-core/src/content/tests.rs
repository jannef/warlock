use super::*;
use crate::geom::{Dir, Pos};
use std::path::Path;

#[test]
fn builtin_content_loads() {
    let content = Content::builtin().expect("built-in content must be valid");
    assert!(content.scenario("showcase").is_some());
    assert!(content.level("hub").is_some());
    assert!(content.terrain_id("wall").is_some());
    assert!(content.glyphs().contains(&'#'));
}

#[test]
fn fingerprint_is_stable_and_tracks_ids() {
    let a = Content::builtin()
        .expect("content")
        .fingerprint_for("showcase");
    assert_eq!(
        a,
        Content::builtin()
            .expect("content")
            .fingerprint_for("showcase")
    );
    let mut files = ContentFiles::builtin();
    let hub = (files.get("campaigns/showcase/levels/hub.toml"))
        .expect("hub")
        .to_string();
    files.insert("campaigns/showcase/levels/hub_copy.toml", &hub);
    let other = Content::load(&files).expect("loads");
    assert_ne!(a, other.fingerprint_for("showcase"));
    // Spells enter it: saves will name them by index (spec §8 "Spells").
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/spells/zzz_zap.toml",
        "[[spell]]\nid = \"zap\"\nname = \"Zap\"\ndescription = \"x\"\nlevel = 0\n\
         school = \"evocation\"\nkind = \"damage\"\nshape = \"bolt\"\nresolution = \"auto\"\n\
         damage_type = \"electricity\"\nrange = 5\ndamage = \"1d4\"\n",
    );
    let with_spell = Content::load(&files).expect("loads");
    let last = SpellId(u16::try_from(with_spell.spells().len() - 1).expect("few"));
    assert_eq!(with_spell.spell_id("zap"), Some(last), "last in path order");
    assert_ne!(a, with_spell.fingerprint_for("showcase"));
}

#[test]
fn a_bottom_is_a_level_of_its_campaign_with_a_way_back_up() {
    const TOWN: &str = "campaigns/ascmearc/levels/gimmwic.toml";
    const BRANCH: &str = "campaigns/ascmearc/data/dungeons/deopgang.toml";
    let edited = |path: &str, from: &str, to: &str| {
        let mut files = ContentFiles::builtin();
        let text = files.get(path).expect("the file").to_string();
        assert!(text.contains(from), "{from}");
        files.insert(path, &text.replace(from, to));
        Content::load(&files).unwrap_err().to_string()
    };
    // The town's `<` must lead back to the deepest floor.
    let err = edited(TOWN, "to = \"deopgang:11\"", "to = \"deopgang:10\"");
    assert!(err.contains("`bottom = \"gimmwic\"`"), "{err}");
    // A level of another campaign is no bottom.
    let err = edited(BRANCH, "bottom = \"gimmwic\"", "bottom = \"hub\"");
    assert!(err.contains("`bottom = \"hub\"`"), "{err}");
}

fn minimal_files(level_map: &str, extra_legend: &str) -> ContentFiles {
    let mut files = ContentFiles::default();
    files.insert("data/item_classes.toml", BUILTIN_ITEM_CLASSES);
    files.insert(
        "data/terrain.toml",
        r##"
        [[terrain]]
        id = "void"
        name = "rock"
        glyph = " "
        fg = "#000000"
        walkable = false
        transparent = false
        [[terrain]]
        id = "floor"
        name = "floor"
        glyph = "."
        fg = "#808080"
        [[terrain]]
        id = "wall"
        name = "wall"
        glyph = "#"
        fg = "#a0a0a0"
        walkable = false
        transparent = false
        "##,
    );
    files.insert(
        "data/creatures.toml",
        r##"
        [[creature]]
        id = "player"
        name = "you"
        glyph = "@"
        fg = "#ffffff"
        weight = 70000
        [[creature]]
        id = "rat"
        name = "rat"
        glyph = "r"
        fg = "#806040"
        weight = 4000
        "##,
    );
    files.insert(
        "data/legend.toml",
        "[legend]\n\" \" = \"void\"\n\".\" = \"floor\"\n\"#\" = \"wall\"\n",
    );
    files.insert(
        "levels/test.toml",
        &format!(
            "name = \"Test\"\nmap = '''\n{level_map}'''\n[legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n{extra_legend}"
        ),
    );
    files.insert(
        "scenarios/test.toml",
        "name = \"T\"\nstart_level = \"test\"\n",
    );
    files
}

/// A creature entry of nothing but the required fields.
fn creature_entry(id: &str, glyph: char) -> String {
    format!(
        "[[creature]]\nid = \"{id}\"\nname = \"{id}\"\nglyph = \"{glyph}\"\nfg = \"#ffffff\"\n\
         weight = 1000\n"
    )
}

/// A table split over its single file and a directory: the single file's entries come
/// first, then the directory's files in path order (subdirectories included), a file of
/// comments only adds nothing, and `remove` drops a directory whole.
#[test]
fn a_table_directory_loads_in_path_order_after_the_single_file() {
    let mut files = minimal_files("#@#\n", "");
    files.insert("data/creatures/README.toml", "# Notes only.\n");
    files.insert(
        "data/creatures/b/kobold.toml",
        &creature_entry("kobold", 'k'),
    );
    files.insert("data/creatures/a.toml", &creature_entry("bat", 'b'));
    files.insert(
        "data/creatures/c.toml",
        &format!(
            "{}{}",
            creature_entry("deer", 'd'),
            creature_entry("beetle", 'a')
        ),
    );
    let table: Vec<&str> = files.table("creatures").map(|(p, _)| p).collect();
    assert_eq!(
        table,
        [
            "data/creatures.toml",
            "data/creatures/README.toml",
            "data/creatures/a.toml",
            "data/creatures/b/kobold.toml",
            "data/creatures/c.toml",
        ]
    );
    let content = Content::load(&files).expect("valid");
    let ids: Vec<&str> = (content.creature_defs().iter())
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(ids, ["player", "rat", "bat", "kobold", "deer", "beetle"]);
    assert_eq!(content.creature_id("deer"), Some(CreatureId(4)));
    // Without the single file, the directory alone is the table.
    files.remove("data/creatures.toml");
    files.insert("data/creatures/player.toml", &creature_entry("player", '@'));
    let content = Content::load(&files).expect("valid");
    let ids: Vec<&str> = (content.creature_defs().iter())
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(ids, ["bat", "kobold", "deer", "beetle", "player"]);
    // `edit_table` rewrites every file of the table.
    files.edit_table("creatures", |t| t.replace("weight = 1000", "weight = 2000"));
    let content = Content::load(&files).expect("valid");
    assert!(content.creature_defs().iter().all(|c| c.weight == 2000));
    // Removing the directory leaves no creature file at all.
    files.remove("data/creatures");
    assert_eq!(files.table("creatures").count(), 0);
    let err = Content::load(&files).unwrap_err();
    assert_eq!(err.file, "data/creatures.toml");
    assert!(err.message.contains("missing"), "{err}");
}

/// An id in two files of a table is an error naming both files; a file that does not parse
/// is an error naming it.
#[test]
fn a_duplicate_id_across_table_files_names_both_files() {
    let mut files = minimal_files("#@#\n", "");
    files.insert("data/creatures/rat.toml", &creature_entry("rat", 'r'));
    let err = Content::load(&files).unwrap_err();
    assert_eq!(err.file, "data/creatures/rat.toml");
    assert_eq!(
        err.message,
        "duplicate creature id `rat` (also in data/creatures.toml)"
    );
    files.remove("data/creatures/");
    let iron = "[[material]]\nid = \"iron\"\nname = \"iron\"\n";
    files.insert("data/materials.toml", iron);
    files.insert("data/materials/iron.toml", iron);
    let err = Content::load(&files).unwrap_err();
    assert_eq!(err.file, "data/materials/iron.toml");
    assert_eq!(
        err.message,
        "duplicate material id `iron` (also in data/materials.toml)"
    );
    files.remove("data/materials/");
    let nail = "[[object]]\nid = \"nail\"\nname = \"nail\"\nglyph = \"(\"\nfg = \"#ffffff\"\nclass = \"tools\"\n\
                materials = [\"iron\"]\nweight = 5\nvalue = 1\n";
    files.insert("data/objects/a.toml", nail);
    files.insert("data/objects/b.toml", nail);
    let err = Content::load(&files).unwrap_err();
    assert_eq!(err.file, "data/objects/b.toml");
    assert_eq!(
        err.message,
        "duplicate object id `nail` (also in data/objects/a.toml)"
    );
    files.insert("data/objects/b.toml", "[[object]\n");
    let err = Content::load(&files).unwrap_err();
    assert_eq!(err.file, "data/objects/b.toml");
    files.remove("data/objects/b.toml");
    let content = Content::load(&files).expect("valid");
    assert_eq!(content.object_id("nail"), Some(ObjectId(0)));
}

/// `from_dir` reads subdirectories like `build.rs` embeds them (one walk, `walk.rs`): the
/// same paths, outside `campaigns/`, `data/`, and `vaults/` nothing, the built-in set being
/// exactly what is on disk under `assets/` (with the `showcase` feature or without).
#[test]
fn from_dir_reads_nested_directories_like_the_build() {
    let root = std::env::temp_dir().join(format!("warlock-content-{}", std::process::id()));
    let deep = root.join("data/creatures/deep");
    std::fs::create_dir_all(&deep).expect("temp dir");
    std::fs::create_dir_all(root.join("levels")).expect("temp dir");
    std::fs::write(root.join("data/legend.toml"), "x = 1\n").expect("write");
    std::fs::write(root.join("data/creatures/rat.toml"), "a = 1\n").expect("write");
    std::fs::write(deep.join("bat.toml"), "b = 2\n").expect("write");
    std::fs::write(deep.join("notes.txt"), "not content\n").expect("write");
    std::fs::write(root.join("levels/hub.toml"), "c = 3\n").expect("write");
    let files = ContentFiles::from_dir(&root).expect("readable");
    std::fs::remove_dir_all(&root).expect("cleanup");
    let paths: Vec<&str> = files.files.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            "data/creatures/deep/bat.toml",
            "data/creatures/rat.toml",
            "data/legend.toml",
        ]
    );
    let table: Vec<(&str, &str)> = files.table("creatures").collect();
    assert_eq!(
        table,
        [
            ("data/creatures/deep/bat.toml", "b = 2\n"),
            ("data/creatures/rat.toml", "a = 1\n"),
        ]
    );
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let on_disk = ContentFiles::from_dir(&assets).expect("assets");
    assert_eq!(on_disk.files, ContentFiles::builtin().files);
}

#[test]
fn short_rows_are_padded_with_void() {
    let content = Content::load(&minimal_files("####\n#@.\n####\n", "")).unwrap();
    let level = content.level("test").unwrap();
    assert_eq!((level.terrain.width(), level.terrain.height()), (4, 3));
    assert_eq!(
        level.terrain[Pos::new(3, 1)],
        content.terrain_id("void").unwrap()
    );
    assert_eq!(level.player_start, Some(Pos::new(1, 1)));
}

#[test]
fn unknown_legend_character_is_reported_with_position() {
    let err = Content::load(&minimal_files("#@X#\n", "")).unwrap_err();
    assert_eq!(err.file, "levels/test.toml");
    assert!(err.message.contains("`X` at (2, 0)"), "{}", err.message);
}

#[test]
fn patrol_routes_must_loop_and_stay_walkable() {
    let map = "#####\n#@.r#\n#...#\n#####\n";
    let good = "\"r\" = { terrain = \"floor\", creature = \"rat\", route = \"S W S\" }";
    let err = Content::load(&minimal_files(map, good)).unwrap_err();
    assert!(
        err.message.contains("leaves walkable terrain"),
        "{}",
        err.message
    );

    let not_loop = "\"r\" = { terrain = \"floor\", creature = \"rat\", route = \"S W\" }";
    let err = Content::load(&minimal_files(map, not_loop)).unwrap_err();
    assert!(err.message.contains("loop back"), "{}", err.message);

    let ok = "\"r\" = { terrain = \"floor\", creature = \"rat\", route = \"S W N E\" }";
    let content = Content::load(&minimal_files(map, ok)).unwrap();
    assert_eq!(content.level("test").unwrap().creatures[0].route.len(), 4);
}

#[test]
fn effects_load_and_their_glyphs_join_the_atlas() {
    let content = Content::builtin().unwrap();
    let brazier = content.terrain(content.terrain_id("brazier").unwrap());
    assert!(brazier.glow > 0.0 && brazier.haze > 0.0);
    assert!(
        brazier
            .particles
            .iter()
            .any(|e| e.on == ParticleTrigger::Ambient)
    );
    let water = content.terrain(content.terrain_id("water").unwrap());
    assert!(
        water
            .particles
            .iter()
            .any(|e| e.on == ParticleTrigger::Step)
    );
    let glyphs = content.glyphs();
    for e in brazier.particles.iter().chain(&water.particles) {
        assert!(e.glyphs.chars().all(|c| glyphs.contains(&c)));
    }
}

#[test]
fn bad_effects_are_reported_with_the_terrain() {
    let mut files = minimal_files("#@#\n", "");
    let terrain = files.get("data/terrain.toml").unwrap().to_string();
    let sparks = "glyphs = \"'\"\ncolors = []\nlife = 1\n";
    files.insert(
        "data/terrain.toml",
        &format!("{terrain}\n[[terrain.particles]]\n{sparks}"),
    );
    let err = Content::load(&files).unwrap_err();
    assert_eq!(err.file, "data/terrain.toml");
    assert!(
        err.message
            .contains("terrain `wall`: particles: particles need at least one color"),
        "{}",
        err.message
    );
    files.insert("data/terrain.toml", &format!("{terrain}glow = nan\n"));
    let err = Content::load(&files).unwrap_err();
    assert!(err.message.contains("`glow` and `haze`"), "{}", err.message);
    let sparks = sparks.replace("[]", "[\"#ffa040\"]");
    files.insert(
        "data/terrain.toml",
        &format!("{terrain}\n[[terrain.particles]]\n{sparks}rate = 2\n"),
    );
    assert!(Content::load(&files).is_ok());
}

const LIGHTS: &str = "[[light]]\nid = \"torch\"\ncolor = [300, 200, 100, 0]\nradius = 6\n\
                      stability.intensity = { amount = 0.2, speed = 1.5 }\n\
                      stability.hue = { degrees = 5, speed = 0.4 }\n";

#[test]
fn builtin_skills_and_who_trains_them() {
    let content = Content::builtin().expect("content");
    let ids: Vec<&str> = content.skills().iter().map(|s| s.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "alchemy",
            "appraise",
            "athletics",
            "balance",
            "bluff",
            "concentration",
            "decipher_script",
            "detect_traps",
            "disable_device",
            "enchanting",
            "escape_artist",
            "handle_animal",
            "heal",
            "herbalism",
            "initiative",
            "intimidate",
            "jewelcraft",
            "knowledge",
            "listen",
            "metalcraft",
            "mining",
            "music",
            "negotiation",
            "open_lock",
            "search",
            "sneak_attack",
            "spellcraft",
            "spot",
            "steal",
            "stealth",
            "survival",
            "swim",
            "tumble",
            "woodcraft"
        ]
    );
    for (id, skill) in content.skill_ids() {
        assert_eq!(content.skill_id(&skill.id), Some(id));
        assert!(!skill.name.is_empty() && !skill.description.is_empty());
    }
    let names = |grants: &[(SkillId, crate::dice::Dice)]| -> Vec<&str> {
        grants
            .iter()
            .map(|&(id, _)| content.skill(id).id.as_str())
            .collect()
    };
    let class = |id: &str| names(&content.class(id).expect("class").skills);
    let race = |id: &str| names(&content.race(id).expect("race").skills);
    assert_eq!(
        class("thief"),
        [
            "appraise",
            "detect_traps",
            "disable_device",
            "escape_artist",
            "initiative",
            "negotiation",
            "open_lock",
            "search",
            "sneak_attack",
            "spot",
            "steal",
            "stealth",
            "tumble"
        ]
    );
    assert_eq!(
        class("ranger"),
        [
            "athletics",
            "decipher_script",
            "handle_animal",
            "initiative",
            "listen",
            "spellcraft",
            "spot",
            "survival",
            "swim",
            "woodcraft"
        ]
    );
    assert_eq!(
        class("wizard"),
        [
            "concentration",
            "decipher_script",
            "knowledge",
            "spellcraft"
        ]
    );
    assert_eq!(class("commoner"), ["athletics", "handle_animal"]);
    assert!(content.classes().iter().all(|c| !c.skills.is_empty()));
    assert_eq!(race("elf"), ["listen", "spot"]);
    assert_eq!(race("dwarf"), ["metalcraft", "mining", "search"]);
    assert_eq!(race("halfling"), ["herbalism", "listen", "stealth"]);
    assert_eq!(race("gnome"), ["appraise", "jewelcraft", "listen"]);
    assert_eq!(race("half_orc"), ["intimidate"]);
    assert_eq!(race("kobold"), ["detect_traps", "disable_device", "mining"]);
    assert!(race("human").is_empty());
    assert!(!content.race("human").expect("human").class_skill.is_zero());
    // Creatures may list skills; the player's come from the character.
    let deer = content.creature_id("deer").expect("deer");
    let listen = content.skill_id("listen").expect("listen");
    assert_eq!(content.creature(deer).skills.value(listen), Some(30));
    let player = content.creature_id("player").expect("player");
    assert!(content.creature(player).skills.is_empty());
}

#[test]
fn creature_skills_must_exist_and_the_player_lists_none() {
    // The built-in creature table as one text, for the checks that an edit applies.
    let creatures: String = (ContentFiles::builtin().table("creatures"))
        .map(|(_, t)| t.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    for (from, to, why) in [
        (
            "skills = { listen = 30 }",
            "skills = { tracking = 30 }",
            "creature `deer`: skills: unknown skill `tracking`",
        ),
        (
            "hearing = 10\nweight = 70000",
            "hearing = 10\nweight = 70000\nskills = { stealth = 5 }",
            "the player's skills come from their race and class",
        ),
    ] {
        assert!(creatures.contains(from), "{from}");
        let mut files = ContentFiles::builtin();
        files.edit_table("creatures", |t| t.replacen(from, to, 1));
        let err = Content::load(&files).unwrap_err();
        assert!(err.message.contains(why), "{why}: {err}");
    }
    // Without a skills file, nothing may name a skill.
    let mut files = ContentFiles::builtin();
    files.remove("data/skills.toml");
    let err = Content::load(&files).unwrap_err();
    assert!(err.message.contains("unknown skill"), "{err}");
}

#[test]
fn every_drawn_creature_has_a_rarity() {
    // Spec §7 "Danger levels": the shipped tiers, commonest first, and every type that a
    // population draws names one; the more dangerous and rewarding, the rarer.
    let content = Content::builtin().expect("content");
    let tiers: Vec<(&str, u16)> = (content.creature_rarities().iter())
        .map(|r| (r.id.as_str(), r.factor))
        .collect();
    assert_eq!(
        tiers,
        [
            ("common", 100),
            ("uncommon", 50),
            ("rare", 25),
            ("very_rare", 10)
        ]
    );
    let factor = |id: &str| {
        let def = content.creature(content.creature_id(id).expect(id));
        content.creature_rarity(def.rarity).factor
    };
    assert!(factor("kobold_shaman") < factor("kobold_archer"));
    assert!(factor("kobold_archer") < factor("kobold"));
    assert!(factor("goblin_chief") < factor("goblin"));
    assert_eq!((factor("rat"), factor("bog_hag")), (100, 10));
    let branches = content.dungeons().filter_map(|d| d.spawning.as_ref());
    let levels = content
        .levels()
        .filter_map(|l| Some(&l.spawning.as_ref()?.1));
    for s in branches.chain(levels) {
        for &c in &s.population {
            let def = content.creature(c);
            assert!(def.rarity < content.creature_rarities().len(), "{}", def.id);
        }
    }
}

/// The generic creature list strays come from (spec §7 "Danger levels"): every type the
/// campaign knows that may stray, whether a population names it or not, and what lives in
/// its dungeons; never a unique, a person, a keeper, a hand-placed guardian, or an animated
/// object outside the populations.
#[test]
fn the_generic_list_is_every_hostile_type_without_uniques_or_people() {
    let content = Content::builtin().expect("content");
    let generic = |branch: &str| -> BTreeSet<&str> {
        let s = content.dungeon(branch).and_then(|d| d.spawning.as_ref());
        (s.expect(branch).generic.iter())
            .map(|&c| content.creature(c).id.as_str())
            .collect()
    };
    let barrow = generic("cealdhlaw");
    // One list for the whole campaign's open branches.
    assert_eq!(barrow, generic("fennmynster"));
    assert_eq!(barrow, generic("fyrdelf"));
    // Every hostile type of no campaign or of the Ascmearc that is no unique and no
    // animated object is in it, populations or none.
    for def in content.creature_defs() {
        let id = def.id.as_str();
        let campaign = def.campaign.as_deref();
        let player = content.creature_id("player").expect("player");
        let c = content.creature_id(id).expect("id");
        let hostile = content.stance(c, player) == Stance::Hostile;
        // The mimics come only through chests (`stray = false`), the gates only through a
        // miscast (#1295), and what another type dies into only out of its corpse (the
        // hydra's heads).
        let offspring = (content.creature_defs().iter())
            .any(|d| d.dies_into.as_ref().is_some_and(|i| i.pool.contains(&c)));
        // A Pit's warden only through its hall (#1298).
        let gate = (def.tags.iter().any(|t| t == "gate") || content.is_layer_boss(c)) && !def.stray;
        let placed =
            def.animated.is_some() || def.mimic.is_some() || gate || (offspring && !def.stray);
        if hostile && !def.unique && !placed && campaign != Some("showcase") {
            assert!(
                def.stray,
                "{id}: no shipped type is marked `stray = false` yet"
            );
            assert!(barrow.contains(id), "{id} strays");
        }
    }
    for id in [
        "rat",
        "skeleton",
        "giant_leech",
        "cult_priest",
        "ogre",
        "cinder_elemental",
        // In no population: met only as strays.
        "lich",
        "hobgoblin_warlord",
        "thunderhead",
    ] {
        assert!(barrow.contains(id), "{id} strays");
    }
    for id in [
        "player",
        "bog_hag",
        "fire_drake",
        "shopkeeper",
        "duergar_trader",
        "villager",
        "innkeeper",
        "reeve",
        "sister",
        "mine_factor",
        "hermit",
        "bard",
        "gardener",
        // Neutral to the player.
        "deer",
        "dryad",
    ] {
        assert!(!barrow.contains(id), "{id} never strays");
    }
    // Sorted by type, as the draws need.
    let s = content.dungeon("fyrdelf").and_then(|d| d.spawning.as_ref());
    assert!(s.expect("fyrdelf").generic.is_sorted());
}

/// A branch with `stray_pool = "populations"` (the guild's four lairs) keeps the old list:
/// only what the campaign's populations name, uniques left out.
#[test]
fn a_lair_draws_its_strays_from_the_populations_only() {
    let content = Content::builtin().expect("content");
    let spawning =
        |branch: &str| (content.dungeon(branch).and_then(|d| d.spawning.as_ref())).expect(branch);
    let mut lives = BTreeSet::new();
    for d in content.dungeons.values() {
        if d.campaign.as_deref() == Some("ascmearc")
            && let Some(s) = &d.spawning
        {
            lives.extend((s.population.iter()).filter(|&&c| !content.creature(c).unique));
        }
    }
    let lives: Vec<CreatureId> = lives.into_iter().collect();
    for lair in ["wyrtholm", "wulfeshol", "banfeld", "brocenburg"] {
        let s = spawning(lair);
        assert_eq!(s.pool, danger::StrayPool::Populations, "{lair}");
        assert_eq!(s.generic, lives, "{lair}");
    }
    let lich = content.creature_id("lich").expect("lich");
    assert!(!spawning("wyrtholm").generic.contains(&lich));
    assert!(spawning("cealdhlaw").generic.contains(&lich));
    assert_eq!(spawning("cealdhlaw").pool, danger::StrayPool::All);
}

/// The owner's rule (2026-10-27): with the exploding out-of-depth draw, anything the
/// campaign knows may come to a level-1 floor, even a type no population names.
#[test]
fn a_first_floor_draw_gone_far_out_of_depth_brings_a_type_of_no_population() {
    let content = Content::builtin().expect("content");
    let barrow = (content
        .dungeon("cealdhlaw")
        .and_then(|d| d.spawning.as_ref()))
    .expect("cealdhlaw");
    let lives: BTreeSet<CreatureId> = (content.dungeons.values())
        .filter_map(|d| d.spawning.as_ref())
        .flat_map(|s| s.population.iter().copied())
        .collect();
    let once = BTreeSet::new();
    let own = content.candidates(&barrow.population, &once);
    let generic = content.candidates(&barrow.generic, &once);
    // Every roll deeper hits, as the 5 % may again and again: 51 dangers past the first.
    let odds = danger::Odds {
        stray: 0,
        out_of_depth: 100,
        deeper: 100,
        deeper_cap: 50,
        dread_one_in: 0,
        neutral: 100,
    };
    let mut rng = crate::rng::Rng::stream(1, "test");
    for _ in 0..50 {
        let c = danger::draw(&mut rng, &own, &generic, 1, &odds).expect("a draw");
        let def = content.creature(c);
        assert!(def.level >= 40, "{} at level {}", def.id, def.level);
        assert!(!lives.contains(&c), "{} lives in no population", def.id);
    }
}

#[test]
fn builtin_races_and_classes_load_in_the_order_offered() {
    let content = Content::builtin().expect("content");
    let races: Vec<&str> = (content.races().list.iter())
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(
        races,
        [
            "elf",
            "half_elf",
            "human",
            "dwarf",
            "halfling",
            "gnome",
            "half_orc",
            "goblin",
            "drow",
            "kobold",
            "lizardfolk",
            "duergar",
            "ratfolk"
        ]
    );
    // Every race offers names to every gender, and every scenario names its character.
    for race in &content.races().list {
        for gender in crate::character::Gender::ALL {
            let names = content.names().offered(&race.id, gender);
            assert!(names.len() >= 10, "{} {gender:?}: {names:?}", race.id);
        }
    }
    // (But the spell circles' hall, which has no character: its player is given every buff
    // and hex spell instead of a class, docs/design/buff-circles.md §13.2.)
    for scenario in content.scenarios() {
        let named = scenario.character_name.is_some();
        assert_eq!(named, scenario.id != "circles", "{}", scenario.id);
    }
    let classes: Vec<&str> = content.classes().iter().map(|c| c.id.as_str()).collect();
    assert_eq!(
        classes,
        [
            "berserker",
            "bard",
            "priest",
            "druid",
            "ascetic",
            "paladin",
            "ranger",
            "thief",
            "wizard",
            "witch",
            "warrior",
            "aristocrat",
            "commoner",
            "craftsperson",
            "merchant"
        ]
    );
    for race in &content.races().list {
        assert!(content.vision(&race.vision).is_some(), "{}", race.id);
        assert!(!race.description.is_empty() && !race.name.is_empty());
    }
    for class in content.classes() {
        assert!(!class.description.is_empty() && !class.name.is_empty());
    }
    assert_eq!(
        content.race("dwarf").map(|r| r.vision.as_str()),
        Some("darkvision")
    );
    // Sizes (spec §8 "Burden"): halflings and gnomes are small; dwarves shrug off Burdened.
    let size = |race: &str| {
        let race = content.race(race).expect(race);
        let size = content.size(race.size);
        (size.id.as_str(), size.carry, race.ignores_burdened)
    };
    assert_eq!(size("halfling"), ("small", 75, false));
    assert_eq!(size("gnome"), ("small", 75, false));
    assert_eq!(size("dwarf"), ("medium", 100, true));
    assert_eq!(size("human"), ("medium", 100, false));
    let creature = |id: &str| {
        let def = content.creature(content.creature_id(id).expect(id));
        content.size(def.size).id.as_str()
    };
    assert_eq!((creature("kobold"), creature("deer")), ("small", "medium"));
    assert_eq!(
        content
            .class("wizard")
            .map(|c| c.stats[crate::stats::Stat::Intelligence].to_string()),
        Some("2d2+1".to_string())
    );
    let h = content.horoscope().expect("a horoscope");
    assert_eq!(
        (h.animals.len(), h.elements.len(), h.great_cycle()),
        (12, 5, 60)
    );
    let elements: Vec<&str> = h.elements.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(elements, ["Wood", "Fire", "Earth", "Metal", "Water"]);
    for part in h.animals.iter().chain(&h.elements) {
        assert!(
            !part.description.is_empty() && !part.name.is_empty(),
            "{}",
            part.id
        );
        // The horoscope stays flat and small.
        for (_, by) in part.stats.iter() {
            assert!(by.is_fixed() && by.min().abs() <= 2, "{} {by}", part.id);
        }
    }
    assert!(content.race("orc").is_none() && content.class("necromancer").is_none());
}

#[test]
fn scenario_characters_and_race_eyes_must_exist() {
    let bad = "name = \"B\"\nstart_level = \"hub\"\n\
               character = { race = \"orc\", gender = \"male\", class = \"warrior\" }\n";
    let mut files = ContentFiles::builtin();
    files.insert("scenarios/bad.toml", bad);
    let err = Content::load(&files).unwrap_err();
    assert_eq!(err.file, "scenarios/bad.toml");
    assert!(
        err.message.contains("character: unknown race `orc`"),
        "{}",
        err.message
    );
    // Content without races has no characters to give.
    let mut files = minimal_files("#@#\n", "");
    files.insert(
        "scenarios/bad.toml",
        &bad.replace("start_level = \"hub\"", "start_level = \"test\""),
    );
    let err = Content::load(&files).unwrap_err();
    assert!(
        err.message.contains("unknown race `orc`"),
        "{}",
        err.message
    );
    // A race's eyes are a known vision profile.
    let mut files = ContentFiles::builtin();
    let races = files.get("data/races.toml").unwrap().replacen(
        "vision = \"darkvision\"",
        "vision = \"tremorsense\"",
        1,
    );
    files.insert("data/races.toml", &races);
    let err = Content::load(&files).unwrap_err();
    assert_eq!(err.file, "data/races.toml");
    assert!(
        err.message
            .contains("race `dwarf`: unknown vision profile `tremorsense`"),
        "{}",
        err.message
    );
}

#[test]
fn builtin_lights_and_vision_profiles_load() {
    let content = Content::builtin().unwrap();
    let ids: Vec<_> = content.vision_profiles().iter().map(|v| &v.id).collect();
    assert_eq!(ids, ["normal", "low_light", "darkvision", "uv_sight"]);
    assert_eq!(content.vision("darkvision").map(|v| v.darkvision), Some(5));
    let torch = content.light(content.light_id("torch").unwrap());
    assert!(!torch.stability.is_stable() && torch.radius > 0);
    let crystal = content.light(content.light_id("crystal_red").unwrap());
    assert!(crystal.stability.is_stable(), "magical light is constant");
    let kobold = content.creature(content.creature_id("kobold").unwrap());
    assert_eq!(kobold.light, content.light_id("torch"));
    let hub = content.level("hub").unwrap();
    assert!(!hub.lights.is_empty());
}

#[test]
fn missing_vision_file_means_light_is_not_needed() {
    let content = Content::load(&minimal_files("#@#\n", "")).unwrap();
    let profiles = content.vision_profiles();
    assert_eq!(profiles.len(), 1);
    assert_eq!(
        (profiles[0].id.as_str(), profiles[0].threshold),
        ("normal", 0)
    );
    assert!(content.light_presets().is_empty());
}

#[test]
fn levels_place_lights_and_set_ambient_light() {
    let mut files = minimal_files(
        "#@T#\n",
        "\"T\" = { terrain = \"floor\", light = \"torch\" }",
    );
    files.insert("data/lights.toml", LIGHTS);
    let text = files
        .get("levels/test.toml")
        .unwrap()
        .replace("name = \"Test\"", "name = \"Test\"\nambient = [1, 2, 3, 4]");
    files.insert("levels/test.toml", &text);
    let content = Content::load(&files).unwrap();
    let level = content.level("test").unwrap();
    assert_eq!(level.ambient, crate::light::Light::new(1, 2, 3, 4));
    assert_eq!(
        level.lights.get(&Pos::new(2, 0)).copied(),
        content.light_id("torch")
    );
}

#[test]
fn light_references_are_validated() {
    let unknown = "\"T\" = { terrain = \"floor\", light = \"sun\" }";
    let err = Content::load(&minimal_files("#@T#\n", unknown)).unwrap_err();
    assert!(
        err.message.contains("unknown light `sun`"),
        "{}",
        err.message
    );

    let mut files = minimal_files("#@#\n", "");
    files.insert(
        "data/lights.toml",
        "[[light]]\nid = \"fire\"\ncolor = [1, 1, 1, 0]\nradius = 3\n\
         stability.dips = { depth = 0.1, every = 1, length = 2 }\n",
    );
    let err = Content::load(&files).unwrap_err();
    assert!(
        err.message.contains("light `fire`: stability: dips"),
        "{}",
        err.message
    );

    let mut files = minimal_files("#@#\n", "");
    files.insert(
        "scenarios/test.toml",
        "name = \"T\"\nstart_level = \"test\"\nvision = \"x-ray\"\n",
    );
    let err = Content::load(&files).unwrap_err();
    assert!(
        err.message.contains("vision profile `x-ray`"),
        "{}",
        err.message
    );

    let mut files = minimal_files("#@#\n", "");
    files.insert(
        "scenarios/test.toml",
        "name = \"T\"\nstart_level = \"test\"\nwield = [\"torch\"]\n",
    );
    let err = Content::load(&files).unwrap_err();
    assert!(err.message.starts_with("wield: "), "{}", err.message);
}

#[test]
fn stairs_and_scenarios_may_lead_into_dungeon_branches() {
    let content = Content::builtin().unwrap();
    assert!(
        content.has_level("showcase", "depths:8") && !content.has_level("showcase", "depths:9")
    );
    let (branch, depth) = content.branch_level("depths:3").unwrap();
    assert_eq!((branch.id.as_str(), depth), ("depths", 3));
    assert_eq!(content.scenario("depths").unwrap().start_level, "depths:1");

    let with_target = |to: &str| {
        let mut files = ContentFiles::builtin();
        let hub = files
            .get("campaigns/showcase/levels/hub.toml")
            .unwrap()
            .to_string();
        files.insert(
            "campaigns/showcase/levels/hub.toml",
            &hub.replace("to = \"vast\"", &format!("to = \"{to}\"")),
        );
        Content::load(&files)
    };
    assert!(with_target("depths:2").is_ok());
    for bad in ["depths:9", "depths:0", "caves:1"] {
        let err = with_target(bad).unwrap_err();
        assert!(
            err.message.contains(&format!("unknown level `{bad}`")),
            "{}",
            err.message
        );
    }

    let mut files = ContentFiles::builtin();
    files.insert(
        "scenarios/deep.toml",
        "name = \"D\"\nstart_level = \"depths:9\"\n",
    );
    let err = Content::load(&files).unwrap_err();
    assert!(
        err.message.contains("unknown start level"),
        "{}",
        err.message
    );
}

#[test]
fn dungeons_need_the_terrain_generators_build_with() {
    let mut files = minimal_files("#@#\n", "");
    files.insert(
        "data/dungeons.toml",
        "[[dungeon]]\nid = \"d\"\ngenerator = \"rooms\"\ndepth = 1\nsize = [80, 21]\n",
    );
    let err = Content::load(&files).unwrap_err();
    assert_eq!(err.file, "data/dungeons.toml");
    assert!(
        err.message.contains("needs terrain `door_closed`"),
        "{}",
        err.message
    );
}

#[test]
fn vaults_are_validated() {
    let cases = [
        ("####\n#..#\n####\n", "needs at least one entrance"),
        ("##+##\n#.<.#\n#####\n", "cannot have stairs"),
        ("##+##\n#...#\n#.#.#\n##.##\n", "must be solid"),
        ("+###\n#..#\n####\n", "not on a corner"),
    ];
    for (map, expected) in cases {
        let mut files = ContentFiles::builtin();
        files.insert(
            "vaults/bad.toml",
            &format!("name = \"Bad\"\nmap = '''\n{map}'''\n"),
        );
        let err = Content::load(&files).unwrap_err();
        assert_eq!(err.file, "vaults/bad.toml");
        assert!(err.message.contains(expected), "{map}: {}", err.message);
    }
    let content = Content::builtin().unwrap();
    let ids: Vec<_> = content.vaults().iter().map(Vault::id).collect();
    assert_eq!(
        ids,
        [
            "archers_walk",
            "bell_and_barracks",
            "broken_bridge",
            "burning_hall",
            "burnt_steading",
            "catacomb",
            "champions_ring",
            "conventicle",
            "counting_house",
            "dark_chapel",
            "dead_mens_gear",
            "delving",
            "dormitory",
            "drowned_mill",
            "drowned_stair",
            "drunken_hall",
            "fairy_ring",
            "flooded_cell",
            "four_fonts",
            "gallows_yard",
            "gauntlet",
            "guard_post",
            "hermits_hut",
            "hoar_stone",
            "kennels",
            "lectern",
            "mead_hall",
            "old_battle",
            "old_enemies",
            "one_of_them",
            "ossuary",
            "pillared_hall",
            "rat_run",
            "reeves_strongroom",
            "reliquary",
            "ruined_chapel",
            "ruined_tower",
            "rune_stone",
            "sealed_barrow",
            "shrine",
            "smithy",
            "spiteful_coffer",
            "stone_circle",
            "the_mere",
            "tithe_barn",
            "trolls_bridge",
            "wardens_hall",
            "watch_fire",
            "watchmen",
            "well_house",
            "wights_niches",
            "wyrms_bed",
            // The campaigns' own, after the shared ones.
            "alberichs_delving",
            "attorcoppes_web",
            "barrow_chamber",
            "captains_hall",
            "crystal_garden",
            "crystal_hollow",
            "drakes_hoard",
            "foremans_cabin",
            "gnaegans_midden",
            "grundwyrgens_mere",
            "hrimwyrms_bed",
            "hunters_statue",
            "kobolds_cache",
            "ligetdrys_stones",
            "moss_bower",
            "nihtscinnas_warren",
            "nithdracas_hoard",
            "robbers_hoard",
            "staneotens_gallery",
            "still_room",
            "sunken_crypt",
            "unhallowed_chapel",
            "waelcyrges_field",
            "wealafs_hall"
        ]
    );
    let shrine = (content.vaults().iter()).find(|v| v.id() == "shrine");
    assert_eq!(shrine.expect("the shrine").entrances.len(), 3);
    let outdoor: Vec<_> = content.vaults().iter().filter(|v| v.is_outdoor()).collect();
    assert_eq!(outdoor.len(), 9);
}

#[test]
fn outdoor_vaults_are_open() {
    let load = |map: &str, kind: &str| {
        let mut files = ContentFiles::builtin();
        files.insert(
            "vaults/ruin.toml",
            &format!("name = \"Ruin\"\nkind = \"{kind}\"\nmap = '''\n{map}'''\n"),
        );
        Content::load(&files)
    };
    // Open ground (spaces) and walkable cells on the edge are ways in; no doors needed.
    let vault = load(" ### \n #.  \n ### \n", "outdoor").expect("an open ruin");
    let ruin = vault
        .vaults()
        .iter()
        .find(|v| v.id() == "ruin")
        .expect("ruin");
    assert!(ruin.is_outdoor());
    assert!(ruin.entrances.len() > 5);
    // A ruin with no open edge has no way in.
    let err = load("###\n###\n###\n", "outdoor").unwrap_err();
    assert!(
        err.message.contains("needs an open cell on its edge"),
        "{}",
        err.message
    );
    assert!(load("###\n#.#\n###\n", "overworld").is_err());
}

/// The built-in content plus a level `stream` with `map` and `legend`.
fn stream(map: &str, legend: &str) -> Result<Content, ContentError> {
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/stream.toml",
        &format!("name = \"Stream\"\nmap = '''\n{map}'''\n[legend]\n{legend}"),
    );
    Content::load(&files)
}

#[test]
fn legend_arrows_are_running_water_with_a_flow() {
    let content = stream("#####\n#→⇓↙#\n#####\n", "").unwrap();
    let level = content.level("stream").unwrap();
    let running = content.terrain_id("running_water").unwrap();
    assert!(content.terrain(running).flows());
    assert!(
        !content
            .terrain(content.terrain_id("water").unwrap())
            .flows()
    );
    assert_eq!(level.terrain[Pos::new(1, 1)], running);
    let flows: Vec<_> = level.flow.iter().map(|(p, f)| (*p, *f)).collect();
    assert_eq!(
        flows,
        [
            (Pos::new(1, 1), Flow::new(Dir::E, 100)),
            (Pos::new(2, 1), Flow::new(Dir::S, 200)),
            (Pos::new(3, 1), Flow::new(Dir::SW, 100)),
        ]
    );
    // A level's own legend may give any speed in range.
    let slow = "\"s\" = { terrain = \"running_water\", flow = \"N\", flow_speed = 40 }\n";
    let content = stream("###\n#s#\n###\n", slow).unwrap();
    let flow = content.level("stream").unwrap().flow[&Pos::new(1, 1)];
    assert_eq!(flow, Flow::new(Dir::N, 40));
}

#[test]
fn flow_is_required_on_running_water_and_only_there() {
    let cases = [
        ("\"s\" = \"running_water\"\n", "needs a flow direction"),
        (
            "\"s\" = { terrain = \"floor\", flow = \"S\" }\n",
            "is not running water",
        ),
        (
            "\"s\" = { terrain = \"floor\", flow_speed = 50 }\n",
            "`flow_speed` but no `flow`",
        ),
        (
            "\"s\" = { terrain = \"running_water\", flow = \"Down\" }\n",
            "flow = \"Down\"",
        ),
    ];
    for (legend, expected) in cases {
        let err = stream("###\n#s#\n###\n", legend).unwrap_err();
        assert_eq!(err.file, "levels/stream.toml");
        assert!(err.message.contains(expected), "{legend}: {}", err.message);
    }
}

#[test]
fn water_animations_are_checked() {
    let cases = [
        (
            "anim = { kind = \"wave\", glint = -1 }",
            "finite and not negative",
        ),
        (
            "anim = { kind = \"flow\", speed = -1 }",
            "finite and not negative",
        ),
    ];
    for (anim, expected) in cases {
        let mut files = minimal_files("#@#\n", "");
        let terrain = files.get("data/terrain.toml").unwrap().replace(
            "glyph = \".\"",
            &format!("glyph = \".\"\n            {anim}"),
        );
        files.insert("data/terrain.toml", &terrain);
        let err = Content::load(&files).unwrap_err();
        assert!(err.message.contains(expected), "{anim}: {}", err.message);
    }
    // Foam needs running water to foam in.
    let mut files = minimal_files("#@#\n", "");
    let terrain = files.get("data/terrain.toml").unwrap().to_string();
    let foam = "on = \"obstacle\"\nglyphs = \".\"\ncolors = [\"#ffffff\"]\nrate = 1\nlife = 1\n";
    files.insert(
        "data/terrain.toml",
        &format!("{terrain}\n[[terrain.particles]]\n{foam}"),
    );
    let err = Content::load(&files).unwrap_err();
    assert!(
        err.message.contains("need a `flow` animation"),
        "{}",
        err.message
    );
}

#[test]
fn every_object_has_a_glyph_color_material_and_weight() {
    let content = Content::builtin().unwrap();
    let objects = content.object_defs();
    // With a corpse for every creature type but the player, and the magic base items.
    // And a grimoire for every spell scroll.
    assert!((15..=800).contains(&objects.len()), "{}", objects.len());
    let glyphs = content.glyphs();
    for def in objects {
        assert!(def.weight > 0 && glyphs.contains(&def.glyph), "{}", def.id);
        assert!(!content.material(def.material()).name.is_empty());
        assert_eq!(
            content
                .object_id(&def.id)
                .map(|id| content.object(id).id.as_str()),
            Some(def.id.as_str())
        );
    }
    let classes: BTreeSet<char> = objects.iter().map(|o| o.glyph).collect();
    for class in [')', '[', '(', '!', '?', '%', '*', '$', '=', '"'] {
        assert!(classes.contains(&class), "an object drawn `{class}`");
    }
    let materials: BTreeSet<MaterialId> = objects.iter().map(|o| o.material()).collect();
    assert!(materials.len() >= 10, "objects of many materials");
    let rubies = object::parse_stacks(&["gem:ruby*2".into()], &content).expect("rubies");
    assert_eq!(rubies[0].describe(&content), "2 rubies");
    // Every creature has a weight; the hub places objects and its kobold carries some.
    assert!(content.creature_defs().iter().all(|c| c.weight > 0));
    let hub = content.level("hub").unwrap();
    assert!(hub.objects.len() >= 8);
    assert!(hub.creatures.iter().any(|c| !c.carry.is_empty()));
}

#[test]
fn bad_objects_are_reported() {
    const MATERIALS: &str = "[[material]]\nid = \"iron\"\nname = \"iron\"\n";
    let object = |fields: &str| {
        format!(
            "[[object]]\nid = \"key\"\nname = \"key\"\nglyph = \"(\"\nfg = \"#ffffff\"\nclass = \"tools\"\n{fields}"
        )
    };
    let cases = [
        (
            object("materials = [\"iron\"]\nweight = 5\n"),
            "missing field `value`",
        ),
        (
            object("materials = [\"bronze\"]\nweight = 5\nvalue = 1\n"),
            "unknown material `bronze`",
        ),
        (
            object("materials = [\"iron\"]\nweight = 5\nvalue = 1\n").replace("tools", "toys"),
            "unknown item class `toys`",
        ),
        (
            object("materials = [\"iron\"]\nweight = 5\nvalue = 1\n")
                .replace("class = \"tools\"\n", ""),
            "missing field `class`",
        ),
        (
            object("materials = [\"iron\"]\nweight = 5\nvalue = 1\nkind = \"ring\"\n"),
            "`kind = \"ring\"` is of class `rings`, not `tools`",
        ),
        (
            object("materials = [\"iron\"]\nvalue = 1\n"),
            "missing field `weight`",
        ),
        (
            object("weight = 5\nvalue = 1\n"),
            "`materials` lists its parts, at least one",
        ),
        (
            object("material = \"iron\"\nweight = 5\nvalue = 1\n"),
            "unknown field `material`",
        ),
        (
            object(
                "materials = [\"iron\", { part = \"grip\", of = \"iron\", share = 5 }]\n\
                 weight = 5\nvalue = 1\n",
            ),
            "unknown part `grip` (data/parts.toml)",
        ),
        (
            format!(
                "{0}\n{0}",
                object("materials = [\"iron\"]\nweight = 5\nvalue = 1\n")
            ),
            "duplicate object id `key`",
        ),
    ];
    for (objects, expected) in cases {
        let mut files = minimal_files("#@#\n", "");
        files.insert("data/materials.toml", MATERIALS);
        files.insert("data/objects.toml", &objects);
        let err = Content::load(&files).unwrap_err();
        assert_eq!(err.file, "data/objects.toml", "{objects}");
        assert!(err.message.contains(expected), "{objects}: {}", err.message);
    }
}

#[test]
fn bad_equipment_is_reported() {
    const MATERIALS: &str = "[[material]]\nid = \"iron\"\nname = \"iron\"\n";
    let slot = |id: &str, action: &str, hand: bool| {
        format!(
            "[[slot]]\nid = \"{id}\"\nname = \"{id}\"\naction = \"{action}\"\n\
             mark = \"worn\"\nhand = {hand}\n"
        )
    };
    let slots = [
        slot("main_hand", "wield", true),
        slot("off_hand", "wield", true),
        slot("head", "wear", false),
    ]
    .concat();
    let object = |fields: &str| {
        format!(
            "[[object]]\nid = \"key\"\nname = \"key\"\nglyph = \"(\"\nfg = \"#ffffff\"\nclass = \"tools\"\n\
             materials = [\"iron\"]\nweight = 5\nvalue = 1\n{fields}\n"
        )
    };
    let load = |slots: &str, objects: &str| {
        let mut files = minimal_files("#@#\n", "");
        files.insert("data/materials.toml", MATERIALS);
        files.insert("data/objects.toml", objects);
        if !slots.is_empty() {
            files.insert("data/slots.toml", slots);
        }
        Content::load(&files)
    };
    for (fields, expected) in [
        ("slot = \"tail\"", "`slot = \"tail\"` is no slot"),
        ("slot = \"head\"\ntwo_handed = true", "`two_handed` needs"),
        ("two_handed = true", "`two_handed` needs"),
        // Ranged combat: ammunition flies as far as its launcher shoots, only missile
        // weapons reload, and only what flies breaks or is lit.
        ("ammo = \"bolt\"\nrange = 5", "no `range`"),
        ("reloads = true", "only missile weapons"),
        ("breaks = 20", "`breaks` and `lit` are for things that fly"),
        (
            "ammo = \"bolt\"\nlit = \"key\"",
            "must name an object that burns",
        ),
        (
            "light = \"torch\"",
            "`light` goes with `burns` (a thing burning where it lies) or with a `slot` (a carried light)",
        ),
        (
            "burns = 30",
            "`light` goes with `burns` (a thing burning where it lies) or with a `slot` (a carried light)",
        ),
        ("light = \"sun\"\nburns = 30", "unknown light preset `sun`"),
        // Combat: weapons have damage, and only things worn or wielded fight or protect.
        ("damage = \"1d4\"", "a weapon (`damage`) must go in a slot"),
        (
            "slot = \"main_hand\"\nthreat = 20",
            "`threat` is for weapons",
        ),
        ("slot = \"main_hand\"\ncrit = 3", "`crit` is for weapons"),
        (
            "slot = \"main_hand\"\nstrength = 12",
            "`strength` is for weapons",
        ),
        (
            "slot = \"main_hand\"\ndamage = \"2d\"",
            "number of sides is missing",
        ),
        ("armour = 2", "are for things worn or wielded"),
        ("max_dex = 3", "are for things worn or wielded"),
        // Protection is for armour worn; penetration for weapons and ammunition.
        ("protection = 1", "`protection` is for armour worn"),
        (
            "slot = \"head\"\npenetration = 1",
            "`penetration` is for weapons",
        ),
    ] {
        let err = load(&slots, &object(fields)).unwrap_err();
        assert_eq!(err.file, "data/objects.toml", "{fields}");
        assert!(err.message.contains(expected), "{fields}: {}", err.message);
    }
    // The data author is trusted with the numbers: none of these is refused.
    for fields in [
        "slot = \"head\"\nprotection = 2",
        "slot = \"main_hand\"\ndamage = \"1d8\"\npenetration = 2",
        "ammo = \"bolt\"\npenetration = 2",
        "slot = \"main_hand\"\nprotection = 1",
        "slot = \"head\"\narmour = 21\nshield = -1",
        "slot = \"main_hand\"\ndamage = \"1d4\"\nthreat = 1\ncrit = 6\nto_hit = 21",
        "damage = \"1d3\"\nrange = 41",
        "slot = \"head\"\nlaunches = \"bolt\"",
        "ammo = \"bolt\"\nbreaks = 101",
    ] {
        assert!(load(&slots, &object(fields)).is_ok(), "{fields}");
    }
    let staff = object("slot = \"main_hand\"\ntwo_handed = true");
    assert!(load(&slots, &staff).is_ok());
    // Two hands are needed for a two-handed object; no slots file, no slots.
    let one_hand = slot("main_hand", "wield", true);
    assert!(load(&one_hand, &staff).is_err());
    let err = load("", &object("slot = \"head\"")).unwrap_err();
    assert!(err.message.contains("is no slot"), "{}", err.message);
    // A launcher pairs with the ammunition of its kind.
    let pair = format!(
        "{}{}",
        object("slot = \"head\"\nlaunches = \"bolt\"\ndamage = \"1d10+1\"\nrange = 12"),
        object("ammo = \"bolt\"").replace("\"key\"", "\"bolt\"")
    );
    let content = load(&slots, &pair).expect("a crossbow and a bolt");
    let (key, bolt) = (content.object_id("key"), content.object_id("bolt"));
    assert!(content.fires(key.unwrap(), bolt.unwrap()));
    assert!(!content.fires(bolt.unwrap(), key.unwrap()));
    // Slots enter the fingerprint: equipment is saved by slot index.
    let without = load("", &object("")).expect("loads");
    let with = load(&slots, &object("")).expect("loads");
    assert_ne!(
        without.fingerprint_for("showcase"),
        with.fingerprint_for("showcase")
    );
}

#[test]
fn creature_types_have_stats_that_default_to_average() {
    use crate::stats::{Stat, Stats};
    let content = Content::builtin().unwrap();
    let def = |id: &str| content.creature(content.creature_id(id).unwrap());
    assert_eq!(
        def("player").stats,
        Stats::default(),
        "the scenario gives the player's"
    );
    let rat = def("rat").stats;
    assert_eq!(rat.score(Stat::Dexterity), 17);
    assert_eq!(
        rat.potential(Stat::Dexterity),
        17,
        "the potential defaults to the score"
    );
    assert_eq!(rat.score(Stat::Intelligence), 1);
    let shaman = def("kobold_shaman").stats;
    assert!(shaman.score(Stat::Willpower) > def("kobold").stats.score(Stat::Willpower));
    // A creature type without stats is average in each.
    let content = Content::load(&minimal_files("#@#\n", "")).unwrap();
    let rat = content.creature(content.creature_id("rat").unwrap());
    assert_eq!(rat.stats, Stats::default());
}

#[test]
fn bad_creature_stats_are_reported() {
    let cases = [
        (
            "rat",
            "stats = { strength = 41 }",
            "creature `rat`: stats: strength 41",
        ),
        (
            "rat",
            "stats = { presence = 0 }",
            "presence 0 is out of range (1 to 40)",
        ),
        (
            "rat",
            "potential = { toughness = 50 }",
            "potential: toughness 50",
        ),
        (
            "rat",
            "stats = { dexterity = 16 }\npotential = { dexterity = 12 }",
            "dexterity 12 is below its score 16",
        ),
        ("rat", "stats = { luck = 3 }", "unknown stat `luck`"),
        (
            "player",
            "stats = { strength = 12 }",
            "the player's stats, hit points, and mana come from the scenario",
        ),
        (
            "player",
            "potential = { strength = 12 }",
            "come from the scenario",
        ),
        ("player", "base_hp = 3", "come from the scenario"),
        ("player", "mana = 3", "come from the scenario"),
        (
            "rat",
            "attack = { damage = \"1d1\", name = \"bite\", verb = \"bites\" }",
            "2 to 1000 sides",
        ),
        // Without data/sizes.toml there is only medium.
        (
            "rat",
            "size = \"small\"",
            "creature `rat`: unknown size `small`",
        ),
        // Without data/creature_rarities.toml every type is common.
        (
            "rat",
            "rarity = \"rare\"",
            "creature `rat`: unknown rarity `rare`",
        ),
    ];
    for (creature, fields, expected) in cases {
        let mut files = minimal_files("#@#\n", "");
        let marker = format!("id = \"{creature}\"\n");
        let creatures = files
            .get("data/creatures.toml")
            .unwrap()
            .replace(&marker, &format!("{marker}{fields}\n"));
        files.insert("data/creatures.toml", &creatures);
        let err = Content::load(&files).unwrap_err();
        assert_eq!(err.file, "data/creatures.toml", "{fields}");
        assert!(err.message.contains(expected), "{fields}: {}", err.message);
    }
    let mut files = minimal_files("#@#\n", "");
    files.insert(
        "scenarios/test.toml",
        "name = \"T\"\nstart_level = \"test\"\nstats = { willpower = 99 }\n",
    );
    let err = Content::load(&files).unwrap_err();
    assert_eq!(err.file, "scenarios/test.toml");
    assert!(err.message.contains("willpower 99"), "{}", err.message);
    let mut files = minimal_files("#@#\n", "");
    files.insert(
        "scenarios/test.toml",
        "name = \"T\"\nstart_level = \"test\"\nspells = [\"fireball\"]\n",
    );
    let err = Content::load(&files).unwrap_err();
    assert_eq!(err.file, "scenarios/test.toml");
    assert!(
        err.message.contains("spells: unknown spell `fireball`"),
        "{}",
        err.message
    );
}

#[test]
fn levels_place_objects_by_legend_and_by_pile() {
    let map = "#####\n#.d.#\n#####\n";
    let legend = "\"d\" = { terrain = \"floor\", objects = [\"dagger\", \"arrow*12\", \"dagger*2\"] }\n\
                  [[pile]]\nat = [3, 1]\nobjects = [\"gold_coin*5\", \"gold_coin*2\"]\n\
                  [[pile]]\nat = [2, 1]\nobjects = [\"gem:ruby\"]\n";
    let content = stream(map, legend).unwrap();
    let level = content.level("stream").unwrap();
    let id = |name: &str| content.object_id(name).unwrap();
    let stack = |name: &str, count| crate::object::Stack::new(id(name), count);
    let ruby = object::parse_stacks(&["gem:ruby".into()], &content).expect("a ruby");
    assert_eq!(
        level.objects[&Pos::new(2, 1)],
        [
            ruby[0].clone(),
            stack("dagger", 1),
            stack("arrow", 12),
            stack("dagger", 1),
            stack("dagger", 1)
        ],
        "the pile goes on top; daggers do not stack"
    );
    assert_eq!(level.objects[&Pos::new(3, 1)], [stack("gold_coin", 7)]);
}

#[test]
fn a_spawn_may_be_placed_with_statuses() {
    let map = "###\n#k#\n###\n";
    let kobold = "\"k\" = { terrain = \"floor\", creature = \"kobold\"";
    let content = stream(
        map,
        &format!("{kobold}, status = [\"blessed\", \"weakened*2\"] }}\n"),
    )
    .unwrap();
    let spawn = &content.level("stream").unwrap().creatures[0];
    let id = |name: &str| content.status_id(name).unwrap();
    assert_eq!(spawn.status, [(id("blessed"), 1), (id("weakened"), 2)]);
    for (status, why) in [
        ("\"blest\"", "unknown status `blest`"),
        ("\"blessed*4\"", "more than its 3 degrees"),
        ("\"blessed*0\"", "count must be"),
    ] {
        let legend = format!("{kobold}, status = [{status}] }}\n");
        let err = stream(map, &legend).unwrap_err();
        assert!(err.message.contains(why), "{status}: {}", err.message);
    }
}

#[test]
fn misplaced_objects_are_reported() {
    let cases = [
        (
            "\"x\" = { terrain = \"floor\", objects = [\"anvil\"] }",
            "unknown object `anvil`",
        ),
        (
            "\"x\" = { terrain = \"floor\", objects = [\"arrow*0\"] }",
            "count must be",
        ),
        (
            "\"x\" = { terrain = \"floor\", objects = [\"arrow*x\"] }",
            "count must be",
        ),
        (
            "\"x\" = { terrain = \"floor\", carry = [\"dagger\"] }",
            "carry, wield, wear, or status but no creature",
        ),
        (
            "\"x\" = { terrain = \"floor\", status = [\"blessed\"] }",
            "or status but no creature",
        ),
        (
            "\"x\" = \"floor\"\n[[pile]]\nat = [9, 9]\nobjects = [\"dagger\"]",
            "outside",
        ),
    ];
    for (legend, expected) in cases {
        let err = stream("###\n#x#\n###\n", &format!("{legend}\n")).unwrap_err();
        assert_eq!(err.file, "levels/stream.toml");
        assert!(err.message.contains(expected), "{legend}: {}", err.message);
    }
}

#[test]
fn the_hub_has_still_ponds_and_running_rills() {
    let content = Content::builtin().unwrap();
    let hub = content.level("hub").unwrap();
    let still = content.terrain_id("water").unwrap();
    let running = content.terrain_id("running_water").unwrap();
    assert!(hub.terrain.cells().iter().filter(|t| **t == still).count() > 50);
    let rill: Vec<_> = hub
        .terrain
        .iter()
        .filter(|(_, t)| **t == running)
        .map(|(p, _)| p)
        .collect();
    assert!(!rill.is_empty());
    assert!(rill.iter().all(|p| hub.flow.contains_key(p)));
    assert_eq!(hub.flow.len(), rill.len());
    assert!(hub.flow.values().any(|f| f.speed > 100), "a fast spring");
}
