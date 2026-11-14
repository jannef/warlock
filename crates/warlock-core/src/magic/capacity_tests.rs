//! Affix capacity and the trim (docs/design/enchanting.md §3, §4; docs/dev/enchanting.md).

use super::*;
use crate::content::{Cutting, ItemHarm, TalentEffect};
use crate::item_parts::ItemPart;
use crate::magic::{Affix, LootGrade, LootPool, Quality, Rarity, Worked, loot};
use crate::talent::FortuneEffect;

fn content() -> Content {
    Content::builtin().expect("built-in content")
}

fn object(c: &Content, id: &str) -> crate::content::ObjectId {
    c.object_id(id)
        .unwrap_or_else(|| panic!("no object `{id}`"))
}

/// One of `id`, made of `main` if given, of `quality` if given, with a gem of `cut` in its
/// ornament part if given.
fn item(
    c: &Content,
    id: &str,
    main: Option<&str>,
    quality: Option<Quality>,
    gem: Option<(&str, &str)>,
) -> Stack {
    let o = object(c, id);
    let def = c.object(o);
    let mut own = Vec::new();
    if let Some(m) = main {
        own.push(ItemPart::material(
            c,
            None,
            c.material_id(m).expect("material"),
        ));
    }
    if let Some((g, cut)) = gem {
        let gems = c.gems().expect("gems");
        let cutting = Cutting {
            size: gems.plain().size,
            cut: gems.cut_id(cut).expect("cut"),
        };
        let part = crate::gem::ornament_part(c, def);
        own.push(ItemPart::gem(
            c,
            part,
            c.material_id(g).expect("gem"),
            cutting,
        ));
    }
    let mut stack = Stack::one(o);
    if !own.is_empty() {
        stack = stack.with_parts(own);
    }
    if let Some(q) = quality {
        stack.set_worked(q.worked(def).expect("arms or armour"));
    }
    stack
}

#[test]
fn the_examples_of_the_design_hold() {
    let c = content();
    let rows: [(&str, Stack, u8); 16] = [
        (
            "steel long sword",
            item(&c, "long_sword", None, None, None),
            2,
        ),
        (
            "Fine long sword",
            item(&c, "long_sword", None, Some(Quality::Fine), None),
            2,
        ),
        (
            "ruby-pommelled",
            item(&c, "long_sword", None, None, Some(("ruby", "fair"))),
            3,
        ),
        (
            "Marvellous bronze scimitar",
            item(
                &c,
                "scimitar",
                Some("bronze"),
                Some(Quality::Marvellous),
                None,
            ),
            3,
        ),
        (
            "Masterwork great sword, flawless diamond",
            item(
                &c,
                "great_sword",
                None,
                Some(Quality::Masterwork),
                Some(("diamond", "flawless")),
            ),
            6,
        ),
        (
            "Fine mithril long sword, sapphire",
            item(
                &c,
                "long_sword",
                Some("mithril"),
                Some(Quality::Fine),
                Some(("sapphire", "fine")),
            ),
            5,
        ),
        (
            "Masterwork adamantium plate",
            item(
                &c,
                "plate_armour",
                Some("adamantium"),
                Some(Quality::Masterwork),
                None,
            ),
            5,
        ),
        (
            "iron ring",
            item(&c, "gold_ring", Some("iron"), None, None),
            3,
        ),
        ("gold ring", item(&c, "gold_ring", None, None, None), 4),
        (
            "gold and emerald ring",
            item(&c, "gold_ring", None, None, Some(("emerald", "fair"))),
            5,
        ),
        (
            "tin ring",
            item(&c, "gold_ring", Some("tin"), None, None),
            2,
        ),
        ("silver ring", item(&c, "silver_ring", None, None, None), 1),
        (
            "silver ring, flawless gem",
            item(&c, "silver_ring", None, None, Some(("ruby", "flawless"))),
            3,
        ),
        (
            "mithril ring, flawless gem",
            item(
                &c,
                "gold_ring",
                Some("mithril"),
                None,
                Some(("ruby", "flawless")),
            ),
            6,
        ),
        (
            "Marvellous dragonscale scale mail",
            item(
                &c,
                "scale_mail",
                Some("dragonscale"),
                Some(Quality::Marvellous),
                None,
            ),
            5,
        ),
        ("arrows", Stack::new(object(&c, "arrow"), 20), 2),
    ];
    for (name, stack, want) in rows {
        assert_eq!(capacity(&stack, &c), want, "{name}");
    }
}

