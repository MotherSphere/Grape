mod config;
mod library;
mod player;
mod playlist;
mod ui;

use std::path::PathBuf;

use crate::library::Catalog;

fn main() {
    tracing_subscriber::fmt::init();

    let library_root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("library"));

    let catalog = match library::scan_library(&library_root) {
        Ok(catalog) => catalog,
        Err(err) => {
            eprintln!(
                "Erreur lors du scan de {:?}: {err}. Utilisation d'une bibliothèque vide.",
                library_root
            );
            Catalog::empty()
        }
    };

    if let Err(err) = ui::run(catalog) {
        eprintln!("Erreur UI: {err}");
    }
}
