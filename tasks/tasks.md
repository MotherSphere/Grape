# Tâches Grape

## Avancement

- [x] Initialisation du dépôt Rust (Cargo).
- [x] Mise en place des dossiers de base (`docs`, `tasks`, `src`, `assets`).
- [x] Scan local de bibliothèque (structure Artiste/Album/Pistes).
- [x] Lecture des durées via métadonnées (`lofty`).
- [x] Cache JSON local (`.grape_cache.json`).
- [x] Prototype UI Iced (layout global + composants principaux).
- [x] Gestion d'un état UI (sélections, recherche, lecture).
- [x] Module de lecture audio (`rodio`) non câblé à l'UI.

## À faire (priorité MVP)

- [ ] Connecter l'UI au module `player` (play/pause/seek).
- [ ] Ajouter un tri/recherche fonctionnel dans les listes.
- [ ] Gestion de playlists basique.

## À planifier (après MVP)

- [ ] Cache d'indexation plus fin (par dossier, JSON/SQLite).
- [ ] Jaquettes et métadonnées enrichies.
- [ ] Amélioration de la navigation (genres, dossiers).
- [ ] Accessibilité et theming avancé.
