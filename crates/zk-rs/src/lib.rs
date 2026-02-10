mod error;
mod note;
mod parse;

pub use error::Error;
pub use note::Note;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Notebook {
    root: PathBuf,
    notes: Vec<Note>,
    stems: HashMap<String, usize>,
    tags: HashMap<String, Vec<String>>,
}

impl Notebook {
    /// Load all notes from a zk notebook directory.
    /// Returns `Error::NotANotebook` if `root/.zk/` does not exist.
    pub fn load(root: impl Into<PathBuf>) -> Result<Self, Error> {
        let root = root.into();
        let zk_dir = root.join(".zk");
        if !zk_dir.is_dir() {
            return Err(Error::NotANotebook(root));
        }

        let md_files = discover_md_files(&root)?;
        let mut notes = Vec::new();
        let mut stems = HashMap::new();
        let mut tags: HashMap<String, Vec<String>> = HashMap::new();

        for abs_path in md_files {
            let rel_path = abs_path
                .strip_prefix(&root)
                .unwrap_or(&abs_path)
                .to_path_buf();

            let note = load_single_note(abs_path, rel_path)?;
            let idx = notes.len();
            let stem = note.filename_stem().to_owned();
            for tag in note.tags() {
                tags.entry(tag.to_lowercase())
                    .or_default()
                    .push(stem.clone());
            }
            stems.insert(stem, idx);
            notes.push(note);
        }

        Ok(Notebook {
            root,
            notes,
            stems,
            tags,
        })
    }

