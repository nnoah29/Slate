# Slate

Application de notes Markdown native Linux, sobre, rapide et axée sur l'écriture ("feuille de papier").

![Slate Icon](data/icons/hicolor/scalable/apps/com.github.slate.Slate.svg)

> 📖 **Documentation complète** : Retrouvez les guides détaillés dans le dossier [`docs/`](docs/) :
> - [Guide d'utilisation](docs/usage.md)
> - [Guide de configuration (`~/.config/Slate/config.json`)](docs/configuration.md)
> - [Tableau des raccourcis clavier](docs/shortcuts.md)
> - [Intégration Hyprland & Wayland](docs/hyprland.md)
> - [Installation avec `yay` & publication AUR](docs/aur_publishing.md)

---

## 1. Vision & Philosophie

* **Simplicité** : Aucune sidebar permanente, aucune barre encombrante, aucun compte, aucune base de données. L'interface s'efface pour laisser place au texte.
* **Rapidité & Sobriété** : Développée en **Rust**, **GTK 4** et **libadwaita**. Démarrage instantané, empreinte mémoire et CPU minimale.
* **Local-first & Portable** : Les notes sont de simples fichiers Markdown (`.md`) UTF-8 bruts enregistrés directement sur votre machine. Aucune télémétrie, aucune synchronisation imposée, aucun format propriétaire.
* **100% Natif** : Sans Electron, sans Chromium, sans WebView.

---

## 2. Raccourcis Clavier

### Système & Fichiers

| Raccourci | Action |
|-----------|--------|
| `Ctrl+S` | Sauvegarder immédiatement |
| `Ctrl+Shift+S` | Sauvegarder sous... |
| `Ctrl+N` | Nouvelle note |
| `Ctrl+O` | Ouvrir un document Markdown |
| `Ctrl+W` | Fermer le document courant |
| `Ctrl+Q` | Quitter l'application |
| `Ctrl+Z` | Annuler |
| `Ctrl+Shift+Z` | Rétablir |

### Rendu & Affichage

| Raccourci | Action |
|-----------|--------|
| `Ctrl+E` | Basculer entre Édition brute et Prévisualisation Markdown native |
| `Ctrl+L` | Activer / Désactiver le masquage dynamique des balises (mode conceal) |
| `Ctrl++` / `Ctrl+=` | Augmenter la taille de la police |
| `Ctrl+-` | Diminuer la taille de la police |
| `Ctrl+0` | Réinitialiser le zoom |

### Recherche & Remplacement

| Raccourci | Action |
|-----------|--------|
| `Ctrl+F` | Ouvrir la recherche (navigation `Entrée` / `Shift+Entrée`, `Échap` pour fermer) |
| `Ctrl+H` | Ouvrir la recherche et remplacement |

### Formatage Markdown

| Raccourci | Action | Syntaxe générée |
|-----------|--------|-----------------|
| `Ctrl+B` | Gras | `**texte**` |
| `Ctrl+I` | Italique | `*texte*` |
| `Ctrl+K` | Lien | `[texte](url)` |
| `Ctrl+Shift+X` | Barré | `~~texte~~` |
| `Ctrl+Shift+H` | Titre | Cycle `# `, `## `, `### `, `#### `, etc. |
| `Ctrl+Shift+8` | Liste à puces | `- texte` |
| `Ctrl+Shift+7` | Liste numérotée | `1. texte` |
| `Ctrl+Shift+C` | Code | `` `code` `` ou bloc ```` ``` ```` |
| `Ctrl+Shift+Q` | Citation | `> texte` |
| `Tab` | Indenter | 4 espaces |
| `Shift+Tab` | Désindenter | -4 espaces |

---

## 3. Utilisation en ligne de commande (CLI)

```bash
# Ouvrir une nouvelle note vierge
slate

# Ouvrir une note spécifique
slate ~/Documents/projet.md

# Forcer une nouvelle note vierge
slate --new

# Mode debug verbeux
slate --debug

# Version et aide
slate --version
slate --help
```

L'application supporte le multi-instance DBus natif : lancer `slate ma-note.md` alors que l'application est déjà ouverte bascule automatiquement la note dans la fenêtre existante.

---

## 4. Compilation & Tests

### Prérequis (Linux)
- Rust & Cargo (édition 2024 / 1.85+)
- GTK 4 (`gtk4`)
- libadwaita (`libadwaita-1`)
- GtkSourceView 5 (`gtksourceview-5`)

### Lancer les tests unitaires
```bash
cargo test
```

### Compiler et exécuter
```bash
cargo run --release
```

---

## 5. Architecture logicielle

```text
src/
├── main.rs          # Point d'entrée CLI et initialisation
├── lib.rs           # Bibliothèque exportant les modules
├── app.rs           # Gestionnaire d'application libadwaita & IPC
├── window.rs        # Fenêtre principale, cycle de vie, dialogues et minuteurs
├── editor/          # Moteur d'édition de texte (GtkSourceView)
│   ├── mod.rs       # Vue de l'éditeur et marges aérées
│   ├── buffer.rs    # Tampon de texte avec coloration Markdown
│   ├── markdown.rs  # Analyse et manipulations Markdown
│   └── formatting.rs# Application des raccourcis de formatage
├── document/        # Modèle de données de la note
│   ├── mod.rs
│   ├── state.rs     # États Clean, Dirty, Saving, Error
│   └── metadata.rs  # Statistiques de mots, lignes, caractères
├── storage/         # Persistance et surveillance filesystem
│   ├── mod.rs       # Détection de modifications externes et diff
│   ├── reader.rs    # Lecture UTF-8 sécurisée
│   └── writer.rs    # Écriture atomique (protection crash)
├── shortcuts/       # Architecture centralisée des raccourcis
│   └── mod.rs
├── search/          # Recherche et remplacement en incrustation
│   └── mod.rs
├── preview/         # Rendu Markdown 100% natif GTK 4 (sans WebView)
│   └── mod.rs
└── config/          # Configuration utilisateur standard XDG
    └── mod.rs
```

---

## 6. Licence

Projet distribué sous licence MIT.
