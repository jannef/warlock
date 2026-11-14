//! The profane gifts (docs/design/profane-gifts.md §6; `docs/dev/profane-gifts.md`): the
//! permanent changes to the body a layer boss's corpse draws, one file each in `data/profane/`.
//!
//! A gift is a talent: each is registered in the [`TalentBook`] as a talent of the `profane` tree,
//! `granted` (never picked) and a `flaw` when it is a bane, so what it does is read through
//! `TalentMods` where its rule lives, and a save holds it in `Actor::boons` as any talent given
//! for life. [`ProfaneTable`] adds what a talent lacks: the kind, the tier and weight a draw
//! reads, the lines, and the relations (`excludes`, made symmetric here; `replaces`; `undoes`).
//!
//! The checks of §6.5 that need the whole table in place (each tier holding a gift of each kind)
//! or the signatures (`only`, a creature's `gives`) come with the phases that land them.

use super::talent::{Needs, TalentBook, TalentDef, TalentEffect, TalentId};
use super::{Content, ContentError};
use serde::Deserialize;
use std::collections::BTreeMap;

/// What a gift is to its holder: the wiki's grouping and the Talents page's mark (a bane is a
/// flaw).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GiftKind {
    Boon,
    Mixed,
    Bane,
}

impl GiftKind {
    /// The id in data and dumps: `boon`, `mixed`, `bane`.
    pub const fn id(self) -> &'static str {
        match self {
            GiftKind::Boon => "boon",
            GiftKind::Mixed => "mixed",
            GiftKind::Bane => "bane",
        }
    }
}

/// The table a meal's roll draws a gift from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GiftTier {
    Lesser,
    Greater,
    Dire,
}

impl GiftTier {
    pub const ALL: [GiftTier; 3] = [GiftTier::Lesser, GiftTier::Greater, GiftTier::Dire];

    /// The id in data and dumps: `lesser`, `greater`, `dire`.
    pub const fn id(self) -> &'static str {
        match self {
            GiftTier::Lesser => "lesser",
            GiftTier::Greater => "greater",
            GiftTier::Dire => "dire",
        }
    }
}

/// A profane gift: its talent and what the table adds to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GiftDef {
    /// The talent it is (name, description, effects).
    pub talent: TalentId,
    pub id: String,
    pub kind: GiftKind,
    pub tier: GiftTier,
    /// Its weight within its tier, 1 to 100.
    pub weight: u16,
    /// Said as it takes, before "You are <name>."
    pub line: String,
    /// Said when another gift's `replaces` or `undoes` takes it.
    pub undone: Option<String>,
    /// Said in place of `line` when its `undoes` takes a gift held.
    pub instead: Option<String>,
    /// The gifts it cannot be held with (symmetric).
    pub excludes: Vec<TalentId>,
    /// The gifts it takes away as it is taken.
    pub replaces: Vec<TalentId>,
    /// The gifts it takes away instead of being taken, when one is held.
    pub undoes: Vec<TalentId>,
    /// The races it is never drawn for, by id.
    pub not_for: Vec<String>,
}

/// Every gift, in path order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProfaneTable {
    gifts: Vec<GiftDef>,
    by_talent: BTreeMap<TalentId, usize>,
}

impl ProfaneTable {
    pub fn is_empty(&self) -> bool {
        self.gifts.is_empty()
    }

    /// Every gift, in path order.
    pub fn iter(&self) -> impl Iterator<Item = &GiftDef> {
        self.gifts.iter()
    }

