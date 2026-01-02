# Tâches Grape

## Avancement

- [x] Initialisation du dépôt Rust (Cargo).
- [x] Mise en place des dossiers de base (`docs`, `tasks`, `src`, `assets`).
- [x] Scan local de bibliothèque (structure Artiste/Album/Pistes).
- [x] Lecture des durées via métadonnées (`lofty`).
- [x] Cache local `.grape_cache/` (index + cache par dossier).
- [x] Détection et cache des jaquettes d'album.
- [x] Prototype UI Iced (layout global + composants principaux).
- [x] Gestion d'un état UI (sélections, recherche, lecture).
- [x] Onglets Genres/Folders avec panels dédiés.
- [x] Recherche/tri appliqués aux listes.
- [x] Module de lecture audio (`rodio`) branché à la sélection de pistes.
- [x] Queue de lecture simple (Next/Previous) basée sur la sélection.
- [x] Écrans Préférences (General/Appearance/Accessibility/Audio).
- [x] Persistance des préférences (`~/.config/grape/preferences.json`).
- [x] Vue playlist (placeholder UI).

## À faire (priorité MVP)

- [ ] Connecter la playlist UI au modèle (`playlist.rs`) et afficher le contenu.
- [ ] Exposer la queue dans l'UI (vue dédiée + état actif).
- [ ] Persister les playlists (JSON local).
- [ ] Brancher les actions Préférences (réindexation, logs, reset audio).

## À planifier (après MVP)

- [ ] Genres réels (lecture métadonnées + agrégation).
- [ ] Cache d'indexation plus fin (par piste, JSON/SQLite).
- [ ] Amélioration de la navigation (panneaux contextuels, raccourcis).
- [ ] Accessibilité et theming avancé.
