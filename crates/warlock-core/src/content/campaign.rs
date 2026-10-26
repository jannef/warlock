//! Campaigns (spec §3, `docs/dev/campaign.md`): worlds that share the engine's data and nothing
//! else.
//!
//! A campaign is a directory `campaigns/<id>/` with a `campaign.toml` ([`CampaignDef`]) and what
//! is the world's own: `levels/`, `scenarios/`, `vaults/`, and the tables `data/dungeons/`,
//! `data/quests/`, `data/dialogue/`, and `data/creatures/` (its own creature types, numbered
//! after the shared ones). Everything else under the assets root is shared engine data. Two
//! campaigns never link: a campaign's file names levels and creature types of its own campaign
//! or shared ones, never another campaign's ([`Content::check_campaigns`]). Ids are global
//! slugs, unique across every campaign (a clash is a content error naming both files).
//!
//! Files outside `campaigns/` that campaigns would own (`levels/`, `scenarios/`, the single
//! `data/quests.toml`) belong to no campaign: test fixtures use them, and they may name anything.

use super::{Content, ContentError, ContentFiles, LevelKind};
use serde::Deserialize;

/// The directory campaigns live in, under the assets root.
pub const CAMPAIGNS: &str = "campaigns";

/// What a campaign is for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignKind {
    /// A world to play: New run lists its start scenario in every build.
    #[default]
    Campaign,
    /// A test world (the showcase): New run lists its scenarios in developer builds only, and
    /// the player wiki leaves it out.
    Test,
}

/// A campaign: `campaigns/<id>/campaign.toml`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignDef {
    /// The directory's name.
    pub id: String,
    /// What New run groups the campaign's scenarios under.
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// The scenario a new run of the campaign starts: one of its own.
    pub start: String,
    #[serde(default)]
    pub kind: CampaignKind,
}

/// The campaign a content path belongs to: `campaigns/showcase/levels/hub.toml` → `showcase`;
/// `None` outside `campaigns/`.
pub fn of_path(path: &str) -> Option<&str> {
    let rest = path.strip_prefix(CAMPAIGNS)?.strip_prefix('/')?;
    let (id, _) = rest.split_once('/')?;
    Some(id)
}

/// Just the kind of a `campaign.toml`, read leniently to order the tables before the campaigns
/// are parsed properly.
#[derive(Deserialize)]
struct KindOnly {
    #[serde(default)]
    kind: CampaignKind,
}

impl ContentFiles {
    /// The campaign directories, in the order their tables number: those of kind `campaign` in
    /// path order, then the test ones (so a build without the test world numbers a real
    /// campaign's entries exactly as a build with it does).
    pub fn campaign_order(&self) -> Vec<&str> {
        let prefix = format!("{CAMPAIGNS}/");
        let mut ids: Vec<&str> = Vec::new();
        for (path, _) in self.files.range(prefix.clone()..) {
            if !path.starts_with(&prefix) {
                break;
            }
            if let Some(id) = of_path(path)
                && ids.last() != Some(&id)
            {
                ids.push(id);
            }
        }
        let test = |id: &&str| {
            (self.get_optional(&format!("{CAMPAIGNS}/{id}/campaign.toml")))
                .and_then(|text| toml::from_str::<KindOnly>(text).ok())
                .is_some_and(|k| k.kind == CampaignKind::Test)
        };
        let (tests, mut order): (Vec<&str>, Vec<&str>) = ids.into_iter().partition(test);
        order.extend(tests);
        order
    }
}

/// Parses every campaign's `campaign.toml`, in [`ContentFiles::campaign_order`].
pub(super) fn parse(files: &ContentFiles) -> Result<Vec<CampaignDef>, ContentError> {
    let mut campaigns = Vec::new();
    for dir in files.campaign_order() {
        let path = format!("{CAMPAIGNS}/{dir}/campaign.toml");
        let text = files.get(&path)?;
        let def: CampaignDef =
            toml::from_str(text).map_err(|e| ContentError::new(&path, e.to_string()))?;
        if def.id != dir {
            return Err(ContentError::new(
                &path,
                format!("id `{}` is not the directory's name `{dir}`", def.id),
            ));
        }
        campaigns.push(def);
    }
    Ok(campaigns)
}

/// What a file names that must stay in its campaign.
fn misplaced(file: &str, what: &str, id: &str, campaign: &str) -> ContentError {
    ContentError::new(
        file,
        format!("names {what} `{id}`, a {what} of campaign `{campaign}`"),
    )
}

