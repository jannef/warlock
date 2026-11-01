//! Objects in the world (spec §5, §6): stacks lying in piles on cells and carried in
//! inventories, and what things weigh.
//!
//! A **pile** is the ordered list of stacks on a cell, the top first: the top is what the map
//! shows, and dropped objects go on top. An **inventory** is an ordered list of stacks, as many
//! as the actor can carry (weight is the only limit); new stacks go last, and the rules name
//! them by position (screens letter them per page). A stackable
//! object (coins, arrows) joins a stack of the same object wherever it goes; any other object is
//! a stack of one.
//!
//! Weights are integer grams. An actor weighs its creature's base weight plus everything it
//! carries ([`Weight`]).
//!
//! A magic or rare item (spec §8 "Magic items", [`crate::magic`]) is a stack that carries what it
//! rolled ([`Magic`]); it joins only a stack with the very same magic (arrows shot from a magic
//! stack and picked up again). What the player is shown of it goes through
//! [`Stack::describe`] and [`Stack::known_magic`], which keep an unidentified item's affixes
//! hidden.
//!
//! An item made of other parts than its base item (spec §7 "Gems": a loose gem, an item set
//! with a gem; "Materials": an item of another material, a bronze long sword) carries them
//! ([`crate::item_parts`]); its name, value, weight, colour, and what it does in a fight follow
//! from them ([`crate::variant`]), and it joins only a stack made the same way. Materials are not
//! hidden: the player sees them at once.
//!
//! Food (spec §8 "Hunger") remembers when it was made ([`Stack::died`]; a corpse, a stack of one,
//! when its creature died): it rots from then on, and is gone in the end ([`crate::rot`]). Food
//! joins a stack of the same food whatever its age, and the stack keeps the oldest.

use crate::color::Rgb;
use crate::content::{Content, Cutting, DamageTypeId, MaterialId, ObjectId, Part, Rarity, part};
use crate::item_parts::{self, ItemPart};
use crate::magic::{Effect, Magic, Mods, name};
use crate::rng::Rng;
use crate::scroll_label::Lore;
use crate::time::GameTime;
use crate::variant;

pub use crate::piece::break_up;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Some number of one object, and what it rolled if it is magic.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Stack {
    pub object: ObjectId,
    /// At least 1; always 1 for objects that do not stack.
    pub count: u32,
    /// What a magic or rare item rolled (boxed: most stacks have none). Ground truth: the rules
    /// read it, presentation only through [`Stack::known_magic`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    magic: Option<Box<Magic>>,
    /// What it is made of where it differs from its base item ([`crate::item_parts`]): a loose
    /// gem's gem, a gem it was set with. Boxed: most stacks have none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parts: Option<Box<[ItemPart]>>,
    /// Food: when it was made (a corpse: when its creature died), the moment it rots from (rules
    /// and dev tools only: food does not show its age, so presentation cannot read it). `None`
    /// for everything else, and for food in content that has not come into the world yet
    /// ([`crate::rot::made_at`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) died: Option<GameTime>,
    /// A wand's charges used and what the player knows of them (spec §8 "Magic items":
    /// "Wands"); `None` for everything else, and for a wand as it is made: full, its charges
    /// unknown. Ground truth: presentation reads them only through [`Stack::known_charges`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    charges: Option<Charges>,
    /// A wand's rung on its spell's damage ladder (spec §8 "Magic items": "Wands"; the spell
    /// scaling design §7.4), drawn evenly in its kind's `rungs` as it is made
    /// ([`Stack::draw_wand_rung`]); `None` for everything else. A wand without one (made before
    /// rungs were drawn: an older save) casts at rung 0. Ground truth: presentation reads it
    /// only through [`Stack::known_wand_rung`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    wand_rung: Option<u8>,
    /// A carried light's mode (spec §5 "Light sources": a lantern unlit, lit, or hooded), kept
    /// with the item wherever it goes; it gives light only ready in the Tool slot
    /// (`WorldState::carried_light`). Unlit for everything else.
    #[serde(default, skip_serializing_if = "LightMode::is_unlit")]
    light: LightMode,
}

