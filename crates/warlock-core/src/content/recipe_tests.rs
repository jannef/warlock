//! Recipes as data (`data/recipes/<skill>/<id>.toml`, docs/design/professions.md §6; #1334): the
//! built-in ones load, a generic one of every field loads, and every check at load refuses.

use super::{Content, ContentFiles, InputKind, PartSource, RecipeCount, RecipeUse};

fn load_with(files: &[(&str, &str)]) -> Result<Content, super::ContentError> {
    let mut all = ContentFiles::builtin();
    for (path, text) in files {
        all.insert(path, text);
    }
    Content::load(&all)
}

/// The refusal of a recipe file at `path` holding `text`.
fn refused(path: &str, text: &str) -> String {
    match load_with(&[(path, text)]) {
        Ok(_) => panic!("{path} loaded:\n{text}"),
        Err(e) => {
            assert_eq!(e.file, path);
            e.message
        }
    }
}

const SWORD: &str = "data/recipes/metalcraft/long_sword.toml";

/// A long sword at an anvil: the blade of stock, an ornament.
fn sword(extra: &str) -> String {
    format!(
        "[[recipe]]\nid = \"long_sword\"\nresult = {{ object = \"long_sword\" }}\n\
         difficulty = 13\nseconds = 3600\nstation = \"anvil\"\nornament = true\n{extra}"
    )
}

#[test]
fn the_built_in_recipes_are_the_forges_swords_and_temper_and_woodcrafts_missiles_and_whittle() {
    let content = Content::builtin().expect("content");
    let recipes = content.recipes();
    let metal = content.skill_id("metalcraft").expect("metalcraft");
    let wood = content.skill_id("woodcraft").expect("woodcraft");
    assert_eq!(recipes.professions(), [metal, wood]);
    let of = |skill| -> Vec<(&str, Option<RecipeUse>)> {
        (recipes.of(skill))
            .map(|(_, r)| (r.name.as_str(), r.uses))
            .collect()
    };
    assert_eq!(
        of(metal),
        [
            ("Arrowheads", None),
            ("Dagger", None),
            ("Great sword", None),
            ("Knife", None),
            ("Long sword", None),
            ("Rapier", None),
            ("Scimitar", None),
            ("Short sword", None),
            ("Temper", Some(RecipeUse::Temper))
        ]
    );
    // The forge's seven swords (docs/design/recipes.md §6): difficulty, seconds, from, and an
    // ornament on all but the knife, at the anvil, of metal stock.
    let anvil = content.terrain_id("anvil");
    for (id, difficulty, seconds, from, ornament) in [
        ("knife", 10, 1200, 0, false),
        ("dagger", 11, 1800, 0, true),
        ("short_sword", 12, 3600, 10, true),
        ("scimitar", 13, 3600, 20, true),
        ("long_sword", 13, 3600, 20, true),
        ("rapier", 14, 3600, 30, true),
        ("great_sword", 15, 5400, 40, true),
    ] {
        let r = content.recipe(recipes.id(id).expect(id));
        let got = (r.difficulty, r.seconds, r.from, r.ornament, r.station);
        assert_eq!(got, (difficulty, seconds, from, ornament, anvil), "{id}");
        assert_eq!(r.parts.len(), 1, "{id}");
        assert!(!r.anyone, "{id}");
    }
    assert_eq!(
        of(wood),
        [
            ("Arrows", None),
            ("Crossbow bolts", None),
            ("Flaming arrows", None),
            ("Sling stones", None),
            ("Whittle", Some(RecipeUse::Whittle))
        ]
    );
    // The built-ins are open to everyone, as they were before (#1337 Q3).
    let built_in = recipes.list.iter().filter(|r| r.uses.is_some());
    assert!(built_in.clone().all(|r| r.anyone && r.result.is_none()));
    assert_eq!(built_in.count(), 2);
    let temper = content.recipe(recipes.id("temper").expect("temper"));
    assert_eq!(temper.station, content.terrain_id("anvil"));
    assert_eq!(recipes.description(metal), Some("Your recipes as a smith."));
}

