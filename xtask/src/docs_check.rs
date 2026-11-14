//! `cargo xtask docs-check` (#1667): every code path the docs name exists.
//!
//! The docs read before every task (CLAUDE.md, `spec/`, `docs/dev/`; not `docs/design/`, which
//! is history) name code in inline code spans in three forms: a file, `ui/screens.rs`, perhaps
//! with an item in it, `ui/screens.rs::ScreenName`; a type's item, `World::state_hash`; and a
//! module's, `background::tell`, `ui::step_dialogs::trap_dialog`, `warlock_core::GAME_VERSION`.
//! Each is looked up in the sources under `crates/` and `xtask/`:
//!
//! - a file by its path's tail (`world/allies/familiar.rs` matches
//!   `crates/warlock-core/src/world/allies/familiar.rs`);
//! - a type in the files that define it (`struct`, `enum`, `trait`, `type`, `union`) or `impl`
//!   something for it;
//! - a module by the longest run of its leading segments that is a file or folder
//!   (`ui::step_dialogs` → `ui/step_dialogs.rs`), within a crate when it starts with one;
//! - then every remaining segment, an item, must be a word of those files (or of the test files
//!   they pull in with `#[path]`), outside `//` comments.
//!
//! It is a grep, not a compiler: a word found is enough, so it catches a file moved or deleted, a
//! type or function renamed or gone, not every stale detail. Names from outside the workspace
//! (`std`, Bevy, clap, the WESL shader imports) are passed over by their first segment
//! ([`EXTERNAL`]); [`IGNORED`] lists the few intentional examples.

use crate::workspace_root;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

/// First segments of names from outside the workspace: other crates, std's types, Bevy's.
const EXTERNAL: &[&str] = &[
    "alloc",
    "AssetPath",
    "bevy",
    "clap",
    "constants",
    "core",
    "f32",
    "f64",
    "i32",
    "i64",
    "image",
    "insta",
    "KeyCode",
    "Option",
    "Res",
    "Result",
    "serde",
    "std",
    "String",
    "Tonemapping",
    "toml",
    "u8",
    "u16",
    "u32",
    "u64",
    "usize",
    "Vec",
    "wgpu",
];

/// Names that are no path in this workspace on purpose: Bevy's examples and its `Key` (the
/// workspace has a `Key` of its own), and the test-file naming pattern.
const IGNORED: &[&str] = &[
    "_tests.rs",
    "examples/shader/shader_material_2d.rs",
    "examples/window/screenshot.rs",
    "Key::ArrowUp",
    "Key::Character",
];

/// The crates a name may start with, and their source folders.
const CRATES: &[(&str, &str)] = &[
    ("warlock", "crates/warlock/"),
    ("warlock_assets", "crates/warlock-assets/"),
    ("warlock_cli", "crates/warlock-cli/"),
    ("warlock_core", "crates/warlock-core/"),
    ("xtask", "xtask/"),
];

pub fn run() -> Result<(), String> {
    let root = workspace_root();
    let sources = Sources::read(&root)?;
    let mut docs = vec![root.join("CLAUDE.md")];
    for dir in ["spec", "docs/dev"] {
        markdown_files(&root.join(dir), &mut docs)?;
    }
    docs.sort();

    let mut names = 0;
    let mut distinct = BTreeSet::new();
    let mut broken = Vec::new();
    for doc in &docs {
        let text = std::fs::read_to_string(doc)
            .map_err(|e| format!("cannot read {}: {e}", doc.display()))?;
        let shown = doc.strip_prefix(&root).unwrap_or(doc).display().to_string();
        for (line, name) in code_names(&text) {
            if IGNORED.contains(&name.as_str()) {
                continue;
            }
            let Some(result) = sources.resolve(&name) else {
                continue;
            };
            names += 1;
            distinct.insert(name.clone());
            if let Err(why) = result {
                broken.push(format!("{shown}:{line}: `{name}`: {why}"));
            }
        }
    }
    println!(
        "docs-check: {names} code names ({} distinct) in {} files",
        distinct.len(),
        docs.len()
    );
    if broken.is_empty() {
        return Ok(());
    }
    for line in &broken {
        eprintln!("{line}");
    }
    Err(format!(
        "{} code names in the docs name nothing in the sources",
        broken.len()
    ))
}

fn markdown_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            markdown_files(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
    Ok(())
}

