mod library;
mod player;
mod playlist;
mod ui;

use std::path::PathBuf;

use crate::library::Catalog;

fn main() {
    let library_root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("library"));

    let catalog = match library::cache::load(&library_root) {
        Ok(Some(catalog)) => catalog,
        Ok(None) => match library::scan_library(&library_root) {
            Ok(catalog) => {
                if let Err(err) = library::cache::store(&library_root, &catalog) {
                    eprintln!("Erreur lors de l'écriture du cache: {err}");
                }
                catalog
            }
            Err(err) => {
                eprintln!(
                    "Erreur lors du scan de {:?}: {err}. Utilisation d'une bibliothèque vide.",
                    library_root
                );
                Catalog::empty()
            }
        },
        Err(err) => {
            eprintln!("Erreur lors du chargement du cache: {err}");
            match library::scan_library(&library_root) {
                Ok(catalog) => {
                    if let Err(err) = library::cache::store(&library_root, &catalog) {
                        eprintln!("Erreur lors de l'écriture du cache: {err}");
                    }
                    catalog
                }
                Err(err) => {
                    eprintln!(
                        "Erreur lors du scan de {:?}: {err}. Utilisation d'une bibliothèque vide.",
                        library_root
                    );
                    Catalog::empty()
                }
            }
        }
    };

    if let Err(err) = ui::run(catalog) {
        eprintln!("Erreur UI: {err}");
    }
}
