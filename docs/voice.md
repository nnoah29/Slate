# Dictée vocale

Dicter dans une note : les mots apparaissent **pendant qu'on parle**, avec majuscules et ponctuation.
La reconnaissance est **locale** : la voix et le texte dicté ne quittent jamais la machine.
Conception : `docs/superpowers/specs/2026-11-23-dictee-vocale-design.md` (calquée sur la dictée de Mira).

## Deux façons de dicter

| Geste | Comportement |
|---|---|
| **Maintenir Espace** (≥ 400 ms) | on parle tant que la touche est tenue ; le texte est validé au relâchement. Un appui court reste une espace |
| **`Ctrl+D`** (ou palette `Ctrl+P` › « Dictée vocale ») | dictée continue : le micro reste ouvert, `Ctrl+D` à nouveau pour arrêter |
| **Échap** | annule la dictée en cours |

Espace reste une touche ordinaire avec un modificateur (Ctrl, Alt, Maj, Méta), en mode aperçu, quand le
champ de texte n'a pas le focus (barre de recherche), sans modèle installé, ou si le réglage est coupé.
Une autre touche pressée pendant l'attente écrit d'abord l'espace : aucune perte en frappe rapide.
Une pastille « ● À l'écoute » avec le niveau sonore s'affiche en bas à droite pendant la dictée.
Toute la dictée est **une seule étape d'annulation** (`Ctrl+Z`).

## Premier usage : le modèle

Au premier `Ctrl+D`, Slate propose de télécharger le modèle de la langue (environ 57 Mo), en affichant la
source (`github.com/k2-fsa/sherpa-onnx`). **Rien n'est téléchargé sans votre accord.** La taille et l'empreinte
SHA-256 sont vérifiées, et l'archive est extraite sans jamais écrire hors du dossier :
`~/.config/Slate/voice-models/`. Le modèle reste en mémoire 3 minutes après une dictée (réglable) pour que
la suivante démarre sans attente.

## Réglages (`~/.config/Slate/config.json`)

| Clé | Défaut | Rôle |
|---|---|---|
| `voice_language` | `"fr"` | langue de dictée (`"fr"` ou `"en"`), un modèle par langue |
| `voice_push_to_talk` | `true` | active Espace maintenu |
| `voice_hold_ms` | `400` | délai de maintien (200 à 1000) |
| `voice_keep_minutes` | `3` | durée de garde du modèle en mémoire (`0` : jamais, max 30) |
| `voice_device` | absent | source audio PulseAudio/PipeWire (`pactl list short sources`) ; micro par défaut sinon |

## Architecture

```
parecord ─▶ PCM → f32 ─▶ fil sherpa-onnx (OnlineRecognizer) ─▶ Partial / Final ─▶ boucle GTK
                                                                          │
                                                  Inserter ◀──────────────┘ (TextBuffer)
```

Le micro s'ouvre **avant** le chargement du modèle : le son est gardé en réserve (30 s au plus) puis rejoué
dans l'ordre, rien n'est perdu. Arrêter attend la validation du texte restant ; annuler n'écrit rien.
Le texte provisoire est remplacé en direct ; si vous tapez ou déplacez le curseur, il devient du texte ordinaire
et la dictée repart du nouveau curseur. Code : `src/voice/` (moteur), `src/window_dictation.rs` (clavier).

## Prérequis

- `parecord` (paquet `libpulse`, ou PipeWire-pulse). Sans lui : « Micro introuvable ».
- Une dictée dure 5 minutes au plus.

## Licence des modèles : à vérifier avant toute publication

Les modèles **Kroko communautaires** (Banafo) sont annoncés sous **CC-BY-SA** par l'éditeur, sans version
précisée ; le fichier `LICENSE` du dépôt Hugging Face est vide. Slate ne les redistribue pas (téléchargement
par l'utilisateur) et ne les commite jamais. Une confirmation de l'éditeur est à obtenir avant une release.
Moteur : `sherpa-onnx` (Apache-2.0).

## Tests

```
cargo test --test voice_pcm_test --test voice_inserter_test --test voice_push_to_talk_test   # sans modèle
SLATE_VOICE_MODELS=<dossier des modèles décompressés> \
SLATE_VOICE_WAV_FR=<test_wavs/0.wav du modèle français> \
  cargo test --test voice_recognizer_integration_test --test voice_engine_test              # avec modèle
```

Sans ces variables, les tests concernés se signalent « ignoré ». Aucun test n'utilise le micro ni le réseau
(le téléchargement est testé contre un serveur local `127.0.0.1`).

## Limites connues

Noms propres rares, sigles épelés, mots anglais dans une phrase française (modèle de 57 Mo).
Pas de commandes vocales (« point à la ligne »), pas de détection automatique de la langue.
Le téléchargement ne peut pas être annulé une fois lancé. Linux avec PulseAudio ou PipeWire uniquement.
