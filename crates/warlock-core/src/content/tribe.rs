//! Tribes (`assets/data/tribes/<id>.toml` and the campaigns' `data/tribes/`, spec §8 "Quests",
//! docs/design/adventurers-guild.md §6): named groups of creature types a kill quest may ask for
//! as a whole ("Kill 100 goblins.").
//!
//! A tribe is defined *from* factions, so it stays in step with them, with exceptions and
//! additions where the faction is wrong for the word:
//!
//! ```toml
//! [[tribe]]
//! id = "goblins"
//! name = "goblin"              # one of them: "a goblin"
//! plural = "goblins"           # "Kill 100 goblins."
//! factions = ["goblins"]       # every creature type of these factions...
//! except = ["ogre", "troll"]   # ...but these
//! creatures = []               # ...and these besides
//! except_note = "Not the ogres, mind, nor the trolls."   # optional, said after the members
//! count = 100                  # optional: the guild's count for it
//! ```
//!
//! Membership is by creature **type**: an actor's `faction_override` (an outraged keeper) moves
//! it neither into nor out of a tribe. A campaign's creature types join a shared tribe through
//! its `factions`; `except` and `creatures` name shared types in a shared file and the
//! campaign's own or shared types in a campaign's (`Content::check_campaigns`). A tribe with no
//! member is a content error, as is an `except` that names no member or a `creatures` entry that
//! is one already.

use super::creature::CreatureId;
use super::dialogue::{MAX_REPLY, check_lines, is_id};
use super::faction::FactionId;
use super::{Content, ContentError};
use serde::Deserialize;

/// The longest tribe name or plural, in characters (it is said in an objective).
pub const MAX_TRIBE_NAME: usize = 24;
/// The most kills a tribe quest, or a tribe's `count`, asks for.
pub const MAX_TRIBE_COUNT: u32 = 999;

/// Index of a tribe in [`Content::tribes`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TribeId(pub u16);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TribeFile {
    #[serde(default)]
    tribe: Vec<TribeSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TribeSpec {
    id: String,
    name: String,
    plural: String,
    #[serde(default)]
    factions: Vec<String>,
    #[serde(default)]
    except: Vec<String>,
    #[serde(default)]
    creatures: Vec<String>,
    except_note: Option<String>,
    count: Option<u32>,
}

/// A tribe: a name and the creature types that belong to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TribeDef {
    /// Its id in quest targets (`tribe:goblins`).
    pub id: String,
    /// One of them: "goblin".
    pub name: String,
    /// Several: "goblins".
    pub plural: String,
    /// The factions whose types it is drawn from.
    pub factions: Vec<FactionId>,
    /// Its member types, in creature table order.
    pub members: Vec<CreatureId>,
    /// The types named in `except` (for the wiki and the texts).
    pub except: Vec<CreatureId>,
    /// The types named in `creatures`: members besides its factions'.
    pub added: Vec<CreatureId>,
    /// What is said after the members, about those left out ("Not the ogres, mind.").
    pub except_note: Option<String>,
    /// How many of them the guild asks for, if it says.
    pub count: Option<u32>,
    /// The campaign whose `data/tribes/` it is in; `None` for shared data.
    pub campaign: Option<String>,
}

impl TribeDef {
    /// Whether creature type `creature` belongs to it.
    pub fn has(&self, creature: CreatureId) -> bool {
        self.members.contains(&creature)
    }

    /// Its members a run of campaign `campaign` can meet: the shared types and that campaign's
    /// (every member when `campaign` is `None`, a run outside every campaign).
    pub fn members_in<'a>(
        &'a self,
        content: &'a Content,
        campaign: Option<&'a str>,
    ) -> impl Iterator<Item = CreatureId> + 'a {
        self.members.iter().copied().filter(move |&c| {
            let theirs = content.creature(c).campaign.as_deref();
            campaign.is_none() || theirs.is_none() || theirs == campaign
        })
    }
}

impl Content {
    /// The tribes (`data/tribes/`, then each campaign's), in table order.
    pub fn tribes(&self) -> &[TribeDef] {
        &self.tribes
    }

    pub fn tribe(&self, id: TribeId) -> &TribeDef {
        &self.tribes[usize::from(id.0)]
    }

    /// The tribe with id `id`.
    pub fn tribe_id(&self, id: &str) -> Option<TribeId> {
        (self.tribes.iter().position(|t| t.id == id)).map(|i| TribeId(i as u16))
    }
}

/// Parses the tribe table (`data/tribes/` and each campaign's; [`super::ContentFiles::table`])
/// against the creatures and factions loaded already.
pub(super) fn parse<'a>(
    table: impl Iterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<Vec<TribeDef>, ContentError> {
    let mut tribes: Vec<TribeDef> = Vec::new();
    let mut seen: std::collections::BTreeMap<String, &str> = std::collections::BTreeMap::new();
    for (file, text) in table {
        let err = |m: String| ContentError::new(file, m);
        let parsed: TribeFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        for spec in parsed.tribe {
            let id = spec.id.clone();
            if let Some(first) = seen.insert(spec.id.clone(), file) {
                return Err(super::duplicate_id("tribe", &spec.id, file, first));
            }
            let mut tribe =
                resolve(spec, content).map_err(|m| err(format!("tribe `{id}`: {m}")))?;
            tribe.campaign = super::campaign::of_path(file).map(str::to_string);
            tribes.push(tribe);
            if tribes.len() > usize::from(u16::MAX) {
                return Err(err("too many tribes".into()));
            }
        }
    }
    Ok(tribes)
}

