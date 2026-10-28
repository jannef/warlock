//! `cargo xtask wiki --check-links`: every link in the built wiki leads somewhere.
//!
//! Runs after `mdbook build wiki`, on the HTML it wrote (`wiki/book/`), so it sees the pages as
//! readers do: the generated tables included, `.md` links rewritten to `.html`, and the heading
//! ids mdBook gave (lowercase, hyphenated, `-1` on a repeat). For each chapter (a page of
//! `wiki/src/`) it reads every `href` and `src` and checks that a relative target exists and,
//! with a `#fragment`, that the target page has that id, and that no page defines an id twice
//! (a generated row's anchor clashing with a heading's). Absolute links (`https:`, `mailto:`) are
//! not fetched. Every miss is listed, by the Markdown page it came from, and the check fails.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;

/// Checks the book built in `book` against the chapters in `src`.
pub(super) fn check(src: &Path, book: &Path) -> Result<(), String> {
    let mut chapters = Vec::new();
    for entry in std::fs::read_dir(src).map_err(|e| format!("{}: {e}", src.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().is_some_and(|x| x == "md") {
            let stem = path
                .file_stem()
                .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
            if stem != "SUMMARY" {
                chapters.push(stem);
            }
        }
    }
    chapters.sort();

    let mut ids: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut misses = Vec::new();
    let mut links = 0;
    for stem in &chapters {
        let page = format!("{stem}.html");
        let Some(html) = read_page(book, &page, &mut ids) else {
            misses.push(format!(
                "{stem}.md: not built ({page} missing; is it in SUMMARY.md?)"
            ));
            continue;
        };
        let mut seen = BTreeSet::new();
        for id in attribute_values(&html, "id") {
            if !seen.insert(id.clone()) {
                misses.push(format!("{stem}.md: anchor #{id} is defined twice"));
            }
        }
        for target in attribute_values(&html, "href")
            .into_iter()
            .chain(attribute_values(&html, "src"))
        {
            if is_external(&target) {
                continue;
            }
            links += 1;
            let (file, fragment) = match target.split_once('#') {
                Some((file, fragment)) => (file, Some(fragment)),
                None => (target.as_str(), None),
            };
            let file = if file.is_empty() { page.as_str() } else { file };
            if file.ends_with(".md") {
                misses.push(format!(
                    "{stem}.md: link to {target} was not rewritten by mdBook"
                ));
                continue;
            }
            if !book.join(file).exists() {
                misses.push(format!("{stem}.md: no page {file} (link {target})"));
                continue;
            }
            let Some(fragment) = fragment else { continue };
            if fragment.is_empty() || !file.ends_with(".html") {
                continue;
            }
            if read_page(book, file, &mut ids).is_none() {
                continue;
            }
            if !ids[file].contains(fragment) {
                misses.push(format!("{stem}.md: no anchor #{fragment} in {file}"));
            }
        }
    }

    if misses.is_empty() {
        println!(
            "wiki: {links} links in {} pages, all resolve",
            chapters.len()
        );
        Ok(())
    } else {
        for miss in &misses {
            eprintln!("  {miss}");
        }
        Err(format!("wiki: {} broken links", misses.len()))
    }
}

/// The page's HTML, recording its ids the first time it is read; `None` when it does not exist.
fn read_page(
    book: &Path,
    page: &str,
    ids: &mut BTreeMap<String, BTreeSet<String>>,
) -> Option<String> {
    let html = std::fs::read_to_string(book.join(page)).ok()?;
    ids.entry(page.to_string())
        .or_insert_with(|| attribute_values(&html, "id").into_iter().collect());
    Some(html)
}

fn is_external(target: &str) -> bool {
    target.starts_with("//") || target.contains(':')
}

/// Every value of `name="..."` in the HTML, entity `&amp;` decoded.
fn attribute_values(html: &str, name: &str) -> Vec<String> {
    let needle = format!(" {name}=\"");
    html.match_indices(&needle)
        .filter_map(|(at, _)| {
            let rest = &html[at + needle.len()..];
            rest.find('"').map(|end| rest[..end].replace("&amp;", "&"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::attribute_values;

    #[test]
    fn reads_attribute_values() {
        let html = r#"<h2 id="a-b"><a class="header" href="x.html#a-b">A</a></h2><p data-id="no">"#;
        assert_eq!(attribute_values(html, "id"), ["a-b"]);
        assert_eq!(attribute_values(html, "href"), ["x.html#a-b"]);
    }
}