/// The mode of a carried light ([`Stack`]'s `light`; spec §5 "Light sources"): what `u`, the
/// use-tool command (`World::use_tool`), turns a lantern ready in the Tool slot through.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LightMode {
    #[default]
    Unlit,
    Lit,
    /// Lit, with its hood lowered: its light's `hooded` preset, dim and hollow (it lights the
    /// cells around its carrier, not their own).
    Hooded,
}

impl LightMode {
    fn is_unlit(&self) -> bool {
        *self == LightMode::Unlit
    }

    /// As an item's name says it, "lantern (lit)": nothing while unlit.
    pub const fn word(self) -> Option<&'static str> {
        match self {
            LightMode::Unlit => None,
            LightMode::Lit => Some("lit"),
            LightMode::Hooded => Some("hooded"),
        }
    }
}

/// A wand's state ([`Stack`]'s `charges`): how many charges it has used, whether the player
/// knows how many it has left, and how long they have carried it not knowing (Connoisseur).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct Charges {
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    used: u8,
    #[serde(default, skip_serializing_if = "is_false")]
    known: bool,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    carried: u32,
}

fn is_zero_u8(n: &u8) -> bool {
    *n == 0
}

fn is_zero_u32(n: &u32) -> bool {
    *n == 0
}

fn is_false(b: &bool) -> bool {
    !*b
}

impl Stack {
    /// `count` of a normal `object`; food made by it does not know its age yet.
    pub fn new(object: ObjectId, count: u32) -> Self {
        Self {
            object,
            count,
            magic: None,
            parts: None,
            died: None,
            charges: None,
            wand_rung: None,
            light: LightMode::Unlit,
        }
    }

    pub fn one(object: ObjectId) -> Self {
        Self::new(object, 1)
    }

    /// A corpse, `object`, of a creature that died at `died`.
    pub fn corpse(object: ObjectId, died: GameTime) -> Self {
        Self {
            died: Some(died),
            ..Self::new(object, 1)
        }
    }

    /// `count` of `object` that rolled `magic`.
    pub(crate) fn with_magic(object: ObjectId, count: u32, magic: Magic) -> Self {
        Self {
            magic: Some(Box::new(magic)),
            ..Self::new(object, count)
        }
    }

    /// This stack made of `parts` where it differs from its base item ([`crate::item_parts`]).
    pub fn with_parts(self, parts: Vec<ItemPart>) -> Self {
        Self {
            parts: Some(parts.into_boxed_slice()),
            ..self
        }
    }

    /// One of this stack: the same object, magic, and parts.
    pub(crate) fn single(&self) -> Self {
        Self {
            count: 1,
            ..self.clone()
        }
    }

    /// It as the player knows it: without its magic while that is unidentified (what the
    /// character screen's attacks are figured for, `world/attacks.rs`).
    pub(crate) fn as_known(&self) -> Self {
        let mut known = self.clone();
        if !known.is_identified() {
            known.magic = None;
        }
        known
    }

    /// What it rolled, identified or not: ground truth, for the rules and dev tools.
    pub(crate) fn magic(&self) -> Option<&Magic> {
        self.magic.as_deref()
    }

    /// Re-tiers its magic's affixes as a regraded ladder asks ([`Magic::retier`]), a line each.
    pub(crate) fn retier(&mut self, content: &Content) -> Vec<String> {
        self.magic
            .as_mut()
            .map_or_else(Vec::new, |m| m.retier(content))
    }

    /// Sets what it rolled (a forge's tempering, `magic/temper.rs`): `None` makes it plain.
    pub(crate) fn set_magic(&mut self, magic: Option<Magic>) {
        self.magic = magic.map(Box::new);
    }

