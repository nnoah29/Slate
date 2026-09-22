# Publication sur l'AUR (Installation avec `yay`)

Pour permettre à n'importe quel utilisateur d'installer Slate avec `yay` (`yay -S slate-git`), le paquet doit être hébergé sur l'**Arch User Repository (AUR)**.

Les fichiers [`PKGBUILD`](../packaging/aur/slate-git/PKGBUILD) et [`.SRCINFO`](../packaging/aur/slate-git/.SRCINFO) ont déjà été entièrement rédigés, configurés et testés avec succès via `makepkg`.

---

## Étape 1 : Créer son compte AUR (si ce n'est pas déjà fait)

1. Rendez-vous sur [aur.archlinux.org/register](https://aur.archlinux.org/register).
2. Créez votre compte.
3. Allez dans **Mon compte** (My Account).
4. Copiez votre clé SSH publique :
   ```bash
   cat ~/.ssh/id_ed25519.pub
   ```
5. Collez-la dans le champ **Clé publique SSH** (SSH Public Key) puis validez.

---

## Étape 2 : Publier le paquet sur l'AUR

L'AUR fonctionne comme une collection de petits dépôts Git. Pour créer le paquet `slate-git` :

```bash
# 1. Cloner le dépôt AUR officiel de votre paquet (il est vide au début)
git clone ssh://aur@aur.archlinux.org/slate-git.git /tmp/slate-git

# 2. Copier les fichiers PKGBUILD et .SRCINFO pré-générés
cp packaging/aur/slate-git/PKGBUILD /tmp/slate-git/
cp packaging/aur/slate-git/.SRCINFO /tmp/slate-git/

# 3. Commiter et pousser sur l'AUR
cd /tmp/slate-git
git add PKGBUILD .SRCINFO
git commit -m "Initial release of slate-git"
git push origin master
```

---

## Étape 3 : Installer avec `yay`

Dès que la commande `git push` est terminée, le paquet est disponible immédiatement pour tous les utilisateurs d'Arch Linux, Manjaro, EndeavourOS, etc. :

```bash
yay -S slate-git
```

### Pourquoi `slate-git` ?
Le suffixe `-git` (paquet VCS) compile automatiquement la version la plus récente directement depuis votre dépôt GitHub `https://github.com/nnoah29/Slate.git`. Cela signifie que chaque fois que vous ferez un `git push` sur GitHub, quiconque installe ou met à jour avec `yay` aura automatiquement vos dernières modifications sans que vous ayez besoin de republier sur l'AUR à chaque commit !
