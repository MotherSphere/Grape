# Analyse UI — Référence et mapping code

Cette analyse documente la structure UI actuelle et sert de référence pour les prochains
chantiers. La plupart des blocs décrits ci-dessous ont déjà des composants Iced associés.

## Résumé visuel

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Top bar                                                                      │
│ - Logo/app icon (gauche)                                                     │
│   - Menu vertical: Bibliothèque | Playlist | Queue | Préférences             │
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
│ - Sections accordéon (startup, langue, updates, privacy, storage, etc.)       │
│ - Contrôles toggles/sliders + actions (cache, logs, reset)                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

La vue Queue remplace la grille principale quand elle est ouverte :

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Queue                                                                        │
│ - Liste des pistes en file d'attente                                        │
│ - Actions : activer lecture queue, vider, réordonner, supprimer             │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Mapping composants → code

- **Top bar** : `ui::app::top_bar` (`src/ui/app.rs`)
- **Menu logo (Grape)** : `AnchoredOverlay` + `UiMessage::OpenPlaylist` (`src/ui/app.rs`)
- **Playlist view** : `PlaylistView` (`src/ui/components/playlist_view.rs`)
- **Queue view** : `QueueView` (`src/ui/components/queue_view.rs`)
- **Préférences** : `preferences_view` (`src/ui/app.rs`)
- **Equalizer** : `audio_settings` (`src/ui/components/audio_settings.rs`)
- **Artists** : `ArtistsPanel` (`src/ui/components/artists_panel.rs`)
- **Genres** : `GenresPanel` (`src/ui/components/genres_panel.rs`)
- **Albums grid** : `AlbumsGrid` (`src/ui/components/albums_grid.rs`)
- **Folders grid/list** : `FoldersPanel` (`src/ui/components/folders_panel.rs`)
- **Songs list** : `SongsPanel` (`src/ui/components/songs_panel.rs`)
  - Éditeur de métadonnées album (genre/année) dans la liste de pistes
- **Player bar** : `PlayerBar` (`src/ui/components/player_bar.rs`)

## États UI

- Onglet actif : `ActiveTab` (Artists/Genres/Albums/Folders)
- Vue playlist : `ui.playlist_open` (`UiState`)
- Vue queue : `ui.queue_open` (`UiState`)
- Vue préférences : `ui.preferences_open` (`UiState`)
- Sélections : `SelectionState` (artist/album/genre/folder/track)
- Lecture : `PlaybackState` (position, durée, shuffle, repeat)
- Recherche : `SearchState` (query + tri)
- Préférences : `UserSettings` (thème, audio, accessibilité, stockage)
- Sections Préférences : `PreferencesSection` (startup, updates, privacy, notifications, audio...)

## Données affichées

- Les listes proviennent du `Catalog` chargé au démarrage.
- Les durées des pistes sont lues via `lofty` lorsque disponibles.
- Les jaquettes sont copiées dans le cache local si détectées.
- Les identifiants UI sont générés à partir de l'ordre du catalogue.

## Écarts actuels vs design cible

- Les genres sont dérivés (un genre « Unknown » si tags absents).
- Certaines préférences restent déclaratives (updates, privacy, performance).

## Pistes de travail (prochaines étapes)

1. Enrichir les genres via metadata en ligne.
2. Étendre la recherche/tri (filtres avancés, tags).
3. Compléter les actions Préférences (updates/logs/analytics).
