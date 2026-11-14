//! Loading materials, objects, gems, kits, and hunger.

use super::{
    Content, ContentError, ContentFiles, CreatureId, Gems, HungerDef, ItemClassId, MaterialDef,
    MaterialId, ObjectDef, ObjectId, SignPart, duplicate_id, gem, grimoire, hunger, item_class,
    kit, load_table, object, part, slot, validate::check_effects,
};
use serde::Deserialize;
use std::collections::BTreeMap;

impl Content {
    pub(super) fn load_kits(&mut self) -> Result<(), ContentError> {
        let classes = (self.classes.iter())
            .map(|c| {
                kit::resolve(&c.kit_spec, self).map_err(|m| {
                    ContentError::new("data/classes/", format!("class `{}`: {m}", c.id))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let races = (self.races.list.iter())
            .map(|r| {
                let at = |m: String| {
                    ContentError::new("data/races.toml", format!("race `{}`: {m}", r.id))
                };
                let kit = kit::resolve(&r.kit_spec, self).map_err(at)?;
                let bare = (r.bare_spec.iter())
                    .map(|s| {
                        self.slot_id(s)
                            .ok_or_else(|| at(format!("`bare`: unknown slot `{s}`")))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok((kit, bare))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let signs = |parts: &[SignPart]| {
            (parts.iter())
                .map(|p| {
                    kit::resolve(&p.kit_spec, self).map_err(|m| {
                        ContentError::new("data/horoscope.toml", format!("`{}`: {m}", p.id))
                    })
                })
                .collect::<Result<Vec<_>, _>>()
        };
        let horoscope = match &self.horoscope {
            Some(h) => Some((signs(&h.animals)?, signs(&h.elements)?)),
            None => None,
        };
        let scrolls = (self.classes.iter())
            .map(|c| {
                (c.random_scrolls_spec.as_ref())
                    .map(|spec| kit::resolve_scrolls(spec, self))
                    .transpose()
                    .map_err(|m| {
                        ContentError::new("data/classes/", format!("class `{}`: {m}", c.id))
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        for ((class, kit), scrolls) in self.classes.iter_mut().zip(classes).zip(scrolls) {
            class.kit = kit;
            class.random_scrolls = scrolls;
        }
        for (race, (kit, bare)) in self.races.list.iter_mut().zip(races) {
            race.kit = kit;
            race.bare = bare;
        }
        if let (Some(h), Some((animals, elements))) = (&mut self.horoscope, horoscope) {
            for (part, kit) in (h.animals.iter_mut()).zip(animals) {
                part.kit = kit;
            }
            for (part, kit) in (h.elements.iter_mut()).zip(elements) {
                part.kit = kit;
            }
        }
        Ok(())
    }

    /// `data/gems.toml` (spec §7 "Gems"), once the objects, item classes, and part names it
    /// names are known.
    pub(super) fn load_gems(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let path = "data/gems.toml";
        let Some(text) = files.get_optional(path) else {
            return Ok(());
        };
        let object_id = |id: &str| self.object_ids.get(id).copied();
        let known = gem::Known {
            object_id: &object_id,
            objects: &self.objects,
            classes: &self.item_classes,
            parts: &self.parts,
        };
        let gems = Gems::parse(path, text, &known)?;
        self.gems = Some(gems);
        Ok(())
    }

    /// Materials and part names, then the objects made of them.
    /// The material and object tables (`data/materials.toml` and `data/materials/*.toml`,
    /// `data/objects.toml` and `data/objects/*.toml`; both optional), numbered in path order
    /// (the single file first), with `data/parts.toml`, `data/slots.toml`, and
    /// `data/item_classes.toml` between them.
    pub(super) fn load_objects(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        let mut seen: BTreeMap<String, &str> = BTreeMap::new();
        for (i, (path, def)) in (load_table(files, "materials", |f: MaterialFile| f.material)?)
            .into_iter()
            .enumerate()
        {
            let id = MaterialId(
                u16::try_from(i).map_err(|_| ContentError::new(path, "too many materials"))?,
            );
            if let Some(first) = seen.insert(def.id.clone(), path) {
                return Err(duplicate_id("material", &def.id, path, first));
            }
            def.check().map_err(|m| ContentError::new(path, m))?;
            self.material_ids.insert(def.id.clone(), id);
            self.materials.push(def);
        }
        let path = "data/parts.toml";
        if let Some(text) = files.get_optional(path) {
            self.parts = part::parse(path, text)?;
        }
        let path = "data/slots.toml";
        if let Some(text) = files.get_optional(path) {
            self.slots = slot::parse(path, text)?;
        }
        let path = "data/item_classes.toml";
        if let Some(text) = files.get_optional(path) {
            self.item_classes = item_class::parse(path, text)?;
        }
        let mut specs = load_table(files, "objects", |f: ObjectFile| f.object)?;
        // A grimoire for every spell scroll, after every object file (`grimoire.rs`).
        let grimoires = grimoire::specs(files, &specs)?;
        if let Some(generated) = &grimoires {
            specs.extend(generated.specs.iter().cloned());
        }
        let mut seen: BTreeMap<String, &str> = BTreeMap::new();
        // Each object's file, for the errors found once every object is known.
        let mut paths = Vec::with_capacity(specs.len());
        let mut lit = Vec::with_capacity(specs.len());
        for (i, (path, spec)) in specs.into_iter().enumerate() {
            let id = ObjectId(
                u16::try_from(i).map_err(|_| ContentError::new(path, "too many objects"))?,
            );
            if let Some(first) = seen.insert(spec.id.clone(), path) {
                return Err(duplicate_id("object", &spec.id, path, first));
            }
            self.object_ids.insert(spec.id.clone(), id);
            let ornament = match &spec.ornament {
                Some(part) => Some(self.part_id(part).ok_or_else(|| {
                    ContentError::new(
                        path,
                        format!(
                            "object `{}`: `ornament = \"{part}\"` is no part of data/parts.toml",
                            spec.id
                        ),
                    )
                })?),
                None => None,
            };
            let (entries, made_of) = (spec.materials.clone(), spec.made_of.clone());
            let (mut def, lit_as) = spec.resolve(
                path,
                self,
                &self.item_classes,
                |entries| {
                    part::resolve(
                        entries,
                        |m| self.material_ids.get(m).copied(),
                        |p| self.part_id(p),
                    )
                },
                |l| self.light_ids.get(l).copied(),
                &self.slots,
                &self.weapon_skills,
            )?;
            def.ornament = ornament;
            def.construction = part::construction(
                &entries,
                made_of.as_deref(),
                &def.parts,
                &self.materials,
                def.role(),
            )
            .map_err(|m| ContentError::new(path, format!("object `{}`: {m}", def.id)))?;
            (def.name, def.plural) = part::names(
                &def.name,
                &def.plural,
                &def.parts,
                def.material(),
                &self.materials,
                &self.parts,
            );
            check_effects(path, "object", &def.id, None, def.glow, 0.0, &def.particles)?;
            lit.push(lit_as);
            paths.push(path);
            self.objects.push(def);
        }
        // What is lit as it is loosed flies as an object that burns.
        for (i, lit_as) in lit.into_iter().enumerate() {
            let Some(name) = lit_as else { continue };
            let target = self.object_ids.get(&name).copied();
            let def = &self.objects[i];
            let Some(target) = target.filter(|t| self.objects[usize::from(t.0)].burning.is_some())
            else {
                return Err(ContentError::new(
                    paths[i],
                    format!(
                        "object `{}`: `lit = \"{name}\"` must name an object that burns (`light` \
                         and `burns`)",
                        def.id
                    ),
                ));
            };
            self.objects[i].lit = Some(target);
        }
        if let Some(generated) = grimoires {
            let mut of = BTreeMap::new();
            for (grimoire, scroll) in &generated.pairs {
                let (g, s) = (self.object_ids[grimoire], self.object_ids[scroll]);
                let spell = (self.objects[usize::from(s.0)].read.as_ref())
                    .and_then(|r| r.scroll())
                    .map(|scroll| scroll.spell);
                self.objects[usize::from(g.0)].grimoire = spell;
                of.insert(s, g);
            }
            self.grimoires = grimoire::Grimoires::new(generated.chance, of);
        }
        // Potions of one colour show in one colour (spec §8 "Potions": the colour is the hint).
        for (o, path) in self.objects.iter().zip(&paths) {
            let Some(drink) = &o.drink else { continue };
            let other = (self.objects.iter())
                .find(|p| p.drink.as_ref().is_some_and(|d| d.colour == drink.colour));
            if let Some(other) = other.filter(|p| p.fg != o.fg) {
                return Err(ContentError::new(
                    path,
                    format!(
                        "potion `{}` is `{}` as `{}` is, but its `fg` differs: one colour, one fg",
                        o.id, drink.colour, other.id
                    ),
                ));
            }
        }
        Ok(())
    }

    /// `data/hunger.toml` (spec §8 "Hunger"), then a corpse object for each creature type that
    /// leaves one (not the player), after the objects of `data/objects.toml`, in creature order.
    pub(super) fn load_hunger(&mut self, files: &ContentFiles) -> Result<(), ContentError> {
        self.corpses = vec![None; self.creatures.len()];
        let path = "data/hunger.toml";
        let Some(text) = files.get_optional(path) else {
            return Ok(());
        };
        let err = |m: String| ContentError::new(path, m);
        let size_index = |id: &str| self.size_index(id);
        let def = HungerDef::parse(path, text, self.sizes.len(), size_index)?;
        if let Some(corpses) = &def.corpses {
            self.resolve_corpse_gifts(corpses).map_err(err)?;
            let material =
                (self.material_ids.get(&corpses.material).copied()).ok_or_else(|| {
                    err(format!(
                        "corpses: unknown material `{}` (data/materials.toml)",
                        corpses.material
                    ))
                })?;
            let class = (self.item_classes.iter().position(|c| c.corpses))
                .map(|i| ItemClassId(i as u8))
                .ok_or_else(|| {
                    err(
                        "creatures leave corpses: a class in data/item_classes.toml must take \
                         them (`corpses = true`)"
                            .into(),
                    )
                })?;
            for (i, creature) in self.creatures.iter().enumerate() {
                if creature.id == "player" || !creature.corpse {
                    continue;
                }
                let id = ObjectId(
                    u16::try_from(self.objects.len())
                        .map_err(|_| err("too many objects".into()))?,
                );
                let creature_id = CreatureId(i as u16);
                let mut food = corpses.sizes[creature.size];
                if creature.everlasting {
                    // It never rots: the one way to a shelf life of never.
                    food.keeps = None;
                    food.lasts = None;
                }
                let def = ObjectDef::corpse_of(creature_id, creature, class, material, food);
                if self.object_ids.insert(def.id.clone(), id).is_some() {
                    return Err(err(format!(
                        "the corpse of `{}` would be object `{}`, which data/objects.toml has",
                        creature.id, def.id
                    )));
                }
                self.corpses[i] = Some(id);
                self.objects.push(def);
            }
        }
        // Harm from eating is degrees of a status (spec §8 "Status effects").
        let harms = def.rotten.harm.max() > 0
            || (def.corpses.is_some()
                && (self.creatures.iter())
                    .any(|c| c.on_eat.as_ref().is_some_and(|e| e.harm.is_some())));
        match &def.sickness {
            Some(id) if self.status_id(id).is_none() => {
                return Err(err(format!("unknown sickness `{id}` (data/status/)")));
            }
            None if harms => {
                return Err(err(
                    "rotten food's `harm` and `on_eat` harm need a `sickness`, a status in \
                     data/status/"
                        .into(),
                ));
            }
            _ => {}
        }
        self.hunger = Some(def);
        Ok(())
    }

    /// Lays `[corpses.by_tag]` under every corpse-leaving creature's `on_eat` (`OnEat::resolve`),
    /// so the rules and the wiki read one resolved gift per stat.
    pub(super) fn resolve_corpse_gifts(&mut self, corpses: &hunger::Corpses) -> Result<(), String> {
        for creature in &mut self.creatures {
            if creature.id == "player" || !creature.corpse {
                continue;
            }
            let tags = corpses.gifts_of(&creature.tags);
            let mut on_eat = creature.on_eat.take().unwrap_or_default();
            on_eat.resolve(&tags);
            creature.on_eat = (!on_eat.is_empty()).then_some(on_eat);
        }
        // What a corpse teaches is a skill with its words in `[corpses.skills]` (without
        // `data/skills/` there are no skills, and corpses teach nothing).
        if self.skills().is_empty() {
            return Ok(());
        }
        if let Some(id) = (corpses.skills.keys()).find(|id| self.skill_id(id).is_none()) {
            return Err(format!("corpses: skills: unknown skill `{id}`"));
        }
        for creature in &self.creatures {
            let Some(on_eat) = &creature.on_eat else {
                continue;
            };
            if let Some(id) = (on_eat.skills.keys()).find(|id| !corpses.skills.contains_key(*id)) {
                return Err(format!(
                    "creature `{}`: on_eat: skills: `{id}` has no words in [corpses.skills]",
                    creature.id
                ));
            }
        }
        Ok(())
    }
}

/// One file of the material table.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MaterialFile {
    #[serde(default)]
    material: Vec<MaterialDef>,
}

/// One file of the object table.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectFile {
    #[serde(default)]
    object: Vec<object::ObjectSpec>,
}
