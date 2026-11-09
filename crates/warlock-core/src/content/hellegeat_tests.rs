//! Hellegeat, the shared portal branch (#1296; spec §7 "The Pits"): nothing in the content leads
//! into a Pit (stairs, a hidden location, a quest), and its folk stay in it.

use super::{Content, ContentFiles};

/// The built-in content with `edit` made to file `path`.
fn edited(path: &str, edit: impl FnOnce(&str) -> String) -> Result<Content, String> {
    let mut files = ContentFiles::builtin();
    let text = edit(files.get(path).expect(path));
    files.insert(path, &text);
    Content::load(&files).map_err(|e| e.to_string())
}

#[test]
fn no_stairs_lead_into_a_pit() {
    let err = edited("campaigns/ascmearc/levels/ascmearc.toml", |t| {
        t.replace("to = \"ceaptun\"", "to = \"hellegeat@1:1\"")
    })
    .expect_err("a way into a Pit");
    assert!(err.contains("a Pit's floor"), "{err}");
}

#[test]
fn no_quest_leads_into_a_pit() {
    let quest = |kind: &str, target: &str, more: &str| {
        format!(
            "[[quest]]\nid = \"pit\"\nname = \"Pit\"\ngiver = \"shopkeeper\"\ntype = \"{kind}\"\n\
             target = \"{target}\"\n{more}offer = \"O.\"\naccept = \"A.\"\nreminder = \"R.\"\n\
             done = \"D.\"\n"
        )
    };
    let load = |text: String| {
        let mut files = ContentFiles::builtin();
        files.insert("data/quests/zz_pit.toml", &text);
        Content::load(&files).map(|_| ()).map_err(|e| e.to_string())
    };
    assert!(
        load(quest("reach", "grundleas:3", "")).is_ok(),
        "an endless floor may be"
    );
    for bad in [
        quest("reach", "hellegeat@1:1", ""),
        quest("kill", "rat", "level = \"hellegeat\"\n"),
        quest("kill", "rat", "level = \"hellegeat@1:2\"\n"),
    ] {
        let err = load(bad.clone()).expect_err(&bad);
        assert!(err.contains("unknown level"), "{err}");
    }
    let pick = "[[quest]]\nid = \"pit\"\nname = \"Pit\"\ngiver = \"shopkeeper\"\ntype = \"kill\"\n\
                pick = { creatures = [\"hellegeat\"] }\noffer = \"O.\"\naccept = \"A.\"\n\
                reminder = \"R.\"\ndone = \"D.\"\n";
    let err = load(pick.to_string()).expect_err("a pick of a Pit's folk");
    assert!(err.contains("unknown dungeon branch"), "{err}");
}

#[test]
fn a_pits_folk_stay_in_it() {
    let content = Content::builtin().expect("content");
    let hellegeat = content.dungeon("hellegeat").expect("Hellegeat");
    let spawning = hellegeat.spawning.as_ref().expect("a danger");
    let erinyes = content.creature_id("erinyes").expect("the erinyes");
    assert!(spawning.population.contains(&erinyes));
    // The erinyes lives in no other population: no branch that draws its strays from the
    // populations alone meets it, though the Pit is shared.
    for d in content.dungeons().filter(|d| d.portal.is_none()) {
        if let Some(s) = &d.spawning
            && s.pool == super::danger::StrayPool::Populations
        {
            assert!(!s.generic.contains(&erinyes), "{}", d.id);
        }
    }
}
