# Focus Time - Project Blueprint

## 1. Vision produit

Focus Time doit aider un utilisateur a comprendre comment il travaille, pas seulement combien de temps il lance un timer.

Le produit vise 5 piliers :

1. lancer des sessions Pomodoro rapidement
2. suivre automatiquement l'application active pendant le travail
3. consolider les sessions dans un historique clair
4. afficher des statistiques utiles et lisibles
5. ajouter une gamification legere pour encourager la regularite

## 2. Positionnement v1

La v1 doit rester locale, rapide et fiable.

Choix structurants :

- local-first, sans backend cloud obligatoire
- donnees stockees en SQLite sur la machine
- tracking explicite et configurable par l'utilisateur
- fonctionnement hors ligne
- Windows prioritaire, Linux supporte avec nuance selon l'environnement graphique

## 3. Stack recommandee

| Domaine | Choix | Pourquoi |
| --- | --- | --- |
| Shell desktop | `Tauri v2` | faible consommation memoire, integration native, bon fit pour une app locale |
| Coeur natif | `Rust` | fiable pour les timers, performant pour le tracking, tres bon acces systeme |
| UI | `React + TypeScript + Vite` | productivite elevee, ecosysteme mature, interface moderne |
| State management | `Zustand` | simple, leger, adapte a une app desktop locale |
| Data fetching local | `TanStack Query` | gestion propre des lectures/mutations asynchrones vers les commandes Tauri |
| UI primitives | `Radix UI` | composants accessibles sans imposer un design fige |
| Styling | `Tailwind CSS` | iteration rapide pour une UI minimaliste et coherente |
| Graphiques | `Recharts` | suffisant et simple pour les dashboards de stats |
| Base locale | `SQLite` | robuste, portable, parfaite pour une app single-user |
| Acces DB Rust | `sqlx` | migrations propres, requetes explicites, bon support SQLite |
| Tests front | `Vitest + React Testing Library` | rapide pour la couche UI |
| Tests Rust | `cargo test` | naturel pour la logique de domaine et la persistance |
| E2E desktop | `Playwright + tauri-driver` | verifie les parcours critiques de bout en bout |

## 4. Pourquoi cette stack et pas une autre

### Pourquoi Tauri plutot qu'Electron

- Tauri est plus leger en RAM et en taille de binaire
- le tracking natif et la logique timer vivent naturellement cote Rust
- le produit est local-first, donc le couple Tauri + Rust + SQLite est tres coherent

### Pourquoi pas Flutter

- tres bon choix UI, mais moins direct pour l'integration web tooling et les dashboards reactifs locaux
- l'ecosysteme autour des primitives desktop et de l'instrumentation native est moins naturel pour ce type de produit

## 5. Risques et contraintes importantes

### Linux et tracking de fenetre active

Le point le plus delicat du projet n'est pas le Pomodoro, c'est le App Tracker.

- sous Windows, le tracking de fenetre active est tres faisable
- sous Linux X11, c'est generalement faisable
- sous Linux Wayland, l'acces a la fenetre active est souvent limite ou interdit selon le compositor

Conclusion produit :

- la v1 doit assumer `Windows = support prioritaire`
- `Linux X11 = support cible`
- `Linux Wayland = best effort`, avec communication claire dans l'app

### Vie privee

- tout doit rester local par defaut
- l'utilisateur doit pouvoir exclure des apps
- l'utilisateur doit pouvoir desactiver le tracking auto

## 6. Architecture cible

Le repo doit rester simple, mais preparer une separation nette entre :

- l'interface React
- le shell Tauri
- la logique metier Rust
- la persistance SQLite
- les adaptateurs de tracking par plateforme

### Vue logique

1. `apps/desktop/src`
   Interface utilisateur, navigation, ecrans, composants, stores front.
2. `apps/desktop/src-tauri`
   Point d'entree Tauri, commandes exposees au front, migrations, ressources desktop.
3. `crates/focus-domain`
   Regles metier pures : session, timer, achievements, aggregations.
4. `crates/focus-persistence`
   Acces SQLite, repositories, migrations et DTO de stockage.
5. `crates/focus-tracking`
   Detection application/fenetre active, normalisation des evenements de tracking.
6. `crates/focus-stats`
   Calculs de dashboard, series journalieres, repartitions par app et categorie.

### Flux principal