#[test]
fn a_damaged_masterwork_counts_as_plain() {
    let c = content();
    let mut sword = item(&c, "long_sword", None, Some(Quality::Masterwork), None);
    assert_eq!(capacity(&sword, &c), 4);
    let mut worked = sword.worked().cloned().expect("worked");
    worked.damage(ItemHarm::Rust);
    sword.set_worked(worked);
    assert_eq!(capacity(&sword, &c), 2);
    let mut repaired = sword.worked().cloned().expect("worked");
    repaired.repair();
    sword.set_worked(repaired);
    assert_eq!(capacity(&sword, &c), 2, "repair does not raise it again");
    let _ = Worked::default();
}

/// `p` prefixes then `s` suffixes that may roll on a long sword, of distinct categories.
fn affixes(c: &Content, p: usize, s: usize) -> Vec<Affix> {
    let def = c.object(object(c, "long_sword"));
    let kind = def.kind.expect("a weapon");
    let mut out = Vec::new();
    for (want, n) in [(AffixKind::Prefix, p), (AffixKind::Suffix, s)] {
        let mut taken: Vec<&str> = Vec::new();
        for a in c.affixes().all() {
            if out
                .iter()
                .filter(|x: &&Affix| c.affixes().get(&x.id).is_some_and(|d| d.kind == want))
                .count()
                == n
            {
                break;
            }
            if a.kind == want
                && super::super::may_roll(a, kind, def)
                && !taken.contains(&a.category.as_str())
            {
                taken.push(&a.category);
                out.push(Affix {
                    id: a.id.clone(),
                    tier: 1,
                    value: a.tiers[0].max,
                });
            }
        }
    }
    out
}

fn rare(affixes: Vec<Affix>) -> Magic {
    Magic {
        rarity: Rarity::Rare,
        level: 12,
        affixes,
        name: Some("Grim Bite".into()),
        identified: false,
        carried: 0,
        unique: None,
    }
}

fn kinds(c: &Content, m: &Magic) -> (usize, usize) {
    let p = (m.affixes.iter())
        .filter(|a| {
            c.affixes()
                .get(&a.id)
                .is_some_and(|d| d.kind == AffixKind::Prefix)
        })
        .count();
    (p, m.affixes.len() - p)
}

#[test]
fn the_trim_takes_from_the_kind_with_more_and_a_suffix_on_a_tie() {
    let c = content();
    // (prefixes, suffixes, capacity) → what is left.
    let cases = [
        ((2, 2), 1, (1, 0)),
        ((2, 2), 2, (1, 1)),
        ((2, 2), 3, (2, 1)),
        ((3, 3), 1, (1, 0)),
        ((3, 3), 2, (1, 1)),
        ((3, 3), 3, (2, 1)),
        ((3, 3), 4, (2, 2)),
        ((3, 3), 5, (3, 2)),
        ((3, 1), 1, (1, 0)),
        ((3, 1), 2, (1, 1)),
        ((3, 1), 3, (2, 1)),
        ((3, 1), 4, (3, 1)),
        ((1, 3), 2, (1, 1)),
        ((1, 3), 1, (1, 0)),
    ];
    for ((p, s), room, want) in cases {
        for seed in 0..8 {
            let mut m = rare(affixes(&c, p, s));
            assert_eq!(kinds(&c, &m), (p, s));
            trim(&mut m, room, &mut Rng::stream(seed, "trim"), &c);
            assert_eq!(kinds(&c, &m), want, "{p}+{s} at {room}");
            // A trimmed rare stays rare, with its name.
            assert_eq!(m.rarity, Rarity::Rare);
            assert_eq!(m.name.as_deref(), Some("Grim Bite"));
        }
    }
}

