//! Loading statuses and diseases.

use super::{Content, ContentError, ContentFiles, DiseaseId, StatusId, disease, status};

impl Content {
    /// Statuses (optional: none): `data/status.toml` (the healing rules) and a file per status in
    /// `data/status/`, in path order (`NNN-<id>.toml`: the number keeps their order).
    pub(super) fn load_statuses(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let path = "data/status/";
        let (healing, list) = status::parse_table(
            files.table("status"),
            |id| self.damage_type_id(id),
            |id| self.light_id(id),
            |id| self.vision_index(id),
        )?;
        for (i, def) in list.iter().enumerate() {
            let id = u16::try_from(i).map_err(|_| ContentError::new(path, "too many statuses"))?;
            self.status_ids.insert(def.id.clone(), StatusId(id));
        }
        self.healing = healing;
        self.statuses = list;
        Ok(())
    }

    /// Diseases (optional: none). Their status must be one that nothing but diseases gives or
    /// takes away: no ticks, no easing, no wounds, no cures of its own.
    pub(super) fn load_diseases(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let path = "data/diseases.toml";
        let Some(text) = files.get_optional(path) else {
            return Ok(());
        };
        let (status, list) = disease::parse(
            path,
            text,
            |id| self.status_id(id),
            |id| self.terrain_id(id),
        )?;
        if let Some(status) = status {
            let def = self.status(status);
            if def.tick.is_some()
                || def.eases.is_some()
                || def.wound.is_some()
                || !def.cured_by.is_empty()
                || def.follows.is_some()
            {
                return Err(ContentError::new(
                    path,
                    format!(
                        "status `{}`: the status diseases show as lasts while they do: no \
                         `tick`, `eases`, `wound`, `cured_by`, or `follows` (cure the diseases \
                         instead)",
                        def.id
                    ),
                ));
            }
            if let Some(other) = self.statuses.iter().find(|s| s.follows == Some(status)) {
                return Err(ContentError::new(
                    path,
                    format!(
                        "status `{}`: follows `{}`, the status diseases show as (only diseases \
                         give it)",
                        other.id, def.id
                    ),
                ));
            }
        }
        for (i, def) in list.iter().enumerate() {
            let id = u16::try_from(i).map_err(|_| ContentError::new(path, "too many diseases"))?;
            self.disease_ids.insert(def.id.clone(), DiseaseId(id));
        }
        self.disease_status = status;
        self.diseases = list;
        Ok(())
    }
}
