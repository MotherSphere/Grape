# Source — Vue d'ensemble

Ce dossier contient le code applicatif Rust du lecteur Grape.

## Entrée

- `main.rs`
  - Charge le `Catalog` depuis un chemin local.
  - Lance l'application Iced via `ui::run`.

## Modules

- `library.rs`
  - Scan du disque et construction d'un `Catalog`.
  - Convention simple : dossiers `Artiste/Album` + fichiers audio.
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
- Le scan des durées/métadonnées est prévu pour une étape suivante.