    /// The gift talent `talent` is, if it is one.
    pub fn gift(&self, talent: TalentId) -> Option<&GiftDef> {
        self.by_talent.get(&talent).map(|&i| &self.gifts[i])
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GiftFile {
    #[serde(default)]
    gift: Vec<GiftSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GiftSpec {
    id: String,
    name: String,
    kind: GiftKind,
    tier: GiftTier,
    weight: u16,
    description: String,
    line: String,
    undone: Option<String>,
    instead: Option<String>,
    #[serde(default)]
    effects: Vec<TalentEffect>,
    #[serde(default)]
    excludes: Vec<String>,
    #[serde(default)]
    replaces: Vec<String>,
    #[serde(default)]
    undoes: Vec<String>,
    #[serde(default)]
    not_for: Vec<String>,
}

/// The tree the gifts are talents of.
const TREE: &str = "profane";

/// Whether a gift may not carry `effect`: one handed out once that could not be taken back
/// cleanly, or a class's or a pick's business (§6.4). `stat` is the one hand-out allowed.
fn forbidden(effect: &TalentEffect) -> bool {
    effect.is_handed_down()
        || matches!(
            effect,
            TalentEffect::MaxHpPerLevels { .. }
                | TalentEffect::MaxManaPerLevels { .. }
                | TalentEffect::WeaponRanks { .. }
                | TalentEffect::WeaponMarks { .. }
                | TalentEffect::ExtraPick { .. }
                | TalentEffect::SkillRolls { .. }
                | TalentEffect::TakeUpSkills { .. }
                | TalentEffect::GrantsResource { .. }
                | TalentEffect::Companion
                | TalentEffect::Familiar
                | TalentEffect::XpPercent { .. }
        )
}

/// Reads `data/profane/` into the table and registers each gift as a talent of the `profane`
/// tree in `book`. Content without talents has no gifts.
pub(super) fn parse<'a>(
    files: impl IntoIterator<Item = (&'a str, &'a str)>,
    book: &mut TalentBook,
    content: &Content,
) -> Result<ProfaneTable, ContentError> {
    let mut specs: Vec<(&str, GiftSpec)> = Vec::new();
    for (file, text) in files {
        let parsed: GiftFile =
            toml::from_str(text).map_err(|e| ContentError::new(file, e.to_string()))?;
        specs.extend(parsed.gift.into_iter().map(|g| (file, g)));
    }
    let mut table = ProfaneTable::default();
    if specs.is_empty() || book.is_empty() {
        return Ok(table);
    }
    let tree = (book.tree_id(TREE)).ok_or_else(|| {
        ContentError::new(
            specs[0].0,
            "gifts need the `profane` tree in data/talents.toml",
        )
    })?;
    // Every gift a talent first, so a relation may name one defined after it.
    for (file, spec) in &specs {
        let err = |m: String| ContentError::new(file, format!("gift `{}`: {m}", spec.id));
        if spec.id.is_empty() || spec.id.contains(':') || spec.id.contains(char::is_whitespace) {
            return Err(err("no gift id".into()));
        }
        if book.id(&spec.id).is_some() {
            return Err(err("its id is a talent's already".into()));
        }
        if !(1..=100).contains(&spec.weight) {
            return Err(err(format!("weight {} is not 1 to 100", spec.weight)));
        }
        for effect in &spec.effects {
            effect.check(content, None).map_err(err)?;
            if forbidden(effect) {
                return Err(err(format!("a gift may not carry {effect:?}")));
            }
        }
        if let Some(race) = (spec.not_for.iter())
            .find(|r| !content.races().list.is_empty() && content.race(r).is_none())
        {
            return Err(err(format!("`not_for` names an unknown race `{race}`")));
        }
        let talent = book.add(TalentDef {
            id: spec.id.clone(),
            name: spec.name.clone(),
            tree,
            choice: None,
            description: spec.description.clone(),
            needs: Needs::default(),
            effects: spec.effects.clone(),
            effects_while: Vec::new(),
            systems: Vec::new(),
            granted: true,
            flaw: spec.kind == GiftKind::Bane,
            resource: None,
            alignment: None,
        });
        table.by_talent.insert(talent, table.gifts.len());
        table.gifts.push(GiftDef {
            talent,
            id: spec.id.clone(),
            kind: spec.kind,
            tier: spec.tier,
            weight: spec.weight,
            line: spec.line.clone(),
            undone: spec.undone.clone(),
            instead: spec.instead.clone(),
            excludes: Vec::new(),
            replaces: Vec::new(),
            undoes: Vec::new(),
            not_for: spec.not_for.clone(),
        });
    }
    // The relations, each naming another gift.
    for (i, (file, spec)) in specs.iter().enumerate() {
        let err = |m: String| ContentError::new(file, format!("gift `{}`: {m}", spec.id));
        let resolve = |field: &str, ids: &[String]| -> Result<Vec<TalentId>, ContentError> {
            let mut out = Vec::new();
            for id in ids {
                let gift = (book.id(id))
                    .filter(|t| table.by_talent.contains_key(t))
                    .ok_or_else(|| err(format!("`{field}` names no gift `{id}`")))?;
                if *id == spec.id {
                    return Err(err(format!("`{field}` names the gift itself")));
                }
                if !out.contains(&gift) {
                    out.push(gift);
                }
            }
            Ok(out)
        };
        let excludes = resolve("excludes", &spec.excludes)?;
        let replaces = resolve("replaces", &spec.replaces)?;
        let undoes = resolve("undoes", &spec.undoes)?;
        if !undoes.is_empty() && spec.instead.is_none() {
            return Err(err("`undoes` asks for an `instead` line".into()));
        }
        for taken in replaces.iter().chain(&undoes) {
            let other = &table.gifts[table.by_talent[taken]];
            if other.undone.is_none() {
                return Err(err(format!(
                    "takes `{}` away, which has no `undone` line",
                    other.id
                )));
            }
        }
        let me = table.gifts[i].talent;
        for other in &excludes {
            let j = table.by_talent[other];
            if !table.gifts[j].excludes.contains(&me) {
                table.gifts[j].excludes.push(me);
            }
        }
        let gift = &mut table.gifts[i];
        for other in excludes {
            if !gift.excludes.contains(&other) {
                gift.excludes.push(other);
            }
        }
        gift.replaces = replaces;
        gift.undoes = undoes;
    }
    check_acyclic(&table, &specs)?;
    Ok(table)
}

/// `replaces` and `undoes` together never lead from a gift back to itself.
fn check_acyclic(table: &ProfaneTable, specs: &[(&str, GiftSpec)]) -> Result<(), ContentError> {
    for (start, gift) in table.gifts.iter().enumerate() {
        let mut seen = vec![false; table.gifts.len()];
        let mut stack: Vec<usize> = Vec::new();
        let next = |g: &GiftDef| -> Vec<usize> {
            (g.replaces.iter().chain(&g.undoes))
                .map(|t| table.by_talent[t])
                .collect()
        };
        stack.extend(next(gift));
        while let Some(i) = stack.pop() {
            if i == start {
                return Err(ContentError::new(
                    specs[start].0,
                    format!(
                        "gift `{}`: `replaces` and `undoes` lead back to it",
                        gift.id
                    ),
                ));
            }
            if !std::mem::replace(&mut seen[i], true) {
                stack.extend(next(&table.gifts[i]));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};

    /// The built-in content with `data/profane/` replaced by `gifts`.
    fn load(gifts: &[(&str, &str)]) -> Result<Content, String> {
        let mut files = ContentFiles::builtin();
        files.remove("data/profane/");
        for (name, text) in gifts {
            files.insert(&format!("data/profane/{name}.toml"), text);
        }
        Content::load(&files).map_err(|e| e.to_string())
    }

    fn gift(id: &str, extra: &str) -> String {
        format!(
            "[[gift]]\nid = \"{id}\"\nname = \"{id}\"\nkind = \"boon\"\ntier = \"lesser\"\n\
             weight = 5\ndescription = \"d\"\nline = \"l\"\nundone = \"u\"\n{extra}"
        )
    }

    #[test]
    fn the_gifts_are_talents_of_the_profane_tree_and_excludes_is_symmetric() {
        let content = Content::builtin().expect("content");
        let book = content.talents();
        let id = |name: &str| book.id(name).expect(name);
        let profane = content.profane();
        let thick = profane.gift(id("thick_blooded")).expect("a gift");
        assert_eq!(thick.excludes, vec![id("paper_skinned")]);
        let thin = profane.gift(id("paper_skinned")).expect("a gift");
        assert_eq!(thin.excludes, vec![id("thick_blooded")]);
        let def = book.get(id("paper_skinned"));
        assert!(def.granted && def.flaw);
        assert_eq!(book.tree(def.tree).id, "profane");
        assert!(!book.get(id("oak_hearted")).flaw);
        assert!(profane.gift(id("hardy")).is_none());
    }

    #[test]
    fn the_loader_refuses_a_bad_gift() {
        let refused = |gifts: &[(&str, &str)], says: &str| {
            let e = load(gifts).expect_err(says);
            assert!(e.contains(says), "{e}");
        };
        refused(&[("hardy", &gift("hardy", ""))], "a talent's already");
        refused(
            &[("a", &gift("a", "").replace("weight = 5", "weight = 0"))],
            "not 1 to 100",
        );
        refused(
            &[("a", &gift("a", "excludes = [\"b\"]\n"))],
            "names no gift `b`",
        );
        refused(
            &[("a", &gift("a", "not_for = [\"orcs\"]\n"))],
            "unknown race",
        );
        refused(
            &[(
                "a",
                &gift("a", "effects = [{ kind = \"extra_pick\", level = 3 }]\n"),
            )],
            "may not carry",
        );
        refused(
            &[("a", &gift("a", "undoes = [\"a\"]\ninstead = \"i\"\n"))],
            "itself",
        );
        refused(
            &[
                ("a", &gift("a", "undoes = [\"b\"]\n")),
                ("b", &gift("b", "")),
            ],
            "`instead`",
        );
        let undone_less = gift("b", "").replace("undone = \"u\"\n", "");
        refused(
            &[
                ("a", &gift("a", "replaces = [\"b\"]\n")),
                ("b", &undone_less),
            ],
            "no `undone`",
        );
        refused(
            &[
                ("a", &gift("a", "replaces = [\"b\"]\n")),
                ("b", &gift("b", "replaces = [\"a\"]\n")),
            ],
            "lead back",
        );
        assert!(
            load(&[
                ("a", &gift("a", "replaces = [\"b\"]\n")),
                ("b", &gift("b", ""))
            ])
            .is_ok()
        );
    }
}
