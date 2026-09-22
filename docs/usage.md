# Guide d'utilisation de Slate

## 1. Démarrage de Slate

Slate peut être lancé depuis votre lanceur d'applications (Rofi, Wofi, Walker, etc.) ou directement en ligne de commande :

```bash
# Ouvrir une note vierge ou la dernière session
slate

# Ouvrir directement un fichier Markdown précis
slate ~/Documents/projet.md

# Forcer l'ouverture d'un nouveau document sans titre
slate --new

# Lancer en mode verbeux (logs et debug)
slate --debug

# Afficher l'aide et les options
slate --help
```

### Comportement mono-instance (DBus)
Slate implémente l'architecture d'application unique GIO/DBus : si vous exécutez `slate autre-note.md` alors qu'une instance tourne déjà, Slate ne lance pas de processus redondant mais ouvre directement le fichier dans la fenêtre existante et l'amène au premier plan.

---

## 2. Philosophie de l'Interface

L'interface de Slate a été conçue pour éliminer toute distraction visuelle :
- **Aucune barre de titre ni barre d'outils** : Pas de HeaderBar encombrante, pas d'onglets superflus.
- **Fond translucide** : Même couleur de fond et translucidité que le terminal Kitty (`#272626` avec opacité `0.5`).
- **Marges aérées** : Le texte respire avec des marges douces de 10px autour de la zone de saisie.

---

## 3. Le Mode Conceal (Masquage dynamique des balises)

Le mode **Conceal** de Slate est actif par défaut et permet de masquer automatiquement les marqueurs de syntaxe Markdown dès que vous avez fini d'écrire, pour une lecture nette et fluide façon typographie imprimée :

### Comment ça fonctionne ?
- **Révélation sous le curseur** : Lorsque votre curseur entre dans un token (ex: un mot en gras `**important**`), les astérisques `**` réapparaissent instantanément pour vous permettre d'éditer la balise.
- **Masquage hors du curseur** : Dès que le curseur quitte le token ou la ligne, les balises de syntaxe sont à nouveau dissimulées de manière invisible.
- **Raccourci** : Vous pouvez activer ou désactiver le mode Conceal à tout moment avec **`Ctrl + L`**.

### Éléments masqués dynamiquement :
- Titres (`# `, `## `, `### `, etc.)
- Marqueurs gras (`**` ou `__`)
- Marqueurs italiques (`*` ou `_`)
- Marqueurs barrés (`~~`)
- Délimiteurs de code inline (`` ` ``)
- Délimiteurs de liens Markdown (`[` et `](url)`)
- Préfixes de citations (`> `)

---

## 4. Mode Prévisualisation Native (`Ctrl + E`)

En complément du mode Conceal interactif, Slate intègre un moteur de rendu **100% natif GTK 4** (sans moteur Chromium, sans WebView, sans Electron) :
- Appuyez sur **`Ctrl + E`** pour basculer entre l'éditeur de texte et la vue de prévisualisation rendue.
- La prévisualisation interprète les titres, paragraphes, citations avec barres verticales, blocs de code avec coloration et bordure arrondie, et tableaux Markdown.
- Appuyez de nouveau sur **`Ctrl + E`** pour revenir instantanément à l'édition brute.

---

## 5. Recherche et Remplacement

Slate intègre un panneau flottant discret superposé en haut à droite :
- **Recherche simple** : **`Ctrl + F`**
  - Tapez le terme recherché.
  - Parcourez les occurrences avec `Entrée` (suivant) et `Shift + Entrée` (précédent).
  - Fermez le panneau avec `Échap`.
- **Recherche & Remplacement** : **`Ctrl + H`**
  - Permet de remplacer l'occurrence courante ou toutes les occurrences du document d'un coup.

---

## 6. Gestion des Fichiers & Sécurité des Données

### Sauvegarde automatique (Autosave)
Slate dispose d'un système de sauvegarde automatique périodique (par défaut toutes les 15 secondes si le document a été modifié). Vous pouvez configurer l'intervalle dans `config.json` ou le désactiver (`"Disabled"`).

### Sauvegarde atomique (Crash-safe)
Chaque sauvegarde effectuée par Slate s'effectue de manière atomique :
1. Les données sont d'abord écrites dans un fichier temporaire sur le même système de fichiers (`.tmp_slate_*`).
2. Les buffers disque sont flushés avec `sync_all()`.
3. Le fichier temporaire est renommé de façon atomique vers le fichier cible.
Cela garantit qu'en cas de coupure de courant ou de crash, votre note originale n'est jamais corrompue ou vidée.

### Détection de modifications externes
Si un fichier ouvert dans Slate est modifié par un autre programme (ex: `git pull`, éditeur externe), Slate détecte le changement d'horodatage et vous propose :
- De recharger la version disque.
- De conserver la version en mémoire.
- D'afficher un comparateur de différences (diff) visuel en direct.
