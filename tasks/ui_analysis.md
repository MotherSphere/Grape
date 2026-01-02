# Analyse UI — Référence et mapping code

Cette analyse documente la structure UI actuelle et sert de référence pour les prochains
chantiers. La plupart des blocs décrits ci-dessous ont déjà des composants Iced associés.

## Résumé visuel

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Top bar                                                                      │
│ - Logo/app icon (gauche)                                                     │
│   - Menu vertical: Bibliothèque | Playlist | Préférences                     │
│ - Tabs navigation: Artists | Genres | Albums | Folders                       │
│ - Search box + boutons fenêtre (droite)                                      │
├───────────────────────┬─────────────────────────────────┬──────────────────┤
│ Colonne gauche         │ Zone centrale                   │ Colonne droite   │
│ (Artists list)         │ (Albums grid)                   │ (Songs list)     │
│ - Index A–Z            │ - Album covers en grille        │ - Titre album     │
│ - Nombre d’artistes    │ - Titre + artiste sous cover    │ - Liste des       │
│ - Scroll vertical      │ - Scroll vertical               │   pistes          │
│ - Artist sélectionné   │ - Album sélectionné             │ - Durée à droite  │
├─────────────────────────────────────────────────────────────────────────────┤
│ Footer / Player bar                                                          │
│ - Artwork + titre en cours (gauche)                                          │
│ - Contrôles playback (centre)                                                │
│ - Progression + durée (droite)                                               │
│ - Options audio (volume, playlist, etc.)                                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Mapping composants → code

- **Top bar** : `ui::app::top_bar` (`src/ui/app.rs`)
- **Menu logo (Grape)** : `AnchoredOverlay` + `UiMessage::OpenPlaylist` (`src/ui/app.rs`)
- **Playlist view** : `PlaylistView` (`src/ui/components/playlist_view.rs`)
- **Artists** : `ArtistsPanel` (`src/ui/components/artists_panel.rs`)
- **Albums grid** : `AlbumsGrid` (`src/ui/components/albums_grid.rs`)
- **Songs list** : `SongsPanel` (`src/ui/components/songs_panel.rs`)
- **Player bar** : `PlayerBar` (`src/ui/components/player_bar.rs`)

## États UI

- Onglet actif : `ActiveTab` (uniquement Artists/Genres/Albums/Folders)
- Vue playlist : `ui.playlist_open` (`UiState`)
- Sélections : `SelectionState` (artist/album/track)
- Lecture : `PlaybackState` (position, durée, shuffle, repeat)
- Recherche : `SearchState` (query + tri)

## Données affichées

- Les listes proviennent du `Catalog` chargé au démarrage.
- Les durées des pistes sont lues via `lofty` lorsque disponibles.
- Les identifiants UI sont générés à partir de l'ordre du catalogue.

## Écarts actuels vs design cible

- Les actions de lecture ne sont pas branchées au module `player`.
- Les onglets Genres/Folders sont visibles mais non implémentés.
- La playlist est une vue séparée, ouverte via `UiMessage::OpenPlaylist` et
  pilotée uniquement par `ui.playlist_open` (pas par `ActiveTab`).
- Le tri/recherche n'impacte pas encore les listes.

## Pistes de travail (prochaines étapes)

1. Implémenter les vues Genres/Folders en reprenant la logique de navigation des onglets
   (comme Artists/Albums) et en conservant la playlist dans le menu vertical du logo.
2. Mapper les données Genres/Folders depuis `Catalog` pour alimenter ces nouveaux panneaux.
3. Connecter la sélection de piste → `player.load` + `player.play`.
4. Implémenter les interactions de recherche/tri.
5. Ajouter le chargement des jaquettes et métadonnées enrichies.
