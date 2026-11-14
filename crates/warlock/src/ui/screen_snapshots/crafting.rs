//! Screen snapshot cases (`super`): professions and their recipes, the forging wizard, cutting,
//! setting, and enchanting.

use super::*;

/// The hub's secret closet found and its gems picked up (a medium fine sapphire, an uncut tiny
/// garnet).
const GEM_CLOSET: &str = "move:N*3 open:N move:N*2 move:E*16 open:S move:S*6 move:SW move:W*3 \
    search:50 move:S open:W move:W*2 pickup";
/// A craftsperson's creation picks with Gemcutter first: Jewelcraft their trade.
const GEMCUTTER: &str = "gemcutter,smith,hardy,healthy,lean";
/// [`GEM_CLOSET`] by a [`GEMCUTTER`] craftsperson (Jewelcraft 16 at fate 8: Set's 10 and more),
/// the hand axe out of the hand: ready to Set.
const SET_READY: &str = "move:N*3 open:N move:N*2 move:E*16 open:S move:S*6 move:SW move:W*3 \
    search:50 move:S open:W move:W*2 pickup remove:main_hand";
/// A wizard's creation picks with Father's sword (an identified Masterwork rare, carried).
const ENCHANTER: &str = "family_wealth,fathers_sword,hardy";

