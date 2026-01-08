# Analyse UI — Référence et mapping code

Cette analyse documente la structure UI actuelle et sert de référence pour les prochains
chantiers. La plupart des blocs décrits ci-dessous ont déjà des composants Iced associés.

## Résumé visuel

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Top bar                                                                      │
│ - Logo/app icon (gauche)                                                     │
│   - Menu vertical: Bibliothèque | Playlist | Préférences                     │
│ - Tabs navigation: Artists | Genres | Albums | Folders                       │
│ - Search box + boutons fenêtre (droite)                                      │
├────────────────────────┬─────────────────────────────────┬───────────────────┤
│ Colonne gauche         │ Zone centrale                   │ Colonne droite    │
│ (Artists/Genres list)  │ (Albums grid/Folders grid)      │ (Songs list)      │
│ - Index A–Z            │ - Album/folder covers en grille │ - Titre album     │
│ - Nombre d’entrées     │ - Titre + artiste sous cover    │ - Liste des       │
│ - Scroll vertical      │ - Scroll vertical               │   pistes          │
│ - Sélection active     │ - Sélection active              │ - Durée à droite  │
├──────────────────────────────────────────────────────────────────────────────┤
│ Footer / Player bar                                                          │
│ - Artwork + titre en cours (gauche)                                          │
│ - Contrôles playback (centre)                                                │
│ - Progression + durée (droite)                                               │
│ - Options audio (volume, playlist, etc.)                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

La vue Préférences remplace la grille principale quand elle est ouverte :

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Préférences                                                                   │
│ - Onglets : Général | Apparence | Accessibilité | Audio                       │
│ - Sections accordéon (startup, langue, stockage, etc.)                        │
│ - Contrôles toggles/sliders + actions (cache, logs, reset)                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Mapping composants → code

- **Top bar** : `ui::app::top_bar` (`src/ui/app.rs`)
- **Menu logo (Grape)** : `AnchoredOverlay` + `UiMessage::OpenPlaylist` (`src/ui/app.rs`)
- **Playlist view** : `PlaylistView` (`src/ui/components/playlist_view.rs`)
- **Préférences** : `preferences_view` (`src/ui/app.rs`)
- **Artists** : `ArtistsPanel` (`src/ui/components/artists_panel.rs`)
- **Genres** : `GenresPanel` (`src/ui/components/genres_panel.rs`)
- **Albums grid** : `AlbumsGrid` (`src/ui/components/albums_grid.rs`)
- **Folders grid/list** : `FoldersPanel` (`src/ui/components/folders_panel.rs`)
- **Songs list** : `SongsPanel` (`src/ui/components/songs_panel.rs`)
- **Player bar** : `PlayerBar` (`src/ui/components/player_bar.rs`)

## États UI

- Onglet actif : `ActiveTab` (Artists/Genres/Albums/Folders)
- Vue playlist : `ui.playlist_open` (`UiState`)
- Vue préférences : `ui.preferences_open` (`UiState`)
- Sélections : `SelectionState` (artist/album/genre/folder/track)
- Lecture : `PlaybackState` (position, durée, shuffle, repeat)
- Recherche : `SearchState` (query + tri)
- Préférences : `UserSettings` (thème, audio, accessibilité, stockage)

## Données affichées

- Les listes proviennent du `Catalog` chargé au démarrage.
- Les durées des pistes sont lues via `lofty` lorsque disponibles.
- Les jaquettes sont copiées dans le cache local si détectées.
- Les identifiants UI sont générés à partir de l'ordre du catalogue.

## Écarts actuels vs design cible

- La playlist est connectée, mais sans réordonnancement/suppression d'items.
- Les genres sont dérivés (un genre « Unknown » si tags absents).
- La file de lecture existe, mais n'est pas exposée dans une vue dédiée.
- Certaines préférences restent déclaratives (updates, privacy, performance).

## Pistes de travail (prochaines étapes)

1. Ajouter la gestion des playlists (réordre, suppression d'items).
2. Exposer la queue de lecture dans une vue dédiée (gestion de l'ordre).
3. Enrichir les genres via metadata en ligne.
4. Étendre la recherche/tri (filtres avancés, tags).
5. Compléter les actions Préférences (updates/logs/analytics).
