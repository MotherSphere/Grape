# Grape

Grape est un lecteur musique/audio desktop en Rust, inspiré par Dopamine. Le projet vise une
expérience rapide et claire pour explorer une bibliothèque locale et lancer la lecture.

## État actuel

- **UI desktop Iced** : layout complet (top bar, colonnes, player bar) avec navigation et états.
- **Scan local** : lecture de dossiers `Artiste/Album` et création d'un catalogue en mémoire.
- **Durées audio** : lecture des durées via les métadonnées (crate `lofty`).
- **Cache local** : cache par dossier d'album + index global dans `.grape_cache/`.
- **Jaquettes** : détection d'images locales et cache des couvertures d'album.
- **Lecture audio** : module `player` basé sur `rodio`, branché à la sélection de pistes.
- **File de lecture** : constitution d'une queue à partir de la sélection + actions Next/Previous.
- **Navigation enrichie** : onglets Genres/Folders + recherche/tri appliqués aux listes.
- **Préférences UI** : écrans General/Appearance/Accessibility/Audio avec persistance locale.
- **Playlist** : modèle en mémoire + vue dédiée (affichage encore placeholder).

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

Après un scan réussi, Grape conserve un cache dans `.grape_cache/` à la racine de la
bibliothèque :

- `index.json` : index global (hash des dossiers + dates de modification).
- `folders/` : un fichier JSON par dossier d'album.
- `covers/` : jaquettes mises en cache (copie locale des images détectées).

Le cache est invalidé par dossier en comparant la date de modification du répertoire d'album.

Les préférences exposent aussi un chemin de cache configurable (action “Vider le cache”), mais
le scan actuel s'appuie toujours sur la cache locale de la bibliothèque.

## Documentation

- Vue d'ensemble : `docs/docs.md`
- Source & modules : `src/src.md`
- Roadmap : `tasks/roadmap/roadmap.md`
- Analyse UI : `tasks/ui_analysis.md`

## Feuille de route (résumé)

- Finaliser la gestion des playlists (affichage, édition, persistance).
- Brancher les préférences aux actions réelles (réindexation, logs, reset audio).
- Améliorer l'indexation (métadonnées enrichies, cache plus fin, genres réels).