cases! {
    // Beside a forge's anvil with arms and armour picked up: no Here group (Temper is a recipe on
    // Metalcraft's screen, #1334); then the Temper list, metal arms and armour (no lantern).
    abilities_menu_forge: Case {
        name: "abilities-menu-forge",
        scenario: "forge",
        actions: "move:SW move:S pickup move:W pickup move:E*2 pickup move:E pickup move:N*2",
        screen: Some("abilities"),
        ..BASE
    },
    temper_list: Case {
        name: "temper-list",
        scenario: "forge",
        actions: "move:SW move:S pickup move:W pickup move:E*2 pickup move:E pickup move:N*2",
        screen: Some("temper"),
        ..BASE
    },
    // An attempt made and one failed (spec §7 "Special rooms"): the name and numbers follow.
    temper_list_fine: Case {
        name: "temper-list-fine",
        scenario: "forge",
        actions: "move:SW move:S pickup move:W pickup move:E*2 pickup move:E pickup move:N*2 \
                  temper:g",
        screen: Some("temper"),
        ..BASE
    },
    temper_list_damaged: Case {
        name: "temper-list-damaged",
        scenario: "forge",
        actions: "move:SW move:S pickup move:W pickup move:E*2 pickup move:E pickup move:N*2 \
                  temper:g temper:g",
        screen: Some("temper"),
        ..BASE
    },
    // The craftsperson's Make do and mend list (#1422): the damaged long sword alone.
    make_do_list: Case {
        name: "make-do-list",
        scenario: "forge",
        actions: "move:SW move:S pickup move:W pickup move:E*2 pickup move:E pickup move:N*2 \
                  temper:g temper:g",
        screen: Some("make-do"),
        ..BASE
    },
    // A ranger in the Thornwood after a forage (spec §8 "Foraging"): Survival listed as its
    // use, Forage, usable; the message line says what came of it.
    abilities_menu_forage: Case {
        name: "abilities-menu-forage",
        scenario: "forest",
        actions: "forage",
        screen: Some("abilities"),
        ..BASE
    },
    // A ranger in the showcase's hall, no wilderness: Forage greyed out, "not usable here".
    abilities_menu_forage_not_here: Case {
        name: "abilities-menu-forage-not-here",
        character: Some("human,male,ranger"),
        run: true,
        screen: Some("abilities"),
        ..BASE
    },
    // A ranger after a batch in the forest area's glade, where wood is free (spec §8
    // "Fletching"; #1336): Woodcraft… in the skill's place; the message line says what was made.
    abilities_menu_fletch: Case {
        name: "abilities-menu-fletch",
        scenario: "forest",
        character: Some("human,male,ranger"),
        run: true,
        actions: "craft:arrows:-",
        screen: Some("abilities"),
        ..BASE
    },
    // The arrows' wizard in the forest area's glade (#1336): the head step alone (the shaft is
    // cut from the trees), nothing carried to head them, None on `-` for brittle arrows.
    craft_pick_arrows_head: Case {
        name: "craft-pick-arrows-head",
        scenario: "forest",
        character: Some("human,male,ranger"),
        run: true,
        screen: Some("abilities"),
        keys: Some("e p"),
        ..BASE
    },
    // A ranger's Whittle list (#1028): the wooden weapons carried (the bow), not the knife.
    whittle_list: Case {
        name: "whittle-list",
        character: Some("human,male,ranger"),
        run: true,
        screen: Some("whittle"),
        ..BASE
    },
    // A ranger's Salvage list (#1333): what a craft works, carried, less what is worn or wielded.
    salvage_list: Case {
        name: "salvage-list",
        character: Some("human,male,ranger"),
        run: true,
        screen: Some("salvage"),
        ..BASE
    },
    // A Gemcutter craftsperson (Jewelcraft their trade) with the hub's secret closet's gems, a
    // medium fine sapphire and an uncut tiny garnet (#1355): Jewelcraft's screen (the first
    // profession whose recipes they see) lists Cut under Work; the Cut list both gems.
    profession_jewelcraft: Case {
        name: "profession-jewelcraft",
        seed: Some(8),
        character: Some("human,male,craftsperson"),
        talents: Some(GEMCUTTER),
        actions: GEM_CLOSET,
        screen: Some("profession"),
        ..BASE
    },
    cut_list: Case {
        name: "cut-list",
        seed: Some(8),
        character: Some("human,male,craftsperson"),
        talents: Some(GEMCUTTER),
        actions: GEM_CLOSET,
        screen: Some("cut"),
        ..BASE
    },
    // The garnet cut (it splits to nothing at this fate) and the sapphire re-cut a size smaller:
    // the list holds the small fine sapphire (and the trade's uncut garnet), and the message line
    // says what came of it.
    cut_list_after: Case {
        name: "cut-list-after",
        seed: Some(8),
        character: Some("human,male,craftsperson"),
        talents: Some(GEMCUTTER),
        actions: "move:N*3 open:N move:N*2 move:E*16 open:S move:S*6 move:SW move:W*3 \
                  search:50 move:S open:W move:W*2 pickup craft:cut:n craft:cut:m",
        screen: Some("cut"),
        ..BASE
    },
    // Set (#1356) by the same craftsperson, Jewelcraft 16 from their trade, the
    // hand axe taken out of the hand: the wizard's item step lists the axe (its pommel unset),
    set_pick_item: Case {
        name: "set-pick-item",
        seed: Some(8),
        character: Some("human,male,craftsperson"),
        talents: Some(GEMCUTTER),
        actions: SET_READY,
        screen: Some("profession"),
        keys: Some("b"),
        ..BASE
    },
    // its gem step the sapphire alone (never the uncut garnet),
    set_pick_gem: Case {
        name: "set-pick-gem",
        seed: Some(8),
        character: Some("human,male,craftsperson"),
        talents: Some(GEMCUTTER),
        actions: SET_READY,
        screen: Some("profession"),
        keys: Some("b a"),
        ..BASE
    },
    // and the dialog names the axe as it would be.
    set_pick_dialog: Case {
        name: "set-pick-dialog",
        seed: Some(8),
        character: Some("human,male,craftsperson"),
        talents: Some(GEMCUTTER),
        actions: SET_READY,
        screen: Some("profession"),
        keys: Some("b a a"),
        ..BASE
    },
    // Enchanting (#1430) by a wizard trained in it at this fate (11), with Father's sword, a
    // Masterwork rare, in the pack: the six works (#1431) greyed by skill or "nothing to
    // enchant" (no plain item free), Quicken "it holds all it can" (the rare at its capacity,
    // #1432), Unbind ready;
    profession_enchanting: Case {
        name: "profession-enchanting",
        seed: Some(13),
        character: Some("human,female,wizard"),
        talents: Some(ENCHANTER),
        run: true,
        screen: Some("profession"),
        ..BASE
    },
    // Unbind's item step lists the sword alone (never the wielded staff),
    enchant_pick_item: Case {
        name: "enchant-pick-item",
        seed: Some(13),
        character: Some("human,female,wizard"),
        talents: Some(ENCHANTER),
        run: true,
        screen: Some("profession"),
        keys: Some("f"),
        ..BASE
    },
    // and the dialog, with nothing used up, says only the time.
    enchant_dialog: Case {
        name: "enchant-dialog",
        seed: Some(13),
        character: Some("human,female,wizard"),
        talents: Some(ENCHANTER),
        run: true,
        screen: Some("profession"),
        keys: Some("f a"),
        ..BASE
    },
    // A Wright craftsperson whose kit's knife (`b`) is dropped: Woodcraft's screen (its row in the
    // abilities menu, `c`) greys the arrows and Whittle, each with the reason in place of what it
    // takes (#1334). Only the trained have the row (#1387).
    profession_woodcraft_no_knife: Case {
        name: "profession-woodcraft-no-knife",
        character: Some("human,male,craftsperson"),
        talents: Some("wright,smith,healthy,lean,skill_focus:appraise"),
        actions: "drop:b",
        screen: Some("abilities"),
        keys: Some("c"),
        ..BASE
    },
    // A ranger's knife, but their bow put away, so none is ready: Whittle can be made, the arrows
    // not (`e`: Woodcraft…, after the stare, Decipher, Tame, and Forage).
    profession_woodcraft_no_launcher: Case {
        name: "profession-woodcraft-no-launcher",
        character: Some("human,male,ranger"),
        actions: "remove:missile_weapon",
        run: true,
        screen: Some("abilities"),
        keys: Some("e"),
        ..BASE
    },
    // Metalcraft's screen (`--screen profession`, the first profession's) away from an anvil:
    // Temper under "Work", greyed "no anvil beside you".
    profession_metalcraft_no_anvil: Case {
        name: "profession-metalcraft-no-anvil",
        character: Some("human,male,craftsperson"),
        talents: Some("smith,wright,healthy,lean,skill_focus:appraise"),
        run: true,
        screen: Some("profession"),
        ..BASE
    },
    // Beside the forge's anvil: Temper can be made, what it takes under the list.
    profession_metalcraft_forge: Case {
        name: "profession-metalcraft-forge",
        scenario: "forge",
        actions: "move:E",
        screen: Some("profession"),
        ..BASE
    },
    // The forge's stock picked up beside the anvil: the weapons under "Melee weapons", those from
    // a higher Metalcraft greyed "beyond your skill".
    profession_metalcraft_stock: Case {
        name: "profession-metalcraft-stock",
        scenario: "forge",
        actions: "move:E pickup",
        screen: Some("profession"),
        ..BASE
    },
    // The dagger's wizard (#1335): its first step, the metal, the ingots carried with what the
    // recipe takes of each.
    craft_pick_blade: Case {
        name: "craft-pick-blade",
        scenario: "forge",
        actions: "move:E pickup",
        screen: Some("profession"),
        keys: Some("c"),
        ..BASE
    },
    // Its second, the pommel: None on `-`, the cut gems; the running choice in the header.
    craft_pick_pommel: Case {
        name: "craft-pick-pommel",
        scenario: "forge",
        actions: "move:E pickup",
        screen: Some("profession"),
        keys: Some("c c"),
        ..BASE
    },
    // The last pick asks in the one dialog, with the item named as it will be made.
    craft_pick_dialog: Case {
        name: "craft-pick-dialog",
        scenario: "forge",
        actions: "move:E pickup",
        screen: Some("profession"),
        keys: Some("c c a"),
        ..BASE
    },
    // A witch's Alchemy (#1357; `--screen profession`, the first profession she has recipes of)
    // after three gathers at the hub's fungus: the brews, those from a higher Alchemy greyed
    // "beyond your skill", those short of herbs "not enough".
    profession_alchemy_herbs: Case {
        name: "profession-alchemy-herbs",
        character: Some("human,female,witch"),
        actions: "move:N*6 move:E*30 move:N*3 move:W*6 gather:W*3",
        screen: Some("profession"),
        ..BASE
    },
    // A brew chooses nothing: its row asks at once in the one dialog, the potion named truly.
    brew_dialog: Case {
        name: "brew-dialog",
        character: Some("human,female,witch"),
        actions: "move:N*6 move:E*30 move:N*3 move:W*6 gather:W*3",
        screen: Some("profession"),
        keys: Some("> b"),
        ..BASE
    },
    // The smith's javelin after its iron head (#1351): the haft, Woodcraft's wood billets on a
    // Metalcraft recipe, never the ingots.
    craft_pick_haft: Case {
        name: "craft-pick-haft",
        scenario: "forge",
        actions: "move:E pickup",
        screen: Some("profession"),
        keys: Some("i b"),
        ..BASE
    },
    // `--screen craft-pick`: the first recipe with a choice, the amber amulet, with nothing carried.
    craft_pick_first: Case {
        name: "craft-pick-first",
        scenario: "forge",
        actions: "move:E",
        screen: Some("craft-pick"),
        ..BASE
    },
}
