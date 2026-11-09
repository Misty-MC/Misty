# Changelog
## 0.3.8
### Profiles
- The filters of a profile's content (All, Mods, Resource packs, Datapacks, Shaders) now show how many items each one holds, for example "Mods (12)"
### Texture packs
- You can now create texture packs for Minecraft 1.0 to 1.5.2 (before 1.6): the original textures are extracted from the game, they are sorted in the editor like the other versions, and the exported zip uses the old layout (textures at the root, description in pack.txt)

## 0.3.7
### Game console
- New console over the launcher, like PvP launchers: the game's logs live, colored by level (info, warnings, errors, Misty's own messages), for Minecraft and Horizon
- Minecraft's XML logs are turned into readable lines (`[12:34:56] [Render thread/INFO]: …`), with the full stack trace of each error
- Filters (All / Warnings / Errors) with counters, search, auto-scroll that stops when you scroll up and a "Follow" button to come back down
- Copy the logs, open the logs folder, or stop a game that froze
- "Send logs": sends the last lines of the game and gives you a report code (copied for you) to share on Discord to get help
- Opens when a game starts (can be turned off in the settings), and on its own when a game crashes
- New LOGS button next to PLAY, in the same style, to open it anytime
- Follows the theme: light with the light theme, dark with the dark theme

### Several games at once
- PLAY is available again as soon as a game starts: run several Minecraft versions, profiles or Horizon builds at the same time
- The console splits the screen: one panel per game (side by side, 2 × 2 for 4 games, up to 3 × 2), each with its own state, counters and buttons. Maximize one panel with a double-click or its button
- Each game keeps the account that was picked when PLAY was clicked, even if you switch accounts while it downloads
- Microsoft and offline accounts are now in a single list on the home page: pick one, play, pick another, play again. The button next to it adds another Microsoft account

### Quilt
- New mod loader: Quilt can now be picked in the version picker and in the profile creation, for every Minecraft version from 1.14 on that Quilt supports
- Quilt modpacks are now supported: Modrinth modpacks (`quilt-loader`) and CurseForge modpacks install and launch with the right Quilt version. Before, CurseForge Quilt modpacks were refused as an unsupported loader
- Misty picks the latest stable Quilt loader by default and skips beta and release candidate builds
- Quilt profiles also load Fabric mods, so Modrinth and CurseForge searches show both
- Fix: Quilt crashed on recent Minecraft versions because an old loader was picked. Misty now picks the highest stable Quilt loader

### Modpacks and profiles
- New "Import a modpack" button on the Profiles page: pick a Modrinth `.mrpack` or a CurseForge `.zip` file and Misty creates a profile from it, with its mods, configs and the right mod loader
- The profile takes the name written in the modpack. Mods that CurseForge authors don't allow apps to download are listed so you can get them by hand
- A clear error message when the file is not a modpack, and a half-created profile is removed if the import fails
- Mods, resource packs, shaders and datapacks added by hand to a profile's folders now show up right away in the profile, without reopening it

### Updates
- New update window with a progress bar: percentage, MB downloaded and the source, then unpacking and restart
- If GitHub is down or out of quota, updates now come from the Misty website, which keeps a copy of the latest version
- The downloaded update is checked (complete file, right version) before it is installed
- New setting "Pre-release updates" (Settings › Troubleshooting): also get the beta and pre-release versions published on GitHub

### Logs and telemetry
- Misty now keeps its own log (`logs/misty/launcher.log`) and a log file for each of the last 10 games, with a "Misty log" button in Settings › Troubleshooting
- Optional anonymous telemetry, asked once and switchable in the settings: Misty version, Windows, RAM, games started, crashes with their last log lines (username, folders and tokens removed). Never your account, password or email

### Fixes and changes
- Horizon no longer opens a black console window on some PCs
- Version picker: the mod loader row no longer overflows the window on small screens, and the Done button stays inside
- Skin studio: removed the "Wave" animation

