//! Starting kits (spec §8 "Character creation"): what a new character carries, wields, and wears
//! when a run starts. The data is [`crate::content::Kit`]: the class's kit, then the race's, then
//! the birth sign's animal's and element's (a hook for the horoscope; none give anything yet).
//!
//! [`roll`] makes a character's kit from the stream `character/kit/<race>,<gender>,<class>`
//! (its own, so no other roll of character creation moves), going through the kits' lines in
//! that order and drawing only for a line that applies (its `trained` and `gender` hold): one
//! draw to pick among its alternatives when it has several, then, for a `magic` line, the loot
//! draws that make it ([`loot::make`] at item level [`ITEM_LEVEL`]). Each item goes into the pack
//! (a stackable one joins a stack of the same), and a `wield` or `wear` line's item into the first
//! free slot it fits; one that finds none stays in the pack. Then come the class's random spell
//! scrolls, if it has any (`random_scrolls`, the casters'): its count rolled, then one draw per
//! scroll among the spells below its level, each carried. Every spell scroll a kit gives, a
//! line's or a random one, is its grimoire instead by the grimoires' chance (one draw a scroll,
//! after the scroll is made; spec §8 "Spells": "Learning"). Last the `fill` lines, the kits in the
//! same order: a fill line that applies draws its alternative, and its item is made and worn or
//! wielded only if a slot it fits is still free, else left out (a race's everyday clothes and
//! shoes, which yield to the class's). Nothing of any kit is worn or wielded in a slot the race
//! leaves bare (`bare`, the halflings' feet): such an item is left out after its draw.
//!
//! Who starts with it: a run whose character the player chose (New run's character creation,
//! `--character` or `--name` in the game and `warlock-cli`, a replay that says so) starts with the
//! kit and the scenario's `carry` on top; a run on the scenario's own character that nobody chose
//! (`--scenario` alone, tests, golden walks) with the scenario's `carry` alone
//! ([`starting_pack`], [`crate::world::World::start_run`]). Food learns when it was made when the
//! run starts (`rot::made_at`), magic items are unidentified like any loot, and every type with a
//! label that the starting pack holds, kit or scenario's (scrolls, potions, wands, rods), is
//! known from the start ([`known_types`], `World::begin`).

use crate::character::Character;
use crate::content::{Content, Keep, Kit, KitItem, ObjectId, ScenarioDef, SlotId, WeaponSkillId};
use crate::equipment::{self, Equipment};
use crate::magic::loot;
use crate::object::{self, Stack};
use crate::rng::Rng;

/// The item level a kit's magic items are made at.
pub const ITEM_LEVEL: u8 = 1;

/// A starting pack: what is carried, and of that what is worn and wielded.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pack {
    pub items: Vec<Stack>,
    pub equipment: Equipment,
}

impl Pack {
    /// Puts `stack` in the pack (it joins a stack of the same if it stacks, else goes last) and
    /// returns where it is.
    pub(crate) fn put(&mut self, stack: Stack, content: &Content) -> usize {
        let counts: Vec<u32> = self.items.iter().map(|s| s.count).collect();
        object::add_to_inventory(&mut self.items, stack, content);
        (0..counts.len())
            .find(|&i| self.items[i].count != counts[i])
            .unwrap_or(self.items.len() - 1)
    }

    /// Each stack as the player knows it, with where it is worn or wielded: "a dagger
    /// (wielded)", "a gold ring (magic) (on left hand)", "25 gold coins". Things carried that
    /// do not stack are counted together: "2 wool cloaks".
    pub fn names(&self, content: &Content) -> Vec<String> {
        let mut carried: Vec<Stack> = Vec::new();
        let mut names: Vec<(Option<usize>, String)> = Vec::new();
        for (i, stack) in self.items.iter().enumerate() {
            if let Some(mark) = equipment::mark(content, &self.equipment, i) {
                names.push((None, format!("{} ({mark})", stack.describe(content))));
                continue;
            }
            // What does not stack is one to a stack: the same thing again is the same but for
            // the count kept here.
            let same = |c: &Stack| {
                let mut c = c.clone();
                c.count = 1;
                !content.object(c.object).stackable && c == *stack
            };
            match carried.iter().position(same) {
                Some(at) => carried[at].count += stack.count,
                None => {
                    names.push((Some(carried.len()), String::new()));
                    carried.push(stack.clone());
                }
            }
        }
        (names.into_iter())
            .map(|(at, name)| at.map_or(name, |at| carried[at].describe(content)))
            .collect()
    }

    /// Wields or wears the stack at `item` in the first free slot it fits, unless it is already;
    /// it stays carried when no slot is free.
    pub(crate) fn equip(&mut self, item: usize, content: &Content) {
        let object = self.items[item].object;
        if self.equipment.holds(item) {
            return;
        }
        if let Some(slot) = equipment::free_slot(content, &self.equipment, object) {
            let slots = equipment::slots_taken(content, object, slot);
            self.equipment.put(&slots, item);
        }
    }
}

