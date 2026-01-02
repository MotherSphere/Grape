# Source — Vue d'ensemble

Ce dossier contient le code applicatif Rust du lecteur Grape.

## Entrée

- `main.rs`
  - Lance le scan du `Catalog` depuis un chemin local.
  - Lance l'application Iced via `ui::run`.

## Modules

- `config.rs`
  - Préférences utilisateur (thème, audio, accessibilité, stockage).
  - Chargement/sauvegarde JSON dans `~/.config/grape/preferences.json`.
- `library.rs`
  - Scan du disque et construction d'un `Catalog`.
  - Convention : dossiers `Artiste/Album` + fichiers audio.
  - Parsing des années/numéros depuis les noms de dossier/fichier.
  - Détection des jaquettes et cache local.
- `library/cache.rs`
  - Cache JSON `.grape_cache/` à la racine de la bibliothèque.
  - Index global + cache par dossier d'album.
  - Invalidation par date de modification du dossier.
- `library/metadata.rs`
  - Lecture des durées audio via `lofty`.
- `player.rs`
  - Abstraction de lecture audio (`rodio`).
  - Méthodes : `load`, `play`, `pause`, `seek`.
- `playlist.rs`
  - Modèle de playlist + sérialisation JSON.
  - File de lecture `PlaybackQueue` utilisée par Next/Previous.
- `ui/`
  - Layout et états UI.
  - Composants : `ArtistsPanel`, `AlbumsGrid`, `GenresPanel`, `FoldersPanel`,
    `SongsPanel`, `PlayerBar`, `PlaylistView`.
  - Styles centralisés dans `ui/style.rs`.

## Notes

- La lecture audio est branchée à la sélection de pistes dans l'UI.
- La playlist est disponible comme vue, mais pas encore connectée aux données.
- Les préférences modifient l'état UI et sont persistées.