#[test]
fn a_generic_recipe_loads_with_its_parts_in_the_results_order() {
    let arrows = "[[recipe]]\nid = \"tipped\"\nresult = { object = \"arrow\", count = \"2d6\" }\n\
                  difficulty = 12\nseconds = 600\nknife = true\nlauncher = true\nfrom = 25\n\
                  inputs = [{ object = \"arrow\", count = 2, per_made = true }]\n";
    let content = load_with(&[
        (SWORD, &sword("[recipe.parts]\nblade = \"stock\"\n")),
        ("data/recipes/woodcraft/tipped.toml", arrows),
    ])
    .expect("loads");
    let recipes = content.recipes();
    let sword = content.recipe(recipes.id("long_sword").expect("a recipe"));
    assert_eq!(sword.name, "Long sword");
    assert_eq!((sword.difficulty, sword.seconds), (13, 3600));
    assert!(sword.ornament && sword.uses.is_none());
    assert_eq!(sword.parts.len(), 1);
    assert_eq!(sword.parts[0].source, PartSource::Stock);
    let tipped = content.recipe(recipes.id("tipped").expect("a recipe"));
    assert_eq!(tipped.from, 25);
    assert!(tipped.knife && tipped.launcher);
    assert_eq!(tipped.inputs[0].count, 2);
    assert!(tipped.inputs[0].per_made);
}

#[test]
fn every_check_at_load_refuses() {
    let cases: &[(&str, String, &str)] = &[
        (
            "data/recipes/long_sword.toml",
            sword(""),
            "goes in its skill's directory",
        ),
        (
            "data/recipes/flying/long_sword.toml",
            sword(""),
            "no skill `flying`",
        ),
        (
            SWORD,
            sword("").replace("\"long_sword\"\nresult", "\"sword\"\nresult"),
            "its id is its file name",
        ),
        (
            SWORD,
            format!("{}{}", sword(""), sword("")),
            "one `[[recipe]]` a file",
        ),
        (
            SWORD,
            sword("").replace("object = \"long_sword\"", "object = \"moonblade\""),
            "unknown result object",
        ),
        (
            SWORD,
            sword("").replace(
                "object = \"long_sword\"",
                "object = \"long_sword\", count = 3",
            ),
            "does not stack",
        ),
        (
            SWORD,
            sword("").replace("seconds = 3600", "seconds = 0"),
            "seconds",
        ),
        (
            SWORD,
            sword("").replace("difficulty = 13\n", ""),
            "difficulty",
        ),
        (
            SWORD,
            sword("").replace("result = { object = \"long_sword\" }\n", ""),
            "needs a `result` or a `use`",
        ),
        (
            SWORD,
            sword("").replace("\"anvil\"", "\"forge_of_ages\""),
            "unknown station",
        ),
        (
            SWORD,
            sword("[recipe.parts]\nhilt = \"stock\"\n"),
            "no part `hilt`",
        ),
        (
            SWORD,
            sword("[recipe.parts]\ngrip = \"stock\"\n"),
            "profession's stock",
        ),
        (
            SWORD,
            sword("[recipe.parts]\nblade = \"ore\"\n"),
            "is no source",
        ),
        (
            SWORD,
            sword("[recipe.parts]\nblade = { input = \"arrow\" }\n"),
            "none of the inputs",
        ),
        (
            SWORD,
            sword("inputs = [{ object = \"moonstone\", count = 1 }]\n"),
            "unknown input object",
        ),
        (
            SWORD,
            sword("inputs = [{ object = \"arrow\", count = 0 }]\n"),
            "count 0",
        ),
        (
            "data/recipes/woodcraft/arrows.toml",
            "[[recipe]]\nid = \"arrows\"\nresult = { object = \"arrow\" }\ndifficulty = 1\n\
             seconds = 1\nornament = true\n"
                .into(),
            "no ornament part",
        ),
        (
            "data/recipes/metalcraft/hone.toml",
            "[[recipe]]\nid = \"hone\"\nuse = \"hone\"\n".into(),
            "unknown `use` `hone`",
        ),
        (
            "data/recipes/metalcraft/hone.toml",
            "[[recipe]]\nid = \"hone\"\nuse = \"temper\"\nseconds = 60\n".into(),
            "a `use` recipe runs its action",
        ),
        (
            "data/recipes/woodcraft/temper.toml",
            "[[recipe]]\nid = \"temper\"\nuse = \"temper\"\n".into(),
            "duplicate recipe id `temper`",
        ),
    ];
    for (path, text, expected) in cases {
        let message = refused(path, text);
        assert!(
            message.contains(expected),
            "{path}: `{message}` lacks `{expected}`"
        );
    }
}

