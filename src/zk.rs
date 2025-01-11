//! Interface with the `zk` binary.

use serde::Deserialize;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to call `zk` binary: {0}")]
    Process(#[from] std::io::Error),
    #[error("failed to open stdout pipe")]
    Pipe,
    // #[error("failed to load from disk")]
    // Load,
    #[error("failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),
}

/// A zk note.
#[derive(Deserialize, Debug)]
pub struct Note {
    pub filename: String,
    #[serde(rename = "filenameStem")]
    pub filename_stem: String,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

pub struct Notebook {
    pub notes: Vec<Note>,
    /// Maps tag name to [`Note`] filename
    pub tags: HashMap<String, Vec<String>>,
}

/// Construct tag [`HashMap`] from given `notes` [`Note`] slice.
fn tags_from_notes(notes: &[Note]) -> HashMap<String, Vec<String>> {
    let mut tags: HashMap<String, Vec<String>> = HashMap::new();

    for note in notes.iter() {
        for tag in &note.tags {
            let tag = tag.clone();
            tags.entry(tag).or_default().push(note.filename.clone());
        }
    }

    tags
}

impl Notebook {
    pub fn load() -> Result<Self, Error> {
        let mut child = Command::new("zk")
            .args(["list", "--format", "jsonl"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdout = child.stdout.take().ok_or(Error::Pipe)?;

        let notes = BufReader::new(stdout)
            .lines()
            .flat_map(|line| line.map(|line| serde_json::from_str(&line)))
            .collect::<Result<Vec<Note>, _>>()?;

        child.wait()?;

        let tags = tags_from_notes(&notes);

        Ok(Self { notes, tags })
    }

    /// Return note with the given filename stem or `None`.
    pub fn note(&self, stem: &str) -> Option<&Note> {
        self.notes.iter().find(|note| note.filename_stem == stem)
    }

    /// Return notes for given tag.
    pub fn search_tag(&self, tag: &str) -> Vec<&Note> {
        let Some(filenames) = self.tags.get(tag) else {
            return vec![];
        };

        filenames
            .iter()
            .filter_map(|filename| self.notes.iter().find(|note| note.filename == *filename))
            .collect()
    }

    /// Fuzzy search for `query` inside titles and return matching [`Note`]s.
    pub fn search_titles(&self, query: &str, with_tag: Option<&str>) -> Vec<&Note> {
        let mut matcher = nucleo::Matcher::new(nucleo::Config::DEFAULT);
        let mut needle_buf = Vec::new();
        let mut haystack_buf = Vec::new();
        let needle = nucleo::Utf32Str::new(query, &mut needle_buf);

        self.notes
            .iter()
            .filter(|note| {
                let haystack = nucleo::Utf32Str::new(&note.title, &mut haystack_buf);
                let matches = matcher.fuzzy_match(haystack, needle).is_some();
                let has_tag = with_tag.map(|tag| note.has(tag)).unwrap_or(true);

                matches && has_tag
            })
            .collect()
    }
}

impl Note {
    /// Return a truncated snippet version of the note with ellipsis at the end.
    pub(crate) fn snippet(&self) -> String {
        let body = &self.body;

        let snippet = match body.char_indices().nth(30) {
            Some((byte_index, _)) => &body[0..byte_index],
            None => body,
        };

        format!("{snippet}...")
    }

    /// Return `true` if note has `tag`.
    pub(crate) fn has(&self, tag: &str) -> bool {
        self.tags.iter().any(|tagged| *tagged == tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::Alphanumeric;
    use rand::{Rng, thread_rng};

    impl Note {
        /// Generate a random new note.
        fn new(title: &str, body: &str, tags: Vec<String>) -> Self {
            let filename_stem: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(10)
                .map(char::from)
                .collect();
            let filename = format!("{filename_stem}.md");

            Self {
                filename,
                filename_stem,
                title: title.to_owned(),
                body: body.to_owned(),
                tags,
            }
        }
    }

    #[test]
    fn search_titles() {
        let notes = vec![Note::new("foo", "", vec![]), Note::new("bar", "", vec![])];
        let tags = tags_from_notes(&notes);
        let notebook = Notebook { notes, tags };
        let result = notebook.search_titles("fo", None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "foo");
    }
}
