//! A spell's numbers in the words the player reads them (the spell scaling design §11, decision
//! 11: the owner's one exception to hidden numbers, a spell's roll shown as a weapon's is): its
//! roll with its damage type or "healed" (`4d6 fire`, `3d6 healed`), its reach (`range 11`,
//! `radius 2`), and a reroll (`Your 1s are rolled again.`). The abilities menu, the wand's
//! description, and the `spells` dump all say them so; built from what the caster has, never a
//! hidden number of anyone else's.

use super::Actor;
use super::cast::Reach;
use super::spell_rung::CastBy;
use crate::content::{Content, DamageTypeId, Shape, SpellData};
use crate::dice::Roll;

/// The roll a spell makes as its caster would cast it now, and what it does: harm of a kind, or
/// healing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellRoll {
    pub roll: Roll,
    /// The kind of harm; `None` for healing.
    pub damage_type: Option<DamageTypeId>,
    /// The roll as the menu shows it: `4d6 fire`, `3d6 healed`.
    pub text: String,
}

impl Actor {
    /// The roll `spell` makes as it casts it as `by` ([`Actor::spell_roll`]), in words; `None`
    /// for a spell that rolls nothing.
    pub fn spell_roll_words(
        &self,
        content: &Content,
        spell: &SpellData,
        by: CastBy,
    ) -> Option<SpellRoll> {
        let roll = self.spell_roll(content, spell, by)?;
        Some(roll_words(content, spell, roll))
    }
}

/// `roll`, made by `spell`, in words.
pub fn roll_words(content: &Content, spell: &SpellData, roll: Roll) -> SpellRoll {
    let what = match spell.damage_type {
        Some(kind) => content.damage_type(kind).name.as_str(),
        None => "healed",
    };
    SpellRoll {
        text: format!("{roll} {what}"),
        roll,
        damage_type: spell.damage_type,
    }
}

/// A spell of `shape`'s `reach` in words, each a part: `range 11` (a ray's, bolt's, or burst's),
/// `length 9` (a cone's or a line's), `radius 2` (a burst's). A touch's reach of 1 and a spell
/// on oneself say nothing.
pub fn reach_words(shape: Shape, reach: Reach) -> Vec<String> {
    let mut parts = Vec::new();
    match (shape, reach.range) {
        (Shape::Cone | Shape::Line, Some(n)) => parts.push(format!("length {n}")),
        (Shape::Ray | Shape::Bolt | Shape::Burst, Some(n)) => parts.push(format!("range {n}")),
        _ => {}
    }
    if let Some(r) = reach.radius {
        parts.push(format!("radius {r}"));
    }
    parts
}

/// A reroll of the dice showing `most` or less, in words (the spell scaling design §9's note):
/// `Your 1s are rolled again.`, `Your 1s and 2s are rolled again.`; `None` for none.
pub fn reroll_words(most: u32) -> Option<String> {
    let faces: Vec<String> = (1..=most).map(|n| format!("{n}s")).collect();
    let list = match faces.as_slice() {
        [] => return None,
        [one] => one.clone(),
        [first, second] => format!("{first} and {second}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    };
    Some(format!("Your {list} are rolled again."))
}

#[cfg(test)]
mod tests {
    use super::super::World;
    use super::super::caster_tests::{sp, world_with};
    use super::*;
    use crate::stats::Stat;
    use std::sync::{Arc, LazyLock};

    static BUILTIN: LazyLock<Arc<Content>> =
        LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

    /// A wizard at Intelligence 19 with Empower and Sure hand, knowing a harm, a heal, and a
    /// buff besides the class's.
    fn grown() -> World {
        let mut w = world_with(BUILTIN.clone(), "wizard");
        super::super::cast_tests::scores(&mut w, &[(Stat::Intelligence, 19)]);
        for id in ["empower", "sure_hand"] {
            w.player_mut().talents.take(id, None);
        }
        for spell in ["fireball", "cure_wounds", "haste"] {
            w.player_mut().spells.insert(sp(spell));
        }
        w
    }

    #[test]
    fn the_players_own_roll_is_the_cast_roll_in_words() {
        let w = grown();
        let view = w.perception();
        let me = w.state.player();
        for spell in view.own_spells() {
            let def = BUILTIN.spell(spell);
            let words = view.own_spell_roll(spell);
            let roll = me.spell_roll(&BUILTIN, def, CastBy::Own);
            assert_eq!(words.as_ref().map(|w| w.roll), roll, "{}", def.id);
            let Some(words) = words else { continue };
            let what = match def.damage_type {
                Some(kind) => BUILTIN.damage_type(kind).name.clone(),
                None => "healed".into(),
            };
            assert_eq!(words.text, format!("{} {what}", words.roll), "{}", def.id);
        }
        let text = |id: &str| view.own_spell_roll(sp(id)).map(|r| r.text);
        // Fireball at Intelligence 19: rung 3 by stats (5d6), Empower a rung more (5d6+2).
        assert_eq!(text("fireball").as_deref(), Some("5d6+2 fire"));
        assert_eq!(text("cure_wounds").as_deref(), Some("3d6 healed"));
        assert_eq!(text("haste"), None, "a buff rolls nothing");
        assert_eq!(view.own_spell_reroll(sp("fireball")), 1, "Sure hand");
        assert_eq!(
            view.own_spell_reroll(sp("haste")),
            0,
            "nothing to roll again"
        );
    }

    #[test]
    fn the_json_dump_gives_each_spells_roll_type_and_reach() {
        use crate::dump::{json, parse_layers};
        let w = grown();
        let value = json(&w, &[parse_layers("spells").expect("layer")[0]], false);
        let spells = value["spells"].as_array().expect("spells");
        let of = |id: &str| (spells.iter()).find(|s| s["id"] == id).expect(id);
        let fireball = of("fireball");
        assert_eq!(fireball["roll"], "5d6+2 fire");
        assert_eq!(fireball["damage_type"], "fire");
        assert_eq!(fireball["reach"]["radius"], 2);
        assert!(fireball["reach"]["range"].is_number());
        assert!(fireball["reach"]["length"].is_null());
        assert_eq!(of("cure_wounds")["damage_type"], "healed");
        let haste = of("haste");
        assert!(haste["roll"].is_null() && haste["damage_type"].is_null());
        assert!(haste["reach"]["range"].is_null() && haste["reach"]["radius"].is_null());
    }

    #[test]
    fn a_reroll_names_its_faces() {
        assert_eq!(reroll_words(0), None);
        assert_eq!(reroll_words(1).unwrap(), "Your 1s are rolled again.");
        assert_eq!(reroll_words(2).unwrap(), "Your 1s and 2s are rolled again.");
        assert_eq!(
            reroll_words(3).unwrap(),
            "Your 1s, 2s, and 3s are rolled again."
        );
    }

    #[test]
    fn reach_says_range_length_and_radius_by_shape() {
        let reach = |range, radius| Reach { range, radius };
        assert_eq!(
            reach_words(Shape::Burst, reach(Some(11), Some(2))),
            ["range 11", "radius 2"]
        );
        assert_eq!(reach_words(Shape::Cone, reach(Some(3), None)), ["length 3"]);
        assert!(reach_words(Shape::Touch, reach(Some(1), None)).is_empty());
        assert!(reach_words(Shape::Myself, reach(None, None)).is_empty());
    }
}
