//! Materials (`assets/data/materials/`, one file each; docs/dev/content.md): what objects are
//! made of, part by part ([`super::part`]). A material has a name, whether it is `precious` (a gem
//! or a precious metal: it shows in the names of what it is part of), and its `value` (percent,
//! steel 100), which weighs it against the others in what an object of other parts is worth;
//! a gem has a `gem` table besides ([`GemDef`], spec §7 "Gems"); more properties (weight,
//! colour, weapon and armour modifiers, burning, rusting) come later.

use super::gem::GemDef;
use serde::{Deserialize, Serialize};

/// Index of a material in [`super::Content::materials`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MaterialId(pub u16);

/// The most a material's `value` may be, in percent of steel's.
pub const MAX_MATERIAL_VALUE: u32 = 10_000;

/// A material, as written in content files and as loaded.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialDef {
    pub id: String,
    pub name: String,
    /// A gem or a precious metal: names show it (spec §7 "Objects and materials").
    #[serde(default)]
    pub precious: bool,
    /// What it is worth, in percent of steel (100, the default): an object's value follows its
    /// parts' materials by their shares ([`super::ObjectDef::value_with`]). 1 to
    /// [`MAX_MATERIAL_VALUE`].
    #[serde(default = "steel")]
    pub value: u32,
    /// A gem (spec §7 "Gems"): its colour, what a plain one is worth, and how often loot finds
    /// it. A gem is precious.
    #[serde(default)]
    pub gem: Option<GemDef>,
}

fn steel() -> u32 {
    100
}

impl MaterialDef {
    /// Checks what a material file cannot say on its own.
    pub(super) fn check(&self) -> Result<(), String> {
        if self.id.trim().is_empty() || self.name.trim().is_empty() {
            return Err(format!(
                "material `{}`: `id` and `name` must not be empty",
                self.id
            ));
        }
        if !(1..=MAX_MATERIAL_VALUE).contains(&self.value) {
            return Err(format!(
                "material `{}`: `value` is in percent of steel's, 1 to {MAX_MATERIAL_VALUE}",
                self.id
            ));
        }
        if let Some(gem) = &self.gem {
            if !self.precious {
                return Err(format!(
                    "material `{}`: a gem is precious (`precious = true`)",
                    self.id
                ));
            }
            gem.check(&self.id)?;
        }
        Ok(())
    }
}