impl Content {
    /// The campaigns, in [`ContentFiles::campaign_order`].
    pub fn campaigns(&self) -> &[CampaignDef] {
        &self.campaigns
    }

    /// The campaign with id `id`.
    pub fn campaign(&self, id: &str) -> Option<&CampaignDef> {
        self.campaigns.iter().find(|c| c.id == id)
    }

    /// The campaign a hand-made level or a dungeon branch's level belongs to; `None` for one
    /// outside every campaign, for a local area, and for an unknown id.
    pub fn level_campaign(&self, id: &str) -> Option<&str> {
        if let Some(c) = self.level_campaigns.get(id) {
            return c.as_deref();
        }
        let branch = self.branch_level(id).map(|(def, _)| def);
        branch
            .or_else(|| self.dungeon(id))
            .and_then(|d| d.campaign.as_deref())
    }

    /// The scenario a run starts when none is named (`--scenario` left out): the showcase where
    /// the test world is built in, else the first campaign's start (a release build, #331);
    /// `None` without any campaign.
    pub fn default_scenario(&self) -> Option<&str> {
        const SHOWCASE: &str = "showcase";
        if self.scenario(SHOWCASE).is_some() {
            return Some(SHOWCASE);
        }
        self.campaigns.first().map(|c| c.start.as_str())
    }

    /// The overworld of campaign `campaign` (`""`:of the files outside every campaign), if it
    /// has one: a campaign has at most one level of kind `overworld`.
    pub fn overworld_of(&self, campaign: &str) -> Option<&super::LevelTemplate> {
        self.levels.values().find(|l| {
            l.kind == LevelKind::Overworld
                && self.level_campaign(&l.id).unwrap_or_default() == campaign
        })
    }

    /// Checks that no campaign's file names another campaign's level or creature type, and
    /// that each campaign starts in a scenario of its own. A file
    /// outside `campaigns/` that campaigns would own (a level, a scenario, a quest: test
    /// fixtures) may name anything; shared data (`data/`, `vaults/`) names shared creature types
    /// only.
    pub(super) fn check_campaigns(&self) -> Result<(), ContentError> {
        for campaign in &self.campaigns {
            let file = format!("{CAMPAIGNS}/{}/campaign.toml", campaign.id);
            let start = self.scenario(&campaign.start).ok_or_else(|| {
                ContentError::new(
                    &file,
                    format!("unknown start scenario `{}`", campaign.start),
                )
            })?;
            if start.campaign.as_deref() != Some(campaign.id.as_str()) {
                return Err(ContentError::new(
                    &file,
                    format!("start scenario `{}` is not the campaign's own", start.id),
                ));
            }
        }
        // Levels: their stairs and their creatures.
        for level in self.levels.values() {
            let Some(owner) = self.level_campaigns.get(&level.id) else {
                continue;
            };
            let file = self.level_file(&level.id);
            if let Some(owner) = owner {
                for to in level.stairs.iter().filter_map(|s| s.to.as_deref()) {
                    self.check_level(&file, owner, to)?;
                }
            }
            for spawn in &level.creatures {
                self.check_creature(&file, owner.as_deref(), spawn.creature)?;
            }
            for &creature in level.spawning.iter().flat_map(|(_, s)| &s.population) {
                self.check_creature(&file, owner.as_deref(), creature)?;
            }
        }
        for scenario in self.scenarios.values() {
            if let Some(owner) = scenario.campaign.as_deref() {
                let file = format!("{CAMPAIGNS}/{owner}/scenarios/{}.toml", scenario.id);
                self.check_level(&file, owner, &scenario.start_level)?;
            }
        }
        for vault in &self.vaults {
            let owner = vault.campaign.as_deref();
            let file = vault.file();
            for spawn in &vault.template.creatures {
                self.check_creature(&file, owner, spawn.creature)?;
            }
        }
        for dungeon in self.dungeons.values() {
            let file = table_file(dungeon.campaign.as_deref(), "dungeons");
            let lives = dungeon.spawning.iter().flat_map(|s| &s.population);
            for &creature in dungeon.creatures.iter().chain(lives) {
                self.check_creature(&file, dungeon.campaign.as_deref(), creature)?;
            }
        }
        for dialogue in &self.dialogues {
            let file = table_file(dialogue.campaign.as_deref(), "dialogue");
            if let super::Speaker::Creature(c) = dialogue.speaker {
                self.check_creature(&file, dialogue.campaign.as_deref(), c)?;
            }
        }
        for quest in self.quests.iter().filter(|q| q.campaign.is_some()) {
            let owner = quest.campaign.as_deref().unwrap_or_default();
            let file = table_file(quest.campaign.as_deref(), "quests");
            for place in quest.places() {
                self.check_level(&file, owner, place)?;
            }
            // A tag is checked as the quest is read (`quest::check_tag`).
            if let Some(&super::KillTarget::Creature(creature)) = quest.kill_target() {
                self.check_creature(&file, Some(owner), creature)?;
            }
            for speaker in quest.speakers() {
                let theirs = self.dialogue(speaker).campaign.as_deref();
                if let Some(theirs) = theirs.filter(|&c| c != owner) {
                    let name = self.speaker_name(speaker);
                    return Err(misplaced(&file, "speaker", &name, theirs));
                }
            }
        }
        if let Some(shops) = &self.shops {
            self.check_creature("data/shops.toml", None, shops.keeper.creature)?;
        }
        Ok(())
    }

