//! Recipes as data (`data/recipes/<skill>/<id>.toml`, docs/design/professions.md §6; #1334): the
//! built-in ones load, a generic one of every field loads, and every check at load refuses.

use super::{Content, ContentFiles, PartSource, RecipeUse};

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
fn the_built_in_recipes_are_metalcrafts_temper_and_woodcrafts_fletch_and_whittle() {
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
    assert_eq!(of(metal), [("Temper", Some(RecipeUse::Temper))]);
    assert_eq!(
        of(wood),
        [
            ("Fletch", Some(RecipeUse::Fletch)),
            ("Whittle", Some(RecipeUse::Whittle))
        ]
    );
    // Open to everyone, as they were before (#1337 Q3).
    assert!(recipes.list.iter().all(|r| r.anyone && r.result.is_none()));
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

#[test]
fn the_settings_file_refuses_an_unknown_skill_or_a_profession_twice() {
    let message = |text: &str| match load_with(&[("data/recipes.toml", text)]) {
        Ok(_) => panic!("loaded"),
        Err(e) => e.message,
    };
    let row = |skill: &str| format!("[[profession]]\nskill = \"{skill}\"\ndescription = \"X.\"\n");
    assert!(message(&row("flying")).contains("unknown skill `flying`"));
    let twice = row("metalcraft").repeat(2);
    assert!(message(&twice).contains("listed twice"));
    assert!(message("bands = 3\n").contains("unknown field"));
}