/// The kit of `character`, whose class trained the weapon skills and styles `trained` (its fixed
/// ranks and the sets of its choices that were picked), in a run with `seed`: see the module
/// docs.
pub fn roll(
    content: &Content,
    character: &Character,
    trained: &[WeaponSkillId],
    seed: u64,
) -> Pack {
    let spec = &character.spec;
    let mut rng = Rng::stream(seed, &format!("character/kit/{spec}"));
    let mut pack = Pack::default();
    let bare = content.race(&spec.race).map_or(&[][..], |r| &r.bare[..]);
    let kits = kits_of(content, character);
    let lines = || (kits.iter()).flat_map(|k| k.items());
    for line in lines().filter(|l| !l.fill) {
        if line.applies(spec.gender, trained) {
            give(content, &mut pack, &mut rng, line, trained, bare);
        }
    }
    // The class's random spell scrolls, after every other line, so no line's draw moves.
    if let Some(random) = content
        .class(&spec.class)
        .and_then(|c| c.random_scrolls.as_ref())
    {
        for _ in 0..random.count.roll(&mut rng) {
            let scroll = random.scrolls[rng.index(random.scrolls.len())];
            let scroll = content.grimoire_or(&mut rng, scroll);
            pack.put(Stack::new(scroll, 1), content);
        }
    }
    // Last what fills the slots still free.
    for line in lines().filter(|l| l.fill) {
        if line.applies(spec.gender, trained) {
            give(content, &mut pack, &mut rng, line, trained, bare);
        }
    }
    pack
}

/// Gives `line` to `pack`: its alternative drawn, then each of its items made and put in,
/// worn or wielded as it says. Left out after the draw: an item worn or wielded in a slot of
/// `bare`, and a `fill` line's when no slot it fits is free.
fn give(
    content: &Content,
    pack: &mut Pack,
    rng: &mut Rng,
    line: &KitItem,
    trained: &[WeaponSkillId],
    bare: &[SlotId],
) {
    let picked = line.pick(content, rng, trained);
    for stack in &line.one_of[picked] {
        if line.keep != Keep::Carry {
            let slot = equipment::free_slot(content, &Equipment::default(), stack.object);
            let free = equipment::free_slot(content, &pack.equipment, stack.object);
            if slot.is_some_and(|s| bare.contains(&s)) || (line.fill && free.is_none()) {
                continue;
            }
        }
        let stack = match line.magic {
            Some(grade) => {
                let (object, count) = (stack.object, stack.count);
                loot::make(content, rng, object, count, ITEM_LEVEL, grade)
            }
            None => {
                let mut plain = stack.clone();
                plain.draw_wand_rung(content, rng);
                plain
            }
        };
        for stack in with_grimoires(content, rng, stack) {
            let at = pack.put(stack, content);
            if line.keep != Keep::Carry {
                pack.equip(at, content);
            }
        }
    }
}

/// `stack` as a kit gives it (spec §8 "Spells": "Learning"): each spell scroll in it is its
/// grimoire with the grimoires' chance, one draw a scroll, so a stack may part in two (the
/// scrolls, then the grimoires); anything else as it is, with no draw.
fn with_grimoires(content: &Content, rng: &mut Rng, stack: Stack) -> Vec<Stack> {
    let Some(grimoire) = content.grimoires().of(stack.object) else {
        return vec![stack];
    };
    let books = (0..stack.count)
        .filter(|_| content.grimoire_or(rng, stack.object) == grimoire)
        .count() as u32;
    let scrolls = Stack::new(stack.object, stack.count - books);
    (scrolls.count > 0)
        .then_some(scrolls)
        .into_iter()
        .chain((books > 0).then(|| Stack::new(grimoire, books)))
        .collect()
}

/// The types a run starts knowing (spec §8 "Scrolls", "Potions"): every type with a label that
/// the starting pack `items` holds, kit or scenario's alike (scrolls, potions, wands, rods), so
/// whatever the player sets out with shows by its name.
pub fn known_types<'a>(
    content: &'a Content,
    items: impl IntoIterator<Item = &'a Stack> + 'a,
) -> impl Iterator<Item = ObjectId> + 'a {
    (items.into_iter())
        .map(|s| s.object)
        .filter(|&o| is_labelled(content, o))
}

/// Whether objects of type `object` go by a label until identified: a scroll, a potion, a wand,
/// or a rod (`scroll_label.rs`).
pub fn is_labelled(content: &Content, object: ObjectId) -> bool {
    let def = content.object(object);
    def.read.is_some() || def.drink.is_some() || def.wand.is_some() || def.rod.is_some()
}

/// The kits a character's starting kit is made of, in order: their class's, their race's, then
/// their birth sign's animal's and element's. Content without the class or race gives none of
/// it.
pub fn kits_of<'c>(content: &'c Content, character: &Character) -> Vec<&'c Kit> {
    let spec = &character.spec;
    let mut kits = Vec::new();
    kits.extend(content.class(&spec.class).map(|c| &c.kit));
    kits.extend(content.race(&spec.race).map(|r| &r.kit));
    let sign = (content.horoscope()).zip(character.sign.as_ref());
    if let Some((animal, element)) = sign.and_then(|(h, s)| h.parts(s)) {
        kits.extend([&animal.kit, &element.kit]);
    }
    kits
}