const ARROWS: &str = "data/recipes/woodcraft/arrows.toml";

/// Arrows by Fletch's count from arrow "heads" (an iron object stands in for arrowheads), bone
/// or stone, or none (brittle), and `extra`.
fn arrows(extra: &str) -> String {
    format!(
        "[[recipe]]\nid = \"arrows\"\nresult = {{ object = \"arrow\", count = \"fletch\" }}\n\
         difficulty = 12\nseconds = 600\nbrittle = \"brittle_arrow\"\n\
         inputs = [{{ object = \"arrow\", count = 1, per_made = true }}]\n{extra}\
         [recipe.parts]\nhead = {{ input = \"arrow\", stock = [\"bone\", \"stone\"], \
         optional = true }}\n"
    )
}

#[test]
fn the_formats_second_half_loads_part_sources_gems_stock_inputs_and_failures() {
    let spear = "[[recipe]]\nid = \"spear\"\nresult = { object = \"spear\" }\ndifficulty = 12\n\
                 seconds = 1800\nstation = \"anvil\"\n\
                 inputs = [{ material = \"sinew\", count = 1 }, { group = \"metal\", count = 2 }]\n\
                 [recipe.parts]\nhead = { stock = [\"iron\", \"bone\"] }\n\
                 haft = { stock = [\"wood\"] }\n";
    let amulet = "[[recipe]]\nid = \"gold_amulet\"\nresult = { object = \"gold_amulet\" }\n\
                  difficulty = 12\nseconds = 600\nfailure = { object = \"arrow\" }\n\
                  fumble = { object = \"brittle_arrow\" }\n\
                  [recipe.parts]\nsetting = { gem = true }\n";
    let content = load_with(&[
        (ARROWS, &arrows("")),
        ("data/recipes/metalcraft/spear.toml", spear),
        ("data/recipes/jewelcraft/gold_amulet.toml", amulet),
        (
            "data/recipes/jewelcraft/cut.toml",
            "[[recipe]]\nid = \"cut\"\nuse = \"cut\"\n",
        ),
        (
            "data/recipes/jewelcraft/set.toml",
            "[[recipe]]\nid = \"set\"\nuse = \"set\"\nfrom = 10\n",
        ),
    ])
    .expect("loads");
    let recipes = content.recipes();
    let recipe = |id: &str| content.recipe(recipes.id(id).expect("a recipe"));
    let object = |id: &str| content.object_id(id).expect("an object");
    let material = |id: &str| content.material_id(id).expect("a material");

    let arrows = recipe("arrows");
    assert_eq!(arrows.result.expect("made").count, RecipeCount::Fletch);
    assert_eq!(arrows.brittle, Some(object("brittle_arrow")));
    let PartSource::Sources {
        input,
        stock,
        optional,
    } = &arrows.parts[0].source
    else {
        panic!("sources")
    };
    assert_eq!((*input, *optional), (Some(object("arrow")), true));
    // The groups bone and stone (a name both a group and a material is the group), as far as
    // the head's `made_of` holds them: flint, never iron.
    let arrow = content.object(object("arrow"));
    let of_groups: Vec<_> = (arrow.made_of(0).iter().copied())
        .filter(|&m| matches!(content.material(m).group.as_deref(), Some("bone" | "stone")))
        .collect();
    assert_eq!(stock, &of_groups);
    assert!(stock.contains(&material("flint")) && !stock.contains(&material("iron")));

    // Another profession's stock (the haft's wood on a Metalcraft recipe); a list narrows the
    // part's `made_of` to what it names, in `made_of` order: iron alone of the metals.
    let spear = recipe("spear");
    let PartSource::Sources { input, stock, .. } = &spear.parts[0].source else {
        panic!("sources")
    };
    assert_eq!(*input, None);
    assert!(stock.contains(&material("iron")) && stock.contains(&material("bone")));
    assert!(!stock.contains(&material("steel")));
    let PartSource::Sources { stock, .. } = &spear.parts[1].source else {
        panic!("sources")
    };
    assert_eq!(stock, &[material("wood")]);
    assert_eq!(spear.inputs[0].kind, InputKind::Material(material("sinew")));
    assert_eq!(spear.inputs[1].kind, InputKind::Group("metal".into()));
    assert_eq!(spear.inputs[1].object(), None);

    let amulet = recipe("gold_amulet");
    assert_eq!(amulet.parts[0].source, PartSource::Gem);
    assert_eq!(amulet.failure, Some(object("arrow")));
    assert_eq!(amulet.fumble, Some(object("brittle_arrow")));

    // Jewelcraft is a profession once it has a directory; Cut and Set load as built-ins.
    let jewel = content.skill_id("jewelcraft").expect("jewelcraft");
    assert!(recipes.professions().contains(&jewel));
    assert_eq!(recipe("cut").uses, Some(RecipeUse::Cut));
    assert_eq!(recipe("set").uses, Some(RecipeUse::Set));
    assert!(!RecipeUse::Cut.usable() && RecipeUse::Temper.usable());
}

