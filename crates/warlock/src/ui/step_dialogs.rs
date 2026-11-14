//! The dialogs and screens input opens before an action (spec §6): what a step asks first
//! (deep water, leaving a shop owing, a known trap, a hungry journey, an offer, an anvil's
//! screen), selling what is dropped in a shop, the Heal ability's choice, and a burst that may
//! catch the reader. `input` decides when to ask; these say what.

use crate::ui::{Aim, AimKind, Dialog, ScreenAction, ScreenName};
use warlock_core::ability::Ability;
use warlock_core::content::{LevelKind, Offer, RecipeUse, TerrainId};
use warlock_core::hunger::Hunger;
use warlock_core::perception::{CellView, PerceptionView};
use warlock_core::{Action, Dir, Positions};

/// The dialog a step in `dir` asks first, if any: diving into deep water, leaving a shop owing,
/// stepping onto a known trap, setting out hungry on the overworld, or an offer (see the
/// movement arm of `Gameplay::play`, `input/play.rs`).
pub(crate) fn step_dialog(view: &PerceptionView<'_>, dir: Dir) -> Option<Dialog> {
    (deep_water_dialog(view, dir))
        .or_else(|| leaving_dialog(view, dir))
        .or_else(|| trap_dialog(view, dir))
        .or_else(|| hunger_dialog(view, dir))
        .or_else(|| offer_dialog(view, dir))
}

/// The dialog a step in `dir` opens first, if it leaves the shop the player stands in owing
/// anything (spec §7 "Shops"). Out of its door: pay and leave in one action (offered only with
/// the gold; else the dialog says what they owe and have), or not, and the keeper turns them
/// back one step (declining and Esc send the step, which the rules turn back). Any other way
/// out (a wall dug open): pay and leave, leave without paying (theft), or stay.
fn leaving_dialog(view: &PerceptionView<'_>, dir: Dir) -> Option<Dialog> {
    let to = view.viewer().step(dir);
    let (shop, owed) = view.owed_on_leaving(to)?;
    let (keeper, gold) = (shop.keeper(), view.own_gold());
    let pay = format!("Pay {owed} gold and leave");
    let dialog = if shop.is_exit(to) {
        if gold >= owed {
            Dialog::new(shop.sign(), format!("You owe {keeper} {owed} gold."))
                .accept(pay, Action::Pay(Some(dir)))
                .decline("Don't pay", Action::Move(dir))
        } else {
            let have = match gold {
                0 => "no gold".to_string(),
                n => format!("only {n}"),
            };
            let text = format!(
                "You owe {keeper} {owed} gold and have {have}. Put something back, or bring \
                 more gold."
            );
            Dialog::new(shop.sign(), text).decline("Go back", Action::Move(dir))
        }
    } else {
        let text = format!(
            "You owe {keeper} {owed} gold. Leave this way without paying, and {keeper} will \
             call it theft."
        );
        let dialog = Dialog::new(shop.sign(), text);
        let dialog = match gold >= owed {
            true => dialog.accept(pay, Action::Pay(Some(dir))),
            false => dialog,
        };
        dialog
            .plain("Leave without paying", Action::Move(dir))
            .cancel("Stay", ScreenAction::Close)
    };
    Some(dialog)
}

/// The dialog dropping the inventory stacks `letters` opens first, if the shop the player stands
/// in would buy some of them (spec §7 "Shops"): sell them, or keep them (nothing is dropped).
pub(crate) fn sale_dialog(view: &PerceptionView<'_>, letters: &Positions) -> Option<Dialog> {
    let sale = view.sale(letters)?;
    let shop = view.shop_here()?;
    let what = warlock_core::object::phrase(&sale.stacks, view.lore());
    let text = format!("{} offers {} gold for {what}.", shop.keeper(), sale.offer);
    let sell = format!("Sell for {} gold", sale.offer);
    Some(
        Dialog::new(shop.sign(), text)
            .accept(sell, Action::Sell(letters.clone()))
            .cancel("Keep it", ScreenAction::Close),
    )
}

