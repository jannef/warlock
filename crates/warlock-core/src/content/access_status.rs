//! Content's accessors for damage, resources, statuses, and diseases.

use super::{
    Content, DamageTypeDef, DamageTypeId, DiseaseDef, DiseaseId, Healing, ResourceDef, ResourceId,
    StatusDef, StatusId, damage_type,
};
use std::collections::BTreeMap;

impl Content {
    /// The damage types (spec §8 "Damage types"), in file order (by [`DamageTypeId`]); empty
    /// without `data/damage_types.toml`, and then nothing is typed.
    pub fn damage_types(&self) -> &[DamageTypeDef] {
        &self.damage_types
    }

    pub fn damage_type(&self, id: DamageTypeId) -> &DamageTypeDef {
        &self.damage_types[usize::from(id.0)]
    }

    pub fn damage_type_id(&self, name: &str) -> Option<DamageTypeId> {
        let i = self.damage_types.iter().position(|t| t.id == name)?;
        u8::try_from(i).ok().map(DamageTypeId)
    }

    /// The alternative resources (spec §8 "Resources"), in file order (by [`ResourceId`]);
    /// empty without `data/resources.toml`.
    pub fn resources(&self) -> &[ResourceDef] {
        &self.resources
    }

    pub fn resource(&self, id: ResourceId) -> &ResourceDef {
        &self.resources[usize::from(id.0)]
    }

    pub fn resource_id(&self, name: &str) -> Option<ResourceId> {
        let i = self.resources.iter().position(|r| r.id == name)?;
        u16::try_from(i).ok().map(ResourceId)
    }

    /// The kind of damage bare hands, kicks, and shield bashes do (the type marked `unarmed`),
    /// if content names one.
    pub fn unarmed_damage(&self) -> Option<DamageTypeId> {
        let i = self.damage_types.iter().position(|t| t.unarmed)?;
        u8::try_from(i).ok().map(DamageTypeId)
    }

    /// The kinds `names` name (`damage_type` in data: one or two type ids), checked; none
    /// named is an untyped attack (soaked and wounding as every blow did before damage had
    /// types; the built-in data names one everywhere, a test checks).
    pub(crate) fn resolve_kinds(
        &self,
        names: Option<&damage_type::TypeNames>,
    ) -> Result<crate::damage::Kinds, String> {
        use crate::damage::{Kinds, MAX_KINDS};
        let Some(names) = names else {
            return Ok(Kinds::NONE);
        };
        if self.damage_types.is_empty() {
            return Err("`damage_type` needs data/damage_types.toml".into());
        }
        let mut kinds = Vec::new();
        for name in names.names() {
            let id = self
                .damage_type_id(name)
                .ok_or_else(|| format!("unknown damage type `{name}`"))?;
            kinds.push(id);
        }
        Kinds::of(&kinds).ok_or_else(|| format!("`damage_type` names at most {MAX_KINDS} types"))
    }

    /// The percentages per kind `map` gives (`resist = { fire = 25 }` on a creature or an
    /// object; `protects = { slashing = 1 }` on armour, `physical` only: the others are never
    /// read), by damage type id. The rules clamp what they add up ([`crate::damage`]).
    pub(crate) fn resolve_per_kind(
        &self,
        field: &str,
        map: &BTreeMap<String, i32>,
        physical: bool,
    ) -> Result<Vec<(DamageTypeId, i32)>, String> {
        let mut out = Vec::new();
        for (name, value) in map {
            let id = self
                .damage_type_id(name)
                .ok_or_else(|| format!("`{field}`: unknown damage type `{name}`"))?;
            if physical && !self.damage_type(id).physical {
                return Err(format!("`{field}`: `{name}` is no physical damage type"));
            }
            out.push((id, *value));
        }
        // In damage type order, whatever order the file wrote them in.
        out.sort_by_key(|(id, _)| *id);
        Ok(out)
    }

    /// Status effects (`data/status.toml`), most important first; empty without the file.
    pub fn status_defs(&self) -> &[StatusDef] {
        &self.statuses
    }

    pub fn status(&self, id: StatusId) -> &StatusDef {
        &self.statuses[usize::from(id.0)]
    }

    pub fn status_id(&self, name: &str) -> Option<StatusId> {
        self.status_ids.get(name).copied()
    }

    /// The first status that makes its sufferer sleep (`sleeps`: Asleep), if any: the one a
    /// teeming den's sleepers are placed with.
    pub fn sleep_status(&self) -> Option<StatusId> {
        (self.status_ids()).find_map(|(id, def)| def.sleeps.then_some(id))
    }

    /// The first status that hides its sufferer (`hides`: Disguised), if any: the one a mimic is
    /// placed with (spec §8 "Mimics").
    pub fn disguise_status(&self) -> Option<StatusId> {
        (self.status_ids()).find_map(|(id, def)| def.hides.then_some(id))
    }

    /// Every status with its id, most important first.
    pub fn status_ids(&self) -> impl Iterator<Item = (StatusId, &StatusDef)> {
        (0u16..).map(StatusId).zip(&self.statuses)
    }

    /// The passive healing rules (`[healing]` in `data/status.toml`).
    pub fn healing(&self) -> &Healing {
        &self.healing
    }

    /// Diseases (`data/diseases.toml`), in file order; empty without the file.
    pub fn disease_defs(&self) -> &[DiseaseDef] {
        &self.diseases
    }

    pub fn disease(&self, id: DiseaseId) -> &DiseaseDef {
        &self.diseases[usize::from(id.0)]
    }

    pub fn disease_id(&self, name: &str) -> Option<DiseaseId> {
        self.disease_ids.get(name).copied()
    }

    /// Every disease with its id, in file order.
    pub fn disease_ids(&self) -> impl Iterator<Item = (DiseaseId, &DiseaseDef)> {
        (0u16..).map(DiseaseId).zip(&self.diseases)
    }

    /// The status diseases show as ("Diseased"); `None` without diseases.
    pub fn disease_status(&self) -> Option<StatusId> {
        self.disease_status
    }
}
