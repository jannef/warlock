//! Skills (`assets/data/skills.toml`, spec §8 "Skills"): what can be trained, how it trains, and
//! what it gives. Adding a skill is one entry; what an actor has of them is
//! [`crate::skills::Skills`].
//!
//! The file also holds how skills grow at level-up (spec §8 "Skills"): the ladder of dice their
//! marks set ([`Ladder`], `[ladder]`) and the rolls' rules ([`RollRules`], `[rolls]`).

use super::ContentError;
use crate::skills::{Benefit, BenefitSpec, Ladder, MarkRule, RollRules, SkillId, SkillKind};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillFile {
    #[serde(default)]
    ladder: Ladder,
    #[serde(default)]
    rolls: RollRules,
    skill: Vec<SkillSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillSpec {
    id: String,
    name: String,
    description: String,
    kind: SkillKind,
    #[serde(default)]
    mark: Vec<MarkRule>,
    #[serde(default)]
    benefit: Vec<BenefitSpec>,
}

/// A skill, in file order (by [`SkillId`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillDef {
    pub id: String,
    /// Its name, as the Skills screen shows it ("Open lock").
    pub name: String,
    /// One line: what it is about.
    pub description: String,
    pub kind: SkillKind,
    /// When it earns training marks.
    pub marks: Vec<MarkRule>,
    /// Its passive effects, by its value.
    pub benefits: Vec<Benefit>,
}

impl SkillDef {
    /// Whether it has an active use: it is used as an action (`kind = "action"`), so the
    /// abilities menu (#156) lists it. Passive skills and those rules read have none.
    pub fn is_active(&self) -> bool {
        self.kind == SkillKind::Action
    }
}

/// Every skill, and how they grow at level-up.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SkillBook {
    pub list: Vec<SkillDef>,
    /// The dice a level-up roll throws, by the skill's marks.
    pub ladder: Ladder,
    /// How many rolls Intelligence adds, and what a roll costs.
    pub rolls: RollRules,
}

impl SkillBook {
    /// Parses `data/skills.toml`.
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: SkillFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        parsed.ladder.check().map_err(err)?;
        parsed.rolls.check().map_err(err)?;
        if parsed.skill.is_empty() {
            return Err(err("at least one skill is required".into()));
        }
        if parsed.skill.len() > usize::from(u8::MAX) + 1 {
            return Err(err("too many skills (at most 256)".into()));
        }
        let mut list: Vec<SkillDef> = Vec::new();
        for spec in parsed.skill {
            let id = &spec.id;
            if list.iter().any(|s| &s.id == id) {
                return Err(err(format!("duplicate skill id `{id}`")));
            }
            if spec.name.trim().is_empty() {
                return Err(err(format!("skill `{id}`: `name` must not be empty")));
            }
            if let Some(rule) = spec.mark.iter().find(|m| !(1..=100).contains(&m.chance)) {
                return Err(err(format!(
                    "skill `{id}`: a mark's chance is 1 to 100 percent, not {}",
                    rule.chance
                )));
            }
            let benefits = (spec.benefit.into_iter())
                .map(BenefitSpec::resolve)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|m| err(format!("skill `{id}`: {m}")))?;
            list.push(SkillDef {
                id: spec.id,
                name: spec.name,
                description: spec.description,
                kind: spec.kind,
                marks: spec.mark,
                benefits,
            });
        }
        Ok(Self {
            list,
            ladder: parsed.ladder,
            rolls: parsed.rolls,
        })
    }

    /// The skill with data id `id`.
    pub fn id(&self, id: &str) -> Option<SkillId> {
        let i = self.list.iter().position(|s| s.id == id)?;
        u8::try_from(i).ok().map(SkillId)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::{Effect, Use};

    const ONE: &str = "[[skill]]\nid = \"stealth\"\nname = \"Stealth\"\n\
                       description = \"Moving unheard.\"\nkind = \"passive\"\n";

    fn parse(text: &str) -> Result<SkillBook, ContentError> {
        SkillBook::parse("skills.toml", text)
    }

    #[test]
    fn skills_read_their_marks_and_benefits() {
        let book = parse(&format!(
            "[ladder]\ndice = [1, \"1d3\"]\nmarks = [0, 4]\n\
             [rolls]\nintelligence_from = 12\nintelligence_every = 3\nspend = 40\nsoft_cap = 50\n{ONE}\
             mark = [{{ on = \"sneak\", chance = 20 }}]\n\
             benefit = [{{ effect = \"quiet\", curve = [[0, 0], [100, 60]] }}]\n"
        ))
        .expect("valid");
        assert_eq!(book.ladder.die(book.ladder.step(4)).to_string(), "1d3");
        assert_eq!(book.rolls.for_intelligence(15), 2);
        assert_eq!(book.rolls.spent(10), 4);
        let stealth = &book.list[0];
        assert_eq!(stealth.kind, SkillKind::Passive);
        assert_eq!(
            stealth.marks,
            [MarkRule {
                on: Use::Sneak,
                chance: 20
            }]
        );
        assert_eq!(stealth.benefits[0].effect, Effect::Quiet);
        assert_eq!(stealth.benefits[0].at(50), 30);
        assert_eq!(book.id("stealth"), Some(SkillId(0)));
        assert_eq!(book.id("listen"), None);
        // Speed (Athletics, #548): a percent, up to the cap on speed.
        let speed =
            format!("{ONE}benefit = [{{ effect = \"speed\", curve = [[0, 0], [100, 50]] }}]\n");
        let speed = parse(&speed).expect("valid");
        assert_eq!(speed.list[0].benefits[0].effect, Effect::Speed);
        assert_eq!(speed.list[0].benefits[0].at(200), 50);
        // Without `[ladder]` and `[rolls]`, ADOM's ladder and the default rules.
        let plain = parse(ONE).expect("valid");
        assert_eq!(
            (plain.ladder, plain.rolls),
            (Ladder::default(), RollRules::default())
        );
    }

    #[test]
    fn bad_skills_are_refused() {
        for (text, why) in [
            (format!("{ONE}{ONE}"), "duplicate skill id `stealth`"),
            ("skill = []\n".into(), "at least one skill"),
            (ONE.replace("passive", "magic"), "unknown variant `magic`"),
            (
                format!("{ONE}mark = [{{ on = \"sneak\", chance = 0 }}]\n"),
                "1 to 100 percent, not 0",
            ),
            (
                format!("{ONE}mark = [{{ on = \"whistle\", chance = 5 }}]\n"),
                "unknown variant `whistle`",
            ),
            (
                format!("{ONE}benefit = [{{ effect = \"dodge\" }}]\n"),
                "skill `stealth`: a benefit needs",
            ),
            (
                format!("[ladder]\ndice = [1]\nmarks = [3]\n{ONE}"),
                "start at 0",
            ),
            (
                format!(
                    "[rolls]\nintelligence_from = 14\nintelligence_every = 4\nspend = 120\nsoft_cap = 100\n{ONE}"
                ),
                "is a percent",
            ),
            (
                format!("[increase]\nmarks = 4\n{ONE}"),
                "unknown field `increase`",
            ),
            (
                ONE.replace("\"Stealth\"", "\" \""),
                "`name` must not be empty",
            ),
            (format!("{ONE}level = 3\n"), "unknown field `level`"),
            (
                format!("{ONE}benefit = [{{ effect = \"speed\", curve = [[0, 0], [100, 51]] }}]\n"),
                "a `speed` benefit must stay within 0 to 50",
            ),
        ] {
            let e = parse(&text).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }
}
