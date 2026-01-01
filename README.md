# Grape

Grape est un lecteur musique/audio desktop en Rust, inspiré par Dopamine. Le projet vise une
expérience rapide et claire pour explorer une bibliothèque locale et lancer la lecture.

## État actuel

- **UI desktop Iced** : layout complet (top bar, colonnes, player bar) avec navigation et états.
- **Scan local** : lecture de dossiers `Artiste/Album` et création d'un catalogue en mémoire.
- **Durées audio** : lecture des durées via les métadonnées (crate `lofty`).
- **Cache** : catalogue sauvegardé dans `.grape_cache.json` pour accélérer les démarrages.
- **Lecture audio** : module `player` basé sur `rodio` (pas encore relié à l'UI).

## Stack technique

- Rust (édition 2024)
- Iced (UI)
- Rodio (audio)
- Lofty (métadonnées audio)

## Structure du dépôt

- `assets/` : visuels et assets UI (logos, fonts, maquettes, captures).
- `docs/` : documentation produit et technique.
- `src/` : code applicatif (UI, bibliothèque, player).
- `tasks/` : tâches, roadmap, analyse UI.

## Démarrage rapide

```bash
cargo run -- /chemin/vers/ma/library
```

Si aucun chemin n'est fourni, Grape utilise `./library`.

### Structure attendue de la bibliothèque

Grape scanne une structure simple de dossiers :

```
Library/
  Artiste/
    2003 - Album Name/
      01 - Titre.mp3
      02 - Autre titre.flac
```

Formats supportés pour le scan : `mp3`, `flac`, `wav`, `ogg`, `m4a`.

### Cache local

Après un scan réussi, un fichier `.grape_cache.json` est écrit dans le dossier de la bibliothèque.
Le cache est invalidé si la date de modification du dossier racine change.

## Documentation

- Vue d'ensemble : `docs/docs.md`
- Source & modules : `src/src.md`
- Roadmap : `tasks/roadmap/roadmap.md`
- Analyse UI : `tasks/ui_analysis.md`

## Feuille de route (résumé)

- Brancher la lecture audio à l'UI
- Améliorer l'indexation (métadonnées, jaquettes, cache plus fin)
- Ajouter la gestion des playlists
