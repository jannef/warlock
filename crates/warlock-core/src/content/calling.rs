//! Callings (a campaign's `data/callings/`, one file per calling, spec §8 "Prestige classes"):
//! the **door** into a prestige class, never the deed. A calling names the prestige class, the
//! base classes it is open to (`from`), the total level it asks (`level`, at least each base's
//! `max_level`), and what is said when the character is called. How a world makes the calling
//! (a dialogue word said to the right creature, a dev action) is that world's business, and
//! there may be several ways to one door. A base class is named by at most one calling of a
//! campaign: two doors for one class would need a choice, which prestige classes are not.
//!
//! A calling outside every campaign (`data/callings/`) belongs to none: test fixtures use it,
//! and a run of any campaign may walk through it.

use super::{Content, ContentError, PrestigeDef, duplicate_id};
use serde::Deserialize;
use std::collections::BTreeMap;

/// The longest `message`, in characters.
pub const MAX_MESSAGE: usize = 160;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CallingFile {
    #[serde(default)]
    calling: Vec<CallingSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CallingSpec {
    class: String,
    from: Vec<String>,
    level: u8,
    message: String,
}

/// A calling (spec §8 "Prestige classes"): its id is its prestige class's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallingDef {
    /// The prestige class it opens (`data/prestige/`), and its own id.
    pub class: String,
    /// The base classes it is open to, by id, in file order.
    pub from: Vec<String>,
    /// The total level it asks: at least every `from` class's `max_level`.
    pub level: u8,
    /// What the character is told as they are called.
    pub message: String,
    /// The campaign whose table it is in (`None`: outside every campaign, a fixture's).
    pub campaign: Option<String>,
}

impl CallingDef {
    /// Whether it is open to a character of base class `class`.
    pub fn is_from(&self, class: &str) -> bool {
        self.from.iter().any(|f| f == class)
    }
}

/// Parses the callings table (`table`: its files in order) against `content`'s classes and
/// prestige classes, which are loaded by then.
pub(super) fn parse<'a>(
    table: impl Iterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<Vec<CallingDef>, ContentError> {
    let mut callings: Vec<CallingDef> = Vec::new();
    let mut seen: BTreeMap<(Option<String>, String), &str> = BTreeMap::new();
    for (path, text) in table {
        let parsed: CallingFile =
            toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
        for spec in parsed.calling {
            let campaign = super::campaign::of_path(path).map(str::to_string);
            let key = (campaign.clone(), spec.class.clone());
            if let Some(first) = seen.insert(key, path) {
                return Err(duplicate_id("calling", &spec.class, path, first));
            }
            let class = spec.class.clone();
            let def = resolve(spec, campaign, content, &callings)
                .map_err(|m| ContentError::new(path, format!("calling `{class}`: {m}")))?;
            callings.push(def);
        }
    }
    Ok(callings)
}

fn resolve(
    spec: CallingSpec,
    campaign: Option<String>,
    content: &Content,
    before: &[CallingDef],
) -> Result<CallingDef, String> {
    let prestige: &PrestigeDef = (content.prestige(&spec.class))
        .ok_or_else(|| format!("`class`: unknown prestige class `{}`", spec.class))?;
    if spec.from.is_empty() {
        return Err("`from` names at least one class".into());
    }
    for from in &spec.from {
        let class =
            (content.class(from)).ok_or_else(|| format!("`from`: unknown class `{from}`"))?;
        if spec.from.iter().filter(|f| *f == from).count() > 1 {
            return Err(format!("`from`: `{from}` twice"));
        }
        if spec.level < class.max_level {
            return Err(format!(
                "`level` {} is below `{from}`'s highest level, {}: a prestige level would come \
                 before the class is full",
                spec.level, class.max_level
            ));
        }
        if let Some(other) = (before.iter())
            .filter(|c| c.campaign == campaign)
            .find(|c| c.is_from(from))
        {
            return Err(format!(
                "`from`: `{from}` is named by calling `{}` already (one door a class)",
                other.class
            ));
        }
    }
    let message = spec.message.trim();
    if message.is_empty() || message.chars().count() > MAX_MESSAGE {
        return Err(format!("`message` is 1 to {MAX_MESSAGE} characters"));
    }
    Ok(CallingDef {
        class: prestige.id.clone(),
        from: spec.from,
        level: spec.level,
        message: message.to_string(),
        campaign,
    })
}

impl Content {
    /// The callings (spec §8 "Prestige classes"), in table order.
    pub fn callings(&self) -> &[CallingDef] {
        &self.callings
    }

