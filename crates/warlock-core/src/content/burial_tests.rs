//! Burial grounds' load checks (`burial.rs`; `docs/dev/graves.md`): a new crypt is one file, and
//! what a level author gets wrong is refused with the file named.

use super::*;

const CRYPT_1: &str = "campaigns/ascmearc/levels/lictun_crypt_01.toml";

/// Līctūn's chain as built in, and a crypt below its deepest: its file, its depth, and the level
/// above it (so the tests hold however many crypts the campaign has).
struct Below {
    file: String,
    depth: u8,
    up: String,
    /// The places on the chain's levels, all of them.
    places: usize,
}

static BELOW: std::sync::LazyLock<Below> = std::sync::LazyLock::new(|| {
    let content = Content::load(&ContentFiles::builtin()).expect("builtin content");
    let chain = content.burial_chain("lictun");
    let up = chain.last().expect("a chain").clone();
    let places = (chain.iter())
        .filter_map(|id| content.level(id)?.burial.as_ref())
        .map(|b| b.places.len())
        .sum();
    Below {
        file: format!(
            "campaigns/ascmearc/levels/lictun_crypt_{:02}.toml",
            chain.len()
        ),
        depth: u8::try_from(chain.len()).expect("a short chain"),
        up,
        places,
    }
});

/// The builtin files with a small crypt below the deepest, `extra` in its `[burial]` table.
fn new_crypt(extra: &str) -> ContentFiles {
    with(&BELOW.file, &crypt(BELOW.depth, &BELOW.up, extra))
}

/// The builtin files with a small crypt below the deepest, its text edited by `edit`.
fn new_crypt_edited(edit: impl Fn(String) -> String) -> ContentFiles {
    with(&BELOW.file, &edit(crypt(BELOW.depth, &BELOW.up, "")))
}

/// A small crypt at `depth` under `up`.
fn crypt(depth: u8, up: &str, extra: &str) -> String {
    format!(
        "name = \"Test crypt\"\nmap = '''\n#####\n#<.v#\n#N.O#\n#####\n'''\n\n[burial]\n\
         chain = \"lictun\"\ndepth = {depth}\n{extra}\n[legend]\n\"N\" = \"empty_niche\"\n\
         \"O\" = \"open_sarcophagus\"\n\"<\" = {{ terrain = \"stairs_up\", to = \"{up}\" }}\n\
         \"v\" = {{ terrain = \"floor\", burial_stairs = \"stairs_down\" }}\n"
    )
}

/// The builtin files with `path` set to `text`.
fn with(path: &str, text: &str) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert(path, text);
    files
}

/// The error loading `files` gives, as text with its file.
fn refused(files: &ContentFiles) -> String {
    let err = Content::load(files).expect_err("refused");
    format!("{}: {err}", err.file)
}

/// Builtin `path` with `from` replaced by `to`.
fn edited(path: &str, from: &str, to: &str) -> ContentFiles {
    let files = ContentFiles::builtin();
    let text = files.get(path).expect("builtin");
    assert!(text.contains(from), "{from}");
    with(path, &text.replace(from, to))
}

#[test]
fn a_new_crypt_is_one_file_and_the_one_above_leads_down_to_it() {
    let content = Content::load(&new_crypt("")).expect("loads");
    let id = format!("lictun_crypt_{:02}", BELOW.depth);
    let chain = content.burial_chain("lictun");
    assert_eq!(chain.last(), Some(&id));
    let above = content.level(&BELOW.up).and_then(|l| l.burial.as_ref());
    let way = above.and_then(|b| b.stairs.as_ref()).expect("a way down");
    assert_eq!(way.to.as_deref(), Some(id.as_str()));
    let (before, burial) = content.burial_of(&id).expect("a burial ground");
    assert_eq!(before, BELOW.places);
    assert_eq!(burial.places.len(), 2);
    assert_eq!(burial.epitaph, DEFAULT_EPITAPH);
    // The deepest's way down leads nowhere.
    assert_eq!(burial.stairs.as_ref().and_then(|s| s.to.as_deref()), None);
}

#[test]
fn a_chain_runs_from_its_churchyard_without_gaps() {
    let gap = refused(&with(&BELOW.file, &crypt(BELOW.depth + 1, &BELOW.up, "")));
    let missing = format!("depth {} has no level", BELOW.depth);
    assert!(gap.contains(&missing), "{gap}");
    let twice = refused(&with(&BELOW.file, &crypt(2, "lictun_crypt_01", "")));
    assert!(twice.contains("at depth 2 of `lictun` too"), "{twice}");
    let up = refused(&with(&BELOW.file, &crypt(BELOW.depth, "lictun", "")));
    assert!(up.starts_with(&BELOW.file), "{up}");
    assert!(up.contains("no stairs up"), "{up}");
    let yard = "campaigns/ascmearc/levels/lictun.toml";
    let nameless = refused(&edited(yard, "chain = \"lictun\"", "chain = \"aaa\""));
    assert!(
        nameless.contains("named after its depth-0 level"),
        "{nameless}"
    );
    // Only the deepest may lack a way down.
    let shut = refused(&edited(
        CRYPT_1,
        "\"v\" = { terrain = \"dusty_floor\", burial_stairs = \"stairs_down\" }",
        "\"v\" = \"dusty_floor\"",
    ));
    assert!(shut.contains("no way down"), "{shut}");
    let deepest = edited(
        &format!("campaigns/ascmearc/levels/{}.toml", BELOW.up),
        "\"v\" = { terrain = \"floor\", burial_stairs = \"stairs_down\" }",
        "\"v\" = \"floor\"",
    );
    Content::load(&deepest).expect("the deepest may");
}

#[test]
fn what_a_level_author_gets_wrong_is_refused() {
    let burial = format!("[burial]\nchain = \"lictun\"\ndepth = {}\n", BELOW.depth);
    let unmarked = refused(&new_crypt_edited(|t| t.replace(&burial, "")));
    assert!(unmarked.contains("need a `[burial]` table"), "{unmarked}");
    let nobody = refused(&new_crypt("epitaph = \"Here lies someone.\""));
    assert!(nobody.contains("names no one"), "{nobody}");
    let blank = refused(&new_crypt_edited(|t| {
        t.replace("\"empty_niche\"", "\"sealed_niche\"")
    }));
    assert!(blank.contains("has no `epitaph`"), "{blank}");
    let statue = refused(&new_crypt_edited(|t| {
        t.replace(
            "\"O\" = \"open_sarcophagus\"",
            "\"O\" = { terrain = \"wall\", statue_place = true }",
        )
    }));
    assert!(statue.contains("`statue_place`"), "{statue}");
}

#[test]
fn a_place_and_its_grave_are_the_same_in_every_rule() {
    let path = "data/terrain/332-grave_plot.toml";
    let err = refused(&edited(path, "walkable = false\n", ""));
    assert!(err.contains("must be the same in every rule"), "{err}");
    let err = refused(&edited(
        path,
        "burial = \"gravestone\"",
        "burial = \"flagstones\"",
    ));
    assert!(err.contains("must be a grave"), "{err}");
}