#[test]
fn a_recipe_of_the_old_shape_loads_as_before() {
    let content =
        load_with(&[(SWORD, &sword("[recipe.parts]\nblade = \"stock\"\n"))]).expect("loads");
    let sword = content.recipe(content.recipes().id("long_sword").expect("a recipe"));
    let result = sword.result.expect("made");
    assert_eq!(result.count, RecipeCount::Dice(crate::dice::Dice::fixed(1)));
    assert_eq!(
        (sword.failure, sword.fumble, sword.brittle),
        (None, None, None)
    );
    // `{ input = ... }` alone is a table of one source.
    let tipped = "[[recipe]]\nid = \"tipped\"\nresult = { object = \"arrow\", count = 10 }\n\
                  difficulty = 12\nseconds = 600\n\
                  inputs = [{ object = \"arrow\", count = 1 }]\n\
                  [recipe.parts]\nhead = { input = \"arrow\" }\n";
    let content = load_with(&[("data/recipes/woodcraft/tipped.toml", tipped)]).expect("loads");
    let tipped = content.recipe(content.recipes().id("tipped").expect("a recipe"));
    assert_eq!(
        tipped.parts[0].source,
        PartSource::Sources {
            input: content.object_id("arrow"),
            stock: Vec::new(),
            optional: false,
        }
    );
    assert_eq!(tipped.inputs[0].object(), content.object_id("arrow"));
}

