# Grape

Grape est un lecteur musique/audio desktop en Rust, inspiré par Dopamine. Le projet vise une
expérience rapide et claire pour explorer une bibliothèque locale et lancer la lecture.

## État actuel

- **UI desktop Iced** : layout complet (top bar, colonnes, player bar) avec navigation et états.
- **Scan local** : lecture de dossiers `Artiste/Album` (ou albums à la racine) + construction d'un catalogue.
- **Métadonnées audio** : durée, bitrate, codec, année, genre et covers embarquées via `lofty`.
- **Cache local** : index piste + cache album/covers + cache métadonnées dans `.grape_cache/`.
- **Jaquettes** : priorité aux covers embarquées, fallback sur images locales mises en cache.
- **Métadonnées en ligne** : enrichissement optionnel via Last.fm (API key + TTL).
- **Lecture audio** : module `player` basé sur `rodio`, EQ 3/5 bandes, normalisation, sortie audio configurable.
- **File de lecture** : queue basée sur la playlist active + actions Next/Previous.
- **Navigation enrichie** : onglets Genres/Folders + recherche/tri appliqués aux listes.
- **Préférences UI** : écrans General/Appearance/Accessibility/Audio avec persistance locale.
- **Playlists** : création/renommage/suppression + ajout de pistes, persistance JSON locale.

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

- `index.json` : index global des signatures de pistes.
- `folders/` : un fichier JSON par dossier d'album.
- `covers/` : jaquettes mises en cache (covers embarquées ou images locales).
- `metadata/` : métadonnées en ligne mises en cache (Last.fm).

Le cache est invalidé par piste en comparant la signature (taille + date de modification).

Les préférences exposent aussi un chemin de cache configurable (action “Vider le cache”), mais
le scan actuel s'appuie toujours sur la cache locale de la bibliothèque.

## Documentation

- Vue d'ensemble : `docs/docs.md`
- Source & modules : `src/src.md`
- Roadmap : `tasks/roadmap/roadmap.md`
- Analyse UI : `tasks/ui_analysis.md`

## Feuille de route (résumé)

- Compléter l'édition des playlists (reorder, suppression d'items, vue queue dédiée).
- Étendre les métadonnées (sources en ligne, tags avancés, covers hi-res).
- Améliorer l'indexation (métadonnées enrichies, cache plus fin, genres réels).