#[test]
fn the_trim_draws_only_when_it_trims() {
    let c = content();
    let mut rng = Rng::stream(3, "trim");
    let before = rng.clone().below(1_000_000);
    let mut m = rare(affixes(&c, 1, 1));
    trim(&mut m, 2, &mut rng, &c);
    assert_eq!(m.affixes.len(), 2);
    assert_eq!(
        rng.clone().below(1_000_000),
        before,
        "no draw for an item that fits"
    );
    trim(&mut m, 1, &mut rng, &c);
    assert_eq!(m.affixes.len(), 1);
    assert_ne!(rng.below(1_000_000), before, "one draw for the trim");
}

#[test]
fn binding_is_read_from_a_variant_main_part() {
    let c = content();
    let steel = item(&c, "long_sword", None, None, None);
    let mithril = item(&c, "long_sword", Some("mithril"), None, None);
    let silver = item(&c, "long_sword", Some("silver"), Some(Quality::Fine), None);
    assert_eq!(capacity(&steel, &c), 2);
    assert_eq!(capacity(&mithril, &c), 4);
    assert_eq!(capacity(&silver, &c), 1, "2 + 0 − 2, at least 1");
}

/// Every item made at a deep level, by every path, holds no more affixes than its capacity.
#[test]
fn every_loot_path_holds_its_items_to_their_capacity() {
    let c = content();
    let fits = |s: &Stack, path: &str| {
        if let Some(m) = s.magic() {
            let room = capacity(s, &c);
            assert!(
                m.affixes.len() <= usize::from(room),
                "{path}: {} holds {} at capacity {room}",
                c.object(s.object).id,
                m.affixes.len()
            );
            assert!(!m.affixes.is_empty());
        }
    };
    let mut rng = Rng::stream(5, "capacity");
    for level in [1, 6, 12, 20] {
        for _ in 0..400 {
            for grade in [LootGrade::Any, LootGrade::Magic, LootGrade::Rare] {
                if let Some(s) = loot::roll(&c, &mut rng, level, grade) {
                    fits(&s, "roll");
                }
            }
            let pool = LootPool::from(LootGrade::Rare);
            if let Some(s) = loot::roll_pool(&c, &mut rng, level, &pool, loot::Luck::default()) {
                fits(&s, "roll_pool");
            }
            if let Some(s) = loot::favoured(&c, &mut rng, level, &[(6, 1)]) {
                fits(&s, "favoured");
            }
            let sword = object(&c, "long_sword");
            fits(
                &loot::make(&c, &mut rng, sword, 1, level, LootGrade::Rare),
                "make",
            );
            let source = crate::variant::Source::gear(level);
            fits(
                &loot::make_as(&c, &mut rng, sword, 1, (level, LootGrade::Rare), source),
                "make_as",
            );
            fits(
                &loot::heirloom(&c, &mut rng, sword, 6, (level, None)),
                "heirloom",
            );
        }
    }
    // Creature gear: trimmed after its condition, so a damaged piece fits its plain capacity.
    for (i, def) in c.creature_defs().iter().enumerate() {
        if def.gear.lines.is_empty() {
            continue;
        }
        let id = crate::content::CreatureId(i as u16);
        for seed in 0..20 {
            let mut rng = Rng::gear(seed, "capacity");
            let mut pack = crate::gear::roll(&c, &mut rng, id, 12);
            let mut condition = crate::world::items::gear_condition::stream(seed, "capacity");
            crate::world::items::gear_condition::apply(&c, &mut condition, (id, 12), &mut pack);
            crate::gear::finish(&c, &mut rng, &mut pack);
            for s in &pack.items {
                fits(s, "gear");
            }
        }
    }
}