    /// Normal, magic, or rare: this the player sees at once.
    pub fn rarity(&self) -> Rarity {
        self.magic.as_ref().map_or(Rarity::Normal, |m| m.rarity)
    }

    /// Whether the player knows what it does: a normal item always, a magic or rare one once
    /// identified.
    pub fn is_identified(&self) -> bool {
        self.magic.as_ref().is_none_or(|m| m.identified)
    }

    /// What it rolled, if the player has identified it: all presentation may know of its magic.
    pub fn known_magic(&self) -> Option<&Magic> {
        self.magic().filter(|m| m.identified)
    }

    /// Counts `seconds` more of it carried by someone who learns magic items by carrying them
    /// (spec §8 "Class abilities": Connoisseur), if it is magic and unidentified, or a wand
    /// whose charges they do not know; returns whether it has now been carried `needed` seconds.
    pub(crate) fn carry_for(&mut self, content: &Content, seconds: u64, needed: u64) -> bool {
        let carried = match self.magic.as_mut() {
            Some(m) if !m.identified => &mut m.carried,
            _ if content.object(self.object).wand.is_some() => {
                let charges = self.charges.get_or_insert_with(Charges::default);
                if charges.known {
                    return false;
                }
                &mut charges.carried
            }
            _ => return false,
        };
        let total = u64::from(*carried).saturating_add(seconds).min(needed);
        *carried = u32::try_from(total).unwrap_or(u32::MAX);
        total >= needed
    }

    /// The charges it has left, if it is a wand: ground truth, for the rules and dev tools.
    pub(crate) fn charges_left(&self, content: &Content) -> Option<u8> {
        let wand = content.object(self.object).wand?;
        let used = self.charges.map_or(0, |c| c.used);
        Some(wand.charges.saturating_sub(used))
    }

    /// Whether the player knows how many charges it has left (a wand identified); true for
    /// anything that is no wand.
    pub fn charges_known(&self, content: &Content) -> bool {
        content.object(self.object).wand.is_none() || self.charges.is_some_and(|c| c.known)
    }

    /// The charges it has left, if it is a wand whose charges the player knows: all
    /// presentation may know of them.
    pub fn known_charges(&self, content: &Content) -> Option<u8> {
        (self.charges.is_some_and(|c| c.known))
            .then(|| self.charges_left(content))
            .flatten()
    }

    /// Draws its rung if it is a wand without one (as it is made: loot, a shop's stock, a level's
    /// pile, a kit, each from the stream it already draws from): evenly in its kind's `rungs`,
    /// one draw of `rng` (none when the range is one rung). Anything else draws nothing.
    pub(crate) fn draw_wand_rung(&mut self, content: &Content, rng: &mut Rng) {
        if self.wand_rung.is_some() {
            return;
        }
        if let Some(wand) = content.object(self.object).wand {
            let (lo, hi) = (i32::from(wand.min_rung), i32::from(wand.max_rung));
            self.wand_rung = Some(u8::try_from(rng.range(lo, hi)).unwrap_or(wand.min_rung));
        }
    }

    /// The rung it casts at on its spell's damage ladder, if it is a wand (0 for one made
    /// before rungs were drawn): ground truth, for the rules and dev tools.
    pub(crate) fn wand_rung(&self, content: &Content) -> Option<u8> {
        content.object(self.object).wand?;
        Some(self.wand_rung.unwrap_or(0))
    }

    /// Its rung, if it is a wand whose type the player knows (the rung comes with the type):
    /// all presentation may know of it.
    pub fn known_wand_rung(&self, lore: Lore<'_>) -> Option<u8> {
        (lore.is_identified(self.object))
            .then(|| self.wand_rung(lore.content))
            .flatten()
    }

    /// Uses up one of its charges (a wand with one left).
    pub(crate) fn use_charge(&mut self) {
        let charges = self.charges.get_or_insert_with(Charges::default);
        charges.used = charges.used.saturating_add(1);
    }

