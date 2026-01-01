# Documentation Grape

Cette documentation couvre l'état actuel du projet, l'architecture et les choix UI.

## Objectifs produit

- Explorer rapidement une bibliothèque locale.
- Lancer la lecture sans latence visible.
- Proposer une interface claire et moderne.

## Architecture (vue rapide)

- **Entrée** : `src/main.rs`
  - Charge un catalogue via `library::scan_library`.
  - Lance l'UI via `ui::run`.
- **Bibliothèque** : `src/library.rs`
  - Scan de dossiers, structure `Artist/Album/Track`.
  - Construction d'un `Catalog` en mémoire.
- **Lecture audio** : `src/player.rs`
  - Player `rodio` (load/play/pause/seek).
  - Pas encore branché à l'UI.
- **UI** : `src/ui/*`
  - Iced (layout en 3 colonnes + player bar).
  - État UI centralisé (`UiState`).

## UI : layout et composants

La maquette actuelle est structurée ainsi :

```
Top bar  → navigation + recherche
Colonnes → Artistes | Albums | Titres
Footer   → player bar (transport + progression)
```

Composants Iced :

- `ArtistsPanel` (`src/ui/components/artists_panel.rs`)
- `AlbumsGrid` (`src/ui/components/albums_grid.rs`)
- `SongsPanel` (`src/ui/components/songs_panel.rs`)
- `PlayerBar` (`src/ui/components/player_bar.rs`)

## État UI

- `ActiveTab` : Artists / Genres / Albums / Folders.
- `SelectionState` : artiste, album, piste.
- `PlaybackState` : position, durée, lecture, shuffle, repeat.
- `SearchState` : query + tri.

## Assets

Le dossier `assets/` est dédié aux éléments visuels (logos, captures, icônes). Il sera
alimenté au fur et à mesure du design.

## Limitations actuelles

- Pas de parsing de métadonnées audio (durées à 0).
- La lecture audio n'est pas reliée à l'UI.
- Pas de cache persistant pour la bibliothèque.

## Prochaines étapes suggérées

- Brancher les actions UI au module `player`.
- Ajouter un cache d'indexation (JSON/SQLite).
- Récupérer les métadonnées et jaquettes.