## 0.3.6
### Horizon
- Fix: Horizon builds failed to start with "unrecognized argument: --launch-api". Misty now works with every Horizon build: stable versions (v0.2.x) and the latest nightlies
- Fix: Mojang republished the Minecraft 26.3 files, which Horizon refused. Misty now downloads the exact version Horizon expects
- Horizon can be played without a Microsoft account: with an offline account, the game starts offline under your username. With a Microsoft account, you are still signed in and can join online servers
- Clearer error messages when Horizon can't start (only the useful line, not the whole help text)

### Skin studio
- Clicking a cosmetics category turns and zooms the camera onto what it dresses: the back for capes, the head for hats, the shoulder for pets, the name tag for tags…
- Zoom in and out with the mouse wheel. Double-click, Front / Side / Back or Rotate bring back the full view
- New hand cursor on the character: open hand on hover, closed fist while you turn it
- The Misty cape and elytra tiles now show their real texture, like the Mojang capes
- Nothing moves by itself anymore: no more floating character, breathing halo, fireflies or drifting buttons
- Removed the NameMC profile link

### Version picker
- Each update banner now has its own landmark: trial spawner (1.21), cherry tree (1.20), the Warden (1.19), snowy cliff (1.18), amethyst and axolotl (1.17), Nether portal (1.16), bee nest (1.15), village house (1.14), dolphin (1.13), colored concrete (1.12), woodland mansion (1.11), polar bear (1.10), End city (1.9), ocean monument (1.8) and Ferris the crab for Horizon
- Large banners now show the ground and the landmark
- Removed the mod loader filters at the top (Fabric, Forge…): the mod loader is still chosen at the bottom of the picker

### Friends (preview)
- New Friends page in the sidebar: add a friend by username, Online / All / Requests tabs, and a chat. It is only the interface for now: friends and chat are coming soon

### Launcher
- Release notes in the launcher now show only the releases published on GitHub
- Offline accounts added in the settings show up right away in the account list next to Play
- Removed the floating particles and their setting
- Removed the Versions tab from the settings: the version picker has its own filters
- Removed the "Reload the version list" button from the home page
- Texture packs: the empty page is centered, like the profiles page
- Removed the small hover animations on the sidebar icons and buttons
- Fix: a Misty shortcut pinned to the taskbar or the Start menu stopped working after an update. Misty now repairs it on startup
- Misty.exe and the installer are now signed with the ENDERR.WIN certificate (self-signed for now, so Windows SmartScreen may still show a warning)
- The old C# version was removed from the repository

## 0.3.5
### New look
- New settings window inspired by the Modrinth App: it opens as a popup over a blurred background, with categories in a sidebar
- Night theme based on the Misty website, and smoother animations everywhere in the launcher
- The window can now be resized and maximized, and the layout adapts to large windows
- New cloud logo, replacing the grass block
- The pixel landscape of the Misty website is now the launcher background, with a blur you can change in the settings (2 by default)
- Seasonal themes: Halloween and Christmas decorations appear automatically 3 days before and after the holiday
- Loading screen at startup, and faster switching between tabs
- Latest news card on the home page
- Lighter launch bar at the bottom of the home page, with a blue Play button and a simpler version card (for example "26.3 VANILLA")

### Version picker
- New full-size version picker, inspired by Lunar Client: versions are grouped by update (1.21 Tricky Trials, 1.20 Trails & Tales, 26.x...), each with its own pixel-art banner
- Filters at the top for releases, snapshots, betas and alphas, and for the mod loader (Vanilla, Fabric, Forge, NeoForge, OptiFine...): only the compatible versions are shown
- Search bar to find a version quickly
- Profiles tab inside the picker, to launch one of your profiles directly
- Fix: the text in the update banners no longer moves slightly

### Horizon (Minecraft rewritten in Rust)
- Misty can now play Horizon (playhorizon.dev), an open source (MIT) rewrite of Minecraft: Java Edition 26.3 in Rust. It is not official
- Horizon is pinned at the top of the version picker, with the same layout as the other versions: a banner, then its builds
- Builds come straight from Horizon's GitHub releases. Nightlies and stable versions can be filtered, and you can install several builds and choose which one to play. The Horizon launcher is not needed
- Click a build to download it (the chip fills up while it downloads), then click it again to select it. Installed builds can be deleted
- The first launch installs Minecraft 26.3 and its Java, then prepares the build once. Horizon needs a Microsoft account that owns the game

