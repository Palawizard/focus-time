# Focus Time - Working Rules

Ces regles s'appliquent a toute la suite du projet.

## 1. Respect des documents

- Toujours respecter ce qui est ecrit dans les fichiers de documentation existants.
- Avant de lancer un nouvel epic, relire les documents lies au projet pour rester aligne avec les choix deja poses.
- Quand une tache d'un epic est reellement terminee, cocher sa case avec un `x` dans le markdown.
- Quand tous les elements d'un epic sont termines, toutes les cases de cet epic doivent etre cochees dans la roadmap.
- Ne pas laisser des cases cochees si le travail n'est pas effectivement livre et verifie.

## 2. Texte visible dans l'application

- Les textes affiches dans l'application doivent etre ecrits comme dans un produit reel.
- Ne jamais afficher de texte de developpement, de bootstrap, de scaffold, d'epic, de runtime technique ou de commentaire interne a l'utilisateur final.
- L'interface doit rester simple, minimale et credible comme une application en production.

## 3. Workflow Git par epic

- Au debut de chaque epic, creer une branche dediee avant de commencer le travail.
- Faire un commit par element de la roadmap quand cet element est termine.
- Ne pas merger automatiquement dans `dev` a la fin d'un epic.
- Attendre une validation explicite de l'utilisateur avant de merger la branche de l'epic dans `dev`.
- Une fois le merge demande, pousser la branche et `dev`.

## 4. Discipline d'execution

- Toujours preferer un travail propre, verifie et coherent avec la doc plutot qu'un scaffold rapide.
- Quand une erreur de mise en oeuvre contredit ces regles, la corriger avant d'avancer sur l'epic suivant.