    /// Lets the player know its charges, if it is a wand; returns whether there was anything to
    /// learn.
    pub(crate) fn identify_charges(&mut self, content: &Content) -> bool {
        if self.charges_known(content) {
            return false;
        }
        self.charges.get_or_insert_with(Charges::default).known = true;
        true
    }

    /// Identifies it; returns whether there was anything to learn.
    pub(crate) fn identify(&mut self) -> bool {
        match self.magic.as_mut() {
            Some(m) if !m.identified => {
                m.identified = true;
                true
            }
            _ => false,
        }
    }

    /// Forgets what it is (spec §7 "Special rooms": a prayer's Muddled): an identified magic
    /// item is unidentified again, and what carrying it taught is lost. Whether it was known.
    pub(crate) fn forget(&mut self) -> bool {
        match self.magic.as_mut() {
            Some(m) if m.identified => {
                m.identified = false;
                m.carried = 0;
                true
            }
            _ => false,
        }
    }

    /// Grams: its object's weight (a loose gem: its size's; made of other materials, scaled by
    /// them, [`variant::weight`]), less what a weight affix takes off, times the count.
    pub fn weight(&self, content: &Content) -> u64 {
        let def = content.object(self.object);
        let each = match (self.gem(content), content.gems()) {
            (Some((_, cutting)), Some(gems)) => u64::from(gems.size(cutting.size).weight),
            _ if self.parts.is_some() => {
                u64::from(variant::weight(def, &self.parts(content), content))
            }
            _ => u64::from(def.weight),
        };
        let lighter = match self.magic {
            Some(_) => Mods::local(self, content).get(Effect::Weight),
            None => 0,
        };
        crate::magic::mods::less(each, lighter).max(1) * u64::from(self.count)
    }

    /// Its Protection while worn (spec §8 "Combat"): its object's (scaled by another material,
    /// [`Stack::base_protection`]), plus what a Protection affix adds (the one reader of that
    /// effect; rules only, as the player knows it only once identified).
    pub(crate) fn protection(&self, content: &Content) -> i32 {
        let more = match self.magic {
            Some(_) => Mods::local(self, content).get(Effect::Protection),
            None => 0,
        };
        self.base_protection(content) + more
    }

    /// Its own Protection, without magic: its object's, scaled by its main part's material if
    /// that is another than its base item's ([`variant::protection`]). Materials are not
    /// hidden: the item description shows it.
    pub fn base_protection(&self, content: &Content) -> i32 {
        let def = content.object(self.object);
        match self.parts {
            Some(_) => variant::protection(def, &self.parts(content), content),
            None => def.protection,
        }
    }

