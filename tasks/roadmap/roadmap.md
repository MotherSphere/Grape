# Roadmap

## Phase 1 — MVP

- Scan local fonctionnel (`library.rs`).
- UI desktop Iced (layout + navigation).
- Métadonnées audio via `lofty` (durées, tags, covers embarquées).
- Cache JSON `.grape_cache/` (index piste + cache par dossier).
- Jaquettes en cache local (covers).
- Lecture audio connectée à l'UI (sélection + play/pause/seek).
- Queue de lecture (Next/Previous) et états shuffle/repeat.
- Vue dédiée de la queue + actions (vider/réordonner/supprimer).
- Préférences UI (General/Appearance/Accessibility/Audio) + persistance locale.
- Playlists persistées (JSON local) + vue dédiée + édition (réordonnancement/suppression).
- EQ et options de sortie audio (périphérique + sample rate).

## Phase 2 — Expérience

- Recherche avancée et filtres.
- Cache d'indexation plus fin (JSON/SQLite) par piste.
- Genres réels via métadonnées + sources en ligne.
- Actions Préférences avancées (logs détaillés, reset audio).

## Phase 3 — Qualité audio & finition

- Jaquettes et métadonnées enrichies (embed + online).
- Accessibilité, thèmes, polish UX.