/// The code names in a Markdown text's inline code spans, with their line numbers (fenced blocks
/// are left out): runs of path and identifier characters that name a `.rs` file or hold a `::`.
fn code_names(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut fenced = false;
    for (i, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        for span in line.split('`').skip(1).step_by(2) {
            let mut start = None;
            for (at, c) in span.char_indices().chain([(span.len(), ' ')]) {
                let part = c.is_ascii_alphanumeric() || "_./:-".contains(c);
                match (part, start) {
                    (true, None) => start = Some(at),
                    (false, Some(from)) => {
                        start = None;
                        // A glob (`*_tests.rs`) is a pattern, not a path.
                        if span[..from].ends_with('*') {
                            continue;
                        }
                        if let Some(name) = code_name(&span[from..at]) {
                            out.push((i + 1, name));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    out
}

/// A run's code name: up to and including `.rs` and any `::` items after it, or a `::` path of
/// identifiers; `None` for anything else (action text, TOML, a Markdown file).
fn code_name(run: &str) -> Option<String> {
    // `shoot_tests::a_shade_in_a_wall_is_...` names an item by the start of its name.
    let ellipsis = if run.ends_with("...") { "..." } else { "" };
    code_name_whole(run.trim_end_matches('.')).map(|name| name + ellipsis)
}

fn code_name_whole(run: &str) -> Option<String> {
    let run = run.trim_matches(|c| c == '.' || c == ':' || c == '-');
    if let Some(at) = run.find(".rs") {
        let (path, rest) = run.split_at(at + 3);
        let stem = path.trim_end_matches(".rs");
        // `<id>.rs` and `<topic>/mod.rs` are placeholders, not paths.
        if stem.is_empty() || stem.starts_with('/') || stem.ends_with('/') || !path_like(stem) {
            return None;
        }
        let items = rest
            .strip_prefix("::")
            .map(|r| r.split("::").take_while(|s| ident(s)));
        let mut name = path.trim_start_matches("./").to_string();
        for item in items.into_iter().flatten() {
            name.push_str("::");
            name.push_str(item);
        }
        return Some(name);
    }
    if !run.contains("::") || run.contains('/') {
        return None;
    }
    let segments: Vec<&str> = run.split("::").collect();
    segments.iter().all(|s| ident(s)).then(|| run.to_string())
}

fn ident(s: &str) -> bool {
    s.chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// A type's name: `World`, `ScreenName`, not a constant (`GAME_VERSION`) or a module.
fn type_name(s: &str) -> bool {
    s.starts_with(|c: char| c.is_ascii_uppercase()) && s.contains(|c: char| c.is_ascii_lowercase())
}

fn path_like(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || "_./-".contains(c))
}

/// The workspace's Rust sources: each file's words and the files it pulls in with `#[path]`,
/// and where each type is defined or implemented.
struct Sources {
    /// Paths relative to the workspace root, `/`-separated, sorted.
    files: Vec<String>,
    words: Vec<HashSet<String>>,
    /// Per file, the files its `#[path = "..."]` modules are.
    linked: Vec<Vec<usize>>,
    types: BTreeMap<String, BTreeSet<usize>>,
}

impl Sources {
    fn read(root: &Path) -> Result<Sources, String> {
        let mut paths = Vec::new();
        for dir in ["crates", "xtask"] {
            rust_files(root, &root.join(dir), &mut paths)?;
        }
        paths.sort();
        let index: BTreeMap<&str, usize> = paths
            .iter()
            .enumerate()
            .map(|(i, p)| (p.as_str(), i))
            .collect();
        let mut words = Vec::new();
        let mut linked = Vec::new();
        let mut types: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
        for (i, path) in paths.iter().enumerate() {
            let text = std::fs::read_to_string(root.join(path))
                .map_err(|e| format!("cannot read {path}: {e}"))?;
            let code: String = text
                .lines()
                .map(|l| l.find("//").map_or(l, |at| &l[..at]))
                .collect::<Vec<_>>()
                .join("\n");
            let tokens = idents(&code);
            for pair in tokens.windows(2) {
                if ["struct", "enum", "trait", "type", "union"].contains(&pair[0]) {
                    types.entry(pair[1].to_string()).or_default().insert(i);
                }
            }
            for implemented in impl_targets(&code) {
                types.entry(implemented).or_default().insert(i);
            }
            words.push(tokens.iter().map(|t| t.to_string()).collect());
            let dir = path.rsplit_once('/').map_or("", |(d, _)| d);
            linked.push(
                text.split("#[path = \"")
                    .skip(1)
                    .filter_map(|rest| rest.split_once('"'))
                    .filter_map(|(file, _)| index.get(format!("{dir}/{file}").as_str()).copied())
                    .collect(),
            );
        }
        Ok(Sources {
            files: paths,
            words,
            linked,
            types,
        })
    }

    /// Whether `name` names something in the sources: `None` when it is from outside them,
    /// else `Ok` or why not.
    fn resolve(&self, name: &str) -> Option<Result<(), String>> {
        if let Some(at) = name.find(".rs") {
            let (path, rest) = name.split_at(at + 3);
            let files: Vec<usize> = (0..self.files.len())
                .filter(|&i| {
                    let f = &self.files[i];
                    f == path || f.ends_with(&format!("/{path}"))
                })
                .collect();
            if files.is_empty() {
                return Some(Err("no such file".into()));
            }
            let items: Vec<&str> = rest.split("::").filter(|s| !s.is_empty()).collect();
            return Some(self.has_items(&files, &items));
        }
        let mut segments: Vec<&str> = name.split("::").collect();
        while matches!(segments.first(), Some(&("crate" | "self" | "super"))) {
            segments.remove(0);
        }
        let mut scope = "";
        if let Some(&(_, dir)) = CRATES.iter().find(|(c, _)| Some(c) == segments.first()) {
            scope = dir;
            segments.remove(0);
        }
        let first = *segments.first()?;
        let module_first = !type_name(first);
        if scope.is_empty()
            && (EXTERNAL.contains(&first) || first.starts_with("bevy") || first.starts_with("Bevy"))
        {
            return None;
        }
        let in_scope = |i: &usize| self.files[*i].starts_with(scope);
        let (files, items): (Vec<usize>, &[&str]) = if !module_first {
            let files = self.types.get(first).into_iter().flatten().copied();
            (files.filter(in_scope).collect(), &segments[1..])
        } else {
            // The longest run of leading segments that is a module file or folder.
            let module = (1..=segments.len()).rev().find_map(|k| {
                let tail = segments[..k].join("/");
                let files: Vec<usize> = (0..self.files.len())
                    .filter(in_scope)
                    .filter(|&i| {
                        let f = format!("/{}", self.files[i]);
                        f.ends_with(&format!("/{tail}.rs")) || f.contains(&format!("/{tail}/"))
                    })
                    .collect();
                (!files.is_empty()).then_some((files, k))
            });
            match module {
                Some((files, k)) => (files, &segments[k..]),
                None if !scope.is_empty() => (
                    (0..self.files.len()).filter(in_scope).collect(),
                    &segments[..],
                ),
                None => (Vec::new(), &segments[..]),
            }
        };
        if files.is_empty() {
            return Some(Err(format!("no type or module `{first}`")));
        }
        let found = self.has_items(&files, items);
        // A module's type may be defined elsewhere and used from there
        // (`input::Context::Play`): its items are then looked for where the type is.
        if found.is_err()
            && module_first
            && let Some(at) = items.iter().position(|s| type_name(s))
            && let Some(files) = self.types.get(items[at])
        {
            let files: Vec<usize> = files.iter().copied().collect();
            if self.has_items(&files, &items[at + 1..]).is_ok() {
                return Some(Ok(()));
            }
        }
        Some(found)
    }

    /// Whether every item is a word of `files` or the files they pull in.
    fn has_items(&self, files: &[usize], items: &[&str]) -> Result<(), String> {
        let mut all: Vec<usize> = files.to_vec();
        for &f in files {
            all.extend(&self.linked[f]);
        }
        let found = |item: &str| match item.strip_suffix("...") {
            Some(start) => all
                .iter()
                .any(|&f| self.words[f].iter().any(|w| w.starts_with(start))),
            None => all.iter().any(|&f| self.words[f].contains(item)),
        };
        match items.iter().find(|item| !found(item)) {
            Some(item) => Err(format!("no `{item}` there")),
            None => Ok(()),
        }
    }
}

fn rust_files(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), String> {
    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n != "target") {
                rust_files(root, &path, out)?;
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

/// The identifiers of a text, in order.
fn idents(text: &str) -> Vec<&str> {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|w| w.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_'))
        .collect()
}

/// The types a text's `impl` blocks are for: `impl<T> Trait for path::Type<T> where … {` gives
/// `Type`.
fn impl_targets(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (at, _) in text.match_indices("impl") {
        let before = text[..at].chars().next_back();
        if before.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        let rest = &text[at + 4..];
        if !rest.starts_with([' ', '<']) {
            continue;
        }
        let head = rest.split(['{', ';']).next().unwrap_or("");
        let head = head.split(" where").next().unwrap_or(head);
        let target = head.rsplit_once(" for ").map_or(head, |(_, t)| t);
        // Drop the impl's own generics, then the target's.
        let target = if target.trim_start().starts_with('<') {
            skip_generics(target.trim_start())
        } else {
            target
        };
        let path = target.trim().split('<').next().unwrap_or("");
        if let Some(name) = path.rsplit("::").next().map(str::trim)
            && ident(name)
        {
            out.push(name.to_string());
        }
    }
    out
}

/// What follows a leading `<...>`, nesting counted.
fn skip_generics(s: &str) -> &str {
    let mut depth = 0;
    for (at, c) in s.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return &s[at + 1..];
                }
            }
            _ => {}
        }
    }
    ""
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_names_are_files_items_and_paths() {
        let text = "See `world/mod.rs::Spent` and `World::state_hash(force)`, `move:N`, \
                    `*_tests.rs`, `docs/dev/cli.md`, `ui::step_dialogs::trap_dialog`, `x_tests::a_b_...`.\n\
                    ```\nignored::Fenced\n```\n`a.rs`";
        let names: Vec<String> = code_names(text).into_iter().map(|(_, n)| n).collect();
        assert_eq!(
            names,
            [
                "world/mod.rs::Spent",
                "World::state_hash",
                "ui::step_dialogs::trap_dialog",
                "x_tests::a_b_...",
                "a.rs"
            ]
        );
    }

    #[test]
    fn impl_targets_skip_generics_and_traits() {
        let text = "impl<T: Clone> Foo for crate::x::Bar<T> where T: Copy {}\nimpl Baz {}";
        assert_eq!(impl_targets(text), ["Bar", "Baz"]);
    }
}
