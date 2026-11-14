//! What the craft talents do, in the wiki's words (`warlock_core::talent::CraftEffect`, #1495): one
//! arm a variant, beside the area's enum and sum in `warlock-core/src/talent/craft.rs`.

use crate::wiki::util::signed;
use warlock_core::content::Content;
use warlock_core::talent::CraftEffect;

/// What `effect` does, as the talents' tables say it.
pub(super) fn text(content: &Content, effect: &CraftEffect) -> String {
    match effect.clone() {
        CraftEffect::Fletcher {
            check,
            count_times,
            time_percent,
        } => format!(
            "[fletching](skill-uses.md#fletching) takes {time_percent} % of its time, +{check} on its \
             check, {count_times} times the arrows or bolts after the check, and they are plain \
             at any Woodcraft"
        ),
        CraftEffect::Trade { skill, dice, .. } => {
            let name =
                (content.skill_id(&skill)).map_or(skill.clone(), |s| content.skill(s).name.clone());
            format!(
                "at creation, [{name}](skills.md#skill-{}) trained at {dice}, \
                 and the [tools of the trade](talents-origins.md#crafts-a-craftspersons-trades); a \
                 [skill roll](skills.md#improving-a-skill) on it throws the die one step up, as on a \
                 skill your class is known for (never two steps)",
                skill.replace('_', "-")
            )
        }
        CraftEffect::TradeBonus { value } => {
            format!(
                "each trade you take at creation is trained {} more",
                signed(value)
            )
        }
        CraftEffect::EyeForMake => format!(
            "an item's description reads its make: what [salvage](salvage.md) would \
             give of it if every check went well ({}% of each material's weight, a gem whole; \
             never the odds), how many more improvements \
             [the anvil](altars-forges-springs.md#forges) or a knife could give it, or that it is \
             damaged, and how hard its main material is to work (\"very hard\" for a \
             [work](crafting.md#which-skill-makes-what) of +4 or more, \"hard\" for +1 to +3, \"soft \
             and easy\" for -2 or less)",
            content.salvage().map_or(0, |s| s.percent)
        ),
        CraftEffect::WasteNot { salvage, failed } => format!(
            "[salvage](salvage.md) gives back {salvage}% of each material's weight \
             (everyone else: {}%), and a [failed recipe](crafting.md#how-well-it-comes-out) gives back \
             {failed}% of its stock and inputs (everyone else: {}%; a failed brew is still spent \
             on its murky draught)",
            content.salvage().map_or(0, |s| s.percent),
            content
                .recipes()
                .settings
                .as_ref()
                .map_or(0, |s| s.yield_percent)
        ),
        CraftEffect::TemperSafe => "a failed [Temper](altars-forges-springs.md#forges) or \
             [Whittle](skill-uses.md#whittling) damages nothing: the item stays as it was, though \
             the attempt still counts toward the next one's difficulty"
            .to_string(),
        CraftEffect::MakersMark { worth_percent } => format!(
            "the arms, armour, and jewellery you [make](crafting.md#the-recipes) bear your \
             mark, whatever their grade: each is worth {worth_percent}% more (and \
             [your patron](patron.md) pays for that worth), and it holds one \
             [enchantment](magic-items.md#capacity) more"
        ),
        CraftEffect::CraftTimePercent { value } => format!(
            "every craft takes {value}% of its time: [recipes](crafting.md#the-recipes) \
             (after [Fletcher](class-ranger.md#talent-fletcher)'s share), \
             [Temper](altars-forges-springs.md#forges), [Whittle](skill-uses.md#whittling), \
             [cutting and setting gems](gem-cutting.md), [salvage](salvage.md), and \
             the craftsperson's own works"
        ),
        CraftEffect::GradeEase { value } => format!(
            "{} to the margin of a made check as its [grade](crafting.md#how-well-it-comes-out) \
             or a [gem's cut](gem-cutting.md) is read: every grade and every cut comes {value} \
             sooner; whether the work is made is unchanged",
            signed(value)
        ),
        CraftEffect::FieldForge { seconds } => format!(
            "active (aimed in a direction, at open ground beside you, {} minutes of work; no \
             mana, cooldown, or check): an [ingot](crafting.md) of any metal you carry is set on a \
             heap of stones there, a [makeshift \
             anvil](altars-forges-springs.md#the-makeshift-anvil) that serves every recipe and Temper as a \
             forge's anvil does, and stays for good. Interrupted, nothing is made and the ingot \
             kept. Refused, with no time spent, without an ingot, on the overworld, or where the \
             ground is not open floor",
            seconds / 60
        ),
        CraftEffect::MakeDo {
            seconds,
            difficulty,
        } => format!(
            "active (on a damaged thing you carry, not worn or wielded, anywhere, with no anvil \
             or tool; {} of work, cut short by anything that would stop a rest): one check, 2d12 \
             plus the [modifier](stats.md#modifiers) of the stat of the craft that works its \
             main material and that craft's points (a stat alone, untrained), against \
             {difficulty} plus the material's [work](crafting.md#which-skill-makes-what). Made, the \
             damage is gone (the improvements it lost stay lost, and the next attempt at the \
             anvil is as hard as before); failed, nothing happens to it, and you may try again. \
             It never destroys anything. Refused, with no time spent, for something worn or \
             wielded, [protected](materials.md#protected-items), unpaid, or of a material no \
             craft works (cloth, glass)",
            warlock_core::time::Duration::seconds(u64::from(seconds)).spoken()
        ),
        CraftEffect::Rigs => "active (a rig from the rig list, then a direction): a trap is \
            rigged on the cell beside you, from arrows, a billet of wood, or a pit dug with the \
            digging tool you have ready; what it kills is your kill. See \
            [Rigging traps](traps.md#rigging-traps)"
            .to_string(),
    }
}
