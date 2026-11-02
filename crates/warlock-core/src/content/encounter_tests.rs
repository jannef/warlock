//! `data/encounters.toml` and the campaigns' encounter tables: parsing and the load checks.

use super::encounter::*;
use super::{Content, ContentFiles, Landscape};

const SHIPPED: &str = include_str!("../../../../assets/data/encounters.toml");

#[test]
fn the_shipped_file_holds_at_three() {
    let rules = EncounterRules::parse("data/encounters.toml", SHIPPED).unwrap();
    assert_eq!(rules.edge_reach, 3);
}

#[test]
fn an_edge_reach_out_of_range_is_refused() {
    let err = EncounterRules::parse("x", "edge_reach = 11").unwrap_err();
    assert!(err.to_string().contains("out of range"), "{err}");
    assert!(EncounterRules::parse("x", "edge_reach = -1").is_err());
    let rules = EncounterRules::parse("x", "edge_reach = 0").unwrap();
    assert_eq!(rules.edge_reach, 0);
    assert_eq!(rules.meetings, None, "no tables: nobody meets anybody");
}

#[test]
fn the_shipped_meetings_read_as_the_design_has_them() {
    let rules = EncounterRules::parse("data/encounters.toml", SHIPPED).unwrap();
    let m = rules.meetings.expect("meetings");
    assert_eq!(m.rate_of(Landscape::Road), 100);
    assert_eq!(m.rate_of(Landscape::Swamp), 800);
    assert_eq!(m.rate_of(Landscape::River), 0);
    assert_eq!(m.cover_of(Landscape::DenseForest), 4);
    assert_eq!(m.phase.night, 200);
    assert_eq!(m.ambush_wait, 600);
    assert_eq!(m.rest_percent, 25, "a quarter of the rate an hour of rest");
    let p = m.positions.player;
    // The more the encounter favours the player, the nearer the edge they stand.
    assert!(p.ambush < p.fight && p.fight < p.close && p.close < p.ambushed);
    assert_eq!(p.ambushed, 100, "ambushed in the middle");
}

#[test]
fn the_meetings_are_all_given_or_none_and_in_range() {
    let some = "edge_reach = 3\n[phase]\nday = 100\ntwilight = 100\nnight = 100\n";
    let err = EncounterRules::parse("x", some).unwrap_err().to_string();
    assert!(err.contains("all given or none"), "{err}");
    let no_swamp = SHIPPED.replace("swamp = 800\n", "");
    let err = EncounterRules::parse("x", &no_swamp)
        .unwrap_err()
        .to_string();
    assert!(err.contains("names no `swamp`"), "{err}");
    let far = SHIPPED.replace("fight = 7 ", "fight = 30 ");
    assert!(EncounterRules::parse("x", &far).is_err());
    let past = SHIPPED.replace("ambushed = 100 }", "ambushed = 101 }");
    assert!(EncounterRules::parse("x", &past).is_err());
}

/// A fixture table in the showcase campaign.
pub(crate) const TABLE: &str = r#"
[[table]]
id = "wilds"
[[table.entry]]
id = "kobolds"
name = "a band of kobolds"
weight = 3
group = [{ creature = "kobold", count = "1d2+1" }]
"#;

fn load(table: &str) -> Result<Content, String> {
    let mut files = ContentFiles::builtin();
    files.insert("campaigns/showcase/data/encounters/test.toml", table);
    Content::load(&files).map_err(|e| e.to_string())
}

#[test]
fn a_campaigns_table_loads() {
    let content = load(TABLE).expect("loads");
    let tables: Vec<_> = content.encounter_tables("showcase").collect();
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0].rate_percent, 100);
    let (_, entry) = content
        .encounter_entry("showcase", "kobolds")
        .expect("entry");
    assert_eq!(entry.weight, 3);
    assert_eq!(entry.lead, content.creature_id("kobold"));
    assert_eq!(entry.stance, EntryStance::Hostile);
    // Another campaign's tables are its own.
    assert!(
        content
            .encounter_tables("ascmearc")
            .all(|t| t.campaign == "ascmearc")
    );
}

