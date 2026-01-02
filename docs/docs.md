# Documentation Grape

Cette documentation couvre l'état actuel du projet, l'architecture et les choix UI.

## Objectifs produit

- Explorer rapidement une bibliothèque locale.
- Lancer la lecture sans latence visible.
- Proposer une interface claire et moderne.

## Architecture (vue rapide)

- **Entrée** : `src/main.rs`
  - Lance un scan du catalogue via `library::scan_library`.
  - Lance l'UI via `ui::run`.
- **Bibliothèque** : `src/library.rs` + `src/library/`
  - Scan de dossiers, structure `Artist/Album/Track`.
  - Parsing des noms de dossiers/fichiers pour année et numéro de piste.
  - Lecture des durées audio via `library::metadata` (crate `lofty`).
  - Détection de jaquettes et cache des couvertures.
- **Cache** : `src/library/cache.rs`
  - Dossier `.grape_cache/` en racine de la bibliothèque.
  - Index global + un fichier JSON par dossier d'album.
  - Invalidation par dossier en fonction de la date de modification.
- **Lecture audio** : `src/player.rs`
  - Player `rodio` (load/play/pause/seek).
  - Branché sur la sélection de piste dans l'UI.
- **UI** : `src/ui/*`
  - Iced (layout en 3 colonnes + player bar).
  - État UI centralisé (`UiState`).
  - Vues dédiées pour Artists/Albums/Genres/Folders + playlist.

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

## Données du catalogue

- Les artistes et albums sont chargés depuis le scan local.
- Les durées proviennent des métadonnées (`lofty`) quand elles sont disponibles.
- Les jaquettes sont copiées dans le cache local si détectées.
- Les onglets Genres/Folders sont alimentés par des résumés dérivés du catalogue.

## Assets

Le dossier `assets/` est dédié aux éléments visuels (logos, fonts, captures, icônes).

## Limitations actuelles

- La playlist est une vue placeholder (modèle non connecté à l'UI).
- Les genres sont dérivés (actuellement un genre « Unknown » global).
- Le cache est indexé par dossier d'album, sans détection fine au niveau piste.

## Prochaines étapes suggérées

- Relier le modèle de playlist (`playlist.rs`) à l'UI.
- Enrichir les métadonnées (genres réels, jaquettes embarquées).
- Ajouter une file de lecture et des actions Next/Previous réelles.
