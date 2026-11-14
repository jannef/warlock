//! Abilities (spec §6 "Abilities"): every ability the abilities menu can list, from the one
//! registry (`warlock_core::ability`).

use super::util::{Term, anchored, at_normal_pace, link, percent_of};
use std::fmt::Write as _;
use warlock_core::ability::{Ability, Aim, Source};
use warlock_core::check;
use warlock_core::content::{
    ActionKind, BASE_TURN, ClassActive, Content, Offer, SignActive, SongOn, TalentEffect, price,
};
use warlock_core::skills::Effect;
use warlock_core::stats::Stat;

/// Every ability, grouped as the menu groups them (by where they come from) and by name within a
/// group: who has it, what it asks for, what it does, how long it takes at a normal pace, and the
/// section that explains it. A skill used as an action whose use is not in the game yet says so.
pub(super) fn abilities(content: &Content) -> String {
    let mut out = String::from(
        "| Ability | In the menu under | Who has it | Asks for | What it does | Takes at a normal \
         pace | Read more |\n|---|---|---|---|---|---:|---|\n",
    );
    let mut every = Ability::every(content);
    every.sort_by_key(|a| (a.source(), a.name(content).to_string()));
    for a in every {
        let who = match a {
            Ability::Sign(active) => sign_holders(content, active),
            Ability::Class(active) => class_holders(content, active),
            Ability::Spell(spell) => {
                let id = content.spell_of(spell).map_or("", |s| s.id.as_str());
                let who = table_holders(
                    content,
                    |e| matches!(e, TalentEffect::Spell { spell } if spell == id),
                );
                let who = if who.is_empty() {
                    "whoever learns it from a scroll".to_string()
                } else {
                    format!("{who}, or whoever learns it from a scroll")
                };
                let spell = content.spell_of(spell).zip(content.spell_rules());
                let (level, cost) = spell.map_or((0, 0), |(s, r)| (s.level, r.mana(s, 0)));
                format!("{who} (a level-{level} spell, {cost} mana a cast)")
            }
            Ability::Skill(_) => "everyone trained in the skill".into(),
            Ability::Disarm => "everyone trained in Disable device".into(),
            Ability::Listen => "everyone trained in Listen".into(),
            Ability::PickLock => "everyone trained in Open lock".into(),
            Ability::Steal => "everyone trained in Steal".into(),
            Ability::Wedge => "everyone trained in Woodcraft".into(),
            Ability::Feint => "everyone trained in Bluff".into(),
            Ability::Tame => {
                "everyone trained in Handle animal, at any level (not while they have a companion; \
                 see [Taming](companions.md#taming))"
                    .into()
            }
            Ability::Intimidate => "everyone trained in Intimidate".into(),
            Ability::Decipher => {
                "everyone trained in Decipher script (usable with a scroll of a kind they have not \
                 identified in the pack; see [Scrolls](scrolls.md))"
                    .into()
            }
            Ability::Heal => "everyone trained in Heal (with something to treat)".into(),
            Ability::Profession(skill) => {
                format!("everyone trained in {}", content.skill(skill).name)
            }
            Ability::Forage => {
                "everyone trained in Survival (usable in the wild: the overworld's land and its \
                 local areas)"
                    .into()
            }
            Ability::SecondWind => "everyone with the Second wind talent (not while winded)".into(),
            Ability::BloodMagic => "everyone with the Blood magic talent".into(),
            Ability::ShieldBash => "everyone (usable with a shield in hand)".into(),
            Ability::ThrowVoice => {
                table_holders(content, |e| matches!(e, TalentEffect::ThrowVoice))
            }
            Ability::Pray => {
                "everyone beside a forgotten altar not yet spent (a prayer takes a minute)".into()
            }
            Ability::Salvage => {
                "everyone (anywhere, with nothing; see [Crafting](crafting.md#salvage))".into()
            }
            Ability::Drink => "everyone beside a healing spring (a drink takes ten seconds)".into(),
            Ability::Gather => {
                "everyone trained in [Herbalism](skills.md#skill-herbalism) beside the [glowing fungus](herbs.md)".into()
            }
            Ability::Heel | Ability::Hold | Ability::SetOn => {
                "everyone with an ally (usable while one is on the level)".into()
            }
            Ability::Give => "everyone who keeps a beast, a companion or a familiar (usable while one is beside them)".into(),
            // Weapon abilities have a table of their own (`weapon_abilities.rs`).
            Ability::Weapon(..) => continue,
        };
        let (asks, takes) = match a.cost() {
            // The breath between actions (Step of the wind).
            Some(_) if matches!(a, Ability::Class(active) if active.takes_no_time()) => {
                (aim(a.aim(content)), "no time".to_string())
            }
            // A long action: its prayer's time, whatever the pace.
            Some(_) if a == Ability::Class(ClassActive::Consecrate) => {
                let book = content.talents();
                let time = (book.id("consecrate").into_iter())
                    .flat_map(|t| book.get(t).effects.iter())
                    .find_map(|e| match e {
                        TalentEffect::Consecrate { time, .. } => Some(time.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                (aim(a.aim(content)), format!("{time} (a prayer)"))
            }
            // A long action too: the rite's kneeling.
            Some(_) if a == Ability::Class(ClassActive::Sanctuary) => {
                let book = content.talents();
                let time = (book.id("sanctuary").into_iter())
                    .flat_map(|t| book.get(t).effects.iter())
                    .find_map(|e| match e {
                        TalentEffect::Sanctuary { time, .. } => Some(time.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                (aim(a.aim(content)), format!("{time} (a rite)"))
            }
            // A place's: its outcome table's time.
            Some(_) if place_seconds(content, a).is_some() => {
                let seconds = place_seconds(content, a).unwrap_or_default();
                (aim(a.aim(content)), format!("{seconds} seconds"))
            }
            // A long action too: a study's turns of writing, by the spell's level.
            Some(_) if a == Ability::Class(ClassActive::Scribe) => {
                let study = content.study_rules();
                let (base, per) = study.map_or((0, 0), |r| (r.turns_base, r.turns_per_level));
                let turn = at_normal_pace(price(
                    content.action_costs().cost(ActionKind::Scribe),
                    BASE_TURN,
                ));
                let time =
                    format!("{base} turns and {per} more a spell level, each {turn} (a study's)");
                (aim(a.aim(content)), time)
            }
            // A long action too: the calling's turns.
            Some(_) if a == Ability::Class(ClassActive::CallFamiliar) => {
                let turns = content.familiars().map_or(0, |r| r.call_turns);
                let turn = at_normal_pace(price(
                    content.action_costs().cost(ActionKind::CallFamiliar),
                    BASE_TURN,
                ));
                (
                    aim(a.aim(content)),
                    format!("{turns} turns, each {turn} (a ritual)"),
                )
            }
            // A long action too: the forage's time.
            Some(_) if a == Ability::Decipher => {
                let minutes = content.decipher().map_or(0, |r| r.seconds / 60);
                (aim(a.aim(content)), format!("{minutes} minutes (a try)"))
            }
            Some(_) if a == Ability::Salvage => {
                let minutes = content.salvage().map_or(0, |r| r.seconds / 60);
                (aim(a.aim(content)), format!("{minutes} minutes (a try)"))
            }
            Some(_) if a == Ability::Forage => {
                let minutes = content.forage().map_or(0, |r| r.seconds / 60);
                (aim(a.aim(content)), format!("{minutes} minutes (a forage)"))
            }
            Some(_) if a == Ability::Gather => {
                let minutes = content.herbs().map_or(0, |r| r.seconds / 60);
                (aim(a.aim(content)), format!("{minutes} minutes (a gather)"))
            }
            Some(kind) => {
                let cost = content.action_costs().cost(kind);
                (aim(a.aim(content)), at_normal_pace(price(cost, BASE_TURN)))
            }
            // An order is a shout: no time at all.
            None if a.source() == Source::Ally => (aim(a.aim(content)), "no time".to_string()),
            // A profession's list: each recipe takes its own time.
            None if matches!(a, Ability::Profession(_)) => {
                (aim(a.aim(content)), "the recipe's time".to_string())
            }
            None => ("—", "not usable yet".to_string()),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {who} | {asks} | {} | {takes} | {} |",
            anchored(Term::Ability, a.id(content), &a.name(content)),
            a.source().name(),
            a.description(content),
            read_more(content, a),
        );
    }
    out
}

/// The section that explains ability `a`, as a link: a skill use's section, a class ability's
/// class (or the page of its own a few have), a spell's row, a birth sign's gift.
fn read_more(content: &Content, a: Ability) -> String {
    let page = |name: &str, target: &str| format!("[{name}]({target})");
    match a {
        Ability::Sign(active) => content.sign_active(active).map_or_else(
            || page("Horoscope", "horoscope.md#what-a-sign-does"),
            // A birth sign's gift is anchored on the horoscope page (`horoscope.rs`).
            |t| {
                page(
                    "Horoscope",
                    &format!("horoscope.md#talent-{}", t.id.replace('_', "-")),
                )
            },
        ),
        Ability::Class(ClassActive::Exploit) => page("Exploit", "resources.md#exploit"),
        Ability::Class(ClassActive::ThrownExploit) => {
            page("Thrown exploit", "resources.md#thrown-exploit")
        }
        Ability::Class(ClassActive::Inspire | ClassActive::Lullaby) => page("Songs", "#songs"),
        Ability::Class(ClassActive::Orison) => page("Prayer", "prayer.md"),
        Ability::Class(ClassActive::CallFamiliar) => page("Familiars", "familiars.md"),
        Ability::Class(ClassActive::FieldForge) => page(
            "The makeshift anvil",
            "altars-forges-springs.md#the-makeshift-anvil",
        ),
        Ability::Class(ClassActive::Rig) => page("Rigging traps", "traps.md#rigging-traps"),
        Ability::Class(active) => class_of(content, active),
        Ability::Spell(spell) => content.spell_of(spell).map_or_else(
            || page("Magic", "magic.md"),
            |s| link(Term::Spell, &s.id, "Magic"),
        ),
        Ability::Skill(skill) => link(Term::Skill, &content.skill(skill).id, "Skills"),
        Ability::Disarm => page("Disarming traps", "traps.md#disarming-traps"),
        Ability::Listen => page("Listening at a door", "hearing.md#listening-at-a-door"),
        Ability::PickLock => page("Picking locks", "doors.md#picking-locks"),
        Ability::Steal => page("Stealing", "skill-uses.md#stealing"),
        Ability::Wedge => page("Wedging a door", "doors.md#wedging-a-door"),
        Ability::Feint => page("Feinting", "skill-uses.md#feinting"),
        Ability::Tame => page("Taming", "companions.md#taming"),
        Ability::Intimidate => page("Intimidating", "skill-uses.md#intimidating"),
        Ability::Decipher => page("Making out a scroll", "scrolls.md#making-out-a-scroll"),
        Ability::Heal => page("Treating yourself", "skill-uses.md#treating-yourself"),
        // A profession with a page of its own links it.
        Ability::Profession(skill) => match content.skill(skill).id.as_str() {
            "alchemy" => page("Brewing potions", "recipes-alchemy.md"),
            "enchanting" => page("Enchanting", "enchanting.md"),
            "jewelcraft" => page("Gem cutting", "gem-cutting.md"),
            _ => page("Professions", "skill-uses.md#professions"),
        },
        Ability::Forage => page("Foraging", "skill-uses.md#foraging"),
        Ability::SecondWind => link(Term::Talent, "second_wind", "Talents"),
        Ability::BloodMagic => link(Term::Talent, "blood_magic", "Talents"),
        Ability::ShieldBash => link(Term::WeaponSkill, "shield_bash", "Weapon skills"),
        Ability::ThrowVoice => page("Throwing your voice", "hearing.md#throwing-your-voice"),
        Ability::Pray => page(
            "Forgotten altars",
            "altars-forges-springs.md#forgotten-altars",
        ),
        Ability::Salvage => page("Salvage", "crafting.md#salvage"),
        Ability::Drink => page(
            "Healing springs",
            "altars-forges-springs.md#healing-springs",
        ),
        Ability::Gather => page("Gathering", "herbs.md#gathering"),
        Ability::Heel | Ability::Hold | Ability::SetOn => page("Orders", "allies.md#orders"),
        Ability::Give => page("The collar", "companions.md#the-collar"),
        Ability::Weapon(..) => page("Weapon skills", "weapon-skills.md#what-a-rank-teaches"),
    }
}

/// The class whose table first gives class active `active`, linked to its section: a class's on
/// the classes page, a prestige class's on the prestige classes page.
fn class_of(content: &Content, active: ClassActive) -> String {
    let book = content.talents();
    let gives = |abilities: &[(u8, String)]| {
        abilities.iter().any(|(_, talent)| {
            (book.id(talent))
                .is_some_and(|t| (book.get(t).effects.iter()).any(|e| e.active() == Some(active)))
        })
    };
    let anchor = |name: &str| name.to_lowercase().replace(' ', "-");
    if let Some(c) = (content.classes().iter()).find(|c| gives(&c.abilities)) {
        return format!("[{}](classes.md#{})", c.name, anchor(&c.name));
    }
    if let Some(p) = (content.prestiges().iter()).find(|p| gives(&p.abilities)) {
        return format!("[{}](prestige-classes.md#{})", p.name, anchor(&p.name));
    }
    "[Classes](classes.md#levels)".to_string()
}

/// How long a place's ability takes: the seconds of the outcome table the place that offers it
/// draws on. `None` for any other ability.
fn place_seconds(content: &Content, a: Ability) -> Option<u32> {
    let offer = match a {
        Ability::Pray => Offer::Pray,
        Ability::Drink => Offer::Drink,
        _ => return None,
    };
    (content.room_effects().defs().iter())
        .filter_map(|e| e.offers.as_ref())
        .find(|(o, _)| *o == offer)
        .and_then(|(_, table)| content.outcomes().get(table.as_deref()?))
        .map(|t| t.seconds)
}

/// Who has class active `active`: the classes whose table gives it, from which level, by name
/// ("paladins from level 12").
fn class_holders(content: &Content, active: ClassActive) -> String {
    table_holders(content, |e| e.active() == Some(active))
}

/// The classes whose table gives a talent with an effect `has` says yes to, from which level, by
/// name ("bards from level 16").
fn table_holders(content: &Content, has: impl Fn(&TalentEffect) -> bool) -> String {
    let book = content.talents();
    let mut who: Vec<(String, u8)> = (content.classes().iter())
        .filter_map(|c| {
            let (level, _) = c.abilities.iter().find(|(_, talent)| {
                (book.id(talent)).is_some_and(|t| (book.get(t).effects.iter()).any(&has))
            })?;
            Some((c.name.to_lowercase(), *level))
        })
        .collect();
    who.sort();
    let mut parts: Vec<String> = (who.iter())
        .map(|(class, level)| format!("{} from level {level}", plural(class)))
        .collect();
    // A prestige class's table counts its own levels (Hunter 1 is the level after its base
    // class's highest).
    let mut prestige: Vec<(String, u8)> = (content.prestiges().iter())
        .filter_map(|p| {
            let (level, _) = p.abilities.iter().find(|(_, talent)| {
                (book.id(talent)).is_some_and(|t| (book.get(t).effects.iter()).any(&has))
            })?;
            Some((p.name.clone(), *level))
        })
        .collect();
    prestige.sort();
    parts.extend((prestige.iter()).map(|(class, level)| {
        format!(
            "{} from {class} level {level}",
            plural(&class.to_lowercase())
        )
    }));
    parts.join(", ")
}

/// Who has the birth sign's active `active`: those born under the animals and elements (and on
/// the lucky days) whose gifts name its talent ("those born in the year of the Unicorn").
fn sign_holders(content: &Content, active: SignActive) -> String {
    let Some(h) = content.horoscope() else {
        return String::new();
    };
    let gives = |grants: &[String]| {
        (grants.iter()).any(|id| content.sign_active(active).is_some_and(|t| &t.id == id))
    };
    let mut who: Vec<String> = (h.animals.iter())
        .filter(|p| gives(&p.grants))
        .map(|p| format!("those born in the year of the {}", p.name))
        .collect();
    who.extend(
        (h.elements.iter())
            .filter(|p| gives(&p.grants))
            .map(|p| format!("the {}-born", p.name)),
    );
    who.extend(
        (h.lucky.iter())
            .filter(|l| gives(&l.grants))
            .map(|l| format!("those born on the day of {}", l.title)),
    );
    // A sign's pick (a Stars talent that asks for the sign) is had by those who take it.
    if let Some(def) = content.sign_active(active) {
        let level = match def.needs.level {
            0 => String::new(),
            l => format!(", from level {l}"),
        };
        let animal = (def.needs.animal.as_deref())
            .and_then(|id| h.animals.iter().find(|p| p.id == id))
            .map(|p| format!("those born in the year of the {}", p.name));
        let element = (def.needs.element.as_deref())
            .and_then(|id| h.elements.iter().find(|p| p.id == id))
            .map(|p| format!("the {}-born", p.name));
        who.extend(
            (animal.into_iter().chain(element))
                .map(|born| format!("{born} who pick it (a sign's pick{level})")),
        );
    }
    if who.is_empty() {
        // An active in the rules that no sign hands out: say so, not an empty cell.
        return "nobody: no birth sign gives it".to_string();
    }
    who.join(", ")
}

/// What an ability asks for before it is used, in words.
fn aim(aim: Aim) -> &'static str {
    match aim {
        Aim::Nothing => "nothing",
        Aim::Direction => "a direction",
        Aim::Cell => "a place you see",
        Aim::Corpse => "a corpse you see",
        Aim::Ailment => "what to treat, when there is more than one",
        Aim::Item(_) => "an item you carry",
        Aim::Throw => "something to throw (never a flask), then a place you see",
        Aim::Shape => "a weapon's shape, from every melee weapon",
        Aim::Rig => "a rig, from the list of them with what each takes, then a direction",
        Aim::Spell => "a spell you know, from the list of them with their prices",
        Aim::Recipes => "a recipe, from the profession's list",
    }
}

/// The songs (spec §8 "Class abilities", the bard's songs): every talent with a `song` effect,
/// by name, with who sings it, what it is paid in, whom it is sung over, the status it brings,
/// and the check its hearers make.
pub(super) fn songs(content: &Content) -> String {
    let mut rows: Vec<(String, String)> = Vec::new();
    for (_, def) in content.talents().iter() {
        for effect in &def.effects {
            let TalentEffect::Song {
                id,
                on,
                status,
                severity,
                resource,
                cost,
                mana,
                check,
                difficulty,
            } = effect
            else {
                continue;
            };
            let who = table_holders(
                content,
                |e| matches!(e, TalentEffect::Song { id: other, .. } if other == id),
            );
            let paid = match resource {
                Some(r) => {
                    let name = (content.resource_id(r))
                        .map_or(r.as_str(), |r| content.resource(r).name.as_str());
                    format!("{cost} {}", link(Term::Resource, r, &name.to_lowercase()))
                }
                None => format!("{mana} [mana](mana.md#spending-it)"),
            };
            let status_name = (content.status_id(status))
                .map_or(status.as_str(), |s| content.status(s).name.as_str());
            let brings = format!(
                "{} at {severity}",
                link(Term::Status, status, &status_name.to_lowercase())
            );
            let (over, against) = match (on, check) {
                (SongOn::Singer, _) => ("you".to_string(), "—".to_string()),
                (SongOn::Hearers, Some(stat)) => (
                    "every hostile creature that hears it".to_string(),
                    format!(
                        "{} against {difficulty} + Music's",
                        link(Term::Stat, &stat.name().to_lowercase(), stat.name())
                    ),
                ),
                (SongOn::Hearers, None) => ("its hearers".to_string(), "—".to_string()),
            };
            let row = format!(
                "| {} | {who} | {paid} | {over} | {brings} | {against} |",
                link(Term::Ability, id, &def.name)
            );
            rows.push((def.name.clone(), row));
        }
    }
    if rows.is_empty() {
        return "Nobody sings yet.\n".into();
    }
    rows.sort();
    let mut out = String::from(
        "| Song | Who sings it | Paid in | Sung over | Brings (degrees) | Their check |\n\
         |---|---|---|---|---|---|\n",
    );
    for (_, row) in rows {
        out.push_str(&row);
        out.push('\n');
    }
    out
}

/// The Music values the song tables show.
const MUSIC_VALUES: [u16; 6] = [0, 50, 100, 150, 200, 300];

/// What Music adds to every song (`skills/`'s `song_degrees` and `song_craft`), by value.
pub(super) fn song_music(content: &Content) -> String {
    let Some(music) = content.skills().iter().find(|s| s.id == "music") else {
        return "Nothing makes a song better.\n".into();
    };
    let at = |effect: Effect, value: u16| {
        (music.benefits.iter())
            .find(|b| b.effect == effect)
            .map_or(0, |b| b.at(value))
    };
    let mut out = String::from(
        "| [Music](skills.md#skill-music) | Degrees added to every song | Added to the hearers' \
         difficulty |\n|---:|:---:|:---:|\n",
    );
    for value in MUSIC_VALUES {
        let _ = writeln!(
            out,
            "| {value} | +{} | +{} |",
            at(Effect::SongDegrees, value),
            at(Effect::SongCraft, value)
        );
    }
    out
}

/// The chance a hearer shakes off each song sung over its hearers, by its edge (its modifier of
/// the stat the song checks) and the singer's Music: 2d12 + the edge against the song's
/// difficulty plus Music's craft (`check::ways`, as the surprise table's).
pub(super) fn song_odds(content: &Content) -> String {
    let Some(music) = content.skills().iter().find(|s| s.id == "music") else {
        return String::new();
    };
    let craft = |value: u16| {
        (music.benefits.iter())
            .find(|b| b.effect == Effect::SongCraft)
            .map_or(0, |b| b.at(value))
    };
    let values: Vec<u16> = (MUSIC_VALUES.into_iter())
        .filter(|v| v % 100 == 0)
        .collect();
    let mut songs: Vec<(&str, Stat, i32)> = Vec::new();
    for (_, def) in content.talents().iter() {
        for effect in &def.effects {
            if let TalentEffect::Song {
                on: SongOn::Hearers,
                check: Some(stat),
                difficulty,
                ..
            } = effect
            {
                songs.push((def.name.as_str(), *stat, *difficulty));
            }
        }
    }
    songs.sort_by_key(|s| s.0);
    let mut out = String::new();
    for (name, stat, difficulty) in songs {
        let _ = write!(
            out,
            "**{name}**, shaken off:\n\n| Their {} edge |",
            stat.name()
        );
        for v in &values {
            let _ = write!(out, " Music {v} ({}) |", difficulty + craft(*v));
        }
        out.push_str("\n|:---:|");
        out.push_str(&":---:|".repeat(values.len()));
        out.push('\n');
        for edge in -2..=8 {
            let sign = if edge < 0 { "\u{2212}" } else { "+" };
            let _ = write!(out, "| {sign}{} |", i32::abs(edge));
            for v in &values {
                let ways = check::ways(edge, difficulty + craft(*v));
                let _ = write!(out, " {} |", percent_of(ways, 144));
            }
            out.push('\n');
        }
        out.push('\n');
    }
    if out.is_empty() {
        out.push_str("No song is sung against anyone yet.\n");
    }
    out
}

/// A class name in the plural: "witch" is "witches", "thief" "thieves", "mystic" "mystics".
fn plural(name: &str) -> String {
    if let Some(stem) = name.strip_suffix("ief") {
        format!("{stem}ieves")
    } else if ["s", "x", "z", "ch", "sh"]
        .iter()
        .any(|e| name.ends_with(e))
    {
        format!("{name}es")
    } else {
        format!("{name}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_song_tables_read_the_data() {
        let content = Content::builtin().expect("content");
        let songs = songs(&content);
        assert!(
            songs.contains("[Inspire](abilities.md#ability-inspire)"),
            "{songs}"
        );
        assert!(songs.contains("bards from level 20"), "{songs}");
        assert!(songs.contains("6 [mana]"), "{songs}");
        let music = song_music(&content);
        assert!(music.contains("| 100 | +2 | +4 |"), "{music}");
        let odds = song_odds(&content);
        assert!(odds.contains("Music 100 (17)"), "{odds}");
        assert!(odds.contains("| +0 | 54.2% | 25% |"), "{odds}");
    }
}
