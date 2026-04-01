# Focus Time

Focus Time est une application desktop locale pensee pour aider un utilisateur a mieux comprendre sa facon de travailler, et pas seulement a lancer un timer.

Le produit combine cinq briques principales :

- un minuteur Pomodoro fiable
- un App Tracker local-first
- un historique clair des sessions
- un dashboard de stats lisible
- une gamification legere et non intrusive

## Etat du projet

Le socle applicatif est en place et les epics suivants sont deja livres :

- fondation du projet
- shell applicatif et design system
- persistance locale SQLite
- moteur Pomodoro
- App Tracker
- historique des sessions
- dashboard de stats
- gamification legere

Les prochains chantiers majeurs concernent les preferences avancees, la fiabilite globale et le packaging beta.

## Stack retenue

- `Tauri v2` pour le shell desktop
- `Rust` pour la logique native, la persistance et le tracking
- `React + TypeScript + Vite` pour l'interface
- `SQLite` pour le stockage local

## Structure du workspace

- `apps/desktop`
  application frontend React
- `apps/desktop/src-tauri`
  shell desktop, commandes Tauri, migrations et services Rust
- `crates/focus-domain`
  modeles et regles metier
- `crates/focus-persistence`
  acces SQLite, repositories et migrations
- `crates/focus-stats`
  agregations et calculs du dashboard
- `crates/focus-tracking`
  detection d'application active et normalisation du tracking
- `docs/`
  cadrage produit, roadmap et regles de travail

## Prerequis

- `Node.js 22+`
- `Rust` installe via `rustup`
- Windows : `Visual Studio Build Tools 2022` avec le workload C++
- Linux : dependances Tauri/WebKitGTK adaptees a la distribution

## Commandes utiles

- `corepack pnpm install`
- `corepack pnpm dev`
- `corepack pnpm check`
- `corepack pnpm build:desktop`

## Documentation a lire en priorite

- `docs/working-rules.md`
  regles de contribution, discipline Git et contraintes de livraison
- `docs/project-blueprint.md`
  vision produit, architecture cible, roadmap et scope v1

## Rappel de fonctionnement

La documentation du projet fait partie de la source de verite. Avant de travailler sur un epic ou un chantier important, il faut relire les fichiers Markdown du projet et rester aligne avec les choix deja poses.
