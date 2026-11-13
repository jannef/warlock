//! The crystals' table (`data/crystals.toml`, docs/design/enchanting.md §5): the built-in one
//! loads with the owner's rules, every creature and affix has the colour the rows say, and the
//! loader refuses what would leave an affix that could never be enchanted.

use crate::content::{Content, ContentFiles};

const PATH: &str = "data/crystals.toml";

fn builtin_text() -> String {
    let files = ContentFiles::builtin();
    files.get_optional(PATH).expect("crystals.toml").to_string()
}

/// The refusal of the built-in table with `from` replaced by `to`.
fn refused(from: &str, to: &str) -> String {
    let text = builtin_text();
    assert!(text.contains(from), "{from}");
    let mut files = ContentFiles::builtin();
    let changed = text.replacen(from, to, 1);
    files.insert(PATH, &changed);
    match Content::load(&files) {
        Ok(_) => panic!("loaded with `{from}` as `{to}`"),
        Err(e) => {
            assert_eq!(e.file, PATH);
            e.message
        }
    }
}

#[test]
fn the_six_colours_load_in_the_owners_order_with_his_two_rules() {
    let content = Content::builtin().expect("content");
    let crystals = content.crystals().expect("crystals");
    let ids: Vec<&str> = crystals.colours.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, ["black", "blue", "white", "red", "green", "gold"]);
    let has = |colour: &str, family: &str| {
        let c = crystals.colour(colour).expect(colour);
        c.families.iter().any(|f| f == family)
    };
    // Blue is never life; red is never mana.
    assert!(!has("blue", "life") && !has("blue", "life_regeneration"));
    for f in ["mana", "mana_regeneration", "mana_cost"] {
        assert!(!has("red", f), "{f}");
    }
    assert!(has("red", "life") && has("white", "life") && has("blue", "mana"));
    assert_eq!(crystals.consume_mana, 6);
    assert_eq!(crystals.consume_marks, 12);
    assert_eq!(
        [1, 9, 10, 19, 20, 40].map(|l| crystals.yield_at(l)),
        [1, 1, 2, 2, 3, 3]
    );
    for c in &crystals.colours {
        let def = content.object(c.object);
        assert!(
            def.stackable && def.value == 25 && def.weight == 20,
            "{}",
            c.id
        );
        assert_eq!(def.name, format!("{} crystal", c.name));
    }
}

#[test]
fn every_affix_has_a_colour_and_a_word_and_a_creature_its_first_tags() {
    let content = Content::builtin().expect("content");
    let crystals = content.crystals().expect("crystals");
    for def in content.affixes().all() {
        assert!(
            crystals.colour_of_affix(&content, &def.id).is_some(),
            "{} ({})",
            def.id,
            def.category
        );
        assert!(
            crystals.family_word(&def.category).is_some(),
            "{}",
            def.category
        );
    }
    let colour = |affix: &str| {
        crystals
            .colour_of_affix(&content, affix)
            .map(|c| c.id.as_str())
    };
    assert_eq!(colour("flaming"), Some("red"));
    assert_eq!(colour("withering"), Some("black"));
    let creature = |id: &str| {
        let c = content.creature_id(id).expect(id);
        crystals.colour_of(&content, c).map(|c| c.id.as_str())
    };
    // A kobold is green; a hobgoblin red (red's row comes before green's).
    assert_eq!(creature("kobold"), Some("green"));
    for def in content.creature_defs() {
        let Some(id) = content.creature_id(&def.id) else {
            continue;
        };
        let got = crystals.colour_of(&content, id).map(|c| c.id.clone());
        let want = (crystals.colours.iter())
            .find(|c| c.tags.iter().any(|t| def.tags.contains(t)))
            .map(|c| c.id.clone());
        assert_eq!(got, want, "{}", def.id);
    }
}

/// Every family a colour names, and every `[[family]]`, is some affix's category.
#[test]
fn every_family_is_an_affix_category() {
    let content = Content::builtin().expect("content");
    let crystals = content.crystals().expect("crystals");
    let is = |f: &str| content.affixes().all().iter().any(|a| a.category == f);
    for c in &crystals.colours {
        for f in &c.families {
            assert!(is(f), "{}: {f}", c.id);
        }
    }
    for (f, _) in &crystals.families {
        assert!(is(f), "[[family]] {f}");
    }
}

/// Every tag a colour names is carried by some creature of the built-in content (not a loader
/// check: a content set without a campaign, as some tests load, may lack a tag's creatures).
#[test]
fn every_colours_tag_is_carried() {
    let content = Content::builtin().expect("content");
    for c in &content.crystals().expect("crystals").colours {
        for tag in &c.tags {
            assert!(content.tagged(tag).next().is_some(), "{}: {tag}", c.id);
        }
    }
}

#[test]
fn the_loader_refuses_a_category_in_no_colour_an_unknown_tag_and_a_loose_object() {
    // `acid_damage` is green's alone.
    let lost = refused(
        "\"poison_resistance\", \"acid_damage\", ",
        "\"poison_resistance\", ",
    );
    assert!(
        lost.contains("acid_damage") && lost.contains("no colour"),
        "{lost}"
    );
    let tag = refused(
        "tags = [\"fey\", \"summoner\"]",
        "tags = [\"fey\", \"no:tag\"]",
    );
    assert!(tag.contains("no:tag"), "{tag}");
    let object = refused("object = \"crystal_gold\"", "object = \"long_sword\"");
    assert!(object.contains("must stack"), "{object}");
    let unknown = refused("object = \"crystal_gold\"", "object = \"no_such_thing\"");
    assert!(unknown.contains("no_such_thing"), "{unknown}");
    let word = refused(
        "[[family]]\ncategory = \"willpower\"\nword = \"willpower\"\n",
        "",
    );
    assert!(
        word.contains("willpower") && word.contains("word"),
        "{word}"
    );
    let twice = refused("id = \"gold\"", "id = \"black\"");
    assert!(twice.contains("twice"), "{twice}");
    let chance = refused("salvage_chance = 3", "salvage_chance = 101");
    assert!(chance.contains("0 to 100"), "{chance}");
    let heart = refused("heart = \"heart_crystal\"", "heart = \"soul_fetter\"");
    assert!(heart.contains("two of the `materials`"), "{heart}");
    let fetter = refused("fetter = \"soul_fetter\"", "fetter = \"crystal_red\"");
    assert!(fetter.contains("two of the `materials`"), "{fetter}");
    let marks = refused("marks = 12", "marks = 0");
    assert!(marks.contains("`marks` must be at least 1"), "{marks}");
}
