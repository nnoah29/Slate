# Guide de configuration de Slate

## 1. Emplacement du fichier de configuration

Le fichier de configuration de Slate se situe selon le standard XDG dans votre dossier utilisateur :

```bash
~/.config/Slate/config.json
```

Si ce dossier ou ce fichier n'existe pas lors du premier lancement, Slate le crée automatiquement avec les valeurs par défaut optimisées.

---

## 2. Exemple de configuration complète

Voici un fichier `config.json` complet avec les valeurs par défaut :

```json
{
  "theme": "Dark",
  "font_size": 12.0,
  "font_family": "Iosevka Nerd Font",
  "auto_save_interval": "Sec15",
  "window_width": 515,
  "window_height": 338,
  "is_maximized": false,
  "save_directory": "/home/nnoah",
  "blur": 0.0,
  "opacity": 0.5,
  "recent_files": [
    "/home/nnoah/ok.md"
  ],
  "custom_shortcuts": {}
}
```

---

## 3. Détail des options

### `save_directory` *(chaîne de caractères)*
- **Description** : Répertoire par défaut vers lequel s'ouvrent les boîtes de dialogue "Enregistrer sous..." (`Ctrl+Shift+S`) et "Ouvrir un document" (`Ctrl+O`).
- **Support du tilde `~`** : Vous pouvez utiliser un chemin absolu comme `"/home/nnoah/Notes"` ou utiliser la notation raccourcie `"~/Notes"`. Slate résoudra automatiquement le chemin absolu.
- **Exemple** :
  ```json
  "save_directory": "~/Notes"
  ```

### `font_family` *(chaîne de caractères)*
- **Description** : Police typographique monospace utilisée dans l'éditeur et l'affichage.
- **Alias acceptés** : `"font"`, `"police"`, `"font_name"`.
- **Valeur par défaut** : `"Iosevka Nerd Font"` (avec fallback automatique sur `monospace` si non installée).
- **Exemple** :
  ```json
  "font_family": "JetBrains Mono"
  ```

### `font_size` *(nombre flottant)*
- **Description** : Taille de la police en points typographiques (`pt`).
- **Alias acceptés** : `"size"`, `"taille_police"`, `"font_size_pt"`.
- **Valeur par défaut** : `12.0` (bornée entre `8.0` et `48.0`).
- **Exemple** :
  ```json
  "font_size": 13.5
  ```

### `blur` *(nombre flottant)*
- **Description** : Degré de flou d'arrière-plan en pixels appliqué via `backdrop-filter`.
- **Alias acceptés** : `"background_blur"`, `"blur_degree"`, `"blur_radius"`.
- **Valeurs** : `0.0` à `100.0`.
  - `0.0` : Désactive le flou (identique au paramètre `background_blur 0` de Kitty).
  - `10.0` ou plus : Floute doucement les éléments situés derrière la fenêtre de Slate.
- **Exemple** :
  ```json
  "blur": 15.0
  ```

### `opacity` *(nombre flottant)*
- **Description** : Niveau d'opacité du fond de la fenêtre.
- **Alias acceptés** : `"background_opacity"`.
- **Valeurs** : `0.0` (entièrement transparent) à `1.0` (opaque).
- **Valeur par défaut** : `0.5` (reproduit fidèlement la translucidité de Kitty).
- **Exemple** :
  ```json
  "opacity": 0.65
  ```

### `auto_save_interval` *(chaîne de caractères)*
- **Description** : Intervalle de déclenchement de la sauvegarde automatique.
- **Valeurs acceptées** :
  - `"Disabled"` : Désactive la sauvegarde automatique.
  - `"Sec5"` : Toutes les 5 secondes.
  - `"Sec15"` : Toutes les 15 secondes *(défaut)*.
  - `"Sec30"` : Toutes les 30 secondes.
  - `"Min1"` : Toutes les 60 secondes.

### `theme` *(chaîne de caractères)*
- **Description** : Thème de couleurs de l'application.
- **Valeurs acceptées** : `"Dark"`, `"Light"`, `"System"`.
- **Valeur par défaut** : `"Dark"`.

### `window_width` & `window_height` *(entiers)*
- **Description** : Dimensions par défaut de la fenêtre lors de l'ouverture (mémorisées automatiquement à la fermeture de l'application).
- **Valeurs par défaut** : `515` x `338`.
- **Limites minimales** : Largeur min `320px`, hauteur min `240px`.

### `is_maximized` *(booléen)*
- **Description** : Indique si la fenêtre doit s'ouvrir maximisée (`true`) ou flottante (`false`).
- **Valeur par défaut** : `false`.

### `recent_files` *(tableau de chemins)*
- **Description** : Liste des 20 derniers fichiers ouverts ou sauvegardés, classés par ordre de consultation récente.

### `custom_shortcuts` *(objet clé/valeur)*
- **Description** : Permet de réassigner certains raccourcis clavier de base si souhaité.

---

## 4. Tolérance et robustesse du JSON

Le parseur de Slate utilise `#[serde(default)]`. Cela signifie que :
- Vous n'êtes pas obligé de spécifier toutes les clés dans votre fichier.
- Si vous créez un fichier ne contenant que :
  ```json
  {
    "save_directory": "~/Documents",
    "blur": 12.0
  }
  ```
  Slate appliquera vos préférences personnalisées et utilisera les valeurs par défaut pour toutes les autres options sans aucune erreur.
