//! A structural check of the optimized web build: the table wasm-bindgen's JS glue grows at
//! startup (`__wbindgen_externrefs`) must be a growable `externref` table. An older binaryen
//! (Ubuntu's apt package) exported the fixed-size function table under that name instead, and the
//! page failed with "WebAssembly.Table.grow(): failed to grow table by 4".

use std::path::Path;

/// The export the JS glue calls `grow` on.
const EXTERNREFS: &str = "__wbindgen_externrefs";
const EXTERNREF: u8 = 0x6f;

/// A table's element type and size limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Table {
    elem: u8,
    min: u64,
    max: Option<u64>,
}

pub fn check(wasm: &Path) -> Result<(), String> {
    let bytes = std::fs::read(wasm).map_err(|e| format!("cannot read {}: {e}", wasm.display()))?;
    match externrefs_table(&bytes)? {
        None => {
            eprintln!("note: {EXTERNREFS} is not exported; skipping the table check");
            Ok(())
        }
        Some(t) if t.elem == EXTERNREF && t.max.is_none_or(|max| max > t.min) => Ok(()),
        Some(t) => Err(format!(
            "{EXTERNREFS} is exported as a table the JS glue cannot grow ({t:?}); \
             wasm-opt is probably too old (use binaryen 133 or newer)"
        )),
    }
}

/// The table exported as [`EXTERNREFS`], if any.
fn externrefs_table(b: &[u8]) -> Result<Option<Table>, String> {
    if b.get(..8) != Some(b"\0asm\x01\0\0\0".as_slice()) {
        return Err("not a wasm module".into());
    }
    let mut tables = Vec::new();
    let mut export = None;
    let mut r = Reader { b, i: 8 };
    while r.i < b.len() {
        let id = r.byte()?;
        let size = r.leb()? as usize;
        let end = r.i + size;
        match id {
            2 => {
                for _ in 0..r.leb()? {
                    r.name()?;
                    r.name()?;
                    match r.byte()? {
                        0 => {
                            r.leb()?;
                        }
                        1 => tables.push(r.table()?),
                        2 => {
                            r.limits()?;
                        }
                        3 => {
                            r.byte()?;
                            r.byte()?;
                        }
                        // Tags and newer import kinds: tables precede nothing we need here.
                        _ => break,
                    }
                }
            }
            4 => {
                for _ in 0..r.leb()? {
                    tables.push(r.table()?);
                }
            }
            7 => {
                for _ in 0..r.leb()? {
                    let name = r.name()?;
                    let kind = r.byte()?;
                    let index = r.leb()? as usize;
                    if kind == 1 && name == EXTERNREFS.as_bytes() {
                        export = Some(index);
                    }
                }
            }
            _ => {}
        }
        r.i = end;
    }
    match export {
        None => Ok(None),
        Some(i) => tables
            .get(i)
            .copied()
            .map(Some)
            .ok_or_else(|| format!("{EXTERNREFS} exports table {i}, which does not exist")),
    }
}

struct Reader<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Reader<'a> {
    fn byte(&mut self) -> Result<u8, String> {
        let x = *self.b.get(self.i).ok_or("truncated wasm")?;
        self.i += 1;
        Ok(x)
    }

    fn leb(&mut self) -> Result<u64, String> {
        let (mut result, mut shift) = (0u64, 0);
        loop {
            let x = self.byte()?;
            result |= u64::from(x & 0x7f) << shift;
            if x < 0x80 {
                return Ok(result);
            }
            shift += 7;
            if shift > 63 {
                return Err("bad LEB128 in wasm".into());
            }
        }
    }

    fn name(&mut self) -> Result<&'a [u8], String> {
        let len = self.leb()? as usize;
        let start = self.i;
        self.i += len;
        self.b
            .get(start..self.i)
            .ok_or_else(|| "truncated wasm".into())
    }

    fn limits(&mut self) -> Result<(u64, Option<u64>), String> {
        let flags = self.byte()?;
        let min = self.leb()?;
        let max = if flags & 1 == 1 {
            Some(self.leb()?)
        } else {
            None
        };
        Ok((min, max))
    }

    fn table(&mut self) -> Result<Table, String> {
        let elem = self.byte()?;
        let (min, max) = self.limits()?;
        Ok(Table { elem, min, max })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A module with a fixed funcref table (0) and a growable externref table (1), exporting
    /// `export` as the externrefs table.
    fn module(export: u8) -> Vec<u8> {
        let mut b = b"\0asm\x01\0\0\0".to_vec();
        // Table section: funcref 3..=3, externref min 2, no max.
        b.extend([4, 8, 2, 0x70, 1, 3, 3, EXTERNREF, 0, 2]);
        let name = EXTERNREFS.as_bytes();
        let mut export_section = vec![1, name.len() as u8];
        export_section.extend(name);
        export_section.extend([1, export]);
        b.push(7);
        b.push(export_section.len() as u8);
        b.extend(export_section);
        b
    }

    #[test]
    fn accepts_the_externref_table_and_rejects_the_function_table() {
        let good = externrefs_table(&module(1)).expect("parses");
        assert_eq!(
            good,
            Some(Table {
                elem: EXTERNREF,
                min: 2,
                max: None
            })
        );
        let bad = externrefs_table(&module(0))
            .expect("parses")
            .expect("exported");
        assert_eq!(bad.elem, 0x70);
        assert_eq!(bad.max, Some(3));
    }
}
