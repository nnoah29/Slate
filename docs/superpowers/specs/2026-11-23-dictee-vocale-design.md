# Dictée vocale — conception

Statut : **approuvé** (utilisateur, 2026-11-23). Branche `feat/dictee-vocale` (issue de `dev`).
Plan : `docs/superpowers/plans/2026-11-23-dictee-vocale.md`. Modèle de référence : la dictée de Mira
(`MIRA/docs/VOICE.md`, `MIRA/docs/superpowers/specs/2026-11-15-dictee-vocale-design.md`).

## Intention

Dicter dans une note au lieu de taper. Les mots apparaissent **pendant qu'on parle** (texte provisoire
remplacé en direct, validé en fin de phrase). Reconnaissance **locale** : la voix et le texte ne quittent
jamais la machine.

## Buts

- Dictée en direct, locale, français et anglais.
- Deux gestes : **Espace maintenu** (≥ 400 ms, on parle tant que la touche est tenue, validation au
  relâchement) et **dictée continue** (`Ctrl+D` ou palette de commandes, le micro reste ouvert).
- Texte utilisable : majuscules et ponctuation viennent du modèle.
- Aucune gêne à la frappe : un appui court sur Espace reste une espace, sans perte en frappe rapide.

## Non-buts

Synthèse vocale, mains libres, mot d'éveil, commandes vocales (« point à la ligne »), reconnaissance
distante, détection automatique de la langue, mots favorisés, Windows/macOS/Android.

## Anti-buts (échec même si « ça marche »)

- Une donnée vocale ou un texte dicté qui part sur le réseau.
- Un modèle téléchargé sans demande ni confirmation de l'utilisateur.
- Une dictée qui écrase ce que l'utilisateur tape, ou qui écrit dans une note fermée.
- Une interface qui fige pendant la reconnaissance.
- Une espace perdue ou doublée en frappe normale.

## Contraintes

- Rust, GTK 4 / libadwaita ; aucune règle d'unsafe, d'`unwrap`/`expect`/`panic!` hors tests.
- Réseau : uniquement le téléchargement du modèle, après confirmation (taille et source affichées),
  empreinte SHA-256 vérifiée. Tests uniquement contre un serveur local `127.0.0.1`.
- Modèles Kroko (Banafo, **CC-BY-SA** annoncée, fichier LICENSE du dépôt vide) : jamais commités ni
  embarqués ; attribution dans la documentation. Confirmation de l'éditeur à obtenir avant une release.
- Aucun test ne dépend du micro ; les tests avec modèle sont optionnels (`SLATE_VOICE_MODELS`).

## Approches envisagées

| | Approche | Verdict |
|---|---|---|
| A | Whisper / serveur externe | Écartée : pas de flux mot à mot, processus externe fragile. |
| B | Service distant | Écartée : la voix quitterait la machine. |
| C | API vocales du système | Écartée : rien de fiable sur Linux. |
| **D** | **`sherpa-onnx` (crate Rust officielle) + modèles Kroko, capture par `parecord`** | **Retenue** : même moteur que Mira, validé par son spike (décodage 40–350 ms par paquet, ponctuation produite). |

## Décisions de l'utilisateur

- Slate a **son propre dossier de modèles** dans le répertoire de config : `~/.config/Slate/voice-models/`.
- Dictée **Espace maintenu** dès la première livraison, mots visibles au fil de la parole.

## Mes choix (à corriger)

- Dictée continue ajoutée (`Ctrl+D` + palette) : elle partage tout le code.
- Langue dans `config.json` (`voice_language`, `fr` par défaut), pas de détection.
- Capture par `parecord` (PulseAudio/PipeWire-pulse), processus enfant, PCM 16 bits mono 16 kHz.
  Sans `parecord` : message « Micro introuvable ».
- Reconnaissance dans un `std::thread` dédié ; l'interface reçoit `Partial` / `Final` par canal
  consommé sur la boucle GTK.
- Modèle gardé en mémoire 3 minutes après la dernière dictée (réglable), puis libéré.
- Dictée active seulement en mode édition (jamais en aperçu), une seule à la fois, 5 minutes au plus.
- Espace maintenu : l'appui est intercepté ; relâché avant 400 ms → une espace est insérée ; autre touche
  pressée entre-temps → l'espace est insérée d'abord ; touches répétées ignorées ; reste une espace avec
  modificateur, hors édition, sans modèle ou réglage coupé.
- Insertion : texte provisoire à une marque du buffer, remplacé à chaque mise à jour ; si l'utilisateur
  tape ou déplace le curseur, le provisoire devient du texte ordinaire et la dictée repart du curseur.

## Architecture

```
parecord ─▶ pcm → f32 ─▶ thread sherpa (OnlineRecognizer) ─▶ Partial / Final ─▶ canal ─▶ boucle GTK
                                                                                  │
                                                          DictationInserter ◀─────┘ (TextBuffer)
```

Arborescence `src/voice/` : `types`, `pcm`, `audio_level`, `microphone`, `model_catalog`, `model_store`,
`model_download`, `model_archive`, `recognizer_config`, `recognizer_worker`, `engine`, `inserter`,
`push_to_talk`, `controller`. Interface : bouton/barres de niveau, boîte de confirmation du modèle,
section Voix de la config.

## Risques

1. Compilation de `sherpa-onnx` (liaison statique, gros binaire) : mesurée à la tâche 1.
2. `parecord` absent ou micro occupé : erreurs typées, jamais de panique.
3. Frappe rapide et Espace maintenu : machine à états testée sans GTK.
4. Qualité : noms propres, sigles, anglais dans une phrase française (limites du modèle de 57 Mo).