/// The Heal ability's choice when the player may treat several things (spec §8 "Skills"): one
/// option for each, in the order the rules list them, and a way out.
pub(crate) fn treat_dialog(ailments: &[warlock_core::heal::OwnAilment]) -> Dialog {
    let dialog = Dialog::new("Heal", "What do you tend to?");
    (ailments.iter())
        .fold(dialog, |d, a| d.plain(a.name.clone(), a.action.clone()))
        .cancel("Never mind", ScreenAction::Close)
}

/// The question before reading a scroll (or casting a spell, spec §8 "Class abilities") whose
/// burst may catch the player too, aimed by `aim` (as far as they know,
/// [`Aim::catches_reader`]): read (cast) it anyway, aim again (back at the same cursor, the
/// target remembered), or not.
pub(crate) fn burst_dialog(view: &PerceptionView<'_>, aim: Aim) -> Option<Dialog> {
    let noun = aim.catches_reader(view)?;
    let read = aim.action(view.content())?;
    let AimKind::Read(item) = aim.kind else {
        return Some(
            Dialog::new(
                "Too close",
                format!("The {noun} may catch you too. Cast it anyway?"),
            )
            .accept("Cast it", read)
            .decline("Aim again", ScreenAction::Aim(aim.kind))
            .cancel("Don't cast it", ScreenAction::Close),
        );
    };
    let stack = view.inventory().get(item)?;
    // An unknown scroll is a gamble (spec §8 "Scrolls"): nothing warns of what it may do.
    if view.lore().label(stack.object).is_some() {
        return None;
    }
    let name = &view.content().object(stack.object).name;
    Some(
        Dialog::new(
            "Too close",
            format!("The {noun} may catch you too. Read the {name} anyway?"),
        )
        .accept("Read it", read)
        .decline("Aim again", ScreenAction::Aim(aim.kind))
        .cancel("Don't read it", ScreenAction::Close),
    )
}

/// The dialog a step in `dir` opens first, if it goes onto a trap the player knows, where no one
/// they see stands (moving there would be an attack, or stop; spec §8 "Traps"): "Really step
/// onto the fireball trap?", to step onto it or stay back (the safe answer, selected first). A
/// door's trap is not stepped on (spec §8 "Trapped doors"), so a door never asks.
fn trap_dialog(view: &PerceptionView<'_>, dir: Dir) -> Option<Dialog> {
    let to = view.viewer().step(dir);
    let door = match view.cell(to) {
        CellView::Visible(t) | CellView::Remembered(t) => view.content().terrain(t).is_door(),
        CellView::Unknown => false,
    };
    let trap = view
        .trap(to)
        .filter(|_| !door && view.actor_at(to).is_none())?;
    let name = &view.content().trap(trap).name;
    Some(
        Dialog::new("Trap", format!("Really step onto the {name}?"))
            .accept("Step onto it", Action::Move(dir))
            .cancel("Stay back", ScreenAction::Close),
    )
}

/// The dialog a step in `dir` opens first, if it dives into deep water from anything else (spec §8
/// "Skills": Swim; [`PerceptionView::dives_at`]): always, no setting turns it off. To dive in or
/// stay back (the safe answer, selected first, and Esc's; it takes no time).
fn deep_water_dialog(view: &PerceptionView<'_>, dir: Dir) -> Option<Dialog> {
    view.dives_at(view.viewer().step(dir)).then(|| {
        Dialog::new("Deep water", "Really dive into the deep water?")
            .accept("Dive in", Action::Move(dir))
            .cancel("Stay back", ScreenAction::Close)
    })
}