    /// The calling into prestige class `class` a run of `campaign` may walk through: the
    /// campaign's own, else one outside every campaign.
    pub fn calling(&self, campaign: &str, class: &str) -> Option<&CallingDef> {
        let mine = (self.callings.iter())
            .find(|c| c.class == class && c.campaign.as_deref() == Some(campaign));
        mine.or_else(|| (self.callings.iter()).find(|c| c.class == class && c.campaign.is_none()))
    }

    /// The callings a run of `campaign` may walk through, in table order: the campaign's own
    /// and those outside every campaign.
    pub fn callings_of(&self, campaign: &str) -> impl Iterator<Item = &CallingDef> {
        (self.callings.iter())
            .filter(move |c| c.campaign.is_none() || c.campaign.as_deref() == Some(campaign))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentFiles;

    const HUNTER: &str = "[[prestige]]\nid = \"hunter\"\nname = \"Hunter\"\n\
                          description = \"Sworn.\"\nlevels = 30\nhp = 7\n";

    fn load(calling: &str, at: &str) -> Result<Content, ContentError> {
        let mut files = ContentFiles::builtin();
        files.insert("data/prestige/hunter.toml", HUNTER);
        files.insert(at, calling);
        Content::load(&files)
    }

    #[test]
    fn a_calling_names_its_class_and_who_may_come() {
        let text = "[[calling]]\nclass = \"hunter\"\nfrom = [\"bard\"]\nlevel = 60\n\
                    message = \"You are a hunter.\"\n";
        let content = load(text, "campaigns/showcase/data/callings/hunter.toml").expect("valid");
        let calling = content
            .calling("showcase", "hunter")
            .expect("the showcase's");
        assert_eq!(calling.campaign.as_deref(), Some("showcase"));
        assert!(calling.is_from("bard") && !calling.is_from("thief"));
        assert_eq!(calling.level, 60);
        assert!(
            content.calling("ascmearc", "hunter").is_none(),
            "another campaign's door is shut"
        );
        let loose = load(text, "data/callings/hunter.toml").expect("valid");
        assert!(
            loose.calling("ascmearc", "hunter").is_some(),
            "a calling outside every campaign is any run's"
        );
    }

    #[test]
    fn a_calling_refuses_what_it_cannot_open() {
        let at = "campaigns/showcase/data/callings/hunter.toml";
        for (text, why) in [
            (
                "[[calling]]\nclass = \"ranger\"\nfrom = [\"bard\"]\nlevel = 60\nmessage = \"x\"\n",
                "unknown prestige class `ranger`",
            ),
            (
                "[[calling]]\nclass = \"hunter\"\nfrom = []\nlevel = 60\nmessage = \"x\"\n",
                "at least one class",
            ),
            (
                "[[calling]]\nclass = \"hunter\"\nfrom = [\"skald\"]\nlevel = 60\nmessage = \"x\"\n",
                "unknown class `skald`",
            ),
            (
                "[[calling]]\nclass = \"hunter\"\nfrom = [\"bard\"]\nlevel = 30\nmessage = \"x\"\n",
                "below `bard`'s highest level, 60",
            ),
            (
                "[[calling]]\nclass = \"hunter\"\nfrom = [\"bard\"]\nlevel = 60\nmessage = \"\"\n",
                "`message` is 1 to",
            ),
            (
                "[[calling]]\nclass = \"hunter\"\nfrom = [\"bard\"]\nlevel = 60\nmessage = \"x\"\n\
                 [[calling]]\nclass = \"hunter\"\nfrom = [\"thief\"]\nlevel = 60\nmessage = \"x\"\n",
                "duplicate",
            ),
        ] {
            let err = load(text, at).expect_err(why);
            assert!(err.to_string().contains(why), "{why}: {err}");
        }
        // Two callings, one class: one door a class.
        let mut files = ContentFiles::builtin();
        files.insert("data/prestige/hunter.toml", HUNTER);
        files.insert(
            "data/prestige/skald.toml",
            &HUNTER.replace("hunter", "skald").replace("Hunter", "Skald"),
        );
        files.insert(
            "campaigns/showcase/data/callings/hunter.toml",
            "[[calling]]\nclass = \"hunter\"\nfrom = [\"bard\"]\nlevel = 60\nmessage = \"x\"\n",
        );
        files.insert(
            "campaigns/showcase/data/callings/skald.toml",
            "[[calling]]\nclass = \"skald\"\nfrom = [\"bard\"]\nlevel = 60\nmessage = \"x\"\n",
        );
        let err = Content::load(&files).expect_err("two doors");
        assert!(err.to_string().contains("one door a class"), "{err}");
    }
}
