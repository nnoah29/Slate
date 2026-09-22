# Intégration Wayland, Hyprland & Gestionnaires de fenêtres

Slate est particulièrement optimisée pour fonctionner harmonieusement avec les gestionnaires de fenêtres Wayland modernes en mode "tiling" tels que **Hyprland**, **Sway**, ou les environnements comme KDE Wayland et GNOME.

---

## 1. Intégration Hyprland

Pour que Slate s'affiche toujours comme une note flottante, épinglée et sans bordure avec le même look que Kitty, ajoutez la règle de fenêtre suivante dans votre configuration Hyprland :

### Configuration Hyprland (format classique `hyprland.conf`)
```ini
# Slate : note flottante, centrée/épinglée, sans bordure
windowrulev2 = float, class:^(com\.github\.slate\.Slate|slate)$
windowrulev2 = size 515 338, class:^(com\.github\.slate\.Slate|slate)$
windowrulev2 = pin, class:^(com\.github\.slate\.Slate|slate)$
windowrulev2 = noborder, class:^(com\.github\.slate\.Slate|slate)$
windowrulev2 = nodim, class:^(com\.github\.slate\.Slate|slate)$
```

### Configuration Hyprland (format Lua / Imperative dots)
Si vous utilisez une configuration modulaire Hyprland en Lua (ex: dans `~/.config/hypr/config/windowrule.lua`) :

```lua
hl.window_rule({
    match = { class = "^(com\\.github\\.slate\\.Slate|slate)$" },
    float = true,
    size = "515 338",
    pin = true,
})
```

---

## 2. Translucidité & Flou (Blur)

Slate fournit un rendu translucide coordonné avec le terminal Kitty :
- **Couleur de fond** : `rgba(39, 38, 38, opacity)` (code hex `#272626`).
- **Opacité** : configurable via `"opacity": 0.5` dans `~/.config/Slate/config.json`.
- **Flou (Blur)** : 
  - Slate supporte le flou natif via la directive CSS `backdrop-filter: blur(Xpx);` configurée via le paramètre `"blur"` de `config.json`.
  - Si le flou global est activé dans votre compositeur Hyprland (`decoration:blur:enabled = true`), Hyprland applique un flou matériel à haute performance sur la région translucide.

---

## 3. Raccourcis globaux recommandés (Keybindings)

Pour faire apparaître Slate instantanément depuis n'importe quel espace de travail en tant que scratchpad ou bloc-notes rapide, vous pouvez lier un raccourci global dans votre gestionnaire de fenêtres :

### Dans Hyprland :
```ini
# Ouvrir ou basculer sur Slate avec Super + N
bind = $mainMod, N, exec, slate
```

Grâce au système mono-instance DBus de Slate, exécuter `slate` lorsque la fenêtre est déjà ouverte ramène instantanément votre note au premier plan sans ouvrir de doublon.
