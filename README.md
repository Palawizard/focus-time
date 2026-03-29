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
