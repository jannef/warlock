//! The loader's whole-set checks: every file read (#1530), the ids the engine names (#1546), and
//! `gear_condition.toml`'s keys (#1556), on the built-in set with one file changed.

use super::{Content, ContentFiles};
use crate::test_support::builtin;

fn load_err(files: &ContentFiles) -> String {
    Content::load(files).expect_err("refused").to_string()
}

#[test]
fn a_file_no_loader_reads_is_refused() {
    let mut files = ContentFiles::builtin();
    files.insert("data/feints.toml", "difficulty = 12\n");
    let err = load_err(&files);
    assert!(
        err.starts_with("data/feints.toml: no loader reads"),
        "{err}"
    );

    let mut files = ContentFiles::builtin();
    files.insert("data/town/cheapside.toml", "");
    assert!(load_err(&files).starts_with("data/town/cheapside.toml"));

    // Comments for the authors are never read, anywhere.
    let mut files = ContentFiles::builtin();
    files.insert("data/town/README.toml", "# notes\n");
    Content::load(&files).expect("a README is no content");
}

#[test]
fn a_required_id_missing_from_its_table_is_refused() {
    let mut files = ContentFiles::builtin();
    let terrain = files.table_text("terrain");
    let renamed = terrain.replace("id = \"makeshift_anvil\"", "id = \"field_anvil\"");
    assert_ne!(renamed, terrain);
    files.replace_table("terrain", &renamed);
    let err = load_err(&files);
    assert!(
        err.contains("the terrain `makeshift_anvil` is missing"),
        "{err}"
    );

    let content = builtin();
    let ids = content.ids();
    assert_eq!(ids.staggered, content.status_id("staggered"));
    assert_eq!(ids.player, content.creature_id("player"));
    assert!(ids.stairs_down.is_some() && ids.neck.is_some() && ids.bones.is_some());
}

#[test]
fn gear_condition_keys_must_name_content() {
    let edited = |from: &str, to: &str| {
        let mut files = ContentFiles::builtin();
        let path = "data/gear_condition.toml";
        let text = (files.get_optional(path).expect("the file")).to_string();
        assert!(text.contains(from), "{from}");
        files.insert(path, &text.replacen(from, to, 1));
        load_err(&files)
    };
    let err = edited("metal = [", "mettal = [");
    assert!(
        err.contains("no material is of the group `mettal`"),
        "{err}"
    );
    let err = edited("tag = \"undead\"", "tag = \"undeed\"");
    assert!(
        err.contains("tribe `undeed`: no creature carries the tag"),
        "{err}"
    );
}