1. le front lance une commande Tauri
2. Tauri delegue a un service Rust
3. le service lit ou ecrit dans SQLite via `focus-persistence`
4. les modules `focus-tracking` et `focus-stats` fournissent les donnees metier
5. le front recupere un payload deja structure pour affichage

## 7. Arborescence retenue

```text
.
|-- .github/
|   `-- workflows/
|-- apps/
|   `-- desktop/
|       |-- public/
|       |-- src/
|       |   |-- app/
|       |   |-- components/
|       |   |-- features/
|       |   |   |-- gamification/
|       |   |   |-- history/
|       |   |   |-- pomodoro/
|       |   |   |-- settings/
|       |   |   |-- stats/
|       |   |   `-- tracking/
|       |   |-- hooks/
|       |   |-- lib/
|       |   |-- routes/
|       |   |-- stores/
|       |   |-- styles/
|       |   |-- tests/
|       |   `-- types/
|       `-- src-tauri/
|           |-- capabilities/
|           |-- icons/
|           |-- migrations/
|           |-- resources/
|           `-- src/
|               |-- bootstrap/
|               |-- commands/
|               |-- platform/
|               |-- services/
|               `-- state/
|-- crates/
|   |-- focus-domain/
|   |   `-- src/
|   |-- focus-persistence/
|   |   `-- src/
|   |-- focus-stats/
|   |   `-- src/
|   `-- focus-tracking/
|       `-- src/
|-- docs/
|   |-- adr/
|   `-- assets/
`-- scripts/
```

## 8. Modeles metier de base

Entites minimales a prevoir :

- `PomodoroPreset`
- `Session`
- `SessionSegment`
- `TrackedApp`
- `TrackedWindowEvent`
- `DailyStat`
- `Streak`
- `Achievement`
- `UserPreference`

### Tables SQLite probables

- `sessions`
- `session_segments`
- `tracked_apps`
- `tracked_window_events`
- `daily_stats`
- `achievements`
- `user_preferences`

## 9. Ecrans cibles v1

- `Home / Focus`
  Timer principal, session courante, bouton start/pause/stop.
- `History`
  Liste des sessions passees, filtres par date, duree, tag, application.
- `Stats`
  Temps focus du jour, semaine, mois, top apps, repartitions et tendances.
- `Tracker`
  Vue de suivi des apps et regles d'exclusion.
- `Gamification`
  Streaks, badges, objectifs simples.
- `Settings`
  Presets Pomodoro, preferences de tracking, exclusions, theme, notifications.

## 10. Regles d'architecture

- la logique timer ne doit pas vivre dans React
- la source de verite des sessions est SQLite
- le front ne parle jamais directement a SQLite
- le tracking plateforme doit etre encapsule dans `focus-tracking`
- les statistiques doivent etre calculables a partir des sessions et evenements bruts
- toute fonctionnalite sensible doit etre feature-flaggee si elle est instable sur Linux

## 11. Roadmap initiale en epics et taches

### Epic 0 - Fondation du projet

- [x] Initialiser le workspace frontend avec `pnpm`, `Vite`, `React`, `TypeScript`.
- [x] Initialiser l'application desktop Tauri v2 dans `apps/desktop`.
- [x] Configurer le workspace Rust pour connecter `src-tauri` et `crates/*`.
- [x] Installer la chaine qualite minimale : `eslint`, `prettier`, `vitest`, `cargo fmt`, `clippy`.
- [x] Configurer les scripts de dev, build, test et lint a la racine.
- [x] Mettre en place une CI simple sur GitHub Actions pour verifier frontend et Rust.

Definition de fini :

- le projet demarre en mode dev
- un build desktop Windows fonctionne
- un build desktop Linux est prepare

### Epic 1 - Design system et shell applicatif

- [x] Definir les tokens visuels : couleurs, surfaces, typographie, espacements, ombres.
- [x] Mettre en place le layout principal avec sidebar ou navigation compacte.
- [x] Integrer `Radix UI` et creer les primitives de base : button, card, dialog, tabs, tooltip.
- [x] Creer les ecrans vides : Focus, History, Stats, Tracker, Gamification, Settings.
- [x] Poser la navigation et les routes.
- [x] Ajouter le theming clair/sombre seulement si necessaire au produit.

Definition de fini :

- l'app ressemble deja a un produit exploitable
- la navigation est stable

### Epic 2 - Modele de donnees et persistance locale