### Skin studio
- The Skin page has been rebuilt like the shop on the Misty website, with a live 3D renderer: drag the character to turn it, with tilt and inertia
- Animations: Idle, Walk, Run, Fly, Wave and Crouch (the character sneaks and stands back up), all smoother than before
- The cape moves with the character
- Misty capes and elytra can be tried on, the cosmetics shop is coming soon
- Your name tag is shown above the head, with a new Tags category
- New Skin button in the sidebar
- Fixes: the Fly animation was upside down, the hand went into the head while waving, Run moved diagonally and stayed leaning forward, the elytra were not spread enough while crouching, the wings were invisible from below, and a grey block stayed after removing a cape

### Profiles and modpacks
- The "Install a modpack" and "New profile" buttons now have the same size, and so do the buttons in the search results (Install, Create profile)

### Discord
- Discord Rich Presence with the Misty logo, a status for each page of the launcher, the version you are playing and "Playing as" with your username

### Other
- README with a screenshot gallery and the themes

## 0.3.4
- "Check for updates": only the two arrows now spin while Misty is searching, the button keeps its size so nothing moves around it anymore
- Removed the white vertical line between the account and the profile on the home page
- The default offline account created at the first launch is now called Misty
- Skin tab: the skin and the capes are loaded in the background at startup and kept on your disk, so the tab opens instantly and switching capes is immediate (the cape change is shown right away and sent to Mojang afterwards)
- Skin tab: the character preview is computed on all CPU cores, and a full 3D Steve is shown when you are not signed in
- When Mojang limits the number of requests, Misty keeps showing your last known skin and capes, tells you with a message, and checks your setup again by itself as soon as the limit is lifted
- The head of your account on the home page now comes from your saved skin instead of an external website
- Changing capes quickly no longer triggers Mojang's request limit: only the last chosen cape is sent, Misty retries by itself if Mojang asks to wait, and a clear message replaces the raw error
- Skin tab: the cape now hangs slightly diagonally behind the character, like on NameMC, and Front / Side / Back buttons under the preview turn the character to that view. Walk and Elytra buttons sit at the top right of the preview: the character walks with legs and arms swinging in opposition, and the wings use your cape texture. Front, Side and Back take the shortest way round, and Side shows the other side
- New "Manage storage" window in the settings: lists what can safely be deleted (Minecraft versions one by one, libraries, assets, Java, downloads, cache, logs and crash reports) with the size of each, detects leftovers and incomplete versions, and explains what happens before deleting. Worlds, instances and other personal data are never touched

## 0.3.3
- Misty now uses the new GitHub repository (Misty-MC/Misty) to look for updates: this version keeps your updates going without reinstalling
- New "Check for updates" button in the settings (About card)
- Several Microsoft accounts: add as many as you want, switch from the home page or from the settings, and forget one without losing the others
- Modpacks: Misty now lets you pick the version to install (Modrinth and CurseForge) and asks for a confirmation before creating the profile
- English is now the default language of the launcher (you can change it in the settings: English, French, Spanish, Italian, Russian)
- New cloud logo, used for the app icon and the sidebar
- The settings button moved to the bottom of the sidebar
- Skin tab: a full 3D Steve is shown when you are not signed in, instead of just a head
- The account head on the home page now uses your real skin when you are signed in with Microsoft
- Fix: dropdown lists no longer open on top of their own button when there is no room below, they open above it
- Fix: the dark background behind the new profile, new texture pack and modpack loading dialogs now covers the whole window
- The empty profile list is now centered, without an icon
- Removed the "experimental" popup when opening texture packs
- The installer license now shows the correct year and name

## 0.3.2
- New SKIN tab on the home page, next to PLAY (vertical text on the far right): change your skin and your cape from the launcher
- The character preview is now a real 3D model: drag it to turn it around and see your cape from behind
- Skin change: pick a PNG file (64x64 or 64x32), choose classic (Steve) or slim (Alex) arms, and it is sent straight to your Microsoft account; the chosen skin is previewed before sending
- Capes: every cape on your account is shown with a clear preview, 4 per row, one click equips it and None removes it
- The account badge on the home page shows the new head as soon as the skin is changed
- The tab needs a Microsoft account (otherwise a message offers to sign in) and is translated in every language
- Added new languages: Spanish, Italian and Russian, with their flags in the language selector