    /// Reload a single note from disk by its stem.
    /// If the stem is unknown, the notebook is re-walked to find new files.
    pub fn reload(&mut self, stem: &str) -> Result<(), Error> {
        if let Some(&idx) = self.stems.get(stem) {
            // Re-read existing note
            let abs_path = self.notes[idx].abs_path().to_path_buf();
            let rel_path = abs_path
                .strip_prefix(&self.root)
                .unwrap_or(&abs_path)
                .to_path_buf();

            let new_note = load_single_note(abs_path, rel_path)?;

            // Remove old tags
            let old_tags: Vec<String> = self.notes[idx].tags().to_vec();
            for tag in &old_tags {
                if let Some(stems) = self.tags.get_mut(&tag.to_lowercase()) {
                    stems.retain(|s| s != stem);
                    if stems.is_empty() {
                        self.tags.remove(&tag.to_lowercase());
                    }
                }
            }

            // Add new tags
            for tag in new_note.tags() {
                self.tags
                    .entry(tag.to_lowercase())
                    .or_default()
                    .push(stem.to_owned());
            }

            self.notes[idx] = new_note;
        } else {
            // Walk to find the new file
            let md_files = discover_md_files(&self.root)?;
            for abs_path in md_files {
                let file_stem = abs_path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                if file_stem == stem {
                    let rel_path = abs_path
                        .strip_prefix(&self.root)
                        .unwrap_or(&abs_path)
                        .to_path_buf();

                    let note = load_single_note(abs_path, rel_path)?;
                    let idx = self.notes.len();
                    for tag in note.tags() {
                        self.tags
                            .entry(tag.to_lowercase())
                            .or_default()
                            .push(stem.to_owned());
                    }
                    self.stems.insert(stem.to_owned(), idx);
                    self.notes.push(note);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Look up a note by its filename stem.
    pub fn note(&self, stem: &str) -> Option<&Note> {
        self.stems.get(stem).map(|&idx| &self.notes[idx])
    }

    /// Return all notes, optionally filtered by tag.
    pub fn all_notes(&self, with_tag: Option<&str>) -> impl Iterator<Item = &Note> {
        let allowed: Option<HashSet<&str>> = with_tag.map(|tag| {
            self.tags
                .get(&tag.to_lowercase())
                .map(|v| v.iter().map(String::as_str).collect())
                .unwrap_or_default()
        });
        self.notes.iter().filter(move |note| {
            allowed
                .as_ref()
                .is_none_or(|s| s.contains(note.filename_stem()))
        })
    }

    /// Return all unique tags across all notes.
    pub fn all_tags(&self) -> impl Iterator<Item = &str> {
        self.tags.keys().map(String::as_str)
    }

    /// Return notes that have ALL of the given tags.
    pub fn notes_with_tags<'a>(&'a self, tags: &'a [&str]) -> impl Iterator<Item = &'a Note> {
        self.notes
            .iter()
            .filter(move |note| tags.is_empty() || tags.iter().all(|tag| note.has(tag)))
    }

    /// Fuzzy search note titles, returning matches ranked by score.
    pub fn search_titles(
        &self,
        query: &str,
        with_tag: Option<&str>,
    ) -> impl Iterator<Item = &Note> {
        if query.is_empty() {
            return self.all_notes(with_tag).collect::<Vec<_>>().into_iter();
        }

        let mut matcher = nucleo::Matcher::new(nucleo::Config::DEFAULT);
        let needle = nucleo::pattern::Pattern::parse(
            query,
            nucleo::pattern::CaseMatching::Ignore,
            nucleo::pattern::Normalization::Smart,
        );

        let mut scored: Vec<(u32, &Note)> = self
            .all_notes(with_tag)
            .filter_map(|note| {
                let mut buf = Vec::new();
                let haystack = nucleo::Utf32Str::new(note.title(), &mut buf);
                let score = needle.score(haystack, &mut matcher)?;
                Some((score, note))
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored
            .into_iter()
            .map(|(_, note)| note)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

/// Read a single markdown file and parse it into a Note.
fn load_single_note(abs_path: PathBuf, rel_path: PathBuf) -> Result<Note, Error> {
    let content = fs::read_to_string(&abs_path)?;
    let meta = fs::metadata(&abs_path)?;
    parse::parse_note(&content, rel_path, abs_path, &meta)
}

/// Recursively discover all `.md` files under `root`, skipping hidden directories.
fn discover_md_files(root: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut files = Vec::new();
    walk_dir(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), Error> {
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        // Skip hidden dirs/files
        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            walk_dir(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "md") {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_notebook() -> TempDir {
        let dir = TempDir::new().unwrap();
        // Create .zk directory
        fs::create_dir(dir.path().join(".zk")).unwrap();

        // Create a note with frontmatter
        fs::write(
            dir.path().join("note1.md"),
            "---\ntitle: First Note\ntags: [rust, testing]\ndate: 2024-06-15\n---\n# First Note\n\nThis is the body.\n\nSecond paragraph.",
        )
        .unwrap();

        // Create a note with just a heading
        fs::write(
            dir.path().join("note2.md"),
            "# Second Note\n\nAnother body here.",
        )
        .unwrap();

        // Create a subdirectory with a note
        fs::create_dir(dir.path().join("subdir")).unwrap();
        fs::write(
            dir.path().join("subdir").join("note3.md"),
            "---\ntitle: Sub Note\ntags: [rust]\n---\nContent of sub note.",
        )
        .unwrap();

        // Create a hidden directory that should be skipped
        fs::create_dir(dir.path().join(".hidden")).unwrap();
        fs::write(
            dir.path().join(".hidden").join("secret.md"),
            "# Secret\nShould not appear.",
        )
        .unwrap();

        dir
    }

    #[test]
    fn test_not_a_notebook() {
        let dir = TempDir::new().unwrap();
        let result = Notebook::load(dir.path());
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(matches!(err, Error::NotANotebook(_)));
    }

    #[test]
    fn test_load_notebook() {
        let dir = setup_notebook();
        let nb = Notebook::load(dir.path()).unwrap();
        assert_eq!(nb.all_notes(None).count(), 3);
    }

    #[test]
    fn test_note_lookup() {
        let dir = setup_notebook();
        let nb = Notebook::load(dir.path()).unwrap();
        let note = nb.note("note1").unwrap();
        assert_eq!(note.title(), "First Note");
        assert_eq!(note.filename(), "note1.md");
        assert_eq!(note.filename_stem(), "note1");
    }

    #[test]
    fn test_tags() {
        let dir = setup_notebook();
        let nb = Notebook::load(dir.path()).unwrap();

        let all_tags: Vec<&str> = nb.all_tags().collect();
        assert!(all_tags.contains(&"rust"));
        assert!(all_tags.contains(&"testing"));

        assert_eq!(nb.all_notes(Some("rust")).count(), 2);
        assert_eq!(nb.all_notes(Some("testing")).count(), 1);
    }

    #[test]
    fn test_notes_with_tags() {
        let dir = setup_notebook();
        let nb = Notebook::load(dir.path()).unwrap();

        let both: Vec<&Note> = nb.notes_with_tags(&["rust", "testing"]).collect();
        assert_eq!(both.len(), 1);
        assert_eq!(both[0].title(), "First Note");
    }

    #[test]
    fn test_hidden_dir_skipped() {
        let dir = setup_notebook();
        let nb = Notebook::load(dir.path()).unwrap();
        assert!(nb.note("secret").is_none());
    }

    #[test]
    fn test_note_fields() {
        let dir = setup_notebook();
        let nb = Notebook::load(dir.path()).unwrap();
        let note = nb.note("note1").unwrap();

        assert!(note.word_count() > 0);
        assert!(!note.link().is_empty());
        assert!(note.link().contains("note1.md"));
        assert_eq!(note.lead(), "# First Note");
        assert!(note.raw_content().contains("First Note"));
    }

    #[test]
    fn test_heading_title() {
        let dir = setup_notebook();
        let nb = Notebook::load(dir.path()).unwrap();
        let note = nb.note("note2").unwrap();
        assert_eq!(note.title(), "Second Note");
        assert_eq!(note.body(), "Another body here.");
    }

    #[test]
    fn test_subdirectory_note() {
        let dir = setup_notebook();
        let nb = Notebook::load(dir.path()).unwrap();
        let note = nb.note("note3").unwrap();
        assert_eq!(note.title(), "Sub Note");
        assert!(note.path().to_string_lossy().contains("subdir"));
    }

    #[test]
    fn test_reload_existing() {
        let dir = setup_notebook();
        let mut nb = Notebook::load(dir.path()).unwrap();

        // Modify the file
        fs::write(
            dir.path().join("note1.md"),
            "---\ntitle: Updated Title\ntags: [updated]\ndate: 2024-06-15\n---\nNew body.",
        )
        .unwrap();

        nb.reload("note1").unwrap();
        let note = nb.note("note1").unwrap();
        assert_eq!(note.title(), "Updated Title");
        assert_eq!(note.tags(), &["updated"]);
    }

    #[test]
    fn test_reload_new_note() {
        let dir = setup_notebook();
        let mut nb = Notebook::load(dir.path()).unwrap();
        assert_eq!(nb.all_notes(None).count(), 3);

        // Add a new file
        fs::write(
            dir.path().join("note4.md"),
            "---\ntitle: Brand New\n---\nNew note content.",
        )
        .unwrap();

        nb.reload("note4").unwrap();
        assert_eq!(nb.all_notes(None).count(), 4);
        let note = nb.note("note4").unwrap();
        assert_eq!(note.title(), "Brand New");
    }

    #[test]
    fn test_search_titles() {
        let dir = setup_notebook();
        let nb = Notebook::load(dir.path()).unwrap();

        let first = nb.search_titles("First", None).next();
        assert!(first.is_some());
        assert_eq!(first.unwrap().title(), "First Note");
    }

    #[test]
    fn test_search_titles_with_tag() {
        let dir = setup_notebook();
        let nb = Notebook::load(dir.path()).unwrap();

        let results: Vec<&Note> = nb.search_titles("Note", Some("testing")).collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title(), "First Note");
    }

    #[test]
    fn test_search_empty_query() {
        let dir = setup_notebook();
        let nb = Notebook::load(dir.path()).unwrap();

        assert_eq!(nb.search_titles("", None).count(), 3);
    }

    #[test]
    fn test_has_tag() {
        let dir = setup_notebook();
        let nb = Notebook::load(dir.path()).unwrap();
        let note = nb.note("note1").unwrap();
        assert!(note.has("rust"));
        assert!(note.has("Rust")); // case-insensitive
        assert!(!note.has("nonexistent"));
    }

    #[test]
    fn test_all_notes_no_filter() {
        let dir = setup_notebook();
        let nb = Notebook::load(dir.path()).unwrap();
        assert_eq!(nb.all_notes(None).count(), 3);
    }
}
