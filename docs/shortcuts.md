# Guide des raccourcis clavier de Slate

Slate est optimisé pour une utilisation 100% au clavier, sans jamais avoir besoin de lever les mains vers la souris.

---

## 1. Gestion des fichiers & Système

| Raccourci | Action | Description |
|-----------|--------|-------------|
| **`Ctrl + S`** | Sauvegarder | Sauvegarde immédiate. Si le document n'a pas de nom, ouvre la boîte de dialogue d'enregistrement. |
| **`Ctrl + Shift + S`** | Sauvegarder sous... | Ouvre le sélecteur de fichier dans `save_directory`. |
| **`Ctrl + N`** | Nouvelle note | Crée un document vierge (invite de sauvegarde si des modifications sont non enregistrées). |
| **`Ctrl + O`** | Ouvrir un document | Ouvre le sélecteur de fichier Markdown dans `save_directory`. |
| **`Ctrl + W`** | Fermer le document | Ferme la note en cours avec contrôle des modifications non enregistrées. |
| **`Ctrl + Q`** | Quitter | Quitte l'application en sauvegardant la géométrie de la fenêtre. |
| **`Ctrl + Z`** | Annuler | Annule la dernière modification de texte. |
| **`Ctrl + Shift + Z`** | Rétablir | Rétablit la modification annulée. |

---

## 2. Formatage Markdown en direct

Les raccourcis de formatage enveloppent la sélection courante ou insèrent les balises à la position du curseur :

| Raccourci | Élément | Syntaxe insérée | Exemple |
|-----------|---------|-----------------|---------|
| **`Ctrl + B`** | Gras | `**texte**` | **texte** |
| **`Ctrl + I`** | Italique | `*texte*` | *texte* |
| **`Ctrl + K`** | Lien | `[texte](url)` | [Lien](https://example.com) |
| **`Ctrl + Shift + X`** | Barré | `~~texte~~` | ~~texte barré~~ |
| **`Ctrl + Shift + H`** | Titre (Cycle) | `# `, `## `, `### `, etc. | Alterne entre les niveaux H1 à H6 puis retour à normal |
| **`Ctrl + Shift + 8`** | Liste à puces | `- texte` | Active ou désactive la liste non ordonnée |
| **`Ctrl + Shift + 7`** | Liste numérotée | `1. texte` | Active ou désactive la liste ordonnée avec numérotation |
| **`Ctrl + Shift + C`** | Code | `` `code` `` ou ```` ``` ```` | Code en ligne sur sélection courte, bloc de code sinon |
| **`Ctrl + Shift + Q`** | Citation | `> texte` | Préfixe la ligne courante ou le bloc sélectionné |
| **`Tab`** | Indentation | 4 espaces | Indente la ligne ou les lignes sélectionnées |
| **`Shift + Tab`** | Désindentation | -4 espaces | Réduit l'indentation |

---

## 3. Affichage, Zoom & Rendu

| Raccourci | Action | Description |
|-----------|--------|-------------|
| **`Ctrl + L`** | Basculer mode Conceal | Active/désactive le masquage dynamique des marqueurs de syntaxe Markdown. |
| **`Ctrl + E`** | Prévisualisation / Édition | Bascule entre la vue d'écriture brute et le rendu natif Markdown GTK 4. |
| **`Ctrl + +`** ou **`Ctrl + =`** | Zoom avant | Augmente la taille de la police de 1pt (jusqu'à 48pt max). |
| **`Ctrl + -`** | Zoom arrière | Réduit la taille de la police de 1pt (jusqu'à 8pt min). |
| **`Ctrl + 0`** | Réinitialiser zoom | Rétablit la taille de police par défaut configurée dans `config.json`. |

---

## 4. Recherche et Remplacement

| Raccourci | Action | Navigation |
|-----------|--------|------------|
| **`Ctrl + F`** | Ouvrir la recherche | `Entrée` : occurrence suivante<br>`Shift + Entrée` : occurrence précédente<br>`Échap` : fermer la barre |
| **`Ctrl + H`** | Ouvrir rechercher/remplacer | Remplacement individuel ou global |
