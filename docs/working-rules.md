# Focus Time - Working Rules

Ces regles s'appliquent a toute la suite du projet. Elles priment sur les habitudes implicites.

## 1. Respect des documents

- Toujours respecter ce qui est ecrit dans les fichiers de documentation existants.
- Avant de lancer un nouvel epic ou un chantier important, relire les documents lies au projet pour rester aligne avec les choix deja poses.
- Quand une tache d'un epic est reellement terminee, cocher sa case avec un `x` dans le Markdown.
- Quand tous les elements d'un epic sont termines, toutes les cases de cet epic doivent etre cochees dans la roadmap.
- Ne pas laisser des cases cochees si le travail n'est pas effectivement livre et verifie.

## 2. Texte visible dans l'application

- Les textes affiches dans l'application doivent etre ecrits comme dans un produit reel.
- Ne jamais afficher de texte de developpement, de bootstrap, de scaffold, d'epic, de runtime technique ou de commentaire interne a l'utilisateur final.
- L'interface doit rester simple, minimale et credible comme une application en production.

## 3. Langue du code et du produit

- Tout le code, les noms techniques, les libelles visibles, les tests et les textes presents dans le projet doivent etre ecrits en anglais.
- Exception explicite : les fichiers Markdown peuvent rester en francais.

## 4. Workflow Git par epic

- Au debut de chaque epic, creer une branche dediee avant de commencer le travail.
- Les noms de branche doivent etre conventionnels, par exemple `feat/...`, `fix/...`, `chore/...`, suivis d'un nom explicite.
- Faire un commit par element de la roadmap quand cet element est termine.
- Chaque sous-partie d'un epic doit aussi etre committee separement quand elle constitue une unite de travail distincte.
- Ne pas regrouper dans un meme commit plusieurs sous-parties differentes d'un epic si elles peuvent etre isolees proprement.
- Un commit peut toucher plusieurs fichiers seulement s'ils appartiennent a la meme sous-partie livree.
- Ne pas merger automatiquement dans `dev` a la fin d'un epic.
- Attendre une validation explicite de l'utilisateur avant de merger la branche de l'epic dans `dev`.
- Une fois le merge demande, pousser la branche de l'epic et `dev`.

## 5. Discipline d'execution

- Toujours preferer un travail propre, verifie et coherent avec la doc plutot qu'un scaffold rapide.
- Corriger d'abord toute mise en oeuvre qui contredit ces regles avant d'avancer sur l'epic suivant.
- Quand une ambiguite apparait entre vitesse et coherence produit, prioriser la coherence.

## 6. Discipline documentaire

- Si une decision produit, technique ou de workflow change, mettre a jour les fichiers Markdown concernes dans le meme chantier.
- Le README doit rester utile pour entrer dans le projet rapidement.
- `docs/working-rules.md` doit rester normatif.
- `docs/project-blueprint.md` doit rester la reference pour la vision, l'architecture et la roadmap.
