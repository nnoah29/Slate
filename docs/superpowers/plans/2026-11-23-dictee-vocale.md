# Plan — dictée vocale

Conception : `docs/superpowers/specs/2026-11-23-dictee-vocale-design.md`. Statuts : ☐ à faire, ◐ en cours, ☑ fait.
Chaque tâche : tests d'abord (boîte noire), un commit par tâche, `clippy -D warnings` vert.

## Lot 1 — Moteur

- ☑ **T1 Dépendances et squelette** : `sherpa-onnx`, `sha2`, `ureq`, `bzip2`, `tar` ; module `voice` ; mesurer le temps de build et le poids du binaire.
  Critère : `cargo build` vert.
- ☑ **T2 Types et PCM** : `VoiceLanguage`, événements, problèmes ; conversion PCM16 → f32 avec octet de reste ; niveau sonore 0–1.
  Critères : paquets coupés en octets impairs sans perte ; silence → 0, saturation → 1.
- ☑ **T3 Catalogue et dossier de modèles** : entrées pinnées FR/EN (URL, taille, SHA-256), `~/.config/Slate/voice-models/`, `is_installed`.
  Critères : dossier sous le répertoire de config de Slate ; installé seulement si les 4 fichiers existent.
- ☑ **T4 Téléchargement vérifié** : HTTP bloquant, progression, SHA-256 et taille vérifiés, extraction `tar.bz2` des 4 fichiers utiles, jamais d'écriture hors du dossier.
  Critères (serveur local `127.0.0.1`) : archive valide installée ; empreinte fausse refusée sans rien laisser ; chemin `../` dans l'archive refusé.
- ☑ **T5 Reconnaisseur** : thread sherpa (init, audio, fin de parole, libération) ; événements `Partial`/`Final`.
  Critères : silence → aucun texte et aucune panique ; avec `SLATE_VOICE_MODELS`, un fichier audio connu donne le texte attendu.
- ☑ **T6 Micro** : `parecord` en processus enfant, flux f32, arrêt propre, erreurs typées.
  Critères : `parecord` introuvable → `MicrophoneMissing` ; arrêt tue le processus.
- ☑ **T7 Moteur** : orchestration micro → réserve (30 s) → reconnaisseur, arrêt jusqu'au bout, annulation, garde du modèle 3 min.

## Lot 2 — Insertion

- ☑ **T8 Inserter** (logique pure sur chaînes + adaptateur `TextBuffer`) : partiel remplacé, validation avec espace, saisie utilisateur respectée.

## Lot 3 — Gestes et interface

- ☑ **T9 Espace maintenu** : machine à états (appui court, maintien, autre touche, modificateur, répétition).
- ☐ **T10 Branchement fenêtre** : `Ctrl+D`, palette, dialogue de confirmation du modèle, barre de niveau, config `voice_*`, messages d'erreur.
- ☐ **T11 Documentation et vérification réelle** : `docs/voice.md`, README, attribution Kroko ; essai à la voix.

## Journal

- 2026-11-23 — T1 : `sherpa-onnx` 2.0.3 en liaison statique ; premier build release complet : **11 min** (téléchargement de la bibliothèque native incluse). `mod voice` retiré de `main.rs` tant que rien ne l'utilise (T10), sinon `dead_code` ; la bibliothèque l'exporte.
- T4 : bug trouvé par test — le nettoyage d'une installation échouée visait le dossier du catalogue réel au lieu de celui de l'entrée ; corrigé dans le code.
- T5 : **la bibliothèque C++ de sherpa peut terminer le processus si un fichier de modèle manque** ; `Session::open` vérifie les 4 fichiers avant de charger (`ModelMissing`). Fin de phrase : `enable_endpoint` est **faux par défaut** dans la crate Rust (vrai dans Dart) ; réglé explicitement (2,4 s / 1,2 s / 20 s).
- T5 : test d'intégration avec le vrai enregistrement `test_wavs/0.wav` du modèle (SLATE_VOICE_WAV_FR) : « demandez », « pays », « faire » reconnus. Une voix espeak-ng ne donne que « Bonjour, » : inutilisable comme fixture.
- T5/T6 : archive FR téléchargée une fois en dev pour extraire le wav ; SHA-256 et taille identiques au catalogue.
- T7 : **plantage intermittent (SIGSEGV) à la fin du processus de test** quand un modèle était libéré par un fil détaché ; cause confirmée (30 exécutions sans plantage après correction). Le moteur garde les fils en cours de libération et les joint dans `Drop` : **l'application devra détruire le moteur à la fermeture (T10)**. Canal de messages renouvelé à chaque libération pour qu'aucun texte tardif ne soit livré après une annulation.
- T7 : un événement provisoire vide (silence) n'est plus émis (comparaison sur le texte normalisé).
- T8 : insertion portée depuis Mira (12 tests, dont accents : positions en caractères). Adaptateur `TextBuffer` à écrire avec T10.
- T9 : machine à états pure ; la touche reste « possédée » après l'écriture de l'espace sur une autre touche (état `Flushed`) pour que les répétitions de la touche tenue n'écrivent rien. Le contrôleur d'événements devra être en phase *capture* (le `TextView` insère l'espace sinon avant nous).
