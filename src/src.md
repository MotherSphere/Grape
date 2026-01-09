# Source — Vue d'ensemble

Ce dossier contient le code applicatif Rust du lecteur Grape.

## Entrée

- `main.rs`
  - Initialise le logging.
  - Lance l'application Iced via `ui::run` (scan déclenché côté UI).

## Modules

- `config.rs`
  - Préférences utilisateur (thème, audio, accessibilité, stockage).
  - Chargement/sauvegarde JSON dans `~/.config/grape/preferences.json`.
- `library.rs`
  - Scan du disque et construction d'un `Catalog`.
  - Convention : dossiers `Artiste/Album` + fichiers audio.
  - Parsing des années/numéros depuis les noms de dossier/fichier.
  - Métadonnées audio (durée, codec, genre, année, cover embarquée).
  - Enrichissement optionnel en ligne (Last.fm).
  - Détection des jaquettes et cache local.
- `library/cache.rs`
  - Cache JSON dans le dossier configuré (par défaut `.grape_cache/` si chemin relatif).
  - Index global de signatures de pistes + cache par dossier d'album.
  - Cache des covers + metadata locales + metadata online.
  - Invalidation par signature (taille + date de modification).
- `library/metadata.rs`
  - Lecture des métadonnées audio via `lofty`.
- `library/metadata/online.rs`
  - Enrichissement album (genre/année) via Last.fm + cache.
- `player.rs`
  - Abstraction de lecture audio (`rodio`).
  - Sortie audio configurable + EQ/normalisation.
  - Méthodes : `load`, `play`, `pause`, `seek`.
- `playlist.rs`
  - Modèle de playlist + sérialisation JSON (`~/.config/grape/playlist.json`).
  - File de lecture `PlaybackQueue` utilisée par Next/Previous + vue queue dédiée.
- `ui/`
  - Layout et états UI.
  - Composants : `ArtistsPanel`, `AlbumsGrid`, `GenresPanel`, `FoldersPanel`,
    `SongsPanel`, `PlayerBar`, `PlaylistView`.
  - Styles centralisés dans `ui/style.rs`.

## Notes

- La lecture audio est branchée à la sélection de pistes dans l'UI.
- La playlist est connectée au modèle et persistée localement.
- Les préférences modifient l'état UI et sont persistées.
