# Documentation Grape

Cette documentation couvre l'état actuel du projet, l'architecture et les choix UI.

## Objectifs produit

- Explorer rapidement une bibliothèque locale.
- Lancer la lecture sans latence visible.
- Proposer une interface claire et moderne.

## Architecture (vue rapide)

- **Entrée** : `src/main.rs`
  - Charge un catalogue via `library::cache::load` puis `library::scan_library` si besoin.
  - Sauvegarde le cache dans `.grape_cache.json`.
  - Lance l'UI via `ui::run`.
- **Bibliothèque** : `src/library.rs` + `src/library/`
  - Scan de dossiers, structure `Artist/Album/Track`.
  - Parsing des noms de dossiers/fichiers pour année et numéro de piste.
  - Lecture des durées audio via `library::metadata` (crate `lofty`).
- **Cache** : `src/library/cache.rs`
  - Fichier `.grape_cache.json` en racine de la bibliothèque.
  - Invalidation simple via la date de modification du dossier racine.
- **Lecture audio** : `src/player.rs`
  - Player `rodio` (load/play/pause/seek).
  - Pas encore branché à l'UI.
- **UI** : `src/ui/*`
  - Iced (layout en 3 colonnes + player bar).
  - État UI centralisé (`UiState`).

## UI : layout et composants

La maquette actuelle est structurée ainsi :

```
Top bar  → navigation + recherche + boutons fenêtre
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
- `SearchState` : query + tri (`SortOption`).

## Données du catalogue

- Les artistes et albums sont chargés depuis le scan local.
- Les durées proviennent des métadonnées (`lofty`) quand elles sont disponibles.
- Le cache évite un scan complet si la bibliothèque n'a pas changé.

## Assets

Le dossier `assets/` est dédié aux éléments visuels (logos, fonts, captures, icônes).

## Limitations actuelles

- La lecture audio n'est pas reliée à l'UI.
- Les onglets Genres/Folders sont visibles mais non implémentés.
- Le cache utilise la date de modification du dossier racine (pas de détection fine).

## Prochaines étapes suggérées

- Brancher les actions UI au module `player`.
- Ajouter un cache d'indexation plus fin (JSON/SQLite) avec détection par dossier.
- Récupérer les métadonnées enrichies et jaquettes.