/// One tribe as written, checked against `content`.
fn resolve(spec: TribeSpec, content: &Content) -> Result<TribeDef, String> {
    if !is_id(&spec.id) {
        return Err("ids are lower-case letters, digits, and underscores".into());
    }
    for (field, text) in [("name", &spec.name), ("plural", &spec.plural)] {
        let n = text.chars().count();
        if n == 0 || n > MAX_TRIBE_NAME {
            return Err(format!("`{field}` is 1 to {MAX_TRIBE_NAME} characters"));
        }
    }
    if let Some(note) = &spec.except_note {
        check_lines([note], MAX_REPLY)?;
    }
    if let Some(n) = spec.count
        && !(1..=MAX_TRIBE_COUNT).contains(&n)
    {
        return Err(format!("`count` is 1 to {MAX_TRIBE_COUNT}"));
    }
    let creature = |name: &String| {
        (content.creature_id(name))
            .filter(|_| name != "player")
            .ok_or_else(|| format!("unknown creature `{name}`"))
    };
    let mut factions = Vec::new();
    for name in &spec.factions {
        let faction = (content.factions().id(name)).ok_or(format!("unknown faction `{name}`"))?;
        if factions.contains(&faction) {
            return Err(format!("`factions` names `{name}` twice"));
        }
        factions.push(faction);
    }
    let by_faction = |c: &&super::CreatureDef| c.faction.is_some_and(|f| factions.contains(&f));
    let of_factions: Vec<CreatureId> = (content.creature_defs().iter().enumerate())
        .filter(|(_, c)| by_faction(c) && c.id != "player")
        .map(|(i, _)| CreatureId(i as u16))
        .collect();
    let mut except = Vec::new();
    for name in &spec.except {
        let c = creature(name)?;
        if !of_factions.contains(&c) {
            return Err(format!(
                "`except` names `{name}`, which is of none of its factions"
            ));
        }
        if except.contains(&c) {
            return Err(format!("`except` names `{name}` twice"));
        }
        except.push(c);
    }
    let mut extra = Vec::new();
    for name in &spec.creatures {
        let c = creature(name)?;
        if of_factions.contains(&c) || extra.contains(&c) {
            return Err(format!("`creatures` names `{name}`, a member already"));
        }
        extra.push(c);
    }
    let mut members: Vec<CreatureId> = (of_factions.into_iter())
        .filter(|c| !except.contains(c))
        .chain(extra.iter().copied())
        .collect();
    members.sort();
    if members.is_empty() {
        return Err("it has no member".into());
    }
    Ok(TribeDef {
        id: spec.id,
        name: spec.name,
        plural: spec.plural,
        factions,
        members,
        except,
        added: extra,
        except_note: spec.except_note,
        count: spec.count,
        campaign: None,
    })
}

#[cfg(test)]
mod tests {
    use super::super::ContentFiles;
    use super::*;

    const FACTIONS: &str = r#"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "goblins"
name = "goblins"

[[faction]]
id = "vermin"
name = "vermin"
"#;

    const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "goblin"
name = "goblin"
glyph = "g"
fg = "#00ff00"
faction = "goblins"
weight = 40000

[[creature]]
id = "goblin_chief"
name = "goblin chief"
glyph = "g"
fg = "#00ff00"
faction = "goblins"
weight = 50000

[[creature]]
id = "ogre"
name = "ogre"
glyph = "O"
fg = "#00ff00"
faction = "goblins"
weight = 200000

[[creature]]
id = "rat"
name = "giant rat"
glyph = "r"
fg = "#806040"
faction = "vermin"
weight = 5000
"##;

    fn load(tribes: &[(&str, &str)]) -> Result<Content, ContentError> {
        let mut files = ContentFiles::default();
        files.insert(
            "data/item_classes.toml",
            crate::content::BUILTIN_ITEM_CLASSES,
        );
        files.insert(
            "data/terrain.toml",
            "[[terrain]]\nid = \"floor\"\nname = \"floor\"\nglyph = \".\"\nfg = \"#808080\"\n",
        );
        files.insert("data/factions.toml", FACTIONS);
        files.insert("data/legend.toml", "[legend]\n\".\" = \"floor\"\n");
        files.insert(
            "levels/test.toml",
            "name = \"Test\"\nmap = \"@.\"\n[legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
        );
        files.insert(
            "scenarios/test.toml",
            "name = \"T\"\nstart_level = \"test\"\n",
        );
        files.insert("data/creatures.toml", CREATURES);
        for (path, text) in tribes {
            files.insert(path, text);
        }
        Content::load(&files)
    }

