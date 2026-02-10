use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("not a zk notebook (missing .zk/ directory): {0}")]
    NotANotebook(PathBuf),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("failed to parse YAML frontmatter in {path}")]
    Yaml {
        path: String,
        source: serde_yaml::Error,
    },
}