- [x] Concevoir le schema SQLite initial.
- [x] Ajouter les migrations SQL versionnees.
- [x] Implementer les repositories Rust pour sessions, preferences, apps trackees et stats.
- [x] Exposer des commandes Tauri pour lire et ecrire les donnees principales.
- [x] Ajouter une couche de seed ou fixtures de dev.
- [x] Verifier les migrations sur base vide et base existante.

Definition de fini :

- la base est versionnee
- les lectures et ecritures critiques sont stables

### Epic 3 - Moteur Pomodoro

- [x] Modeliser presets, session active, pause, reprise, annulation, completion.
- [x] Construire le moteur de timer cote Rust pour eviter les derives de timing.
- [ ] Exposer les commandes start, pause, resume, stop, skip-break.
- [ ] Synchroniser l'etat du timer avec le front.
- [ ] Ajouter notifications desktop et son facultatif.
- [ ] Sauvegarder automatiquement les sessions terminees et interrompues.

Definition de fini :

- un utilisateur peut faire un cycle complet focus + break sans bug d'etat

### Epic 4 - App Tracker

- [ ] Definir le format commun d'un evenement de tracking.
- [ ] Implementer l'adaptateur Windows de detection d'application/fenetre active.
- [ ] Implementer l'adaptateur Linux X11.
- [ ] Ajouter une strategie Linux Wayland de degradation elegante avec message clair si non supporte.
- [ ] Normaliser les noms d'applications et executables.
- [ ] Enregistrer les segments de temps par app pendant une session active.
- [ ] Ajouter les exclusions utilisateur par executable, titre de fenetre ou categorie.
- [ ] Mettre en place une permission explicite et un onboarding clair sur le tracking.

Definition de fini :

- le tracking fonctionne sur Windows
- Linux ne casse pas l'app, meme en mode degrade

### Epic 5 - Historique et revue des sessions

- [ ] Creer la liste des sessions avec pagination ou chargement incremental.
- [ ] Ajouter filtres par date, duree, preset, statut et application.
- [ ] Afficher le detail d'une session : segments, apps, interruptions, note eventuelle.
- [ ] Permettre la suppression ou correction d'une session.
- [ ] Ajouter export local CSV ou JSON.

Definition de fini :

- l'utilisateur peut relire et exploiter ses sessions sans passer par la base

### Epic 6 - Dashboard de stats

- [ ] Definir les KPIs v1 : temps focus, taux de completion, top apps, streak, repartition journaliere.
- [ ] Construire les agregations dans `focus-stats`.
- [ ] Afficher vues jour, semaine, mois.
- [ ] Ajouter graphiques d'evolution et repartition par app.
- [ ] Ajouter comparaisons simples avec la periode precedente.
- [ ] Gerer les etats vides et les grosses plages de donnees.

Definition de fini :

- le dashboard aide a comprendre des tendances reelles

### Epic 7 - Gamification legere

- [ ] Definir un systeme simple de streak quotidien.
- [ ] Ajouter objectifs hebdomadaires configurables.
- [ ] Ajouter badges de progression non intrusifs.
- [ ] Debloquer des achievements a partir de regles deterministes.
- [ ] Afficher la progression sans polluer le flow principal.

Definition de fini :

- la gamification renforce l'usage sans transformer l'app en jeu

### Epic 8 - Preferences, fiabilite et packaging

- [ ] Ajouter l'ecran Settings complet.
- [ ] Configurer autostart, tray, notifications et comportement de fermeture.
- [ ] Ajouter sauvegarde et restauration locale des donnees.
- [ ] Mettre en place logs, tracing et gestion d'erreurs utilisateur.
- [ ] Ecrire les tests E2E des parcours critiques.
- [ ] Preparer les icones, metadata, signatures et bundles de release.

Definition de fini :

- une beta installable peut etre partagee

## 12. Premiere version raisonnable

La meilleure premiere release n'est pas "tout".

Scope recommande pour une `v0.1` :

- timer Pomodoro complet
- sauvegarde locale des sessions
- tracking Windows
- historique simple
- dashboard minimal jour/semaine
- exclusions basiques

Ce qui peut attendre :

- gamification avancee
- sync cloud
- categories intelligentes automatiques
- coaching ou recommandations

## 13. Conclusion

Pour Focus Time, le meilleur compromis est :

- `Tauri + Rust` pour la couche desktop et systeme
- `React + TypeScript` pour une UI rapide a construire et a faire evoluer
- `SQLite` pour une base locale robuste

Cette combinaison est celle qui colle le mieux au besoin reel du produit : une app locale, performante, moderne, avec du tracking natif et des stats riches.