    /// The file a hand-made level was read from, for an error.
    pub(super) fn level_file(&self, id: &str) -> String {
        match self.level_campaigns.get(id).cloned().flatten() {
            Some(c) => format!("{CAMPAIGNS}/{c}/levels/{id}.toml"),
            None => format!("levels/{id}.toml"),
        }
    }

    /// `file`, of campaign `owner`, names level `to` (a level id, a branch level, or a branch):
    /// refused when it is another campaign's.
    fn check_level(&self, file: &str, owner: &str, to: &str) -> Result<(), ContentError> {
        match self.level_campaign(to) {
            Some(theirs) if theirs != owner => Err(misplaced(file, "level", to, theirs)),
            _ => Ok(()),
        }
    }

    /// `file`, of campaign `owner` (`None`: shared data, or a file outside every campaign that
    /// campaigns would own), names creature type `creature`: refused when it is another
    /// campaign's, or a campaign's and `file` is shared data.
    fn check_creature(
        &self,
        file: &str,
        owner: Option<&str>,
        creature: super::CreatureId,
    ) -> Result<(), ContentError> {
        let def = self.creature(creature);
        let Some(theirs) = def.campaign.as_deref() else {
            return Ok(());
        };
        let loose = owner.is_none() && !is_shared(file);
        if loose || owner == Some(theirs) {
            Ok(())
        } else {
            Err(misplaced(file, "creature", &def.id, theirs))
        }
    }
}

/// Whether `file` is shared engine data (`data/...`, `vaults/...`) rather than a campaign's or a
/// test fixture's level, scenario, or table outside every campaign.
fn is_shared(file: &str) -> bool {
    file.starts_with("vaults/") || file == "data/shops.toml"
}