/// The dialog a step in `dir` opens first, if it is a journey on the overworld for a player who
/// is Fainting or Starving (spec §8 "Hunger"), onto ground that may be walked, where no one they
/// see stands (moving there would be an attack, or stop): that it may kill them, to go on or
/// stay (the safe answer, selected first, and Esc's; it takes no time). Judged by the state
/// alone, never by what this step would cost, so it tells no number. Long fast is asked too: it
/// spares its fainting, not its starving.
fn hunger_dialog(view: &PerceptionView<'_>, dir: Dir) -> Option<Dialog> {
    if view.level_kind() != LevelKind::Overworld {
        return None;
    }
    let hunger = view.own_hunger()?;
    let text = match hunger {
        Hunger::Fainting => {
            "You are faint from hunger, and this journey may kill you. Go on without eating?"
        }
        Hunger::Starving => {
            "You are starving to death, and every step takes what life you have left. This \
             journey may well be your last. Go on without eating?"
        }
        _ => return None,
    };
    let to = view.viewer().step(dir);
    let walkable = match view.cell(to) {
        CellView::Visible(t) | CellView::Remembered(t) => view.content().terrain(t).walkable,
        CellView::Unknown => true,
    };
    if !walkable || view.actor_at(to).is_some() {
        return None;
    }
    Some(
        Dialog::new(hunger.name(), text)
            .accept("Go on", Action::Move(dir))
            .cancel("Stay", ScreenAction::Close),
    )
}

/// The feature a step in `dir` walks into and what it offers, if the player sees or remembers
/// it and no one they see stands there (moving there would be an attack, or stop).
fn offered_at(view: &PerceptionView<'_>, dir: Dir) -> Option<(TerrainId, Offer)> {
    let to = view.viewer().step(dir);
    let terrain = match view.cell(to) {
        CellView::Visible(t) | CellView::Remembered(t) => t,
        CellView::Unknown => return None,
    };
    let offer = view.content().offer(terrain)?;
    view.actor_at(to).is_none().then_some((terrain, offer))
}

/// The screen a step in `dir` opens instead, if any: walking into an anvil opens the profession
/// screen of Temper's skill, Metalcraft, with all its recipes, for one trained in it (the owner,
/// #1387); untrained, the step goes to the rules, which say what it needs, taking no time.
pub(crate) fn step_screen(view: &PerceptionView<'_>, dir: Dir) -> Option<ScreenName> {
    let (_, offer) = offered_at(view, dir)?;
    if offer != Offer::Temper {
        return None;
    }
    let skill = view.content().recipes().of_use(RecipeUse::Temper)?.skill;
    let profession = Ability::Profession(skill);
    let own = view.abilities().iter().any(|own| own.ability == profession);
    own.then_some(ScreenName::Profession(skill))
}

