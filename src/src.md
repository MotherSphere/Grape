# Source — Vue d'ensemble

Ce dossier contient le code applicatif Rust du lecteur Grape.

## Entrée

- `main.rs`
  - Charge le `Catalog` depuis un chemin local avec cache.
  - Lance l'application Iced via `ui::run`.

## Modules

- `library.rs`
  - Scan du disque et construction d'un `Catalog`.
  - Convention : dossiers `Artiste/Album` + fichiers audio.
  - Parsing des années/numéros depuis les noms de dossier/fichier.
- `library/cache.rs`
  - Cache JSON `.grape_cache.json` à la racine de la bibliothèque.
  - Invalidation par date de modification du dossier racine.
- `library/metadata.rs`
  - Lecture des durées audio via `lofty`.
- `player.rs`
  - Abstraction de lecture audio (`rodio`).
  - Méthodes : `load`, `play`, `pause`, `seek`.
- `playlist.rs`
  - Modèle minimal de playlist (liste de pistes).
- `ui/`
  - Layout et états UI.
  - Composants : `ArtistsPanel`, `AlbumsGrid`, `SongsPanel`, `PlayerBar`.
  - Styles centralisés dans `ui/style.rs`.

## Notes

- La lecture audio n'est pas encore câblée à l'UI.
- Les durées sont remplies via les métadonnées quand elles existent.