    /// How its Protection stands against each physical kind, more or less than its total
    /// (spec §8 "Damage types": its object's `protects`).
    pub(crate) fn protects<'c>(&self, content: &'c Content) -> &'c [(DamageTypeId, i32)] {
        &content.object(self.object).protects
    }

    /// What it resists while worn or held, in percent per damage type: its object's `resist`
    /// (what its affixes add is global: [`Mods::worn`]).
    pub(crate) fn resists<'c>(&self, content: &'c Content) -> &'c [(DamageTypeId, i32)] {
        &content.object(self.object).resist
    }

    /// What it is made of where it differs from its base item, as saved: its own parts
    /// ([`crate::item_parts`]), if any. Materials are not hidden.
    pub fn own_parts(&self) -> Option<&[ItemPart]> {
        self.parts.as_deref()
    }

    /// What it is made of, part by part, the main part first (spec §7 "Objects and materials"):
    /// its base item's parts with its own in their place ([`item_parts::resolve`]). Materials are
    /// not hidden: the player sees them, identified or not.
    pub fn parts<'a>(&self, content: &'a Content) -> Cow<'a, [Part]> {
        let base = &content.object(self.object).parts;
        match &self.parts {
            Some(own) => Cow::Owned(item_parts::resolve(base, own, content)),
            None => Cow::Borrowed(base),
        }
    }

    /// The gem it is, if it is a loose gem (the gems' object, spec §7 "Gems"): its material and
    /// its size and cut.
    pub fn gem(&self, content: &Content) -> Option<(MaterialId, Cutting)> {
        let gems = content.gems()?;
        if self.object != gems.object {
            return None;
        }
        let main = *self.parts(content).first()?;
        Some((main.material, main.gem?))
    }

    /// What it is worth, in gold coins: each one's value, made of its [`Stack::parts`]
    /// ([`crate::content::ObjectDef::value_with`]: a gem by its size and cut), times the count.
    /// Magic does not change it.
    pub fn value(&self, content: &Content) -> u64 {
        let def = content.object(self.object);
        u64::from(def.value_with(&self.parts(content), content)) * u64::from(self.count)
    }

    /// What it is called as it is made, singular and plural, without an article, a count, or
    /// magic: its object's name ("long sword", "gold and ruby amulet"), named from what its file
    /// writes with its own parts if it has any ("emerald-pommelled long sword", "bronze long
    /// sword"; [`part::names`]), a loose gem's ("large fine emerald", [`crate::gem::names`]), or
    /// a piece's by its material ("piece of tin", [`crate::piece::names`]).
    pub fn names(&self, content: &Content) -> (String, String) {
        let def = content.object(self.object);
        if def.id == crate::piece::ID {
            return crate::piece::names(content, def, self.parts(content)[0].material);
        }
        if self.parts.is_none() {
            return (def.name.clone(), def.plural.clone());
        }
        if let Some((gem, cutting)) = self.gem(content) {
            return crate::gem::names(content, gem, cutting);
        }
        part::names(
            &def.written_name,
            &def.written_plural,
            &self.parts(content),
            def.material(),
            content.materials(),
            content.part_defs(),
        )
    }

    /// The colour its glyph is drawn in: its object's, a loose gem's own (duller uncut), its
    /// gem's on an item that takes its gem's colour (a ring set with one; spec §7 "Gems"), or
    /// else its main part's material's where that is not its base item's (a bronze long sword,
    /// [`variant::color`]).
    pub fn fg(&self, content: &Content) -> Rgb {
        let def = content.object(self.object);
        if self.parts.is_none() {
            return def.fg;
        }
        let gem_color = || {
            if let Some((gem, cutting)) = self.gem(content) {
                return crate::gem::color(content, gem, cutting);
            }
            if !content.gems()?.ornament(def.class)?.colour {
                return None;
            }
            let set = (self.parts(content).iter())
                .find(|p| p.gem.is_some())
                .copied()?;
            crate::gem::color(content, set.material, set.gem?)
        };
        gem_color()
            .or_else(|| variant::color(def, &self.parts(content), content))
            .unwrap_or(def.fg)
    }

    /// As the player knows it: "a dagger", "12 arrows", "a Sharp long sword of the Bear", "Grim
    /// Bite, a long sword", unidentified "a long sword (magic)".
    pub fn describe(&self, content: &Content) -> String {
        let names = self.names(content);
        let known = self.is_identified();
        let text = name::describe(&names, self.count, self.magic(), known, content);
        match self.light.word() {
            Some(mode) => format!("{text} ({mode})"),
            None => text,
        }
    }

    /// Its mode, if it is a carried light (a lantern: unlit, lit, or hooded); unlit otherwise.
    pub fn light_mode(&self) -> LightMode {
        self.light
    }

    /// Sets its mode (`World::use_tool`): only a carried light has one.
    pub(crate) fn set_light_mode(&mut self, mode: LightMode) {
        self.light = mode;
    }

    /// What it is called as the player knows it, without an article or count: "long sword",
    /// "Sharp long sword of the Bear", "Grim Bite".
    pub fn title(&self, content: &Content) -> String {
        let names = self.names(content);
        name::title(&names, self.magic(), self.is_identified(), content)
    }

    /// As it truly is, identified or not: for dev tools only.
    pub(crate) fn describe_truth(&self, content: &Content) -> String {
        let names = self.names(content);
        let mut text = name::describe(&names, self.count, self.magic(), true, content);
        if let Some(mode) = self.light.word() {
            text += &format!(" ({mode})");
        }
        if let Some(left) = self.charges_left(content) {
            text += &format!(" ({})", charges_text(left));
        }
        if let Some(rung) = self.wand_rung(content) {
            text += &format!(" (rung {rung})");
        }
        match self.magic() {
            Some(m) if !m.identified => format!("{text} (unidentified)"),
            _ if !self.charges_known(content) => format!("{text} (unidentified)"),
            _ => text,
        }
    }

    /// Whether the object exists in `content`, the count makes sense, its magic and its own
    /// parts are valid for it (a loose gem has its gem), and it knows when it was made if and
    /// only if it is food (a guard for saves).
    pub(crate) fn is_valid_for(&self, content: &Content) -> bool {
        if usize::from(self.object.0) >= content.object_defs().len() {
            return false;
        }
        let def = content.object(self.object);
        let loose = content.gems().is_some_and(|g| g.object == self.object);
        self.count >= 1
            && (self.count == 1 || def.stackable)
            && (self.magic()).is_none_or(|m| m.check(def, content).is_ok())
            && match self.own_parts() {
                Some(own) => item_parts::check(def, own, content).is_ok(),
                None => !loose,
            }
            && self.died.is_some() == def.food.is_some()
            && (self.charges).is_none_or(|c| def.wand.is_some_and(|w| c.used <= w.charges))
            && (self.wand_rung.is_none() || def.wand.is_some())
            && (self.light.is_unlit() || def.carried_light.is_some())
    }

    /// The same goods as `other`: the same object made the same way (what a shop's bill counts
    /// by, whatever the magic or age).
    pub fn same_goods(&self, other: &Stack) -> bool {
        self.object == other.object && self.parts == other.parts
    }

    /// `count` of these goods as a shop's bill counts them: the object and what it is made of,
    /// without magic or age.
    pub(crate) fn goods(&self, count: u32) -> Stack {
        Stack {
            parts: self.parts.clone(),
            ..Stack::new(self.object, count)
        }
    }
}

