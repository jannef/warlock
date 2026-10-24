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
//! free slot it fits; one that finds none stays in the pack.
//!
//! Who starts with it: a run whose character the player chose (New run's character creation,
//! `--character` or `--name` in the game and `warlock-cli`, a replay that says so) starts with the
//! kit and the scenario's `carry` on top; a run on the scenario's own character that nobody chose
//! (`--scenario` alone, tests, golden walks) with the scenario's `carry` alone
//! ([`starting_pack`], [`crate::world::World::start_run`]). Food learns when it was made when the
//! run starts (`rot::made_at`), and magic items are unidentified like any loot.

use crate::character::Character;
use crate::content::{Content, Keep, Kit, ScenarioDef, WeaponSkillId};
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
    fn put(&mut self, stack: Stack, content: &Content) -> usize {
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
    fn equip(&mut self, item: usize, content: &Content) {
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
    for kit in kits_of(content, character) {
        for line in kit.items() {
            if !line.applies(spec.gender, trained) {
                continue;
            }
            let picked = match line.one_of.len() {
                0 | 1 => 0,
                n => rng.index(n),
            };
            for stack in &line.one_of[picked] {
                let stack = match line.magic {
                    Some(grade) => {
                        let (object, count) = (stack.object, stack.count);
                        loot::make(content, &mut rng, object, count, ITEM_LEVEL, grade)
                    }
                    None => stack.clone(),
                };
                let at = pack.put(stack, content);
                if line.keep != Keep::Carry {
                    pack.equip(at, content);
                }
            }
        }
    }
    pack
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
/// it, then the scenario's `carry` on top (stackable things joining the kit's).
pub fn starting_pack(content: &Content, scenario: &ScenarioDef, kit: Option<&Pack>) -> Pack {
    let mut pack = kit.cloned().unwrap_or_default();
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
    use crate::content::{ClassDef, ContentFiles, KitItem, Rarity};
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
    fn the_barbarians_great_weapon_follows_the_weapon_skill_picked() {
        let axes = CONTENT.weapon_skills().id("axes").expect("axes");
        let blades = CONTENT
            .weapon_skills()
            .id("great_blades")
            .expect("great blades");
        let mut seen = [false; 2];
        for seed in 0..30 {
            let s = start(&CONTENT, "human,male,barbarian", seed);
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
            let found: Vec<_> = (pack.items.iter())
                .map(|s| CONTENT.object(s.object).id.as_str())
                .filter(|o| spells.contains(o))
                .map(str::to_string)
                .collect();
            assert_eq!(found.len(), 1, "{found:?}");
            found[0].clone()
        };
        let picked: std::collections::BTreeSet<String> = (0..40).map(scroll).collect();
        assert_eq!(picked.len(), 3, "{picked:?}");
        for seed in 0..10 {
            assert_eq!(scroll(seed), scroll(seed));
            let a = start(&CONTENT, "dwarf,male,aristocrat", seed);
            let b = start(&CONTENT, "dwarf,male,aristocrat", seed);
            assert_eq!(a.kit, b.kit);
        }
    }

    /// `text` without its `kit` lines (one line or a block up to its closing `]`).
    fn without_kits(text: &str) -> String {
        let mut out = String::new();
        let mut in_kit = false;
        for line in text.lines() {
            if in_kit {
                in_kit = line != "]";
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
        for (path, text) in [
            (
                "data/races.toml",
                include_str!("../../../assets/data/races.toml"),
            ),
            (
                "data/classes.toml",
                include_str!("../../../assets/data/classes.toml"),
            ),
        ] {
            files.insert(path, &without_kits(text));
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
    fn every_kit_item_worn_or_wielded_finds_its_slot() {
        for race in &CONTENT.races().list {
            for class in CONTENT.classes() {
                for gender in Gender::ALL {
                    let character = format!("{},{},{}", race.id, gender.id(), class.id);
                    let s = start(&CONTENT, &character, 1);
                    for trained in trained_sets(class) {
                        let pack = roll(&CONTENT, &s.character, &trained, 1);
                        let kept = (lines(&CONTENT, &s).into_iter())
                            .filter(|l| l.keep != Keep::Carry && l.applies(gender, &trained))
                            .count();
                        assert_eq!(pack.equipment.items().len(), kept, "{character} {pack:?}");
                    }
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

    /// The most a kit line could weigh: its heaviest alternative.
    fn heaviest(line: &KitItem) -> u64 {
        (line.one_of.iter())
            .map(|stacks| stacks.iter().map(|s| s.weight(&CONTENT)).sum::<u64>())
            .max()
            .unwrap_or(0)
    }

    #[test]
    fn nobody_starts_strained_and_only_wealth_burdens() {
        // The weakest character of every race, gender, and class, with the heaviest kit their
        // class's picks allow and the heaviest pack a scenario adds on top.
        let carry = (CONTENT.scenarios())
            .map(|s| s.carry.iter().map(|st| st.weight(&CONTENT)).sum::<u64>())
            .max()
            .unwrap_or(0);
        let jokes = ["aristocrat", "merchant"];
        for race in &CONTENT.races().list {
            for class in CONTENT.classes() {
                for gender in Gender::ALL {
                    let character = format!("{},{},{}", race.id, gender.id(), class.id);
                    let s = start(&CONTENT, &character, 1);
                    let st = least_score(&s, &CONTENT, Stat::Strength);
                    let to = least_score(&s, &CONTENT, Stat::Toughness);
                    let limit = burden::limit(st, to, CONTENT.size(race.size).carry);
                    for trained in trained_sets(class) {
                        let kit: u64 = (lines(&CONTENT, &s).into_iter())
                            .filter(|l| l.applies(gender, &trained))
                            .map(heaviest)
                            .sum();
                        let load = |carried| burden::Load {
                            carried,
                            limit,
                            ignores_burdened: race.ignores_burdened,
                        };
                        let worst = load(kit + carry).penalty();
                        assert!(worst < Burden::Strained, "{character}: {kit} g of {limit}");
                        if !jokes.contains(&class.id.as_str()) {
                            assert_eq!(
                                load(kit + carry).penalty(),
                                Burden::Unburdened,
                                "{character}: {kit} g + {carry} g of {limit}"
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

    #[test]
    fn the_scenarios_pack_comes_on_top_or_alone() {
        // The world's ranger with a scenario's food on top.
        let mut files = ContentFiles::builtin();
        let world = include_str!("../../../assets/scenarios/world.toml");
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
        // golden walks start. So does a chosen one without the kit (`with_character`).
        let packed = World::new(content.clone(), "packed", None).expect("world");
        let chosen = World::with_character(content, "packed", None, Some(&ranger)).expect("world");
        for world in [&packed, &chosen] {
            let player = world.state().player();
            assert!(!world.started_with_kit());
            assert_eq!(player.inventory.len(), 2, "{:?}", player.inventory);
            assert!(player.equipment.is_empty());
        }
        let showcase = World::new(CONTENT.clone(), "showcase", None).expect("showcase");
        assert!(showcase.state().player().inventory.is_empty());
        // A chosen character in the showcase starts with the kit: there is no pack on top.
        let thief: CharacterSpec = "human,female,thief".parse().expect("a character");
        let kitted = World::start_run(CONTENT.clone(), "showcase", None, Some(&thief), None, true)
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
            "kit = [\"bread\"]",
            "kit = [\"bread\", { carry = \"candle\", gender = \"female\" }]",
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
        assert!(start.state().player().inventory.is_empty());
    }
}