/// The dialog a step in `dir` opens first, if it walks into a feature the player sees or remembers
/// that offers an action (spec §7 "Special rooms"; what the abilities menu's Here group reads, so
/// a spent altar blocks as any wall does), where no one they see stands (moving there would be an
/// attack, or stop): "Would you like to pray at the forgotten altar?", to use its ability, just as
/// the menu does, or leave it (the safe answer, selected first, and Esc's; it takes no time). An
/// anvil asks nothing: its Temper is a recipe of Metalcraft's ([`step_screen`]).
fn offer_dialog(view: &PerceptionView<'_>, dir: Dir) -> Option<Dialog> {
    let (terrain, offer) = offered_at(view, dir)?;
    let ability = Ability::offered(offer)?;
    let (label, then) = (
        ability.name(view.content()).into_owned(),
        ScreenAction::UseAbility(ability),
    );
    let name = &view.content().terrain(terrain).name;
    let mut letters = name.chars();
    let title: String = (letters.next().iter().flat_map(|c| c.to_uppercase()))
        .chain(letters)
        .collect();
    Some(
        Dialog::new(
            title,
            format!("Would you like to {} the {name}?", offer.asks()),
        )
        .accept(label, then)
        .cancel("Leave it", ScreenAction::Close),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use warlock_core::ability;
    use warlock_core::time::Duration;
    use warlock_core::{Pos, Rest};

    #[test]
    fn leaving_a_shop_owing_and_selling_open_dialogs() {
        let content = std::sync::Arc::new(warlock_assets::content().unwrap());
        let mut world = warlock_core::World::new(content, "courtyard", None).expect("run");
        let walk = |world: &mut warlock_core::World, actions: &str| {
            for action in warlock_core::action::parse_actions(actions).expect("actions") {
                world.apply(&action);
            }
        };
        // Into Grimbold's store with the purse (a) and the scroll of light (b), on the cell
        // inside the door (to the west).
        walk(
            &mut world,
            "move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 move:S*5 move:E*5",
        );
        assert!(
            leaving_dialog(&world.perception(), Dir::W).is_none(),
            "owing nothing"
        );
        // With an unpaid dagger (c): pay and leave in one action, or not (the step, which the
        // rules turn back).
        walk(&mut world, "move:E move:NE pickup move:SW move:W");
        let view = world.perception();
        let owed = view.shop_here().map(|s| s.owed()).expect("owing");
        assert!(owed <= view.own_gold());
        assert_eq!(
            leaving_dialog(&view, Dir::W),
            Some(
                Dialog::new(
                    "Grimbold's General Store",
                    format!("You owe Grimbold {owed} gold.")
                )
                .accept(
                    format!("Pay {owed} gold and leave"),
                    Action::Pay(Some(Dir::W))
                )
                .decline("Don't pay", Action::Move(Dir::W))
            )
        );
        assert!(leaving_dialog(&view, Dir::E).is_none(), "a step inside");
        // Selling the scroll asks; putting back the dagger does not.
        let offer = view.sale(&Positions::of([2])).expect("an offer").offer;
        // Unread, the scroll goes by its label.
        let scroll = view.lore().describe(&view.inventory()[2]);
        assert!(scroll.starts_with("a scroll labelled "), "{scroll}");
        assert_eq!(
            sale_dialog(&view, &Positions::of([2])),
            Some(
                Dialog::new(
                    "Grimbold's General Store",
                    format!("Grimbold offers {offer} gold for {scroll}.")
                )
                .accept(
                    format!("Sell for {offer} gold"),
                    Action::Sell(Positions::of([2]))
                )
                .cancel("Keep it", ScreenAction::Close)
            )
        );
        assert!(sale_dialog(&view, &Positions::of([3])).is_none());
        // With the short sword too, more than the purse holds: no way to pay, only back.
        walk(
            &mut world,
            "move:E move:NE move:E*2 pickup move:W*3 move:SW",
        );
        let view = world.perception();
        let owed = view.shop_here().map(|s| s.owed()).expect("owing");
        assert!(owed > view.own_gold());
        assert_eq!(
            leaving_dialog(&view, Dir::W),
            Some(
                Dialog::new(
                    "Grimbold's General Store",
                    format!(
                        "You owe Grimbold {owed} gold and have only 60. Put something back, or \
                         bring more gold."
                    )
                )
                .decline("Go back", Action::Move(Dir::W))
            )
        );
    }

    #[test]
    fn stepping_onto_a_known_trap_asks_first() {
        use std::sync::Arc;
        use warlock_core::World;
        use warlock_core::action::parse_actions;
        let content = Arc::new(warlock_assets::content().unwrap());
        let mut world = World::new(content, "traps", None).expect("the trap hall");
        // Below the arrow trap at the west end of the proving row.
        for action in parse_actions("move:NW*14 move:W*7 move:N*2").unwrap() {
            world.apply(&action);
        }
        assert_eq!(world.player_pos(), Pos::new(16, 3));
        let view = world.perception();
        assert!(trap_dialog(&view, Dir::E).is_none());
        assert_eq!(
            trap_dialog(&view, Dir::N),
            Some(
                Dialog::new("Trap", "Really step onto the arrow trap?")
                    .accept("Step onto it", Action::Move(Dir::N))
                    .cancel("Stay back", ScreenAction::Close)
            )
        );
    }

    /// A run of `scenario` that starts with `nutrition`, or if that is below 0 (no run starts so
    /// hungry), has just begun to starve.
    fn hungry_world(scenario: &str, nutrition: i32) -> warlock_core::World {
        let mut files = warlock_assets::files();
        let hunger = include_str!("../../../../assets/data/hunger.toml")
            .replace("start = 3600", &format!("start = {}", nutrition.max(0)));
        files.insert("data/hunger.toml", &hunger);
        let content = std::sync::Arc::new(warlock_core::content::Content::load(&files).unwrap());
        let mut world = warlock_core::World::new(content, scenario, None).expect("a run");
        if nutrition < 0 {
            world.apply(&Action::Rest(Rest::For(Duration::minutes(2))));
        }
        world
    }

    #[test]
    fn a_journey_asks_first_while_fainting_or_starving() {
        for (nutrition, title, words) in [
            (100, "Fainting", "this journey may kill you"),
            (-1, "Starving", "may well be your last"),
        ] {
            let world = hungry_world("world", nutrition);
            let dialog = hunger_dialog(&world.perception(), Dir::E).expect("it asks");
            assert_eq!(dialog.title, title);
            assert!(dialog.text.contains(words), "{}", dialog.text);
            // No numbers: the state in words.
            assert!(!dialog.text.chars().any(|c| c.is_ascii_digit()));
            assert_eq!(
                dialog,
                Dialog::new(title, dialog.text.clone())
                    .accept("Go on", Action::Move(Dir::E))
                    .cancel("Stay", ScreenAction::Close),
                "going on is the step; staying is Esc's, selected first, and does nothing"
            );
        }
        // Weak, and anything better, walks on.
        for nutrition in [500, 1000, 3600, 6000] {
            let world = hungry_world("world", nutrition);
            assert!(hunger_dialog(&world.perception(), Dir::E).is_none());
        }
    }

    #[test]
    fn a_step_off_the_overworld_never_asks_about_hunger() {
        for scenario in ["courtyard", "forest", "showcase"] {
            let world = hungry_world(scenario, -1);
            for dir in [Dir::N, Dir::E, Dir::S, Dir::W] {
                let dialog = hunger_dialog(&world.perception(), dir);
                assert!(dialog.is_none(), "{scenario}");
            }
        }
    }

    /// A run of `scenario` after `walk`.
    fn walked_to(scenario: &str, walk: &str) -> warlock_core::World {
        let content = std::sync::Arc::new(warlock_assets::content().unwrap());
        let mut world = warlock_core::World::new(content, scenario, None).expect("a run");
        for action in warlock_core::action::parse_actions(walk).unwrap() {
            world.apply(&action);
        }
        world
    }

    #[test]
    fn walking_into_what_offers_an_action_asks_first() {
        let cases = [
            (
                "shrine",
                "move:N open:N move:N*3",
                "Forgotten altar",
                "Would you like to pray at the forgotten altar?",
                "Pray",
                Ability::Pray,
            ),
            (
                "spring",
                "move:N*5",
                "Spring",
                "Would you like to drink from the spring?",
                "Drink",
                Ability::Drink,
            ),
        ];
        for (scenario, walk, title, text, label, ability) in cases {
            let world = walked_to(scenario, walk);
            let view = world.perception();
            // It asks by the data, as the Here group lists it.
            assert!(view.abilities().iter().any(|own| own.ability == ability));
            assert_eq!(
                offer_dialog(&view, Dir::N),
                Some(
                    Dialog::new(title, text)
                        .accept(label, ScreenAction::UseAbility(ability))
                        .cancel("Leave it", ScreenAction::Close)
                ),
                "{scenario}"
            );
            // Any other way is plain floor.
            assert!(offer_dialog(&view, Dir::S).is_none(), "{scenario}");
        }
        // An anvil's Temper is a recipe of Metalcraft's, no row of the Here group: walking into
        // the anvil asks nothing, and opens Metalcraft's screen for the trained (#1387).
        let world = walked_to("forge", "move:E");
        let view = world.perception();
        let place = ability::Source::Place;
        assert!(
            !view
                .abilities()
                .iter()
                .any(|own| own.ability.source() == place)
        );
        assert_eq!(offer_dialog(&view, Dir::N), None);
        let metal = view.content().skill_id("metalcraft").expect("metalcraft");
        assert_eq!(
            step_screen(&view, Dir::N),
            Some(ScreenName::Profession(metal))
        );
        assert_eq!(step_screen(&view, Dir::S), None, "plain floor");
        // Untrained, no screen: the step goes to the rules, which say what it needs.
        let content = std::sync::Arc::new(warlock_assets::content().unwrap());
        let thief: warlock_core::character::CharacterSpec = "human,male,thief".parse().unwrap();
        let mut world =
            warlock_core::World::with_character(content, "forge", None, Some(&thief)).unwrap();
        world.apply(&Action::Move(Dir::E));
        assert_eq!(step_screen(&world.perception(), Dir::N), None);
        world.apply(&Action::Move(Dir::N));
        let last = world.messages().iter().last().map(|m| m.text.clone());
        assert_eq!(
            last.as_deref(),
            Some("You need Metalcraft to work at the anvil.")
        );
    }

    #[test]
    fn a_spent_altar_never_asks() {
        let mut world = walked_to("shrine", "move:N open:N move:N*3");
        assert!(offer_dialog(&world.perception(), Dir::N).is_some());
        for _ in 0..8 {
            world.apply(&Action::Pray);
        }
        let view = world.perception();
        assert!(matches!(
            view.cell(Pos::new(13, 3)),
            CellView::Visible(t) if view.content().terrain(t).name == "spent altar"
        ));
        assert!(offer_dialog(&view, Dir::N).is_none());
    }

    /// The spring's cell beside the player, in the dark: they neither see nor remember it.
    #[test]
    fn what_the_player_does_not_know_never_asks() {
        let mut files = warlock_assets::files();
        files.insert(
            "levels/dark_spring.toml",
            "name = \"Dark\"\nambient = [0, 0, 0, 0]\nmap = '''\n#####\n#@S.#\n#####\n'''\n\
             [legend]\n\".\" = \"floor\"\n\"#\" = \"wall\"\n\"S\" = \"spring\"\n\
             \"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
        );
        files.insert(
            "scenarios/dark_spring.toml",
            "name = \"Dark\"\nstart_level = \"dark_spring\"\n",
        );
        let content = std::sync::Arc::new(warlock_core::content::Content::load(&files).unwrap());
        let world = warlock_core::World::new(content, "dark_spring", None).expect("a run");
        let view = world.perception();
        assert_eq!(view.cell(view.viewer().step(Dir::E)), CellView::Unknown);
        assert!(offer_dialog(&view, Dir::E).is_none());
    }

    #[test]
    fn a_trapped_door_never_asks() {
        use std::sync::Arc;
        use warlock_core::World;
        use warlock_core::action::parse_actions;
        let content = Arc::new(warlock_assets::content().unwrap());
        let mut world = World::new(content, "traps", None).expect("the trap hall");
        // Below the west closet's door, which shows its arrow trap: walking into the door opens
        // it (and sets the trap off) without asking, as `o` does.
        for action in parse_actions("move:NW*14 move:W*21").unwrap() {
            world.apply(&action);
        }
        assert_eq!(world.player_pos(), Pos::new(2, 5));
        let view = world.perception();
        assert!(view.trap(Pos::new(2, 4)).is_some());
        assert!(trap_dialog(&view, Dir::N).is_none());
    }
}