/// The age a stack of food keeps when two join: the oldest (a stack that does not know its age
/// takes the other's).
fn oldest(a: Option<GameTime>, b: Option<GameTime>) -> Option<GameTime> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}

/// A wand's charges in words: "7 charges", "1 charge", "empty".
pub fn charges_text(left: u8) -> String {
    match left {
        0 => "empty".into(),
        1 => "1 charge".into(),
        n => format!("{n} charges"),
    }
}

/// Total weight of `stacks`, in grams.
pub fn weight_of(stacks: &[Stack], content: &Content) -> u64 {
    stacks.iter().map(|s| s.weight(content)).sum()
}

/// What an actor weighs, in grams: its creature's base weight plus everything it carries. The
/// rules know every actor's weight; the player is shown only their own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Weight {
    /// The creature itself.
    pub base: u64,
    /// Everything it carries.
    pub carried: u64,
}

impl Weight {
    pub fn total(self) -> u64 {
        self.base + self.carried
    }
}

/// Grams for people: `450 g` below a kilogram, else kilograms to one decimal (`1.3 kg`, `70 kg`).
pub fn format_weight(grams: u64) -> String {
    if grams < 1000 {
        return format!("{grams} g");
    }
    let tenths = (grams + 50) / 100;
    match tenths % 10 {
        0 => format!("{} kg", tenths / 10),
        d => format!("{}.{d} kg", tenths / 10),
    }
}

