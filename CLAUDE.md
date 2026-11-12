# Misty Launcher

Launcher Minecraft pour Windows, écrit en Rust avec une interface Slint. Repo : Misty-MC/Misty-Launcher.
Le mod en jeu (Misty Client) est le sous-module `misty-client/` → Misty-MC/Misty-Client (il a son propre `CLAUDE.md`).

## Cloner et compiler

```powershell
git clone --recursive https://github.com/Misty-MC/Misty-Launcher.git   # ou : git submodule update --init
cd misty-rust
cargo check --all-targets   # vérification rapide
cargo test                  # tests hors ligne (doivent passer avant chaque commit)
cargo test -- --ignored --nocapture   # tests qui utilisent Internet et le dossier Misty
cargo run                   # lancer le launcher
```

- Le mod doit être compilé d'abord (`gradlew build` dans `misty-client/`, JDK 25) pour être intégré à l'exe. Sans lui, le build passe avec l'avertissement « Misty Client n'est pas compilé ».
- Windows uniquement : WebView2, registre, raccourcis. La fenêtre de connexion Microsoft (`src/ms_login.rs`, winit + wry) n'existe que sous `cfg(windows)`.
- Release : `installer\build.ps1` (voir le README, section « Publishing a release »).

## Conventions du code

- **Identifiants et commentaires en français** (`executer`, `fenetre`, `dossier`…), comme le reste du code. Garder ce style.
- Textes de l'interface : jamais en dur. Ajouter la clé dans **les 6 fichiers** `misty-rust/lang/*.json` (en, fr, es, it, pt, ru), puis l'utiliser avec `loc::t("Cle")`.
- Nouvelle clé dans `data.txt` (réglages) : incrémenter `DATA_VERSION` dans `src/services/app_info.rs` et l'ajouter dans `src/services/settings.rs`.
- Ne pas modifier `GITHUB_REPO` (`app_info.rs`) ni l'`AppId` de `installer/Misty.iss` : les mises à jour automatiques et l'installateur en dépendent.
- Le nom affiché est « Misty Launcher », mais l'exe (`Misty.exe`) et les dossiers de données (`%LOCALAPPDATA%\Misty`, `%LOCALAPPDATA%\Programs\Misty`) gardent le nom « Misty » : les renommer casserait les installations existantes.
- `misty-rust/curseforge.key` est privé et ignoré par git : ne jamais le commiter.

## Commits et branches

- Branches : `main` (publiée), `EnderMythex` et `xtheox` (branches perso). Travailler sur sa branche, PR vers `main`. Demander avant tout push ou merge sur `main`.
- Message : sujet en anglais à l'impératif, sans point final (`Add …`, `Fix …`, `Keep …`), corps optionnel qui explique pourquoi. Terminer par `Co-Authored-By: Claude <noreply@anthropic.com>` quand Claude a écrit le changement.
- Tout changement visible par l'utilisateur va dans `CHANGELOG.md`, en anglais, sous la version en cours (`## <version>` puis `### Launcher` ou `### Misty Client (in game)`). Ce fichier est intégré dans l'exe et affiché dans « Patch notes ».
- Mise à jour du mod : `git submodule update --remote misty-client`, puis `git add misty-client` et un commit dédié.
- Version : `VERSION` dans `app_info.rs`, `version` dans `Cargo.toml` et le tag GitHub (sans `v`) doivent être identiques, sinon le launcher propose la mise à jour en boucle.

## Dépendances

Dependabot ouvre des PR chaque semaine (cargo + sous-module). Pour les traiter : les merger ensemble dans une branche locale, puis `cargo check --all-targets` et `cargo test`, et pousser seulement si tout passe. Un bump 0.x → 0.y peut casser l'API.
