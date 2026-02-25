//! Interface with the zk notebook via zk-rs.

use std::path::PathBuf;

use pulldown_cmark::{Event, Parser};

pub use zk_rs::Note;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to convert env var: {0}")]
    Var(#[from] std::env::VarError),
    #[error("zk error: {0}")]
    Zk(#[from] zk_rs::Error),
}

pub struct Notebook {
    /// Path to the notebook.
    pub path: PathBuf,
    inner: zk_rs::Notebook,
}

impl Notebook {
    pub fn load() -> Result<Self, Error> {
        let path = PathBuf::from(std::env::var("ZK_NOTEBOOK_DIR")?);
        let inner = zk_rs::Notebook::load(&path)?;
        Ok(Self { path, inner })
    }

    /// Reload the note with the given stem.
    pub fn reload(&mut self, stem: &str) -> Result<(), Error> {
        self.inner.reload(stem)?;
        Ok(())
    }

    /// Remove the note with the given stem from the in-memory notebook.
    pub fn remove(&mut self, stem: &str) {
        self.inner.remove(stem);
    }

    /// Return note with the given filename stem or `None`.
    pub fn note(&self, stem: &str) -> Option<Note> {
        self.inner.note(stem).cloned()
    }

    /// Return all notes, optionally filtered by a required tag, sorted by last modified (most recent first).
    pub fn all_notes(&self, with_tag: Option<&str>) -> Vec<&Note> {
        let mut notes: Vec<&Note> = self.inner.all_notes(with_tag).collect();
        notes.sort_by(|a, b| b.modified().cmp(&a.modified()));
        notes
    }

    /// Return notes for given tag, sorted by last modified (most recent first).
    pub fn search_tag(&self, tag: &str) -> Vec<&Note> {
        let mut notes: Vec<&Note> = self.inner.all_notes(Some(tag)).collect();
        notes.sort_by(|a, b| b.modified().cmp(&a.modified()));
        notes
    }

    /// Fuzzy search for `query` inside titles and return matching [`Note`]s.
    pub fn search_titles(&self, query: &str, with_tag: Option<&str>) -> Vec<&Note> {
        self.inner.search_titles(query, with_tag).collect()
    }

    /// Return notes that wiki-link to `stem`.
    /// If `authenticated` is false, only public-tagged notes are returned.
    pub fn backlinks(&self, stem: &str, authenticated: bool) -> Vec<Note> {
        self.inner
            .backlinks(stem)
            .into_iter()
            .filter(|n| authenticated || n.has("public"))
            .cloned()
            .collect()
    }

    /// Resolve outgoing wiki-link stems to notes.
    /// If `authenticated` is false, only public-tagged notes are returned.
    pub fn outgoing_links(&self, stems: &[String], authenticated: bool) -> Vec<Note> {
        stems
            .iter()
            .filter_map(|stem| self.inner.note(stem))
            .filter(|n| authenticated || n.has("public"))
            .cloned()
            .collect()
    }
}

/// Extension trait for weave-specific [`Note`] methods.
pub trait NoteExt {
    /// Return a truncated snippet version of the note with ellipsis at the end.
    fn snippet(&self) -> String;
}

impl NoteExt for Note {
    fn snippet(&self) -> String {
        Parser::new(self.body())
            .scan(0, |len, event| {
                if *len >= 40 {
                    return None;
                }
                match event {
                    Event::Text(t) | Event::Code(t) => {
                        *len += t.len();
                        Some(Some(t))
                    }
                    Event::SoftBreak | Event::HardBreak => {
                        *len += 1;
                        Some(Some(" ".into()))
                    }
                    _ => Some(None),
                }
            })
            .flatten()
            .map(|t| t.into_string())
            .collect()
    }
}