/// What a run of `scenario` starts the player with: the character's `kit`, if they start with
/// it, else what the scenario's own character has ready (its `wield`: the lantern), then the
/// scenario's `carry` on top (stackable things joining the kit's).
pub fn starting_pack(content: &Content, scenario: &ScenarioDef, kit: Option<&Pack>) -> Pack {
    let mut pack = kit.cloned().unwrap_or_else(|| {
        let mut ready = Pack::default();
        for stack in &scenario.wield {
            let item = ready.put(stack.clone(), content);
            ready.equip(item, content);
        }
        ready
    });
    for stack in &scenario.carry {
        pack.put(stack.clone(), content);
    }
    pack
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::burden::{self, Burden};
    use crate::character::{CharacterSpec, Gender, Start};
    use crate::content::{ClassDef, ContentFiles, KitItem, Rarity, TalentEffect};
    use crate::replay::Replay;
    use crate::stats::{AVERAGE, SCORES, Stat};
    use crate::world::World;
    use std::sync::{Arc, LazyLock};

    static CONTENT: LazyLock<Arc<Content>> =
        LazyLock::new(|| Arc::new(Content::builtin().expect("built-in content")));

    fn start(content: &Content, character: &str, seed: u64) -> Start {
        let spec: CharacterSpec = character.parse().expect("a character");
        let date = content.scenario("world").expect("world").start;
        spec.start(content, seed, date).expect("starts")
    }

    fn id(name: &str) -> crate::content::ObjectId {
        CONTENT.object_id(name).unwrap_or_else(|| panic!("{name}"))
    }

    /// The objects of a pack in order, by id, with where they are.
    fn held(pack: &Pack) -> Vec<(String, Option<String>)> {
        (pack.items.iter().enumerate())
            .map(|(i, s)| {
                let slots = pack.equipment.slots_of(i);
                let slot = slots.first().map(|&s| CONTENT.slot(s).id.clone());
                (CONTENT.object(s.object).id.clone(), slot)
            })
            .collect()
    }

    fn style(name: &str) -> WeaponSkillId {
        CONTENT.weapon_skills().id(name).expect("a style")
    }

    /// The seeds among the first 40 whose thief's style choice picked `picked` over `other`.
    fn thieves_with(picked: &str, other: &str) -> Vec<u64> {
        (0..40)
            .filter(|&seed| {
                let ranks = start(&CONTENT, "human,female,thief", seed).weapon_skills;
                ranks.rank(style(picked)) > ranks.rank(style(other))
            })
            .collect()
    }

    #[test]
    fn twin_blades_wield_two_daggers_and_the_duelist_keeps_a_stiletto() {
        let twins = thieves_with("twin_blades", "duelist");
        let duelists = thieves_with("duelist", "twin_blades");
        assert!(!twins.is_empty() && !duelists.is_empty());
        for seed in twins {
            let pack = start(&CONTENT, "human,female,thief", seed).kit;
            let held = held(&pack);
            assert_eq!(held[0], ("dagger".into(), Some("main_hand".into())));
            assert_eq!(held[1], ("dagger".into(), Some("off_hand".into())));
            assert!(!held.iter().any(|(o, _)| o == "stiletto"), "{held:?}");
        }
        for seed in duelists {
            let pack = start(&CONTENT, "human,female,thief", seed).kit;
            let held = held(&pack);
            assert_eq!(held[0], ("dagger".into(), Some("main_hand".into())));
            assert_eq!(held[1], ("stiletto".into(), None));
            assert_eq!(held.iter().filter(|(o, _)| o == "dagger").count(), 1);
        }
    }

    #[test]
    fn the_berserkers_great_weapon_follows_the_weapon_skill_picked() {
        let axes = CONTENT.weapon_skills().id("axes").expect("axes");
        let blades = CONTENT
            .weapon_skills()
            .id("great_blades")
            .expect("great blades");
        let mut seen = [false; 2];
        for seed in 0..30 {
            let s = start(&CONTENT, "human,male,berserker", seed);
            let held = held(&s.kit);
            let weapon = &held[0];
            assert_eq!(weapon.1.as_deref(), Some("main_hand"), "{held:?}");
            match weapon.0.as_str() {
                "great_axe" => {
                    seen[0] = true;
                    assert!(s.weapon_skills.rank(axes) >= 2);
                }
                "great_sword" => {
                    seen[1] = true;
                    assert!(s.weapon_skills.rank(blades) >= 2);
                }
                other => panic!("{other}"),
            }
            // Two-handed: in both hands, and only one great weapon.
            assert_eq!(s.kit.equipment.slots_of(0).len(), 2);
        }
        assert_eq!(seen, [true, true]);
    }

    #[test]
    fn choices_follow_the_seed_and_are_the_same_every_time() {
        let scroll = |seed| {
            let pack = start(&CONTENT, "gnome,other,wizard", seed).kit;
            let spells = [
                "scroll_of_fireball",
                "scroll_of_frost",
                "scroll_of_lightning",
            ];
            // A grimoire counts as its scroll: one in ten is (`with_grimoires`).
            let found: Vec<_> = (pack.items.iter())
                .map(|s| CONTENT.object(as_scroll(s.object)).id.as_str())
                .filter(|o| spells.contains(o))
                .map(str::to_string)
                .collect();
            // The kit's pick, and now and then the same spell's scroll among the random low
            // ones (frost is below level 10).
            assert!((1..=2).contains(&found.len()), "{found:?}");
            found
        };
        let picked: std::collections::BTreeSet<String> = (0..40).flat_map(scroll).collect();
        assert_eq!(picked.len(), 3, "{picked:?}");
        for seed in 0..10 {
            assert_eq!(scroll(seed), scroll(seed));
            let a = start(&CONTENT, "dwarf,male,aristocrat", seed);
            let b = start(&CONTENT, "dwarf,male,aristocrat", seed);
            assert_eq!(a.kit, b.kit);
        }
    }

    /// `text` without its `kit` lines (one line or a block up to its closing `]`) and its
    /// `random_scrolls`.
    fn without_kits(text: &str) -> String {
        let mut out = String::new();
        let mut in_kit = false;
        for line in text.lines() {
            if in_kit {
                in_kit = line != "]";
                continue;
            }
            if line.starts_with("random_scrolls = ") {
                continue;
            }
            if line.starts_with("kit = [") {
                in_kit = !line.ends_with(']');
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    #[test]
    fn kits_move_no_other_roll() {
        let mut files = ContentFiles::builtin();
        for table in ["races", "classes"] {
            files.edit_table(table, without_kits);
        }
        let plain = Content::load(&files).expect("valid without kits");
        assert!(plain.classes().iter().all(|c| c.kit.is_empty()));
        for race in &CONTENT.races().list {
            for class in CONTENT.classes() {
                for gender in Gender::ALL {
                    let character = format!("{},{},{}", race.id, gender.id(), class.id);
                    for seed in 0..3 {
                        let with = start(&CONTENT, &character, seed);
                        let without = start(&plain, &character, seed);
                        assert!(without.kit.items.is_empty());
                        assert!(!with.kit.items.is_empty(), "{character}");
                        assert_eq!(
                            Start {
                                kit: Pack::default(),
                                ..with
                            },
                            without,
                            "{character} {seed}"
                        );
                    }
                }
            }
        }
    }

    /// The trained sets a class may roll: its fixed ranks and each pick of its choices.
    fn trained_sets(class: &ClassDef) -> Vec<Vec<WeaponSkillId>> {
        let ids = |set: &[(WeaponSkillId, crate::dice::Dice)]| -> Vec<WeaponSkillId> {
            set.iter().map(|&(id, _)| id).collect()
        };
        let picks = |choice: &[Vec<(WeaponSkillId, crate::dice::Dice)>]| match choice {
            [] => vec![Vec::new()],
            sets => sets.iter().map(|s| ids(s)).collect(),
        };
        let mut out = Vec::new();
        for weapon in picks(&class.weapon_skill_choice) {
            for style in picks(&class.style_choice) {
                let mut set = ids(&class.weapon_skills);
                set.extend(ids(&class.styles));
                set.extend(weapon.iter().copied());
                set.extend(style.iter().copied());
                out.push(set);
            }
        }
        out
    }

    fn lines<'c>(content: &'c Content, start: &Start) -> Vec<&'c KitItem> {
        kits_of(content, &start.character)
            .into_iter()
            .flat_map(|k| k.items())
            .collect()
    }

    #[test]
    fn every_kit_item_worn_or_wielded_finds_its_slot_unless_it_is_taken() {
        // A race's bow or gauntlets may find the slot taken by the class's own (an elven ranger,
        // a dwarven paladin): then it is carried. Otherwise it is worn or wielded.
        for race in &CONTENT.races().list {
            for class in CONTENT.classes() {
                for gender in Gender::ALL {
                    let character = format!("{},{},{}", race.id, gender.id(), class.id);
                    let s = start(&CONTENT, &character, 1);
                    for trained in trained_sets(class) {
                        let pack = roll(&CONTENT, &s.character, &trained, 1);
                        let objects = |carry: bool| -> Vec<_> {
                            (lines(&CONTENT, &s).into_iter())
                                .filter(|l| (l.keep == Keep::Carry) == carry)
                                .filter(|l| l.applies(gender, &trained))
                                .flat_map(|l| l.one_of.iter().flatten().map(|s| s.object))
                                .collect()
                        };
                        // (Only what no line carries: a half-orc craftsperson wields a hand axe
                        // and carries the race's.)
                        let (kept, carried) = (objects(false), objects(true));
                        for (i, stack) in pack.items.iter().enumerate() {
                            let only_kept =
                                kept.contains(&stack.object) && !carried.contains(&stack.object);
                            if only_kept && !pack.equipment.holds(i) {
                                let free =
                                    equipment::free_slot(&CONTENT, &pack.equipment, stack.object);
                                assert_eq!(free, None, "{character}: {pack:?}");
                            }
                        }
                        assert!(!pack.equipment.items().is_empty(), "{character}");
                    }
                }
            }
        }
    }

    #[test]
    fn every_kit_dresses_the_body_and_the_feet_but_a_bare_races() {
        // Every race, class, gender, and pick, over a few seeds: something worn on the body, and
        // on the feet unless the race goes barefoot (the halflings), when nothing is.
        let slot = |id: &str| CONTENT.slot_id(id).expect("a slot");
        let (body, feet) = (slot("body"), slot("feet"));
        let halfling = CONTENT.race("halfling").expect("halflings");
        assert_eq!(halfling.bare, [feet]);
        for race in &CONTENT.races().list {
            for class in CONTENT.classes() {
                for gender in Gender::ALL {
                    let character = format!("{},{},{}", race.id, gender.id(), class.id);
                    for seed in 0..6 {
                        let s = start(&CONTENT, &character, seed);
                        for trained in trained_sets(class) {
                            let pack = roll(&CONTENT, &s.character, &trained, seed);
                            assert!(pack.equipment.get(body).is_some(), "{character}: {pack:?}");
                            let shod = pack.equipment.get(feet).is_some();
                            assert_eq!(shod, race.bare.is_empty(), "{character}: {pack:?}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn clothing_neither_defends_nor_protects_and_is_never_loot() {
        let clothing = CONTENT.item_class_id("clothing").expect("clothing");
        let clothes: Vec<_> = (CONTENT.object_defs().iter())
            .filter(|o| o.class == clothing)
            .collect();
        assert!(clothes.len() >= 20, "{}", clothes.len());
        for def in clothes {
            assert_eq!((def.armour, def.protection), (0, 0), "{}", def.id);
            assert!(def.kind.is_none() && def.slot.is_some(), "{}", def.id);
        }
    }

    #[test]
    fn a_races_fill_lines_dress_one_whose_class_left_the_slot_free() {
        // The commoner wears nothing of their class but a club in hand: every fill line of
        // their race puts one of its alternatives on.
        for race in &CONTENT.races().list {
            let fills: Vec<&KitItem> = race.kit.items().iter().filter(|l| l.fill).collect();
            assert!(!fills.is_empty(), "{}", race.id);
            let character = format!("{},female,commoner", race.id);
            for seed in 0..4 {
                let s = start(&CONTENT, &character, seed);
                let pack = roll(&CONTENT, &s.character, &[], seed);
                for line in &fills {
                    let worn = (line.one_of.iter().flatten()).any(|alt| {
                        (pack.items.iter().enumerate())
                            .any(|(i, st)| st.object == alt.object && pack.equipment.holds(i))
                    });
                    assert!(worn, "{character}: {line:?} in {pack:?}");
                }
            }
        }
    }

    /// The lowest score in `stat` a character could start with: every die at its least, the
    /// worst birth sign for it, the gender's +1 only where it is sure.
    fn least_score(character: &Start, content: &Content, stat: Stat) -> u8 {
        let spec = &character.character.spec;
        let (race, class) = (spec.race(content).unwrap(), spec.class(content).unwrap());
        let mut by = race.stats[stat].min() + class.stats[stat].min();
        by += i32::from(spec.gender == Gender::Male && stat == Stat::Strength);
        if let Some(h) = content.horoscope() {
            by += (h.animals.iter())
                .map(|a| a.stats[stat].min())
                .min()
                .unwrap_or(0);
            by += (h.elements.iter())
                .map(|e| e.stats[stat].min())
                .min()
                .unwrap_or(0);
        }
        (i32::from(AVERAGE) + by).clamp(i32::from(*SCORES.start()), i32::from(*SCORES.end())) as u8
    }

    /// An ordinary character's score in `stat`: every die of their race and class at its mean,
    /// no birth sign, no gender.
    fn ordinary_score(character: &Start, content: &Content, stat: Stat) -> u8 {
        let spec = &character.character.spec;
        let (race, class) = (spec.race(content).unwrap(), spec.class(content).unwrap());
        let mean = |min: i32, max: i32| (min + max) / 2;
        let by = mean(race.stats[stat].min(), race.stats[stat].max())
            + mean(class.stats[stat].min(), class.stats[stat].max());
        (i32::from(AVERAGE) + by).clamp(i32::from(*SCORES.start()), i32::from(*SCORES.end())) as u8
    }

    /// The most a kit line could weigh: its heaviest alternative.
    /// The slot a line's item is worn or wielded in, if it is (its first alternative's).
    fn slot_of(line: &KitItem) -> Option<SlotId> {
        let object = line.one_of[0][0].object;
        (line.keep != Keep::Carry)
            .then(|| equipment::free_slot(&CONTENT, &Equipment::default(), object))
            .flatten()
    }

    fn heaviest(line: &KitItem) -> u64 {
        (line.one_of.iter())
            .map(|stacks| stacks.iter().map(|s| s.weight(&CONTENT)).sum::<u64>())
            .max()
            .unwrap_or(0)
    }

    /// The most a class's random spell scrolls could weigh: its most of the heaviest.
    fn random_scrolls(class: &ClassDef) -> u64 {
        class.random_scrolls.as_ref().map_or(0, |r| {
            let heaviest = (r.scrolls.iter())
                .map(|&o| Stack::one(o).weight(&CONTENT))
                .max()
                .unwrap_or(0);
            heaviest * u64::try_from(r.count.max()).expect("a count")
        })
    }

    /// The most the heritage talents (#697) could add for `race` carrying `gold` coins: Family
    /// wealth's gold and most gems, and Father's sword's heirloom.
    fn heritage(race: &str, gold: u64) -> u64 {
        let mut most = 0;
        for (_, def) in CONTENT.talents().iter() {
            for effect in &def.effects {
                most += match effect {
                    TalentEffect::StartingGold {
                        percent,
                        at_least,
                        at_most,
                    } => {
                        let more = (gold * u64::from(*percent) / 100)
                            .clamp(u64::from(*at_least), u64::from(*at_most));
                        let coin = CONTENT.object_id(crate::content::GOLD).expect("gold");
                        more * Stack::one(coin).weight(&CONTENT)
                    }
                    TalentEffect::StartingGems { max, .. } => {
                        let gem = CONTENT.gems().expect("gems").object;
                        u64::from(*max) * Stack::one(gem).weight(&CONTENT)
                    }
                    TalentEffect::Heirloom {
                        weapons, fallback, ..
                    } => {
                        let weapon = weapons.get(race).unwrap_or(fallback);
                        Stack::one(id(weapon)).weight(&CONTENT)
                    }
                    _ => 0,
                };
            }
        }
        most
    }

    /// The heaviest line of the background stories: its items (their heaviest alternatives)
    /// and its gold (a story is a line a chapter, and most are plain).
    fn heaviest_story_line() -> u64 {
        use crate::content::StoryEffect;
        let Some(b) = CONTENT.background() else {
            return 0;
        };
        let coin = Stack::one(CONTENT.object_id(crate::content::GOLD).expect("gold"));
        let line_weight = |line: &crate::content::StoryLine| -> u64 {
            (line.effects.iter())
                .map(|e| match e {
                    StoryEffect::Item(item) => heaviest(item),
                    StoryEffect::Gold(dice) => dice.max().max(0) as u64 * coin.weight(&CONTENT),
                    _ => 0,
                })
                .sum()
        };
        (b.chapters.iter().flat_map(|c| &c.lines))
            .map(line_weight)
            .max()
            .unwrap_or(0)
    }

    #[test]
    fn nobody_starts_strained_and_only_wealth_or_armour_burdens() {
        // Every race, gender, and class, with the heaviest kit their class's picks allow and the
        // heaviest pack a scenario adds on top: the weakest such character is never Strained,
        // and an ordinary one starts Unburdened, but for the jokes' wealth and the armour of a
        // small warrior or paladin (#695: a player may toss what is too heavy).
        let carry = (CONTENT.scenarios())
            .map(|s| s.carry.iter().map(|st| st.weight(&CONTENT)).sum::<u64>())
            .max()
            .unwrap_or(0);
        let coin = CONTENT.object_id(crate::content::GOLD).expect("gold");
        let coins = |stacks: &[Stack]| -> u64 {
            (stacks.iter())
                .filter(|st| st.object == coin)
                .map(|st| u64::from(st.count))
                .sum()
        };
        let carried_gold = (CONTENT.scenarios())
            .map(|s| coins(&s.carry))
            .max()
            .unwrap_or(0);
        let story = heaviest_story_line();
        assert!(story > 0, "the stories give things");
        let jokes = ["aristocrat", "merchant"];
        let martial = ["paladin", "warrior"];
        for race in &CONTENT.races().list {
            let size = CONTENT.size(race.size);
            // The frail: a race whose Toughness falls short (elves and drow), whose shoes on top
            // of a warrior's armour may tip them over.
            let frail = race.stats[Stat::Toughness].max() < 0;
            for class in CONTENT.classes() {
                let may_burden = jokes.contains(&class.id.as_str())
                    || ((size.id == "small" || frail) && martial.contains(&class.id.as_str()));
                for gender in Gender::ALL {
                    let character = format!("{},{},{}", race.id, gender.id(), class.id);
                    let s = start(&CONTENT, &character, 1);
                    let limit = |score: fn(&Start, &Content, Stat) -> u8| {
                        let st = score(&s, &CONTENT, Stat::Strength);
                        burden::limit(st, score(&s, &CONTENT, Stat::Toughness), size.carry)
                    };
                    let (weakest, ordinary) = (limit(least_score), limit(ordinary_score));
                    for trained in trained_sets(class) {
                        // A fill line counts unless another line takes its slot for sure, and
                        // nothing for a bare slot does.
                        let applying: Vec<&KitItem> = (lines(&CONTENT, &s).into_iter())
                            .filter(|l| l.applies(gender, &trained))
                            .filter(|l| slot_of(l).is_none_or(|s| !race.bare.contains(&s)))
                            .collect();
                        let taken: Vec<SlotId> = (applying.iter())
                            .filter(|l| !l.fill)
                            .filter_map(|l| slot_of(l))
                            .collect();
                        let kit: u64 = (applying.iter())
                            .filter(|l| !l.fill || slot_of(l).is_none_or(|s| !taken.contains(&s)))
                            .map(|l| heaviest(l))
                            .sum::<u64>()
                            + random_scrolls(class);
                        // The heritage talents' gold, gems, and heirloom on top, and the
                        // heaviest line of a background story (but for the jokes: their
                        // wealth and a weapon by fate may strain the weakest of them).
                        let told = if jokes.contains(&class.id.as_str()) {
                            0
                        } else {
                            story
                        };
                        let heirs = heritage(&race.id, coins(&s.kit.items) + carried_gold) + told;
                        let load = |limit, carried| {
                            let load = burden::Load {
                                carried,
                                limit,
                                ignores_burdened: race.ignores_burdened,
                                stout: false,
                            };
                            load.penalty()
                        };
                        // Without the heritage nobody is ever Strained, but Family wealth's 450
                        // coins (4.5 kg) and Father's sword, a player's own pick, may Strain the
                        // weakest of the small peoples; they never Overload anyone.
                        assert!(
                            load(weakest, kit + carry + told) < Burden::Strained,
                            "{character}: {kit} g + {told} g + {carry} g of {weakest}"
                        );
                        assert!(
                            load(weakest, kit + carry + heirs) < Burden::Overloaded,
                            "{character}: {kit} g + {heirs} g + {carry} g of {weakest}"
                        );
                        if !may_burden {
                            assert_eq!(
                                load(ordinary, kit + carry),
                                Burden::Unburdened,
                                "{character}: {kit} g + {carry} g of {ordinary}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn magic_kit_items_are_rolled_and_unidentified() {
        let ring = id("gold_ring");
        let rapier = id("rapier");
        let mut rapiers = [0; 2];
        for seed in 0..40 {
            let pack = start(&CONTENT, "human,female,aristocrat", seed).kit;
            let stack = |object| pack.items.iter().find(|s| s.object == object).unwrap();
            // The heirloom is always magic or rare, and nobody knows what it does yet.
            assert!(stack(ring).rarity() >= Rarity::Magic, "{seed}");
            assert!(!stack(ring).is_identified() && stack(ring).known_magic().is_none());
            assert!(stack(ring).describe(&CONTENT).contains("ring"));
            rapiers[usize::from(stack(rapier).rarity() > Rarity::Normal)] += 1;
        }
        // The rapier only perhaps.
        assert!(rapiers[0] > 0 && rapiers[1] > 0, "{rapiers:?}");
    }

    /// A grimoire as the spell scroll a kit names (`grimoire_of_X` is `scroll_of_X`); anything
    /// else as it is.
    fn as_scroll(object: ObjectId) -> ObjectId {
        let id = &CONTENT.object(object).id;
        (id.strip_prefix("grimoire_of_"))
            .and_then(|rest| CONTENT.object_id(&format!("scroll_of_{rest}")))
            .unwrap_or(object)
    }

    /// Whether `object` is read: a scroll, or a grimoire.
    fn readable(object: ObjectId) -> bool {
        let def = CONTENT.object(object);
        def.read.is_some() || def.grimoire.is_some()
    }

    #[test]
    fn a_kits_spell_scrolls_are_grimoires_one_in_ten() {
        let (mut scrolls, mut grimoires) = (0, 0);
        for seed in 0..400 {
            let pack = start(&CONTENT, "human,female,wizard", seed).kit;
            for stack in &pack.items {
                let def = CONTENT.object(stack.object);
                if def.grimoire.is_some() {
                    grimoires += stack.count;
                } else if matches!(def.read, Some(crate::content::Reading::Cast(_))) {
                    scrolls += stack.count;
                }
            }
        }
        let share = grimoires * 100 / (scrolls + grimoires);
        assert!(
            (6..=14).contains(&share),
            "{grimoires} of {}",
            scrolls + grimoires
        );
    }

    /// How many scrolls the kit lines of `s` give (each line's first alternative: the wizard's
    /// attack scroll is one whichever it is).
    fn fixed_scrolls(s: &Start) -> u32 {
        (lines(&CONTENT, s).into_iter())
            .flat_map(|l| &l.one_of[0])
            .filter(|st| CONTENT.object(st.object).read.is_some())
            .map(|st| st.count)
            .sum()
    }

    #[test]
    fn casters_start_with_random_low_spell_scrolls_within_their_dice() {
        // The owner's table (#1104): every other class gets none.
        let dice = |class: &str| match class {
            "wizard" | "witch" => (1, 3),
            "priest" | "druid" | "ranger" => (1, 2),
            "bard" => (1, 4),
            _ => (0, 0),
        };
        for class in CONTENT.classes() {
            let (least, most) = dice(&class.id);
            let mut seen = std::collections::BTreeSet::new();
            for seed in 0..120 {
                let s = start(&CONTENT, &format!("human,female,{}", class.id), seed);
                let scrolls: u32 = (s.kit.items.iter())
                    .filter(|st| readable(st.object))
                    .map(|st| st.count)
                    .sum();
                let extra = scrolls - fixed_scrolls(&s);
                assert!((least..=most).contains(&extra), "{}: {extra}", class.id);
                seen.insert(extra);
                // What a kit line does not name is a spell scroll below spell level 10.
                let named = |o| {
                    lines(&CONTENT, &s)
                        .iter()
                        .any(|l| l.one_of.iter().flatten().any(|st| st.object == o))
                };
                for stack in &s.kit.items {
                    let Some(crate::content::Reading::Cast(cast)) =
                        &CONTENT.object(as_scroll(stack.object)).read
                    else {
                        continue;
                    };
                    if !named(as_scroll(stack.object)) {
                        assert!(CONTENT.spell(cast.spell).level < 10, "{}", class.id);
                    }
                }
            }
            assert_eq!(seen, (least..=most).collect(), "{}", class.id);
        }
        // The pool: the scroll of each spell below level 10, once.
        let pool = &(CONTENT.class("wizard").expect("wizard").random_scrolls)
            .as_ref()
            .expect("the wizard's")
            .scrolls;
        let below = (CONTENT.spells().iter()).filter(|s| s.level < 10).count();
        assert_eq!(pool.len(), below);
        assert!(pool.contains(&id("scroll_of_light")) && !pool.contains(&id("scroll_of_fireball")));
        for class in ["paladin", "warrior"] {
            assert!(CONTENT.class(class).expect(class).random_scrolls.is_none());
        }
    }

    #[test]
    fn every_starting_scroll_and_potion_is_known() {
        // Every class, a few runs each: whatever is read or drunk in the starting pack (the
        // kit's lines, the random scrolls, a story's things) is known by type and named so.
        for class in CONTENT.classes() {
            for seed in 0..4 {
                let spec: CharacterSpec = (format!("elf,male,{}", class.id)).parse().expect("spec");
                let world = World::start_run(
                    CONTENT.clone(),
                    "world",
                    Some(seed),
                    Some(&spec),
                    None,
                    true,
                )
                .expect("world");
                let dump = crate::dump::ascii(&world, &[crate::dump::Layer::Inventory]);
                for stack in &world.state().player().inventory {
                    let def = CONTENT.object(stack.object);
                    if def.read.is_none() && def.drink.is_none() {
                        continue;
                    }
                    assert!(
                        world.state().known_scrolls.contains(&stack.object),
                        "{}: {}",
                        class.id,
                        def.id
                    );
                    // By its true name ("2 scrolls of shatter": what follows "of").
                    let kind = def.name.rsplit(" of ").next().expect("a name");
                    assert!(
                        dump.contains(&format!(" of {kind}")),
                        "{}: {} in\n{dump}",
                        class.id,
                        def.name
                    );
                }
            }
        }
    }

    #[test]
    fn a_scenarios_own_pack_is_known_by_type_too() {
        // A scenario whose own character sets out with a scroll and a potion: both known.
        let mut files = ContentFiles::builtin();
        let world = include_str!("../../../assets/campaigns/showcase/scenarios/world.toml");
        files.insert(
            "scenarios/packed.toml",
            &format!("{world}carry = [\"scroll_of_frost\", \"potion_of_healing\"]\n"),
        );
        let content = Arc::new(Content::load(&files).expect("valid"));
        let world = World::new(content.clone(), "packed", None).expect("world");
        assert!(!world.started_with_kit());
        let known = &world.state().known_scrolls;
        assert!(known.contains(&id("scroll_of_frost")) && known.contains(&id("potion_of_healing")));
        // Only what has a label: the lantern is not counted.
        assert_eq!(known.len(), 2, "{known:?}");
        let dump = crate::dump::ascii(&world, &[crate::dump::Layer::Inventory]);
        assert!(dump.contains("a scroll of frost"), "{dump}");
        assert!(dump.contains("a potion of healing"), "{dump}");
    }

    #[test]
    fn the_scenarios_pack_comes_on_top_or_alone() {
        // The world's ranger with a scenario's food on top.
        // Without background stories: the kit and the pack alone (a story's things come on top,
        // `world/background_tests.rs`).
        let mut files = ContentFiles::builtin();
        files.remove("data/background.toml");
        let world = include_str!("../../../assets/campaigns/showcase/scenarios/world.toml");
        files.insert(
            "scenarios/packed.toml",
            &format!("{world}carry = [\"ration*3\", \"dried_meat*2\"]\n"),
        );
        let content = Arc::new(Content::load(&files).expect("valid"));
        let ranger: CharacterSpec = "human,male,ranger".parse().expect("a character");
        let world = World::start_run(content.clone(), "packed", None, Some(&ranger), None, true)
            .expect("world");
        assert!(world.started_with_kit());
        let player = world.state().player();
        let s = start(&CONTENT, "human,male,ranger", 1);
        // The ranger's kit, first, then the world's rations; its dried meat joins the kit's.
        assert_eq!(player.equipment, s.kit.equipment);
        let count = |name: &str| {
            (player.inventory.iter())
                .filter(|st| st.object == id(name))
                .map(|st| st.count)
                .sum::<u32>()
        };
        assert_eq!(count("dried_meat"), 3 + 2);
        assert_eq!(count("ration"), 3);
        assert_eq!(count("short_bow"), 1);
        assert_eq!(player.inventory.len(), s.kit.items.len() + 1);
        // Food from the kit is as fresh as the run.
        let date = CONTENT.scenario("world").expect("world").start;
        for stack in &player.inventory {
            let food = CONTENT.object(stack.object).food.is_some();
            assert_eq!(stack.died, food.then_some(date), "{stack:?}");
        }
        // Nobody chose the scenario's own character: the scenario's pack alone, as tests and
        // golden walks start, the lantern of its `wield` ready first. So does a chosen one
        // without the kit (`with_character`).
        let packed = World::new(content.clone(), "packed", None).expect("world");
        let chosen =
            World::with_character(content.clone(), "packed", None, Some(&ranger)).expect("world");
        for world in [&packed, &chosen] {
            let player = world.state().player();
            assert!(!world.started_with_kit());
            assert_eq!(player.inventory.len(), 3, "{:?}", player.inventory);
            assert_eq!(player.inventory[0].object, id("lantern"));
            assert_eq!(player.ready_light(&CONTENT), Some(0));
        }
        let showcase = World::new(CONTENT.clone(), "showcase", None).expect("showcase");
        assert_eq!(
            showcase.state().player().inventory,
            [Stack::one(id("lantern"))]
        );
        // A chosen character in the showcase starts with the kit: there is no pack on top.
        let thief: CharacterSpec = "human,female,thief".parse().expect("a character");
        let kitted = World::start_run(content, "showcase", None, Some(&thief), None, true)
            .expect("showcase");
        let kit = start(&CONTENT, "human,female,thief", 1).kit;
        // (The same stacks; the food now knows when it was made.)
        let goods = |stacks: &[Stack]| -> Vec<(crate::content::ObjectId, u32)> {
            stacks.iter().map(|s| (s.object, s.count)).collect()
        };
        assert_eq!(goods(&kitted.state().player().inventory), goods(&kit.items));
        assert_eq!(kitted.state().player().equipment, kit.equipment);
    }

    #[test]
    fn genders_and_birth_signs_may_give_more() {
        let mut files = ContentFiles::builtin();
        let races = include_str!("../../../assets/data/races.toml").replace(
            "    \"gold_coin*5\",\n",
            "    { carry = \"candle\", gender = \"female\" },\n",
        );
        files.insert("data/races.toml", &races);
        let signs = include_str!("../../../assets/data/horoscope.toml").replace(
            "\n[[element]]\n",
            "\n[[element]]\nkit = [{ wear = \"iron_ring\" }]\n",
        );
        files.insert("data/horoscope.toml", &signs);
        let content = Content::load(&files).expect("valid");
        let has = |pack: &Pack, name: &str| {
            (pack.items.iter()).position(|s| content.object(s.object).id == name)
        };
        for seed in 0..5 {
            let woman = start(&content, "human,female,warrior", seed).kit;
            let man = start(&content, "human,male,warrior", seed).kit;
            assert!(has(&woman, "candle").is_some() && has(&man, "candle").is_none());
            // Every element gives a ring, worn, after the class's and the race's things.
            for pack in [&woman, &man] {
                let ring = has(pack, "iron_ring").expect("the sign's ring");
                assert!(pack.equipment.holds(ring));
                assert!(ring > has(pack, "bread").expect("the race's bread"));
            }
        }
    }

    #[test]
    fn a_run_with_a_kit_replays() {
        let spec: CharacterSpec = "dwarf,female,thief".parse().expect("a character");
        let mut world =
            World::start_run(CONTENT.clone(), "world", Some(7), Some(&spec), None, true)
                .expect("world");
        let mut replay = Replay::new(&world);
        assert!(replay.kit && replay.to_json().contains("\"kit\": true"));
        for action in
            crate::action::parse_actions("move:E remove:main_hand drop:a wait").expect("actions")
        {
            world.apply(&action);
            replay.push(&action, &world);
        }
        let (again, divergence) = replay.play(CONTENT.clone()).expect("plays");
        assert!(divergence.is_none(), "{divergence:?}");
        assert_eq!(again.state_hash(), world.state_hash());
        // The pick-axe readied in the Tool slot, as dwarves start.
        let player = world.state().player();
        let tool = CONTENT
            .slot_ids()
            .find(|(_, s)| s.id == "tool")
            .expect("a tool slot");
        let item = player.equipment.get(tool.0).expect("a tool");
        assert_eq!(player.inventory[item].object, id("pick_axe"));
        // A replay without the flag (from before kits, or of a run on the scenario's own
        // character) starts with the scenario's pack, as it was recorded.
        let plain = Replay {
            kit: false,
            ..replay
        };
        assert!(!plain.to_json().contains("\"kit\""));
        let start = plain.start(CONTENT.clone()).expect("starts");
        let lantern = (start.state().player().inventory.iter()).map(|s| s.object);
        assert_eq!(
            lantern.collect::<Vec<_>>(),
            [id("lantern")],
            "the world's `wield`"
        );
    }
}
