# Analyse UI (référence) + base pour la liste de tâches

## Contexte
Cette analyse sert de base à la création de la liste de tâches UI. Elle est dérivée de la capture d’écran fournie et doit guider la structure de l’interface desktop.

## Prompt d’origine (à conserver)
"""
Parfait.

Crée moi une liste de tâche. Tu as bien capturé ce que je voulais. N'oublie pas de sauvegarder ton analyse dans un .md en le détaillant le plus possible pour la création des tâches. Et tu utilises ça ensuite pour la création des tâches. Lance la série des tâches, je vais les lancer une par une.
"""

## Structure globale (desktop) — basée sur l’image

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Top bar                                                                      │
│ - Logo/app icon (gauche)                                                     │
│ - Tabs navigation: Artists | Genres | Albums | Folders                       │
│ - Search box (droite) + menu/controls fenêtre                                │
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

## Détails précis des zones (d’après l’image)

### 1) Top bar
- Logo/app icon à gauche.
- Tabs de navigation : **Artists**, **Genres**, **Albums**, **Folders**.
- Champ de recherche à droite.
- Contrôles fenêtre (min/max/close) alignés à droite.

### 2) Colonne gauche — Artists
- Liste verticale d’artistes.
- Index alphabétique A–Z (repères de navigation rapide).
- Compteur global (ex. “65 Song artists”).
- Élément sélectionné en surbrillance.
- Scroll vertical.

### 3) Zone centrale — Albums
- Grille d’albums (jaquettes carrées).
- Légende sous chaque cover : **titre album** + **artiste**.
- Indication de tri (ex. “A–Z”).
- Album sélectionné visuellement (surbrillance).
- Scroll vertical.

### 4) Colonne droite — Songs
- En‑tête avec le **nombre de titres** (ex. “11 Songs”).
- Affichage de l’album sélectionné (ex. *Fallen* / Evanescence).
- Liste des pistes :
  - Numéro à gauche.
  - Titre principal + artiste en sous‑ligne.
  - Durée alignée à droite.
- Scroll vertical.

### 5) Footer / Player bar
- Mini artwork + titre/artist en cours (gauche).
- Contrôles playback (shuffle, prev, play/pause, next, repeat) au centre.
- Barre de progression + temps écoulé / durée.
- Icônes audio (volume, playlist/queue, etc.).

## Notes d’implémentation UI (Iced)
- Layout en 3 colonnes + footer fixe.
- Navigation par onglets (tabs) en top bar.
- États UI nécessaires :
  - Onglet actif (Artists/Genres/Albums/Folders)
  - Artiste sélectionné
  - Album sélectionné
  - Piste en cours
  - Position de lecture + durée
  - Recherche (texte)
- Données UI à prévoir :
  - Liste artistes triable A–Z
  - Liste albums d’un artiste
  - Liste pistes d’un album
  - Mini-métadonnées (durées, jaquettes, etc.)

## Sortie attendue pour les tâches
- Structuration des modules UI.
- Création d’un état global UI.
- Mise en place du layout (TopBar, Sidebar, MainGrid, RightPanel, PlayerBar).
- Gestion des messages/actions (sélection, navigation, lecture, recherche).
- Implémentation progressive des vues et composants.

## Liste de tâches détaillée (séquence à lancer une par une)

### Tâche 1 — Définir l’architecture UI et le layout global (desktop)
- Créer un module UI racine (`src/ui/mod.rs`) et un état global.
- Mettre en place la structure de layout : **Top bar + 3 colonnes + footer**.
- Implémenter la **Top bar** : tabs *Artists/Genres/Albums/Folders*, champ de recherche, icône app.
- Préparer les zones : **Colonne gauche (Artists)**, **Zone centrale (Albums)**, **Colonne droite (Songs)**, **Footer/Player bar**.

### Tâche 2 — Modéliser l’état UI (sélections + navigation)
- Définir l’onglet actif (Artists/Genres/Albums/Folders) et l’état de navigation.
- Ajouter les sélections UI :
  - artiste sélectionné,
  - album sélectionné,
  - piste en cours.
- Ajouter l’état de lecture :
  - position courante,
  - durée totale,
  - is_playing/shuffle/repeat.
- Ajouter l’état de recherche :
  - texte de recherche,
  - filtres éventuels (A–Z, By album).
- Prévoir une structure de données UI minimaliste pour la grille d’albums et la liste de pistes.

### Tâche 3 — Implémenter la colonne gauche (Artists)
- Liste verticale d’artistes avec scroll.
- Index alphabétique A–Z (repères de navigation rapide).
- Compteur global (“65 Song artists”).
- État sélectionné en surbrillance.

### Tâche 4 — Implémenter la zone centrale (Albums grid)
- Grille d’albums (jaquettes carrées).
- Légende sous chaque cover : titre + artiste.
- Indication de tri (A–Z).
- Surbrillance de l’album sélectionné.
- Scroll vertical.

### Tâche 5 — Implémenter la colonne droite (Songs)
- En‑tête avec nombre de titres (“11 Songs”).
- Titre album + artiste en tête de panneau.
- Liste des pistes :
  - numéro à gauche,
  - titre principal + artiste en sous‑ligne,
  - durée alignée à droite.
- Scroll vertical.
- Mise en évidence de la piste en cours.

### Tâche 6 — Implémenter la barre de lecture (Player bar)
- Mini‑artwork + piste en cours (gauche).
- Contrôles playback (shuffle, prev, play/pause, next, repeat) au centre.
- Barre de progression + temps écoulé / durée.
- Icônes audio (volume, playlist/queue, etc.).

### Tâche 7 — Câbler les messages Iced
- Navigation entre tabs.
- Sélection artiste/album/piste.
- Play/Pause + Next/Prev.
- Recherche.
