use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Note {
    pub(crate) filename: String,
    pub(crate) filename_stem: String,
    pub(crate) path: PathBuf,
    pub(crate) abs_path: PathBuf,
    pub(crate) title: String,
    pub(crate) link: String,
    pub(crate) lead: String,
    pub(crate) body: String,
    pub(crate) raw_content: String,
    pub(crate) word_count: usize,
    pub(crate) tags: Vec<String>,
    pub(crate) aliases: Vec<String>,
    pub(crate) created: jiff::Timestamp,
    pub(crate) modified: jiff::Timestamp,
}

impl Note {
    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn filename_stem(&self) -> &str {
        &self.filename_stem
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn abs_path(&self) -> &Path {
        &self.abs_path
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn link(&self) -> &str {
        &self.link
    }

    pub fn lead(&self) -> &str {
        &self.lead
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn raw_content(&self) -> &str {
        &self.raw_content
    }

    pub fn word_count(&self) -> usize {
        self.word_count
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }

    pub fn created(&self) -> jiff::Timestamp {
        self.created
    }

    pub fn modified(&self) -> jiff::Timestamp {
        self.modified
    }

    pub fn has(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
    }
}