/// The table directory of campaign `campaign` (`data/<name>` outside every campaign), for an
/// error.
fn table_file(campaign: Option<&str>, name: &str) -> String {
    match campaign {
        Some(c) => format!("{CAMPAIGNS}/{c}/data/{name}"),
        None => format!("data/{name}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;
    use std::sync::Arc;

    const ASH: &str = "campaigns/ashland";

    /// A town level of the Ashland, with `legend` added (`v` is a villager by default).
    fn town(extra: &str) -> String {
        format!(
            "name = \"Town\"\nmap = '''\n#######\n#@..v.#\n#######\n'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"v\" = {{ terrain = \"floor\", creature = \"villager\" }}\n{extra}"
        )
    }

    /// The built-in content and a second world, the Ashland (kind `campaign`): a town level, a
    /// start scenario, and a creature type of its own, the villager. The Ascmearc is left out:
    /// the fixture is the one playable campaign.
    fn with_ashland() -> ContentFiles {
        let mut files = ContentFiles::builtin();
        files.remove("campaigns/ascmearc/");
        files.insert(
            &format!("{ASH}/campaign.toml"),
            "id = \"ashland\"\nname = \"The Ashland\"\nstart = \"ashland\"\n",
        );
        files.insert(&format!("{ASH}/levels/town.toml"), &town(""));
        files.insert(
            &format!("{ASH}/scenarios/ashland.toml"),
            "name = \"Ashland\"\nstart_level = \"town\"\n",
        );
        let deer = files
            .get("data/creatures/deer.toml")
            .expect("deer")
            .to_string();
        let villager = deer
            .replace("id = \"deer\"", "id = \"villager\"")
            .replace("name = \"deer\"", "name = \"villager\"")
            .replace("plural = \"deer\"", "plural = \"villagers\"");
        files.insert(&format!("{ASH}/data/creatures/villager.toml"), &villager);
        files
    }

    fn load(files: &ContentFiles) -> Result<Content, String> {
        Content::load(files).map_err(|e| e.to_string())
    }

    #[test]
    fn the_showcase_is_the_test_campaign_and_owns_its_world() {
        let content = Content::builtin().expect("content");
        let ids: Vec<_> = content
            .campaigns()
            .iter()
            .map(|c| (c.id.as_str(), c.kind))
            .collect();
        assert_eq!(
            ids,
            [
                ("ascmearc", CampaignKind::Campaign),
                ("showcase", CampaignKind::Test)
            ]
        );
        for scenario in content.scenarios() {
            let owner = if ["ascmearc", "ceaptun", "cealdhlaw", "fennmynster", "fyrdelf"]
                .contains(&scenario.id.as_str())
            {
                "ascmearc"
            } else {
                "showcase"
            };
            assert_eq!(scenario.campaign.as_deref(), Some(owner), "{}", scenario.id);
        }
        assert_eq!(content.level_campaign("ascmearc"), Some("ascmearc"));
        assert_eq!(content.level_campaign("ceaptun"), Some("ascmearc"));
        for level in ["hub", "overworld", "vast", "depths:3", "depths"] {
            assert_eq!(content.level_campaign(level), Some("showcase"), "{level}");
        }
        assert_eq!(content.level_campaign("local:49,23"), None);
        let campaign_of = |id: &str| {
            let c = content.creature_id(id).expect(id);
            content.creature(c).campaign.clone()
        };
        for own in ["hermit", "gardener", "bard"] {
            assert_eq!(campaign_of(own).as_deref(), Some("showcase"), "{own}");
        }
        for shared in ["player", "shopkeeper", "deer", "kobold", "rat"] {
            assert_eq!(campaign_of(shared), None, "{shared}");
        }
        // The showcase's types are numbered last, after every shared one and the real
        // campaigns' own.
        let first_own = content
            .creature_defs()
            .iter()
            .position(|c| c.campaign.as_deref() == Some("showcase"));
        assert_eq!(first_own, Some(content.creature_defs().len() - 3));
        let first_campaign = content
            .creature_defs()
            .iter()
            .position(|c| c.campaign.is_some());
        assert!(
            (content.creature_defs()[first_campaign.expect("a campaign type")..].iter())
                .all(|c| c.campaign.is_some())
        );
        assert!(
            content
                .quests()
                .iter()
                .all(|q| matches!(q.campaign.as_deref(), Some("showcase" | "ascmearc")))
        );
        let speakers: Vec<_> = (content.dialogues().iter().enumerate())
            .map(|(i, d)| {
                let name = content.speaker_name(super::super::DialogueId(i as u16));
                (name, d.campaign.as_deref())
            })
            .collect();
        assert!(
            speakers.contains(&("shopkeeper".into(), None)),
            "{speakers:?}"
        );
        assert!(
            speakers.contains(&("old hermit".into(), Some("showcase"))),
            "{speakers:?}"
        );
        assert!(content.overworld_of("showcase").is_some());
        assert!(content.overworld_of("").is_none());
    }

    #[test]
    fn a_second_campaign_numbers_its_types_before_the_test_world_and_runs_apart() {
        let files = with_ashland();
        let content = Arc::new(Content::load(&files).expect("loads"));
        let order: Vec<_> = content.campaigns().iter().map(|c| c.id.as_str()).collect();
        assert_eq!(order, ["ashland", "showcase"]);
        // Shared types, then the Ashland's, then the showcase's.
        let index = |id: &str| content.creature_id(id).expect(id).0;
        assert!(index("shopkeeper") < index("villager"));
        assert!(index("villager") < index("bard") && index("villager") < index("hermit"));
        assert_eq!(
            content.scenario("ashland").unwrap().campaign.as_deref(),
            Some("ashland")
        );
        let world = World::new(content.clone(), "ashland", None).expect("starts");
        assert_eq!(world.campaign(), "ashland");
        assert_eq!(world.fingerprint(), content.fingerprint_for("ashland"));
        let showcase = World::new(content.clone(), "showcase", None).expect("starts");
        assert_eq!(showcase.campaign(), "showcase");

        // Without the test world (a release build), the Ashland's numbers and fingerprint are
        // the same; the showcase's fingerprint is not, its types having moved.
        let mut release = files.clone();
        release.remove("campaigns/showcase/");
        let release = Content::load(&release).expect("loads without the showcase");
        // A run that names no scenario: the showcase with the test world, else the first
        // campaign's start.
        assert_eq!(content.default_scenario(), Some("showcase"));
        assert_eq!(release.default_scenario(), Some("ashland"));
        assert_eq!(
            release.fingerprint_for("ashland"),
            content.fingerprint_for("ashland")
        );
        assert_eq!(
            release.creature_id("villager"),
            content.creature_id("villager")
        );
        let builtin = Content::builtin().expect("content");
        assert_ne!(
            builtin.fingerprint_for("showcase"),
            content.fingerprint_for("showcase")
        );
        // A save of the Ashland loads in the build without the test world.
        let bytes = crate::save::encode(&world);
        let restored = crate::save::decode(&bytes, Arc::new(release)).expect("loads");
        assert_eq!(restored.state_hash(), world.state_hash());
    }

    #[test]
    fn campaigns_never_link_to_each_other() {
        let refused = |path: &str, text: &str, expect: &str| {
            let mut files = with_ashland();
            files.insert(path, text);
            let err = load(&files).expect_err(path);
            assert!(err.contains(expect), "{path}: {err} (expected {expect:?})");
        };
        let town_file = format!("{ASH}/levels/town.toml");
        refused(
            &town_file,
            &town("\">\" = { terrain = \"stairs_down\", to = \"hub\" }\n").replace("..v", ".>v"),
            "names level `hub`, a level of campaign `showcase`",
        );
        refused(
            &town_file,
            &town("").replace("creature = \"villager\"", "creature = \"hermit\""),
            "names creature `hermit`, a creature of campaign `showcase`",
        );
        refused(
            &format!("{ASH}/scenarios/arrival.toml"),
            "name = \"A\"\nstart_level = \"depths:1\"\n",
            "names level `depths:1`, a level of campaign `showcase`",
        );
        refused(
            &format!("{ASH}/data/dialogue/hermit.toml"),
            "[[dialogue]]\ncreature = \"hermit\"\nlines = [\"Hm.\"]\n",
            "already one (also in campaigns/ashland/data/dialogue/hermit.toml)",
        );
        refused(
            &format!("{ASH}/data/dungeons/barrow.toml"),
            "[[dungeon]]\nid = \"barrow\"\ngenerator = \"rooms\"\ndepth = 2\nsize = [80, 21]\n\
             creatures = [\"bard\"]\n",
            "names creature `bard`, a creature of campaign `showcase`",
        );
        refused(
            &format!("{ASH}/data/quests/rats.toml"),
            "[[quest]]\nid = \"far_rats\"\nname = \"Rats\"\ngiver = \"gardener\"\ntype = \"kill\"\n\
             target = \"rat\"\noffer = \"O.\"\naccept = \"A.\"\nreminder = \"R.\"\ndone = \"D.\"\n",
            "a speaker of campaign `showcase`",
        );
        // A shared file names shared types only.
        let mut files = with_ashland();
        let shops = files.get("data/shops.toml").expect("shops").to_string();
        let keeper = shops.replacen("creature = \"shopkeeper\"", "creature = \"villager\"", 1);
        assert_ne!(keeper, shops);
        files.insert("data/shops.toml", &keeper);
        let err = load(&files).expect_err("a shared keeper");
        assert!(err.contains("a creature of campaign `ashland`"), "{err}");
    }

    #[test]
    fn ids_are_unique_across_campaigns_and_names_must_match() {
        let clash = |path: &str, text: &str, expect: &str| {
            let mut files = with_ashland();
            files.insert(path, text);
            let err = load(&files).expect_err(path);
            assert!(err.contains(expect), "{path}: {err} (expected {expect:?})");
        };
        clash(
            &format!("{ASH}/levels/hub.toml"),
            &town(""),
            "duplicate level id `hub` (also in campaigns/ashland/levels/hub.toml)",
        );
        clash(
            &format!("{ASH}/scenarios/arena.toml"),
            "name = \"A\"\nstart_level = \"town\"\n",
            "duplicate scenario id `arena`",
        );
        let deer = ContentFiles::builtin()
            .get("data/creatures/deer.toml")
            .unwrap()
            .to_string();
        clash(
            &format!("{ASH}/data/creatures/deer.toml"),
            &deer,
            "duplicate creature id `deer` (also in data/creatures/deer.toml)",
        );
        clash(
            &format!("{ASH}/data/quests/rats.toml"),
            "[[quest]]\nid = \"rats\"\nname = \"Rats\"\ngiver = \"shopkeeper\"\ntype = \"kill\"\n\
             target = \"rat\"\noffer = \"O.\"\naccept = \"A.\"\nreminder = \"R.\"\ndone = \"D.\"\n",
            "duplicate quest id `rats`",
        );
        clash(
            &format!("{ASH}/data/dungeons/depths.toml"),
            "[[dungeon]]\nid = \"depths\"\ngenerator = \"rooms\"\ndepth = 2\nsize = [80, 21]\n",
            "duplicate dungeon id `depths`",
        );
        clash(
            &format!("{ASH}/campaign.toml"),
            "id = \"ash\"\nname = \"A\"\nstart = \"ashland\"\n",
            "id `ash` is not the directory's name `ashland`",
        );
        clash(
            &format!("{ASH}/campaign.toml"),
            "id = \"ashland\"\nname = \"A\"\nstart = \"showcase\"\n",
            "start scenario `showcase` is not the campaign's own",
        );
    }

    #[test]
    fn each_campaign_has_one_overworld_of_its_own() {
        let overworld = |name: &str| {
            format!(
                "name = \"{name}\"\nkind = \"overworld\"\nmap = '''\n...\n.@.\n...\n'''\n\
                 [legend]\n\".\" = \"plains\"\n\"@\" = {{ terrain = \"plains\", spawn = \"player\" }}\n"
            )
        };
        let mut files = with_ashland();
        files.insert(&format!("{ASH}/levels/ash.toml"), &overworld("Ash"));
        let content = Content::load(&files).expect("one overworld a campaign");
        assert_eq!(
            content.overworld_of("ashland").map(|l| l.id.as_str()),
            Some("ash")
        );
        assert_eq!(
            content.overworld_of("showcase").map(|l| l.id.as_str()),
            Some("overworld")
        );
        // `local:0,0` is a plains cell of the Ashland's 3×3 map, and out of none of the showcase's.
        assert!(content.local_area("ashland", "local:0,0").is_some());
        assert!(content.has_level("ashland", "local:2,2"));
        assert!(!content.has_level("ashland", "local:3,3"));
        assert!(content.has_level("showcase", "local:49,23"));
        assert!(!content.has_level("ashland", "local:49,23"));
        files.insert(&format!("{ASH}/levels/cinder.toml"), &overworld("Cinder"));
        let err = load(&files).expect_err("two");
        assert!(err.contains("only one level of kind `overworld`"), "{err}");
    }
}

/// A release build's content (`--no-default-features`, #331): the test world is not in it. Run
/// by CI's release-configuration job (`cargo test --no-default-features -p warlock-core
/// release_content`); a default build leaves it out.
#[cfg(all(test, not(feature = "showcase")))]
mod release {
    use super::*;

    #[test]
    fn release_content() {
        let content = Content::builtin().expect("a release build's content loads");
        for c in content.campaigns() {
            assert_eq!(c.kind, CampaignKind::Campaign, "test campaign `{}`", c.id);
        }
        assert!(content.campaign("showcase").is_none());
        assert!(content.scenario("showcase").is_none());
        // Whatever names a campaign names a playable one; scenarios and hand-made levels belong
        // to one (nothing is left outside `campaigns/`).
        let playable = |c: Option<&str>| {
            c.is_none_or(|id| {
                (content.campaign(id)).is_some_and(|c| c.kind == CampaignKind::Campaign)
            })
        };
        for s in content.scenarios() {
            let campaign = s.campaign.as_deref();
            assert!(
                campaign.is_some() && playable(campaign),
                "scenario {}",
                s.id
            );
        }
        for l in content.levels() {
            let campaign = content.level_campaign(&l.id);
            assert!(campaign.is_some() && playable(campaign), "level {}", l.id);
        }
        for d in content.dungeons() {
            assert!(playable(d.campaign.as_deref()), "dungeon {}", d.id);
        }
        for q in content.quests() {
            assert!(playable(q.campaign.as_deref()), "quest {}", q.id);
        }
        for d in content.dialogues() {
            assert!(playable(d.campaign.as_deref()), "dialogue {:?}", d.speaker);
        }
        for c in content.creature_defs() {
            assert!(playable(c.campaign.as_deref()), "creature {}", c.id);
        }
        for v in content.vaults() {
            assert!(playable(v.campaign.as_deref()), "vault {}", v.template.id);
        }
        // A run that names no scenario starts the first campaign's start.
        assert_eq!(
            content.default_scenario(),
            content.campaigns().first().map(|c| c.start.as_str())
        );
    }
}
