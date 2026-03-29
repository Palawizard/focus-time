# Focus Time

Focus Time est une application bureau moderne et minimaliste pensée pour Windows et Linux.
Le produit cible combine :

- un minuteur Pomodoro
- un App Tracker local-first
- un historique de sessions
- des statistiques d'usage
- une couche légère de gamification

La stack retenue pour le cadrage initial est :

- `Tauri v2` pour l'application desktop
- `Rust` pour le coeur natif, le tracking et la persistance
- `React + TypeScript + Vite` pour l'interface
- `SQLite` pour le stockage local

Le cadrage produit et technique détaillé est dans `docs/project-blueprint.md`.

## Workspace

- `apps/desktop`
  application React + Tauri
- `apps/desktop/src-tauri`
  shell desktop et commandes Rust
- `crates/*`
  crates Rust du domaine, de la persistence, des stats et du tracking

## Prerequisites

- `Node.js 22+`
- `Rust` via `rustup`
- Windows : `Visual Studio Build Tools 2022` avec le workload C++
- Linux : pre-requis Tauri/WebKitGTK documentes dans la CI

## Commandes utiles

- `corepack pnpm install`
- `corepack pnpm dev`
- `corepack pnpm check`
- `corepack pnpm build:desktop`