/// The affixes written for each shipped unique and each level's `[[item]]`, from the asset
/// files: (where, unique id or level stem, `at`, object, how many).
#[allow(clippy::type_complexity)]
fn written() -> Vec<(String, String, Option<[i64; 2]>, String, usize)> {
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("assets").flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "toml") {
                out.push(path);
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let mut files = Vec::new();
    walk(&root, &mut files);
    files.sort();
    let mut found = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path).expect("read");
        let Ok(value) = text.parse::<toml::Table>() else {
            continue;
        };
        let at_of = |t: &toml::Table| {
            let at = t.get("at")?.as_array()?;
            Some([at.first()?.as_integer()?, at.get(1)?.as_integer()?])
        };
        let count = |t: &toml::Table| {
            t.get("affixes")
                .and_then(|a| a.as_array())
                .map_or(0, Vec::len)
        };
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        for (key, uniques) in [("unique", true), ("item", false)] {
            let Some(rows) = value.get(key).and_then(|v| v.as_array()) else {
                continue;
            };
            for row in rows.iter().filter_map(|r| r.as_table()) {
                let n = count(row);
                if n == 0 {
                    continue;
                }
                let object = row
                    .get("object")
                    .and_then(|o| o.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = match uniques {
                    true => row
                        .get("id")
                        .and_then(|i| i.as_str())
                        .unwrap_or("")
                        .to_string(),
                    false => stem.clone(),
                };
                found.push((key.to_string(), name, at_of(row), object, n));
            }
        }
    }
    found
}

#[test]
fn hand_made_items_are_never_trimmed() {
    let c = content();
    let mut short = Vec::new();
    let written = written();
    assert!(
        written.iter().any(|w| w.0 == "unique"),
        "the uniques are found"
    );
    assert!(
        written.iter().any(|w| w.0 == "item"),
        "the levels' items are found"
    );
    for (kind, name, at, object, n) in written {
        let stack = match kind.as_str() {
            "unique" => c.unique(&name).map(|u| u.stack.clone()),
            _ => {
                let [x, y] = at.expect("an [[item]] has `at`");
                let pos = crate::geom::Pos {
                    x: x as i32,
                    y: y as i32,
                };
                (c.level(&name))
                    .and_then(|l| l.objects.get(&pos))
                    .and_then(|pile| {
                        (pile.iter())
                            .find(|s| c.object(s.object).id == object && s.magic().is_some())
                            .cloned()
                    })
            }
        };
        let Some(stack) = stack else {
            // A vault's `[[item]]` is placed when the vault is: not checked here.
            continue;
        };
        let room = usize::from(capacity(&stack, &c));
        let held = stack.magic().map_or(0, |m| m.affixes.len());
        if n > room || held < n {
            short.push(format!(
                "{kind} `{name}` ({object}): {n} affixes, capacity {room}"
            ));
        }
    }
    // The heirloom: every race's weapon at the top count, of the effect's make.
    for (_, def) in c.talents().iter() {
        for effect in &def.effects {
            let TalentEffect::Fortune(FortuneEffect::Heirloom {
                weapons,
                fallback,
                max,
                quality,
                ..
            }) = effect
            else {
                continue;
            };
            for weapon in weapons.values().chain([fallback]) {
                let o = object(&c, weapon);
                let mut stack = Stack::one(o);
                if let Some(worked) = quality.and_then(|q| q.worked(c.object(o))) {
                    stack.set_worked(worked);
                }
                let room = capacity(&stack, &c);
                if *max > room {
                    short.push(format!(
                        "heirloom `{weapon}`: {max} affixes, capacity {room}"
                    ));
                }
            }
        }
    }
    assert!(
        short.is_empty(),
        "hand-made items over their capacity: {short:#?}"
    );
}
