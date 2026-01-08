# Documentation Grape

Cette documentation couvre l'état actuel du projet, l'architecture et les choix UI.

## Objectifs produit

- Explorer rapidement une bibliothèque locale.
- Lancer la lecture sans latence visible.
- Proposer une interface claire et moderne.

## Architecture (vue rapide)

- **Entrée** : `src/main.rs`
  - Initialise le logging.
  - Démarre l'UI via `ui::run`, qui pilote le scan initial.
- **Bibliothèque** : `src/library.rs` + `src/library/`
  - Scan de dossiers, structure `Artist/Album/Track`.
  - Parsing des noms de dossiers/fichiers pour année et numéro de piste.
  - Lecture des métadonnées audio via `library::metadata` (crate `lofty`).
  - Détection de covers embarquées + fallback sur images locales.
  - Enrichissement optionnel via `library::metadata::online` (Last.fm).
- **Cache** : `src/library/cache.rs`
  - Dossier `.grape_cache/` en racine de la bibliothèque (ou chemin configuré).
  - Index global des signatures de pistes + JSON par dossier d'album.
  - Cache covers + cache metadata (Last.fm).
  - Invalidation par signature (taille + date de modification).
- **Lecture audio** : `src/player.rs`
  - Player `rodio` (load/play/pause/seek).
  - Sortie audio configurable (périphérique + sample rate).
  - Traitement EQ et normalisation de volume.
- **Playlists & queue** : `src/playlist.rs`
  - Modèle de playlist + sérialisation JSON (`~/.config/grape/playlist.json`).
  - Queue de lecture (`PlaybackQueue`) basée sur la playlist active.
- **UI** : `src/ui/*`
  - Iced (layout en 3 colonnes + player bar).
  - État UI centralisé (`UiState`).
  - Vues dédiées pour Artists/Albums/Genres/Folders + playlist.
- **Préférences** : `src/config.rs`
  - Paramètres persistés dans `~/.config/grape/preferences.json`.
  - Actions locales (clear cache, clear history, reset audio, reindex) exposées dans l'UI.

## UI : layout et composants

La maquette actuelle est structurée ainsi :

```
Top bar  → navigation + recherche + boutons fenêtre
Colonnes → Artistes | Albums | Titres (ou Genres/Folders selon l'onglet)
Footer   → player bar (transport + progression)
```

Composants Iced :

- `ArtistsPanel` (`src/ui/components/artists_panel.rs`)
- `AlbumsGrid` (`src/ui/components/albums_grid.rs`)
- `GenresPanel` (`src/ui/components/genres_panel.rs`)
- `FoldersPanel` (`src/ui/components/folders_panel.rs`)
- `SongsPanel` (`src/ui/components/songs_panel.rs`)
- `PlayerBar` (`src/ui/components/player_bar.rs`)
- `PlaylistView` (`src/ui/components/playlist_view.rs`)

## État UI

- `ActiveTab` : Artists / Genres / Albums / Folders.
- `SelectionState` : artiste, album, genre, dossier, piste.
- `PlaybackState` : position, durée, lecture, shuffle, repeat.
- `SearchState` : query + tri (`SortOption`).
- `UiState` : menu, playlist ouverte, états combinés.
- `UserSettings` : préférences (apparence, accessibilité, audio, stockage, etc.).

## Données du catalogue

- Les artistes et albums sont chargés depuis le scan local.
- Les métadonnées proviennent de `lofty` (durées, codec, bitrate, genre, année).
- Les jaquettes embarquées sont prioritaires, sinon copie locale dans le cache.
- Les onglets Genres/Folders sont alimentés par des résumés dérivés du catalogue.
- L'enrichissement en ligne (Last.fm) est optionnel via clé API.

## Assets

Le dossier `assets/` est dédié aux éléments visuels (logos, fonts, captures, icônes).

## Limitations actuelles

- L'édition de playlist est limitée (pas de réordonnancement ni suppression d'items depuis l'UI).
- Les genres restent « Unknown » si les tags audio sont absents.
- L'égaliseur est limité aux bandes préconfigurées (3 ou 5) avec des gains entre -12 dB et +12 dB.
- Si un périphérique audio sélectionné n'est pas disponible, la sortie repasse sur le système.

## Prochaines étapes suggérées

- Compléter les actions playlist (réorder, suppression de pistes, vue queue dédiée).
- Enrichir les métadonnées (genres réels, sources en ligne supplémentaires).
- Étendre les préférences (actions système avancées, logs détaillés).
