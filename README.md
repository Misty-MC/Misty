<div align="center">

<img src="https://misty-launcher.online/assets/icone.png" alt="Misty" width="110" />

# Misty Launcher

**The Minecraft launcher, light as the mist.**

Written in Rust with a [Slint](https://slint.dev) interface: a single `Misty.exe`, no runtime to install.

[![Website](https://img.shields.io/badge/website-misty--launcher.online-8ab4f8?style=flat-square)](https://misty-launcher.online/)
[![Version](https://img.shields.io/github/v/release/Misty-MC/Misty-Launcher?label=version&style=flat-square)](https://github.com/Misty-MC/Misty-Launcher/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/Misty-MC/Misty-Launcher/total?style=flat-square)](https://github.com/Misty-MC/Misty-Launcher/releases)
[![Rust](https://img.shields.io/badge/written%20in-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Slint](https://img.shields.io/badge/UI-Slint-2379F4?style=flat-square)](https://slint.dev)
[![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?style=flat-square&logo=windows)](https://misty-launcher.online/)
[![License](https://img.shields.io/badge/license-Misty%20(no%20resale)-blue?style=flat-square)](LICENSE.txt)

[**Download**](https://misty-launcher.online/) · [**Docs**](https://misty-launcher.online/docs) · [**Patch notes**](https://misty-launcher.online/patch-notes) · [**Releases**](https://github.com/Misty-MC/Misty-Launcher/releases) · [**Website source**](https://github.com/Misty-MC/Misty-Web)

<img src="https://raw.githubusercontent.com/Misty-MC/Misty-Web/main/public/assets/screens/en/home.png" alt="Misty home screen" width="800" />

<table>
  <tr>
    <td><img src="https://raw.githubusercontent.com/Misty-MC/Misty-Web/main/public/assets/screens/en/profiles.png" alt="Profiles" /></td>
    <td><img src="https://raw.githubusercontent.com/Misty-MC/Misty-Web/main/public/assets/screens/en/textures.png" alt="Texture packs" /></td>
    <td><img src="https://raw.githubusercontent.com/Misty-MC/Misty-Web/main/public/assets/screens/en/settings.png" alt="Settings" /></td>
  </tr>
  <tr>
    <td align="center"><sub>Profiles · Modrinth &amp; CurseForge modpacks</sub></td>
    <td align="center"><sub>Texture pack editor</sub></td>
    <td align="center"><sub>Settings · Mist, Night &amp; seasonal themes</sub></td>
  </tr>
</table>

</div>

## ✨ Features

- **Every Minecraft version**: releases, plus snapshots, betas and alphas/indev as options
- **Profiles** (like Modrinth App instances): each one has its own version, mod loader and folder (mods, packs, shaders, worlds, options); versions, libraries and assets stay shared
- **Modrinth and CurseForge content**: search and install mods, resource packs, datapacks and shaders filtered by the profile's version and loader, with required dependencies. Files added by hand are identified by hash (Modrinth) or fingerprint (CurseForge)
- **Modpacks** (`.mrpack` and CurseForge): one click creates a ready-to-play profile
- **Mod loaders**: Forge, NeoForge, Fabric, Quilt, OptiFine and LiteLoader
- **Misty Client in game** (Fabric 26.3 profiles): Misty main menu, HUD you can arrange (FPS, CPS, keystrokes, coordinates…), the Misty cloud next to Misty players in the player list and the chat, Misty cape and elytra
- **Skins and capes**: 3D preview (walk, elytra, front / side / back), cape switching, local cache that survives Mojang's rate limit
- **Texture pack editor** (experimental): drawing tools, recent colors, pack icon, export, live reload from an external editor
- **Accounts**: several Microsoft accounts and offline accounts
- **Themes**: Mist (light), Night (dark) or synced with Windows, plus Halloween and Christmas themes that switch on by themselves
- **Languages**: English, French, Spanish, Italian, Portuguese, Russian
- **Storage manager**, adjustable RAM, **Discord Rich Presence**, **automatic updates** from GitHub releases

## 📥 Installation

Download `Misty-Setup-<version>.exe` from the [latest release](https://github.com/Misty-MC/Misty-Launcher/releases/latest) and run it.

- No admin rights needed: Misty installs into `%LOCALAPPDATA%\Programs\Misty`.
- Nothing else to install: Java is downloaded automatically when the game needs it.
- **Portable version**: download the zip, extract it and run `Misty-<version>/Misty.exe`.

Updates install themselves afterwards.

## 🛠️ Build from source

Requirements: Windows 10/11 and [Rust](https://rustup.rs) (stable, MSVC target) with the Visual Studio C++ build tools.

```powershell
cd misty-rust
cargo run              # development build
cargo build --release  # final exe: target\release\Misty.exe
cargo test -- --ignored --nocapture   # uses the Internet and your Misty folder, never launches the game
```

**Misty Client** (the in-game mod, git submodule of [Misty-MC/Misty-Client](https://github.com/Misty-MC/Misty-Client), clone with `git clone --recursive` or run `git submodule update --init`): build it first with `gradlew build` in `misty-client` (JDK 25) and the launcher embeds it. Without it, the launcher still builds, just without the in-game features.

**CurseForge API key**: CurseForge requires a personal (free) key from [console.curseforge.com](https://console.curseforge.com). Put it in `misty-rust/curseforge.key` (git-ignored) or in the `MISTY_CURSEFORGE_KEY` environment variable; it is embedded in the exe at build time. Without it, Modrinth still works.

## 🚀 Publishing a release

1. Bump `VERSION` in [misty-rust/src/services/app_info.rs](misty-rust/src/services/app_info.rs) and `version` in [misty-rust/Cargo.toml](misty-rust/Cargo.toml), then add the entry to [CHANGELOG.md](CHANGELOG.md) (embedded in the exe, shown in "Patch notes").
2. Run the release script (needs a JDK 25 for Misty Client and [Inno Setup 6](https://jrsoftware.org/isdl.php): `winget install JRSoftware.InnoSetup`):

   ```powershell
   powershell -ExecutionPolicy Bypass -File installer\build.ps1
   ```

   It produces in `publish\` the `Misty-<version>` folder, `Misty-<version>.zip` (used by the updater) and `Misty-Setup-<version>.exe`. `-SansInstallateur` skips the installer.
3. Create a GitHub release tagged `<version>` and attach **the zip and the installer**.

> [!WARNING]
> `VERSION`, the GitHub tag (without `v`) and the `Misty-<version>` folder name must match, otherwise the launcher will offer the update in a loop.

## 🗂️ Project structure

```
CHANGELOG.md           patch notes (embedded in the exe)
installer/             release script (build.ps1) + Inno Setup installer (Misty.iss)
misty-client/          Misty Client (submodule → Misty-MC/Misty-Client): the in-game Fabric mod, embedded in the exe
misty-rust/
├── build.rs           embeds the icon, CurseForge key, changelog and Misty Client
├── misty-ui/          crate that compiles the Slint interface (kept apart so Rust edits rebuild fast)
├── assets/            icon, Silkscreen pixel font (OFL), flags
├── lang/              translations (en, fr, es, it, ru), embedded in the exe
├── ui/                Slint interface (theme, widgets, state, one file per page)
└── src/
    ├── main.rs        startup
    ├── ms_login.rs    Microsoft sign-in window (subprocess "Misty.exe --ms-login")
    ├── ui/            code behind each page
    ├── minecraft/     game install and launch (versions, Java, loaders)
    └── services/      logic without UI (profiles, content, Modrinth, CurseForge,
                       modpacks, skins, storage, updates, settings, Discord…)
```

User data lives in `%LOCALAPPDATA%\Misty\` (`data.txt` settings, `msa.json` Microsoft account, `instances/<id>/` profiles, plus versions, libraries, assets and Java). When a key is added to or removed from `data.txt`, bump `DATA_VERSION` in `app_info.rs` and add the key to `services/settings.rs`.

## 📄 License

The code is released under the [Misty License](LICENSE.txt): free to use, modify, share and contribute to, but it may not be sold. Versions published before this license remain under MIT.
The Silkscreen font is distributed under the SIL Open Font License.

---

<div align="center">
<sub>
Misty is an independent launcher and is not affiliated with Mojang Studios or Microsoft.<br/>
No Minecraft files are shipped: the game is downloaded from Mojang's official servers on first launch.
</sub>
</div>
