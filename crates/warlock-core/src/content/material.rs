//! Materials (`assets/data/materials/`, one file each; docs/dev/content.md): what objects are
//! made of. Only a name for now;
//! properties (hardness, burning, rusting) come later.

use serde::{Deserialize, Serialize};

/// Index of a material in [`super::Content::materials`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MaterialId(pub u16);

/// A material, as written in content files and as loaded.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialDef {
    pub id: String,
    pub name: String,
}
