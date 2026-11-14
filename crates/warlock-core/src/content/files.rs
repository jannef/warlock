//! The raw content files a [`Content`] is loaded from, and which of them were read.

use super::{CAMPAIGNS, ContentError, walk};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;

mod builtin {
    include!(concat!(env!("OUT_DIR"), "/builtin_assets.rs"));
}

/// Raw content files keyed by path relative to the assets root, e.g. `data/creatures/rat.toml`.
#[derive(Clone, Debug, Default)]
pub struct ContentFiles {
    pub(super) files: BTreeMap<String, String>,
    /// The paths a loader has looked up (#1530): [`Content::load`] fails on any file it was
    /// given that is not among them ([`ContentFiles::check_all_read`]).
    pub(super) read: RefCell<BTreeSet<String>>,
}

impl ContentFiles {
    /// The content compiled into the binary from `assets/`.
    pub fn builtin() -> Self {
        let mut files = Self::default();
        for (path, text) in builtin::FILES {
            files.insert(path, text);
        }
        files
    }

    /// Reads the `campaigns/`, `data/`, and `vaults/` TOML files under `root`, subdirectories
    /// included (`data/creatures/kobold.toml`): the same set, under the same paths, as `build.rs`
    /// embeds from `assets/` (one walk, `content/walk.rs`), the test world left out without the
    /// `showcase` feature.
    pub fn from_dir(root: &Path) -> std::io::Result<Self> {
        let mut files = Self::default();
        for (rel, path) in walk::walk(root, cfg!(feature = "showcase"), &mut |_| {})? {
            files.insert(&rel, &std::fs::read_to_string(&path)?);
        }
        Ok(files)
    }

    pub fn insert(&mut self, path: &str, text: &str) {
        self.files.insert(path.to_string(), text.to_string());
    }

    /// Leaves a file out, e.g. `data/weather.toml` for an always clear sky, or a whole directory
    /// (`data/creatures/`, with or without the slash) for a test that brings its own table.
    pub fn remove(&mut self, path: &str) {
        self.files.remove(path);
        let dir = format!("{}/", path.trim_end_matches('/'));
        self.files.retain(|p, _| !p.starts_with(&dir));
    }

    /// The files of the content table `name`, in the order its entries are numbered:
    /// `data/<name>.toml` first, if there is one, then every file under `data/<name>/`
    /// (subdirectories included) in path order, then every campaign's `data/<name>/` the same
    /// way, the campaigns in [`ContentFiles::campaign_order`]. Each is `(path, text)`.
    pub fn table<'a>(&'a self, name: &str) -> impl Iterator<Item = (&'a str, &'a str)> {
        let all = self.table_unread(name);
        self.mark(all.iter().map(|(p, _)| *p));
        all.into_iter()
    }

    /// [`ContentFiles::table`], without counting its files as read.
    pub(super) fn table_unread<'a>(&'a self, name: &str) -> Vec<(&'a str, &'a str)> {
        let single = format!("data/{name}.toml");
        let mut all: Vec<(&str, &str)> = (self.files.get_key_value(&single).into_iter())
            .map(|(p, t)| (p.as_str(), t.as_str()))
            .collect();
        let campaigns = self.campaign_order();
        let dirs = std::iter::once(format!("data/{name}/")).chain(
            campaigns
                .iter()
                .map(|c| format!("{CAMPAIGNS}/{c}/data/{name}/")),
        );
        for dir in dirs {
            let files = (self.files.range(dir.clone()..))
                .take_while(|(p, _)| p.starts_with(&dir))
                .filter(|(p, _)| p.ends_with(".toml"));
            all.extend(files.map(|(p, t)| (p.as_str(), t.as_str())));
        }
        all
    }

    /// Rewrites every file of table `name` with `edit` (tests: a built-in creature with a line
    /// changed, whichever file it is in).
    pub fn edit_table(&mut self, name: &str, mut edit: impl FnMut(&str) -> String) {
        let edited: Vec<(String, String)> = (self.table_unread(name).into_iter())
            .map(|(p, t)| (p.to_string(), edit(t)))
            .collect();
        for (path, text) in edited {
            self.insert(&path, &text);
        }
    }

    pub(super) fn get(&self, path: &str) -> Result<&str, ContentError> {
        self.get_optional(path)
            .ok_or_else(|| ContentError::new(path, "file is missing"))
    }

    pub(super) fn get_optional(&self, path: &str) -> Option<&str> {
        self.mark([path]);
        self.files.get(path).map(String::as_str)
    }

    /// Counts `paths` as read by a loader.
    pub(super) fn mark<'a>(&self, paths: impl IntoIterator<Item = &'a str>) {
        let mut read = self.read.borrow_mut();
        for path in paths {
            if !read.contains(path) {
                read.insert(path.to_string());
            }
        }
    }

    /// Fails on the first file no loader read (#1530): a misnamed file (`data/feints.toml`) or
    /// directory (`data/town/`), or a file in a place nothing looks, would otherwise load cleanly
    /// with its feature quietly off. `README.toml` files, comments for the authors, are never
    /// read. Checks whatever set was loaded, so a file added on top of the built-in set is held to
    /// it too.
    pub(super) fn check_all_read(&self) -> Result<(), ContentError> {
        let read = self.read.borrow();
        let unread = (self.files.keys())
            .filter(|p| !p.ends_with("/README.toml") && p.as_str() != "README.toml")
            .find(|p| !read.contains(p.as_str()));
        match unread {
            Some(path) => Err(ContentError::new(
                path,
                "no loader reads this file: is its name or directory misspelt?",
            )),
            None => Ok(()),
        }
    }

    /// Files directly under `dir/` with their stem (`levels/hub.toml` → `hub`), then those
    /// directly under each campaign's `dir/` (`campaigns/showcase/levels/hub.toml`), the
    /// campaigns in [`ContentFiles::campaign_order`]: each `(path, stem, text)`.
    pub(super) fn in_dir<'a>(&'a self, dir: &'a str) -> Vec<(&'a str, &'a str, &'a str)> {
        let under = |prefix: String| {
            self.files.iter().filter_map(move |(path, text)| {
                let rest = path.strip_prefix(&prefix)?;
                let stem = rest.strip_suffix(".toml")?;
                (!stem.contains('/')).then_some((path.as_str(), stem, text.as_str()))
            })
        };
        let mut all: Vec<_> = under(format!("{dir}/")).collect();
        for campaign in self.campaign_order() {
            all.extend(under(format!("{CAMPAIGNS}/{campaign}/{dir}/")));
        }
        self.mark(all.iter().map(|(p, ..)| *p));
        all
    }
}