## 0.3.1
- Fenêtre aux coins arrondis : le fond de l'application épouse maintenant des angles arrondis, avec un affichage net sur tous les bords
- Amélioration : le sélecteur de version (menu général) s'ouvre directement sur la version choisie au lieu de revenir tout en haut de la liste, ce qui permet de choisir facilement la version juste avant ou après
- Correction : le même comportement est appliqué à toutes les listes déroulantes (profil, mod loader, …)
- Correction de bugs mineurs d'affichage
- Correction de Optifine dans les dernières versions

## 0.3.0
- Baisse considérable de l'utilisation de la RAM du launcher

## 0.2.4.3
- Séparer les textures modifié avec le template
- Ajout du sélecteur des langues
- Toute l'interface est traduite (français / anglais), y compris les messages et les boîtes de dialogue ; le changement de langue s'applique immédiatement
- Installateur Windows (Misty-Setup.exe) : sans droits administrateur, installe .NET 10 si besoin, raccourcis menu Démarrer / bureau
- Éditeur de packs de textures repensé : catégories avec compteurs, outils et couleurs plus lisibles, couleurs récentes, icône du pack, export depuis l'éditeur
- Éditeur : les traits rapides n'ont plus de trous, une seule annulation par trait, raccourcis (Ctrl+Z/Y/S, B, E, G, I), zoom Ctrl + molette
- Éditeur : la texture modifiée dans Paint.NET est rechargée automatiquement, la texture en cours est enregistrée en quittant la page
- Packs : pack_format correct pour les versions avant 1.14 et après 1.21.9, message clair pour les versions trop anciennes
- Correctifs : Rich Presence Discord (un seul client, l'option est respectée partout), bouton JOUER qui restait en français, avertissement « expérimental » affiché une seule fois

## 0.2.4.2
- Amélioration du modificateur de pack de textures

## 0.2.4.1
- Possibilité de supprimer un profil directement où il y a la liste des profils
- Fix curseforge key 
- Ajout de la modification de pack de ressources (Expérimentale)
- Possibilité de désactiver le rich presence

## 0.2.4
- CurseForge en plus de Modrinth : choix de la plateforme dans l'explorateur (mods, packs de ressources, datapacks, shaders, modpacks)
- Modpacks CurseForge : les fichiers que leurs auteurs réservent au site CurseForge sont listés, avec leur page à ouvrir
- Profils (instances) : chacun a sa version, son mod loader et son propre dossier de jeu ; sélection du profil depuis l'accueil
- Explorateur Modrinth dans chaque profil : mods, packs de ressources, datapacks et shaders, avec installation des dépendances
- Contenu d'un profil : activation / désactivation, suppression, identification des fichiers ajoutés à la main
- Installation de modpacks Modrinth (.mrpack) en un clic
- Les pseudos hors-ligne deviennent des « comptes hors-ligne »
- Nouvelle interface en WPF : fond ciel, cartes en verre dépoli, titres en police pixel, animations fluides
- Navigation latérale Jouer / Options / Notes de version, boîtes de dialogue intégrées
- Options repensées : profils en pastilles, RAM au curseur, interrupteurs pour les versions et le raccourci
- Tête du skin affichée à côté du compte, progression du téléchargement dans le bouton JOUER
- La mise à jour n'est plus proposée quand la release GitHub est plus ancienne que la version installée
- Ajout du modloader quilt
- Réorganisation du projet : `src/Misty` avec `Forms/` et `Services/`, formulaires renommés (`MainForm`, `SettingsForm`, `ChangelogForm`), contrôles renommés
- Correctifs : le raccourci est bien recréé après une mise à jour, les options snapshot/beta/alpha sont appliquées même avec un compte premium, la migration de `C:\TEXT` ne supprime plus l'ancien dossier si la copie a échoué
- Ajout du README et du CHANGELOG

## 0.2.3.9.5.1
- Calcul de la taille du dossier, possibilité de le vider

## 0.2.3.9.5
- Changement du dossier de sauvegarde (`C:\TEXT` → `%LOCALAPPDATA%\Misty`)