/// Stacks in a sentence: "a dagger", "a dagger and 12 arrows", "a dagger, 12 arrows, and a
/// ruby"; more than three stacks are counted ("17 objects").
pub fn phrase(stacks: &[Stack], lore: Lore<'_>) -> String {
    phrase_with(stacks, lore, |_| None)
}

/// [`phrase`] with what `tag` says of the stack at each position after its name, in
/// parentheses: "a dagger (for sale, 45 gold) and 12 arrows (for sale, 18 gold)". More than
/// three stacks are counted, untagged.
pub fn phrase_with(
    stacks: &[Stack],
    lore: Lore<'_>,
    tag: impl Fn(usize) -> Option<String>,
) -> String {
    let names: Vec<String> = (stacks.iter().enumerate())
        .map(|(i, s)| match tag(i) {
            Some(tag) => format!("{} ({tag})", lore.describe(s)),
            None => lore.describe(s),
        })
        .collect();
    match names.as_slice() {
        [] => "nothing".into(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [a, b, c] => format!("{a}, {b}, and {c}"),
        _ => {
            let count: u64 = stacks.iter().map(|s| u64::from(s.count)).sum();
            format!("{count} objects")
        }
    }
}

/// Joins `stack` to a stack of the same object (and the same magic and parts, if any) in
/// `stacks` if it stacks; returns whether it did. Food joins whatever its age, and keeps the
/// oldest.
fn join(stacks: &mut [Stack], stack: &Stack, content: &Content) -> bool {
    if !content.object(stack.object).stackable {
        return false;
    }
    match (stacks.iter_mut()).find(|s| s.same_goods(stack) && s.magic == stack.magic) {
        Some(same) => {
            same.count += stack.count;
            same.died = oldest(same.died, stack.died);
            true
        }
        None => false,
    }
}

/// Puts `stacks` on top of `pile`, keeping their order (the first ends on top). Stackable
/// objects join a stack of the same object already there.
pub fn add_to_pile(pile: &mut Vec<Stack>, stacks: &[Stack], content: &Content) {
    for stack in stacks.iter().rev() {
        if !join(pile, stack, content) {
            pile.insert(0, stack.clone());
        }
    }
}

/// Adds `stack` to `inventory`: joins a stack of the same object if it stacks, else goes last.
pub fn add_to_inventory(inventory: &mut Vec<Stack>, stack: Stack, content: &Content) {
    if !join(inventory, &stack, content) {
        inventory.push(stack);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content() -> Content {
        Content::builtin().expect("built-in content")
    }

    fn stack(content: &Content, id: &str, count: u32) -> Stack {
        let object = content.object_id(id).unwrap_or_else(|| panic!("{id}"));
        Stack::new(object, count)
    }

    /// A loose gem of the plain size and cut.
    fn gem(content: &Content, gem: &str) -> Stack {
        let gems = content.gems().expect("gems");
        let material = content.material_id(gem).expect(gem);
        let part = ItemPart::gem(content, None, material, gems.plain());
        Stack::one(gems.object).with_parts(vec![part])
    }

    #[test]
    fn weights_read_like_a_person_would_say_them() {
        assert_eq!(format_weight(0), "0 g");
        assert_eq!(format_weight(450), "450 g");
        assert_eq!(format_weight(999), "999 g");
        assert_eq!(format_weight(1000), "1 kg");
        assert_eq!(format_weight(1250), "1.3 kg");
        assert_eq!(format_weight(1949), "1.9 kg");
        assert_eq!(format_weight(70_000), "70 kg");
        assert_eq!(format_weight(71_960), "72 kg");
    }

    #[test]
    fn stackable_objects_join_and_others_stay_apart() {
        let c = content();
        let mut pile = vec![stack(&c, "dagger", 1)];
        add_to_pile(
            &mut pile,
            &[stack(&c, "arrow", 5), stack(&c, "dagger", 1)],
            &c,
        );
        assert_eq!(
            pile,
            [
                stack(&c, "arrow", 5),
                stack(&c, "dagger", 1),
                stack(&c, "dagger", 1)
            ]
        );
        // More arrows join the stack where it lies; the top stays the top.
        add_to_pile(&mut pile, &[stack(&c, "arrow", 7)], &c);
        assert_eq!(pile.len(), 3);
        assert_eq!(pile[0], stack(&c, "arrow", 12));

        let mut inventory = Vec::new();
        add_to_inventory(&mut inventory, stack(&c, "gold_coin", 3), &c);
        add_to_inventory(&mut inventory, stack(&c, "gold_coin", 4), &c);
        add_to_inventory(&mut inventory, stack(&c, "dagger", 1), &c);
        add_to_inventory(&mut inventory, stack(&c, "dagger", 1), &c);
        assert_eq!(inventory.len(), 3);
        assert_eq!(inventory[0].count, 7);
    }

    #[test]
    fn food_joins_whatever_its_age_and_keeps_the_oldest() {
        let c = content();
        let made = |count, hour: u64| Stack {
            died: Some(GameTime::from_seconds(hour * 3600)),
            ..stack(&c, "bread", count)
        };
        let mut inventory = vec![made(2, 10)];
        add_to_inventory(&mut inventory, made(1, 4), &c);
        add_to_inventory(&mut inventory, made(3, 20), &c);
        assert_eq!(inventory, [made(6, 4)], "one stack, as old as the oldest");
        let mut pile = vec![made(1, 30)];
        add_to_pile(&mut pile, &[made(2, 12)], &c);
        assert_eq!(pile, [made(3, 12)]);
        // A loaf that does not know its age yet takes the stack's.
        add_to_pile(&mut pile, &[stack(&c, "bread", 1)], &c);
        assert_eq!(pile, [made(4, 12)]);
        // Corpses never join: each keeps the moment its creature died.
        let rat = c.object_id("rat_corpse").expect("a rat corpse");
        let hour = |h: u64| GameTime::from_seconds(h * 3600);
        let mut pile = vec![Stack::corpse(rat, hour(1))];
        add_to_pile(&mut pile, &[Stack::corpse(rat, hour(2))], &c);
        assert_eq!(pile.len(), 2);
    }

    #[test]
    fn an_inventory_holds_any_number_of_stacks() {
        let c = content();
        let mut inventory = Vec::new();
        for _ in 0..40 {
            add_to_inventory(&mut inventory, stack(&c, "dagger", 1), &c);
        }
        assert_eq!(inventory.len(), 40);
        // A stackable object joins its stack wherever it is, past the 26th too.
        add_to_inventory(&mut inventory, stack(&c, "arrow", 1), &c);
        add_to_inventory(&mut inventory, stack(&c, "arrow", 3), &c);
        assert_eq!((inventory.len(), inventory[40].count), (41, 4));
    }

    #[test]
    fn weights_and_phrases() {
        let c = content();
        let dagger = stack(&c, "dagger", 1);
        let arrows = stack(&c, "arrow", 12);
        let each = u64::from(c.object(arrows.object).weight);
        assert_eq!(arrows.weight(&c), 12 * each);
        let (d, a) = (|| dagger.clone(), || arrows.clone());
        assert_eq!(weight_of(&[d(), a()], &c), dagger.weight(&c) + 12 * each);
        assert_eq!(phrase(&[d()], Lore::plain(&c)), "a dagger");
        assert_eq!(
            phrase(&[d(), a()], Lore::plain(&c)),
            "a dagger and 12 arrows"
        );
        let ruby = gem(&c, "ruby");
        assert_eq!(
            phrase(&[d(), a(), ruby.clone()], Lore::plain(&c)),
            "a dagger, 12 arrows, and a ruby"
        );
        assert_eq!(
            phrase(&[d(), a(), ruby, d()], Lore::plain(&c)),
            "15 objects"
        );
        let w = Weight {
            base: 70_000,
            carried: 1_500,
        };
        assert_eq!(w.total(), 71_500);
    }
}
