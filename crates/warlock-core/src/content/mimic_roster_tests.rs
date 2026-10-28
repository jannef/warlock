//! The mimics (spec §8 "Mimics", `data/mimics.toml`; docs/design/chests-and-treasure.md §5.6):
//! one type a level, named by bands that match the chest bands, every level a brute on the
//! roster's formulas, and placed by nothing but a chest.

use super::Content;
use super::undead_roster_tests::{Role, dice_average, max_hp, within};
use crate::stats::Stat;

#[test]
fn one_mimic_a_level_on_the_brute_formulas() {
    let content = Content::builtin().expect("content");
    let mimics = content.mimics();
    assert_eq!(mimics.levels, [1, 60]);
    let vermin = content.factions().id("vermin");
    let guideline = content.experience().guideline();
    let first = content.creature_id("mimic_1").expect("mimic_1");
    for level in 1..=60u8 {
        let id = content.mimic_at(level).expect("a mimic");
        let def = content.creature(id);
        assert_eq!(def.id, format!("mimic_{level}"));
        assert_eq!(def.level, level);
        assert_eq!(content.family(id), first, "{}", def.id);
        assert_eq!(content.family_key(id), "mimic");
        assert_eq!(def.xp, guideline.xp(level), "{}: off the guideline", def.id);
        assert!(!def.stray && def.faction == vermin, "{}", def.id);
        assert!(def.tags.iter().any(|t| t == "mimic"));
        assert_eq!(def.budge, super::Budge::Never);
        let l = f64::from(level);
        let hp = f64::from(def.stats.score(Stat::Toughness)) + f64::from(def.base_hp);
        let want = max_hp(l, Role::Brute);
        assert!(
            within(hp, want),
            "{}: max HP {hp}, the formula {want:.1}",
            def.id
        );
        let attack = def.attack.as_ref().expect("a slam");
        let damage = attack.attack.damage;
        let average = f64::from(damage.min() + damage.max()) / 2.0;
        let want = dice_average(l, Role::Brute, false);
        assert!(within(average, want), "{}: dice {average}, {want}", def.id);
        let held = content.status_id("held").expect("held");
        assert!(
            (attack.on_hit.iter()).any(
                |h| matches!(h.what, super::Affliction::Hold { status, .. } if status == held)
            ),
            "{}: its slam holds",
            def.id
        );
    }
    // Beyond the family, the top; below it, the least.
    assert_eq!(content.mimic_at(99), content.creature_id("mimic_60"));
    assert_eq!(content.mimic_at(0), Some(first));
    assert_eq!(
        content.creature_id("mimic"),
        Some(first),
        "the family's name"
    );
}

#[test]
fn the_bands_are_the_chests_and_name_them() {
    let content = Content::builtin().expect("content");
    let names: Vec<&str> = content
        .mimics()
        .bands
        .iter()
        .map(|b| b.name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "mimicling",
            "young mimic",
            "mimic",
            "old mimic",
            "greater mimic",
            "elder mimic",
            "hoary mimic",
            "ancient mimic",
            "primeval mimic",
            "eldest mimic"
        ]
    );
    for (mimic, chest) in content.mimics().bands.iter().zip(&content.chests().bands) {
        assert_eq!(mimic.from, chest.from, "{}", mimic.id);
    }
    let name = |l| &content.creature(content.mimic_at(l).expect("a mimic")).name;
    assert_eq!(name(1), "mimicling");
    assert_eq!(name(7), "mimic");
    assert_eq!(name(60), "eldest mimic");
}

#[test]
fn no_population_names_a_mimic() {
    let content = Content::builtin().expect("content");
    for def in content.dungeons() {
        let population = def.spawning.iter().flat_map(|s| &s.population);
        for &c in population {
            assert!(!content.is_mimic(c), "{}: names a mimic", def.id);
        }
    }
}