#[test]
fn every_check_of_the_formats_second_half_refuses() {
    let amulet = |extra: &str| {
        format!(
            "[[recipe]]\nid = \"gold_amulet\"\nresult = {{ object = \"gold_amulet\" }}\n\
             difficulty = 12\nseconds = 600\n{extra}"
        )
    };
    const AMULET: &str = "data/recipes/jewelcraft/gold_amulet.toml";
    let cases: &[(&str, String, &str)] = &[
        (
            ARROWS,
            arrows("").replace("\"bone\", \"stone\"", "\"bone\", \"moonrock\""),
            "unknown stock group or material `moonrock`",
        ),
        (
            ARROWS,
            arrows("").replace("\"bone\", \"stone\"", "\"bone\", \"leather\""),
            "stock `leather` is not what it may be made of",
        ),
        (
            ARROWS,
            arrows("").replace("\"bone\", \"stone\"", "\"wood\""),
            "none of the group is what it may be made of",
        ),
        (
            ARROWS,
            arrows("").replace("stock = [\"bone\", \"stone\"], ", "stock = [], "),
            "an empty `stock` list",
        ),
        (
            ARROWS,
            arrows("").replace("brittle = \"brittle_arrow\"\n", ""),
            "needs the recipe's `brittle`",
        ),
        (
            ARROWS,
            arrows("").replace("\"brittle_arrow\"", "\"splinter\""),
            "unknown brittle object `splinter`",
        ),
        (
            ARROWS,
            arrows("").replace(", optional = true", ""),
            "`brittle` is only for a recipe with an `optional` part",
        ),
        (
            ARROWS,
            arrows("")
                .replace("input = \"arrow\", ", "input = \"flaming_arrow\", ")
                .replace(
                    "object = \"arrow\", count = 1",
                    "object = \"flaming_arrow\", count = 1",
                ),
            "its material may not make the part",
        ),
        (
            ARROWS,
            arrows("").replace("input = \"arrow\", stock = [\"bone\", \"stone\"], ", ""),
            "names no source",
        ),
        (
            ARROWS,
            arrows("").replace("optional = true", "gem = true"),
            "a `gem` part has no other source",
        ),
        (
            ARROWS,
            arrows("").replace("count = \"fletch\"", "count = \"twenty\""),
            "neither dice nor",
        ),
        (
            SWORD,
            sword("").replace(
                "object = \"long_sword\"",
                "object = \"long_sword\", count = \"fletch\"",
            ),
            "does not stack",
        ),
        (
            SWORD,
            sword("[recipe.parts]\ngrip = { gem = true }\n"),
            "may be made of no gem",
        ),
        (
            AMULET,
            amulet("inputs = [{ material = \"moonrock\", count = 1 }]\n"),
            "unknown input material `moonrock`",
        ),
        (
            AMULET,
            amulet("inputs = [{ group = \"moonrock\", count = 1 }]\n"),
            "unknown input group `moonrock`",
        ),
        (
            AMULET,
            amulet("inputs = [{ group = \"ruby\", count = 1 }]\n"),
            "a gem is no group",
        ),
        (
            AMULET,
            amulet("inputs = [{ group = \"metal\", material = \"iron\", count = 1 }]\n"),
            "one of `object`, `material`, and `group`",
        ),
        (
            AMULET,
            amulet("inputs = [{ material = \"sinew\", count = 0 }]\n"),
            "input `sinew`: count 0",
        ),
        (
            AMULET,
            amulet("failure = { object = \"moonstone\" }\n"),
            "unknown failure object `moonstone`",
        ),
        (
            AMULET,
            amulet("failure = { object = \"long_sword\" }\n"),
            "failure object `long_sword` does not stack",
        ),
        (
            AMULET,
            amulet("fumble = { object = \"moonstone\" }\n"),
            "unknown fumble object `moonstone`",
        ),
        (
            AMULET,
            amulet("fumble = { object = \"long_sword\" }\n"),
            "fumble object `long_sword` does not stack",
        ),
        (
            "data/recipes/jewelcraft/polish.toml",
            "[[recipe]]\nid = \"polish\"\nuse = \"polish\"\n".into(),
            "(temper, whittle, cut, set)",
        ),
        (
            "data/recipes/jewelcraft/cut.toml",
            "[[recipe]]\nid = \"cut\"\nuse = \"cut\"\nfailure = { object = \"arrow\" }\n".into(),
            "a `use` recipe runs its action",
        ),
    ];
    for (path, text, expected) in cases {
        let message = refused(path, text);
        assert!(
            message.contains(expected),
            "{path}: `{message}` lacks `{expected}`"
        );
    }
}

/// The settings file as shipped, with `from` replaced by `to`.
fn settings(from: &str, to: &str) -> String {
    let text = include_str!("../../../../assets/data/recipes.toml");
    assert!(text.contains(from), "{from}");
    text.replace(from, to)
}

#[test]
fn the_settings_file_holds_the_bands() {
    let content = Content::builtin().expect("content");
    let bands = content.recipes().settings.as_ref().expect("settings");
    assert_eq!(
        (
            bands.fine,
            bands.marvellous,
            bands.masterwork,
            bands.cut_step
        ),
        (6, 12, 20, 5)
    );
    assert_eq!(
        (bands.half_within, bands.one_within, bands.yield_percent),
        (3, 6, 50)
    );
    assert_eq!(
        bands.free_wood,
        [
            super::Landscape::Forest,
            super::Landscape::DenseForest,
            super::Landscape::Swamp
        ]
    );
}

