fn main() -> anyhow::Result<()> {
    if let Some(path) = std::env::args().into_iter().skip(1).next() {
        let path = std::path::PathBuf::from(path);
        let notebook = zk_rs::Notebook::load(path)?;

        for note in notebook.all_notes(None) {
            println!("{}: {}", note.filename_stem(), note.title());
        }
    } else {
        eprintln!("usage: list <path>");
    }

    Ok(())
}
