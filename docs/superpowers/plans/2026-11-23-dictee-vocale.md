# Plan — dictée vocale

Conception : `docs/superpowers/specs/2026-11-23-dictee-vocale-design.md`. Statuts : ☐ à faire, ◐ en cours, ☑ fait.
Chaque tâche : tests d'abord (boîte noire), un commit par tâche, `clippy -D warnings` vert.

## Lot 1 — Moteur

- ☐ **T1 Dépendances et squelette** : `sherpa-onnx`, `sha2`, `ureq`, `bzip2`, `tar` ; module `voice` ; mesurer le temps de build et le poids du binaire.
  Critère : `cargo build` vert.
- ☐ **T2 Types et PCM** : `VoiceLanguage`, événements, problèmes ; conversion PCM16 → f32 avec octet de reste ; niveau sonore 0–1.
  Critères : paquets coupés en octets impairs sans perte ; silence → 0, saturation → 1.
- ☐ **T3 Catalogue et dossier de modèles** : entrées pinnées FR/EN (URL, taille, SHA-256), `~/.config/Slate/voice-models/`, `is_installed`.
  Critères : dossier sous le répertoire de config de Slate ; installé seulement si les 4 fichiers existent.
- ☐ **T4 Téléchargement vérifié** : HTTP bloquant, progression, SHA-256 et taille vérifiés, extraction `tar.bz2` des 4 fichiers utiles, jamais d'écriture hors du dossier.
  Critères (serveur local `127.0.0.1`) : archive valide installée ; empreinte fausse refusée sans rien laisser ; chemin `../` dans l'archive refusé.
- ☐ **T5 Reconnaisseur** : thread sherpa (init, audio, fin de parole, libération) ; événements `Partial`/`Final`.
  Critères : silence → aucun texte et aucune panique ; avec `SLATE_VOICE_MODELS`, un fichier audio connu donne le texte attendu.
- ☐ **T6 Micro** : `parecord` en processus enfant, flux f32, arrêt propre, erreurs typées.
  Critères : `parecord` introuvable → `MicrophoneMissing` ; arrêt tue le processus.
- ☐ **T7 Moteur** : orchestration micro → réserve (30 s) → reconnaisseur, arrêt jusqu'au bout, annulation, garde du modèle 3 min.

## Lot 2 — Insertion

- ☐ **T8 Inserter** (logique pure sur chaînes + adaptateur `TextBuffer`) : partiel remplacé, validation avec espace, saisie utilisateur respectée.

## Lot 3 — Gestes et interface

- ☐ **T9 Espace maintenu** : machine à états (appui court, maintien, autre touche, modificateur, répétition).
- ☐ **T10 Branchement fenêtre** : `Ctrl+D`, palette, dialogue de confirmation du modèle, barre de niveau, config `voice_*`, messages d'erreur.
- ☐ **T11 Documentation et vérification réelle** : `docs/voice.md`, README, attribution Kroko ; essai à la voix.