    fn members(content: &Content, id: &str) -> Vec<String> {
        let tribe = content.tribe(content.tribe_id(id).expect("tribe"));
        (tribe.members.iter())
            .map(|&c| content.creature(c).id.clone())
            .collect()
    }

    fn refused(text: &str) -> String {
        let err = load(&[("data/tribes/t.toml", text)]).expect_err("refused");
        err.to_string()
    }

    #[test]
    fn members_come_from_factions_less_except_and_with_creatures() {
        let content = load(&[
            (
                "data/tribes/goblins.toml",
                "[[tribe]]\nid = \"goblins\"\nname = \"goblin\"\nplural = \"goblins\"\n\
                 factions = [\"goblins\"]\nexcept = [\"ogre\"]\n\
                 except_note = \"Not the ogres, mind.\"\ncount = 100\n",
            ),
            (
                "data/tribes/beasts.toml",
                "[[tribe]]\nid = \"beasts\"\nname = \"beast\"\nplural = \"beasts\"\n\
                 factions = [\"vermin\"]\ncreatures = [\"ogre\"]\n",
            ),
            (
                "data/tribes/big.toml",
                "[[tribe]]\nid = \"big\"\nname = \"big one\"\nplural = \"big ones\"\n\
                 creatures = [\"ogre\", \"goblin_chief\"]\n",
            ),
        ])
        .expect("valid");
        assert_eq!(members(&content, "goblins"), ["goblin", "goblin_chief"]);
        assert_eq!(members(&content, "beasts"), ["ogre", "rat"], "table order");
        assert_eq!(members(&content, "big"), ["goblin_chief", "ogre"]);
        let goblins = content.tribe(content.tribe_id("goblins").expect("tribe"));
        assert_eq!(goblins.count, Some(100));
        assert_eq!(goblins.except_note.as_deref(), Some("Not the ogres, mind."));
        let ogre = content.creature_id("ogre").expect("ogre");
        assert!(!goblins.has(ogre));
        assert_eq!(content.tribe_id("nobody"), None);
    }

    #[test]
    fn bad_tribes_are_refused() {
        let tribe = |rest: &str| {
            format!("[[tribe]]\nid = \"t\"\nname = \"thing\"\nplural = \"things\"\n{rest}\n")
        };
        for (rest, says) in [
            ("factions = [\"elves\"]", "unknown faction `elves`"),
            ("creatures = [\"dragon\"]", "unknown creature `dragon`"),
            ("creatures = [\"player\"]", "unknown creature `player`"),
            (
                "factions = [\"goblins\"]\nexcept = [\"wyrm\"]",
                "unknown creature `wyrm`",
            ),
            (
                "factions = [\"goblins\"]\nexcept = [\"rat\"]",
                "of none of its factions",
            ),
            (
                "factions = [\"goblins\"]\ncreatures = [\"goblin\"]",
                "a member already",
            ),
            ("", "it has no member"),
            (
                "factions = [\"vermin\"]\nexcept = [\"rat\"]",
                "it has no member",
            ),
            ("creatures = [\"rat\"]\ncount = 0", "`count` is 1 to 999"),
            ("creatures = [\"rat\"]\ncolour = \"red\"", "unknown field"),
        ] {
            let error = refused(&tribe(rest));
            assert!(error.contains(says), "{rest:?}: {error}");
        }
        let twice = format!(
            "{}{}",
            tribe("creatures = [\"rat\"]"),
            tribe("creatures = [\"rat\"]")
        );
        assert!(refused(&twice).contains("duplicate tribe id `t`"));
        let error = refused("[[tribe]]\nid = \"T\"\nname = \"x\"\nplural = \"xs\"\n");
        assert!(error.contains("ids are lower-case"), "{error}");
    }

    #[test]
    fn the_builtin_tribes_are_the_seven() {
        let content = Content::builtin().expect("content");
        let ids: Vec<_> = content.tribes().iter().map(|t| t.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "duergar", "fire", "goblins", "kobolds", "marsh", "undead", "vermin"
            ]
        );
        assert_eq!(
            members(&content, "goblins"),
            ["goblin", "goblin_archer", "goblin_chief"],
            "the ogre and the troll left out"
        );
        assert_eq!(
            members(&content, "kobolds"),
            ["kobold", "kobold_archer", "kobold_shaman"]
        );
        let vermin = members(&content, "vermin");
        assert!(!vermin.contains(&"basilisk".to_string()), "{vermin:?}");
        assert!(vermin.contains(&"rat".to_string()), "{vermin:?}");
        assert!(!members(&content, "duergar").contains(&"duergar_trader".to_string()));
        assert!(members(&content, "marsh").contains(&"bog_hag".to_string()));
        assert!(members(&content, "fire").contains(&"fire_drake".to_string()));
        for tribe in content.tribes() {
            assert!(!tribe.members.is_empty(), "{}", tribe.id);
        }
    }
}