#[test]
fn the_settings_file_refuses_an_unknown_skill_a_profession_twice_or_bands_out_of_order() {
    let message = |text: &str| match load_with(&[("data/recipes.toml", text)]) {
        Ok(_) => panic!("loaded:\n{text}"),
        Err(e) => e.message,
    };
    let metal = "skill = \"metalcraft\"";
    assert!(message(&settings(metal, "skill = \"flying\"")).contains("unknown skill `flying`"));
    let twice = "[[profession]]\nskill = \"woodcraft\"";
    assert!(
        message(&settings(twice, &format!("[[profession]]\n{metal}"))).contains("listed twice")
    );
    assert!(message(&settings("fine = 6", "fine = 6\nbands = 3")).contains("unknown field"));
    let cases = [
        ("fine = 6", "fine = 0", "`fine` (0) must be above 0"),
        ("marvellous = 12", "marvellous = 6", "above `fine`"),
        ("masterwork = 20", "masterwork = 11", "above `marvellous`"),
        ("cut_step = 5", "cut_step = 0", "`cut_step` (0)"),
        ("half_within = 3", "half_within = -2", "`half_within` (-2)"),
        ("one_within = 6", "one_within = 3", "above `half_within`"),
        ("yield = 50", "yield = 101", "above 100"),
        ("\"swamp\"]", "\"swamp\", \"forest\"]", "twice"),
        ("\"swamp\"]", "\"bog\"]", "unknown variant"),
        ("cut_step = 5\n", "", "missing field `cut_step`"),
    ];
    for (from, to, expected) in cases {
        let got = message(&settings(from, to));
        assert!(got.contains(expected), "{to}: `{got}` lacks `{expected}`");
    }
}

/// The shipped missiles (docs/design/recipes.md §6 "Missiles"; #1336): the arrows' head from
/// arrowheads, then bone and stone, or none (the brittle arrow); counted; and the head's sources
/// checked against the arrow's `made_of`.
#[test]
fn the_shipped_arrows_take_a_head_of_three_sources_and_the_loader_checks_them() {
    let content = Content::builtin().expect("content");
    let recipe = |id: &str| content.recipe(content.recipes().id(id).expect(id));
    let arrows = recipe("arrows");
    assert_eq!(arrows.result.expect("made").count, RecipeCount::Fletch);
    assert_eq!(arrows.brittle, content.object_id("brittle_arrow"));
    assert!(arrows.knife && arrows.launcher && arrows.anyone);
    let PartSource::Sources {
        input,
        stock,
        optional,
    } = &arrows.parts[0].source
    else {
        panic!("the head's sources");
    };
    assert_eq!(*input, content.object_id("arrowhead"));
    assert!(optional);
    let names: Vec<&str> = (stock.iter())
        .map(|&m| content.material(m).id.as_str())
        .collect();
    // Plain stone strikes as nothing: ammunition's head keeps only what can be a weapon's.
    for m in [
        "bone",
        "horn",
        "chitin",
        "ivory",
        "dragonbone",
        "flint",
        "obsidian",
    ] {
        assert!(names.contains(&m), "{m}: {names:?}");
    }
    assert!(!names.contains(&"iron"), "metal only as arrowheads");
    assert_eq!(arrows.parts[1].source, PartSource::Stock);
    assert_eq!(recipe("bolts").brittle, content.object_id("brittle_bolt"));
    let heads = recipe("arrowheads").result.expect("made");
    assert_eq!(heads.count, RecipeCount::Dice(crate::dice::Dice::fixed(20)));
    let flaming = recipe("flaming_arrows");
    assert!(flaming.inputs[0].per_made && !flaming.inputs[1].per_made);
    // A head of a group or a material the arrow may not be made of is refused.
    let text = include_str!("../../../../assets/data/recipes/woodcraft/arrows.toml");
    let head = "stock = [\"bone\", \"stone\"]";
    for (to, says) in [
        ("stock = [\"bone\", \"cloth\"]", "none of the group"),
        (
            "stock = [\"bone\", \"silk\"]",
            "is not what it may be made of",
        ),
    ] {
        let bad = text.replace(head, to);
        assert_ne!(bad, text, "{to}");
        let mut files = ContentFiles::builtin();
        files.insert("data/recipes/woodcraft/arrows.toml", &bad);
        let error = Content::load(&files).expect_err(to).to_string();
        assert!(error.contains(says), "{to}: {error}");
    }
    let bad = text.replace("brittle = \"brittle_arrow\"\n", "");
    let mut files = ContentFiles::builtin();
    files.insert("data/recipes/woodcraft/arrows.toml", &bad);
    let error = Content::load(&files).expect_err("no brittle").to_string();
    assert!(error.contains("`brittle`"), "{error}");
}
