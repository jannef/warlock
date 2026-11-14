//! A vault file's `danger = { from, to }` (#1073): what loads, what is refused, and that a vault
//! drawn at random keeps to the floors of its range.

use super::*;
use crate::content::{ContentFiles, DangerRange};

/// A small hall behind a door, its statue named so a test can find it on a floor.
fn vault_file(head: &str) -> String {
    format!(
        "name = \"Marked\"\n{head}\nmap = '''\n#####\n#.S.#\n#...#\n##+##\n'''\n\
         [legend]\n\"S\" = {{ terrain = \"statue\", name = \"the marker\" }}\n"
    )
}

/// The builtin content without its shared vaults (so the random vaults of the branch are the
/// test's), the vault `marked` from `head` and three plain ones (so the deal puts `marked` in any
/// of the four phases), and a rooms branch `graded` of twelve floors whose danger is its depth.
fn content_with(head: &str) -> Result<Content, String> {
    let mut files = ContentFiles::builtin();
    files.remove_shared_vaults();
    files.insert("vaults/marked.toml", &vault_file(head));
    for plain in ["plain_a", "plain_b", "plain_c"] {
        files.insert(
            &format!("vaults/{plain}.toml"),
            "name = \"Plain\"\nmap = '''\n##+##\n#...#\n#####\n'''\n",
        );
    }
    let branch = "\n[[dungeon]]\nid = \"graded\"\nname = \"Graded\"\ngenerator = \"rooms\"\n\
         depth = 12\nsize = [80, 30]\ndanger = { base = 1, per_depth = 1 }\n\
         population = [\"kobold\", \"rat\"]\npopulation_count = 10\n";
    let dungeons = format!(
        "{}{branch}",
        include_str!("../../../../assets/data/dungeons.toml")
    );
    files.insert("data/dungeons.toml", &dungeons);
    Content::load(&files).map_err(|e| e.message)
}

fn marked(content: &Content) -> &Vault {
    (content.vaults().iter())
        .find(|v| v.id() == "marked")
        .expect("the vault")
}

/// Whether the vault stamped on floor `depth` of seed `seed`.
fn stamped(content: &Content, seed: u64, depth: u32) -> bool {
    let id = format!("graded:{depth}");
    let level = generate(content, ("showcase", seed), &id, None).expect("the floor");
    level.places.values().any(|p| p.name == "the marker")
}

/// Over many seeds, the floors (by danger, which here is the depth) the vault stamped on.
fn stamped_dangers(content: &Content) -> std::collections::BTreeSet<u32> {
    let mut dangers = std::collections::BTreeSet::new();
    for seed in 0..80 {
        for depth in 1..=12 {
            if stamped(content, seed, depth) {
                dangers.insert(depth);
            }
        }
    }
    dangers
}

#[test]
fn the_field_parses_and_holds_its_ends() {
    let content = content_with("danger = { from = 2, to = 40 }").expect("content");
    let range = marked(&content).danger;
    assert_eq!(
        range,
        DangerRange {
            from: Some(2),
            to: Some(40)
        }
    );
    assert!(!range.holds(1));
    assert!(range.holds(2) && range.holds(40));
    assert!(!range.holds(41));
    let content = content_with("").expect("content");
    assert!(!marked(&content).danger.is_given(), "absent: every danger");
    assert!(marked(&content).suits(0) && marked(&content).suits(99));
}

#[test]
fn a_vault_from_a_danger_stamps_only_at_or_above_it() {
    let content = content_with("danger = { from = 6 }").expect("content");
    let dangers = stamped_dangers(&content);
    assert!(dangers.iter().all(|d| *d >= 6), "{dangers:?}");
    assert!(dangers.contains(&6) && dangers.len() > 3, "{dangers:?}");
}

#[test]
fn a_vault_to_a_danger_stamps_only_at_or_below_it() {
    let content = content_with("danger = { to = 6 }").expect("content");
    let dangers = stamped_dangers(&content);
    assert!(dangers.iter().all(|d| *d <= 6), "{dangers:?}");
    assert!(dangers.contains(&6) && dangers.len() > 3, "{dangers:?}");
}

#[test]
fn the_deal_is_made_before_the_danger_is_asked() {
    // A vault's `danger` takes it out of a floor's list; every other floor's list is the deal's.
    let ranged = content_with("danger = { from = 6 }").expect("content");
    let free = content_with("").expect("content");
    for seed in 0..8 {
        for depth in 1..=12 {
            let ids = |content: &Content, danger: u8| -> Vec<String> {
                let def = content.dungeon("graded").expect("the branch");
                (spaced_vaults(content, seed, (def, depth), danger).iter())
                    .map(|v| v.id().to_string())
                    .collect()
            };
            let danger = u8::try_from(depth).expect("a danger");
            let mut want = ids(&free, danger);
            want.retain(|v| danger >= 6 || v != "marked");
            assert_eq!(ids(&ranged, danger), want, "seed {seed} depth {depth}");
        }
    }
}

#[test]
fn what_the_field_refuses_is_refused_with_a_message() {
    let refused = |head: &str, says: &str| {
        let err = content_with(head).expect_err(head);
        assert!(err.contains(says), "{head}: {err}");
    };
    // A vault draws at the floor's danger: a level's rule is no vault's.
    refused(
        "danger = 3\npopulation = [\"kobold\"]",
        "no danger rule of its own",
    );
    // A typo or a number past a danger is no range: refused, not read as every danger (#1588).
    for (head, says) in [
        ("danger = { form = 8 }", "`danger`: unknown field `form`"),
        (
            "danger = { from = 300 }",
            "`danger`: invalid value: integer `300`",
        ),
        (
            "danger = { from = 2, too = 6 }",
            "`danger`: unknown field `too`",
        ),
        (
            "danger = { base = 2, from = 3 }",
            "`danger`: unknown field `base`",
        ),
    ] {
        refused(head, says);
    }

    // An outdoor vault: a local area has no danger.
    let mut files = ContentFiles::builtin();
    files.insert(
        "vaults/open.toml",
        "name = \"Open\"\nkind = \"outdoor\"\ndanger = { from = 3 }\nmap = '''\n...\n.#.\n...\n'''\n",
    );
    let err = Content::load(&files).expect_err("outdoor").message;
    assert!(err.contains("an outdoor vault has no `danger`"), "{err}");

    // A level: only a vault keeps to a range.
    let mut files = ContentFiles::builtin();
    files.insert(
        "levels/ranged.toml",
        "name = \"Ranged\"\ndanger = { from = 3 }\nmap = '''\n###\n#.#\n###\n'''\n",
    );
    let err = Content::load(&files).expect_err("a level").message;
    assert!(err.contains("only a vault keeps to a range"), "{err}");
}

#[test]
fn a_vault_a_branch_level_names_takes_no_danger() {
    let mut files = ContentFiles::builtin();
    files.remove_shared_vaults();
    files.insert("vaults/marked.toml", &vault_file("danger = { from = 6 }"));
    let branch = "\n[[dungeon]]\nid = \"pinning\"\nname = \"Pinning\"\ngenerator = \"rooms\"\n\
         depth = 1\nsize = [80, 21]\n[[dungeon.levels]]\nfrom = 1\nvaults = [\"marked\"]\n";
    let dungeons = format!(
        "{}{branch}",
        include_str!("../../../../assets/data/dungeons.toml")
    );
    files.insert("data/dungeons.toml", &dungeons);
    let err = Content::load(&files).expect_err("pinned").message;
    assert!(err.contains("stamped there for certain"), "{err}");
}
