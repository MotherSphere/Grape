# Documentation

Ce dossier regroupera la documentation du projet Grape.

## Contenu prévu

- Guide utilisateur
- Guide de contribution
- Architecture et décisions techniques

## MVP

### Fonctionnalités minimales

- Navigation artistes → albums avec accès rapide aux pistes.
- Lecture audio avec commandes play/pause/seek.
- Indexation locale minimale (scan d'un dossier, extraction des métadonnées essentielles, cache léger).

### Périmètre hors MVP

- Synchronisation cloud, comptes utilisateurs, ou playlists avancées.
- Égaliseur ou traitement audio sophistiqué.
- Gestion avancée des métadonnées (édition, téléchargement de jaquettes).

## Choix UI

### Options évaluées

| Option | Multi-plateforme | Performance | Theming | Accessibilité | Notes |
| --- | --- | --- | --- | --- | --- |
| Iced | ✅ (Desktop natif) | ✅ (GPU via wgpu) | ✅ (thèmes personnalisables) | ⚠️ (a11y encore jeune) | API Rust pure, bon fit pour apps desktop natives. |
| egui | ✅ (Desktop natif) | ✅ (très rapide, immédiate) | ⚠️ (théming simple, custom limité) | ⚠️ (a11y limitée) | Idéal pour outils/UX rapides, moins adapté aux apps grand public. |
| Tauri | ✅ (Desktop + webview) | ⚠️ (dépend du WebView) | ✅ (CSS) | ✅ (a11y via Web) | UI web, contraintes de packaging webview et bridge Rust. |

### Choix retenu : Iced

**Raisons principales :**
- UI 100 % Rust, alignée avec la stack existante.
- Bon compromis entre performance et flexibilité de layout.
- Maintien de la simplicité du déploiement (pas de webview).

### Impacts

- Architecture UI pensée en composants Iced (views + messages) pour les écrans MVP.
- Nécessité d'un thème maison (couleurs, typographies, spacing) pour une identité visuelle cohérente.
- Accessibilité à suivre : ajouter des tests manuels et ajustements si besoin.

## Structure des écrans MVP

### Bibliothèque

- **Entrée principale** : liste des artistes.
- **Second niveau** : liste des albums d'un artiste.
- **Panneau de détails** : pistes d'un album (durée, numéro de piste).
- **Actions** : lancer la lecture d'un album ou d'une piste.

### Lecture

- **Zone principale** : titre, artiste, album, jaquette.
- **Contrôles** : play/pause, seek, précédent/suivant.
- **Barre de progression** : temps écoulé / durée totale.

### File d'attente

- **Liste ordonnée** des pistes à venir.
- **Actions** : réordonner, retirer, sauter à une piste.
- **Indicateur** de la piste en cours de lecture.

## Flow MVP

1. **Arrivée sur la bibliothèque** : l'utilisateur voit la liste des artistes.
2. **Sélection d'un artiste** : affichage des albums disponibles et de leurs pistes.
3. **Lancement d'une lecture** : clic sur un album ou une piste pour démarrer la lecture.
4. **Écran lecture** : affichage du titre en cours, progression, et commandes play/pause.
5. **Retour bibliothèque** : navigation possible vers d'autres artistes/albums.