#[test]
fn the_load_checks_refuse_what_is_not_there() {
    let cases = [
        ("count = \"1d2+1\"", "count = \"1d2-1\"", "at least 1"),
        (
            "creature = \"kobold\"",
            "creature = \"nobody\"",
            "unknown creature",
        ),
        (
            "weight = 3",
            "weight = 3\nwhen = { regions = [\"Nowhere\"] }",
            "no region",
        ),
        (
            "weight = 3",
            "weight = 3\nwhen = { landscapes = [\"sea\"] }",
            "no landscape",
        ),
        (
            "weight = 3",
            "weight = 3\nwhen = { quest_active = \"nope\" }",
            "no quest",
        ),
        (
            "weight = 3",
            "weight = 3\nwhen = { revealed = \"nope\" }",
            "no hidden location",
        ),
        (
            "weight = 3",
            "weight = 3\nwhen = { level = { min = 3 } }",
            "unknown field",
        ),
        (
            "weight = 3",
            "weight = 3\nstance = \"peaceful\"",
            "hostile to the player",
        ),
        (
            "id = \"wilds\"",
            "id = \"wilds\"\nreplaces = [\"gone\"]",
            "no other table",
        ),
    ];
    for (from, to, why) in cases {
        let err = load(&TABLE.replacen(from, to, 1)).expect_err(to);
        assert!(err.contains(why), "{to}: {err}");
    }
    let err = load(&TABLE.replace("kobold\"", "hermit\"")).expect_err("hermit");
    assert!(err.contains("not hostile"), "{err}");
    // A level ceiling is the one level condition (the owner, #796 Q5).
    let max = TABLE.replacen(
        "weight = 3",
        "weight = 3\nwhen = { level = { max = 8 } }",
        1,
    );
    let content = load(&max).expect("loads");
    let (_, e) = content.encounter_entry("showcase", "kobolds").unwrap();
    assert_eq!(e.when.level_max, Some(8));
}

#[test]
fn draws_npcs_and_quest_variants_are_checked() {
    // The fixture in the Ascmearc, beside its own tables.
    let load_in = |campaign: &str, table: &str| -> Result<Content, String> {
        let mut files = ContentFiles::builtin();
        files.insert(
            &format!("campaigns/{campaign}/data/encounters/test.toml"),
            table,
        );
        Content::load(&files).map_err(|e| e.to_string())
    };
    // A draw: a branch of the campaign with a population, instead of a group.
    let group = "group = [{ creature = \"kobold\", count = \"1d2+1\" }]";
    let draw = "draw = { pool = \"wulfeshol\", danger = 3, count = \"1d3\" }";
    let drawn = TABLE.replacen(group, draw, 1);
    let content = load_in("ascmearc", &drawn).expect("a draw loads");
    let (_, e) = content.encounter_entry("ascmearc", "kobolds").unwrap();
    let d = e.draw.as_ref().expect("the draw");
    assert_eq!((d.pool.as_str(), d.danger, e.lead), ("wulfeshol", 3, None));
    assert!(e.group.is_empty());
    let with = |what: &str| TABLE.replacen("weight = 3", &format!("weight = 3\n{what}"), 1);
    let drawn_with = |what: &str| drawn.replacen("weight = 3", &format!("weight = 3\n{what}"), 1);
    let cases = [
        (
            "showcase",
            drawn.clone(),
            "no branch `wulfeshol` of the campaign",
        ),
        (
            "showcase",
            drawn.replace("wulfeshol", "depths"),
            "has no population",
        ),
        (
            "ascmearc",
            drawn.replace("danger = 3", "danger = 0"),
            "1 to 99",
        ),
        ("ascmearc", drawn.replace("1d3", "1d3-1"), "gives 1 to"),
        (
            "ascmearc",
            drawn_with("stance = \"wildlife\""),
            "only a hostile",
        ),
        ("ascmearc", drawn_with("lead = \"kobold\""), "no `lead`"),
        ("ascmearc", with(draw), "not both"),
        // An npc: a dialogue of the campaign's, for a peaceful party.
        ("ascmearc", with("npc = \"nobody\""), "no dialogue of npc"),
        (
            "showcase",
            with("npc = \"road_folk\""),
            "no dialogue of npc",
        ),
        ("ascmearc", with("npc = \"road_folk\""), "only a peaceful"),
        // A quest's variant: a quest of the campaign with that variant.
        (
            "showcase",
            with("when = { quest_variant = { quest = \"guild_wanted\", variant = \"wystan\" } }"),
            "no quest `guild_wanted` of the campaign",
        ),
        (
            "ascmearc",
            with("when = { quest_variant = { quest = \"guild_wanted\", variant = \"nobody\" } }"),
            "has no variant `nobody`",
        ),
    ];
    for (campaign, table, why) in cases {
        let err = load_in(campaign, &table).expect_err(why);
        assert!(err.contains(why), "{why}: {err}");
    }
    // The Ascmearc's own: an npc for its villagers, Wystan's variant.
    let (_, e) = content
        .encounter_entry("ascmearc", "road_villagers")
        .unwrap();
    assert_eq!(e.npc.as_deref(), Some("road_folk"));
    let (t, _) = content
        .encounter_entry("ascmearc", "wystan_archers")
        .unwrap();
    let variant = (t.when.quest_variant.as_ref()).map(|(q, v)| (q.as_str(), v.as_str()));
    assert_eq!(variant, Some(("guild_wanted", "wystan")));
}

#[test]
fn a_table_outside_every_campaign_is_refused() {
    let mut files = ContentFiles::builtin();
    files.insert("data/encounters/loose.toml", TABLE);
    let err = Content::load(&files).expect_err("loose").to_string();
    assert!(err.contains("belong to a campaign"), "{err}");
}
