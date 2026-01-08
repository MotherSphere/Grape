# Tâches Grape

## Avancement

- [x] Initialisation du dépôt Rust (Cargo).
- [x] Mise en place des dossiers de base (`docs`, `tasks`, `src`, `assets`).
- [x] Scan local de bibliothèque (structure Artiste/Album/Pistes).
- [x] Lecture des métadonnées audio via `lofty` (durées, tags, covers).
- [x] Cache local `.grape_cache/` (index piste + cache par dossier).
- [x] Détection et cache des jaquettes d'album (covers embarquées + fichiers).
- [x] Prototype UI Iced (layout global + composants principaux).
- [x] Gestion d'un état UI (sélections, recherche, lecture).
- [x] Onglets Genres/Folders avec panels dédiés.
- [x] Recherche/tri appliqués aux listes.
- [x] Module de lecture audio (`rodio`) branché à la sélection de pistes.
- [x] Queue de lecture simple (Next/Previous) basée sur la playlist active.
- [x] Sortie audio configurable + EQ/normalisation.
- [x] Écrans Préférences (General/Appearance/Accessibility/Audio).
- [x] Persistance des préférences (`~/.config/grape/preferences.json`).
- [x] Playlists connectées (création/renommage/suppression + ajout de pistes).
- [x] Persistance des playlists (`~/.config/grape/playlist.json`).
- [x] Actions Préférences (réindexation, clear cache/history, reset audio).

## À faire (priorité MVP)

- [ ] Permettre le réordonnancement/suppression d'items de playlist dans l'UI.
- [ ] Exposer une vue dédiée de la queue avec actions (clear/reorder).
- [ ] Afficher/éditer les métadonnées en ligne (genre/année) dans l'UI.

## À planifier (après MVP)

- [ ] Sources métadonnées supplémentaires (autres providers).
- [ ] Cache d'indexation plus fin (par piste, JSON/SQLite).
- [ ] Amélioration de la navigation (panneaux contextuels, raccourcis).
- [ ] Accessibilité et theming avancé.
