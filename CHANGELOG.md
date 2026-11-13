# Changelog
## 0.3.8-beta.3
### Launcher
- Your server list and saved hotbars are now the same in every version and profile: add a server once, find it everywhere. The first time, the lists of your profiles are merged (no server is lost) and the servers of the official Minecraft launcher are imported. Can be turned off in Settings → Behavior
- Redesigned version card on the home page: the cloud floats on its own (no more glowing circle or shadow under it), and the version, its label and the loader sit in a glass strip at the bottom
- New loading screen when the launcher starts, the same as Misty Client's in game: the night landscape scrolling by, the Misty cloud and name in pixel art, and a blue progress bar. With the Mist (light) theme it shows the same landscape by day
- New Sidebar settings (Settings → Sidebar): choose which tabs show in the sidebar (Skin, Profiles, Release notes, Texture packs, Friends, game folder). Hidden tabs leave their place to the others
- Redesigned release notes: the versions are listed on the left with their date and a Latest or Beta badge, and the notes of the chosen version show on the right, grouped by Launcher and Misty Client, with NEW and FIX labels and a link to the release on GitHub
- Fixed: with a profile selected, the home page stayed on "Loading versions…" after the launcher started
- Redesigned settings: each category opens with its own header, and the settings are grouped in cards with room around them, so no button or switch sits against an edge anymore
- New language: Portuguese (Portugal), with its flag in the language selector
- Fixed: Misty pinned to the taskbar (or its Start menu and desktop shortcuts) no longer stops working after an update. Shortcuts now point to a launcher whose path never changes (`Misty.exe` next to the version folders), which starts the newest version. Existing shortcuts and pins are redirected to it automatically, and the Misty window and its pin share one taskbar button

### Misty Client (in game)
- New Reach module: the distance of the last player you hit, in blocks (from your eyes to their hitbox). The farther the hit, the redder (green up to 2 blocks, yellow around 2.6, red at 3); the colour can be turned off in the module settings
- The ping module changes colour with your ping: green under 80 ms, yellow under 150 ms, orange under 250 ms, red above. The colour can be turned off
- The ping module shows your ping right away when you join a server and refreshes it every second: Misty now measures it itself instead of waiting for the player list, which the server only updates every few seconds
- New Combo module: the players you hit in a row without being hit back. Made for combat since 1.9: only hits that land count (the target gets hurt), not clicks spammed during the cooldown or on an invulnerable player. Being hit, or 3 seconds without landing a hit, ends the combo
- HUD modules with their own settings have a gear before their switch in the Misty settings: click it to set the module up (for example, turn off the colour of the ping)
- New Hitboxes option (Misty settings, HUD category): shows the hitbox of the entities around you, like F3 + B, without opening F3
- Fixed: dragging the character in the Misty menu and the wardrobe turned it the wrong way
- Fixed: joining a server could fail with error 429 (too many requests at Mojang). To show your Misty badge and cosmetics, Misty Client proves your account to Mojang, the same way the game does when it joins a server, and Mojang refuses an account that does it too often. Misty Client now keeps its Misty session between games instead of proving the account at every start, waits 20 seconds after the game starts and 30 seconds after a server join begins, and no longer retries in a loop when the Misty site refuses it

## 0.3.8-beta.2
### Misty Client (in game)
- Misty Client now runs on Minecraft 1.21.11, 26.1, 26.1.1, 26.1.2, 26.2 and 26.3. Pick one in the Misty Client section of the version picker
- Modpacks that ship an older Fabric Loader no longer crash with Misty Client: it now asks only for the loader it really needs (0.17.3 on 1.21.11, 0.18.4 on 26.1 and 26.2, 0.19.3 on 26.3)
- If a part of Misty Client does not fit a Minecraft version, only that feature is turned off: the game no longer crashes
- Old Misty Client files are cleaned up: at each start, Misty looks at every Misty Client jar in the profile's mods, keeps only the newest one for this Minecraft version and deletes the others
- New Music module: the song playing on your computer (Spotify, Deezer, a browser…) with its cover, artist and progress, read from Windows by the launcher. Nothing to sign in to
- New cosmetics: Cloudly, a little cloud floating over your head (worn with a cape), and the early player cape, kept for the first Misty players. Equip them in the Skin tab or the in-game wardrobe; the other Misty players see them
- The Misty cloud now shows in the name above the Misty players' head, and a new option shows your own name tag in third person (F5)
- Zoom: new "Reset zoom" option (on by default): the mouse wheel only changes the zoom while the key is held, the next zoom starts from your level again
- Fixed: the modules of the HUD editor could not be moved on Minecraft 26.3 (a left click was taken for a right click), and the CPS module counted the left clicks as right clicks

### Profiles
- New "Export the modpack" button on a profile: saves it as a `.mrpack` file (mods, configs, resource packs, shaders and options, without the worlds) that Misty, the Modrinth App and Prism can import. Misty Client goes with it when its box is ticked
- New "Misty Client" box next to Content and Discover on Fabric profiles of a Minecraft version Misty Client runs on: ticked, the mod is added to the profile; unticked, it is removed. Each profile keeps its own choice
- The "Misty Client" setting moved to Settings › Troubleshooting: it now chooses whether the box is ticked by default

### Home and versions
- The Misty Client versions of the version picker wrap onto several lines instead of pushing the panel off the window

### Skin studio
- The 3D preview draws about four times faster

### Discord
- The small image of the Discord status is now your player head

## 0.3.8-beta.1
### Misty Client (in game)
- New Zoom module: hold C to zoom in, the mouse wheel zooms more or less, the mouse turns slower while zoomed. Smooth by default, the key can be changed in Options › Controls
- New Fullbright module: everything is lit, caves, nights and the Nether. Turn it on in the Misty settings, or give it a key in Options › Controls
- The Singleplayer, Multiplayer and Options menus (and every menu outside a world) now have the Misty landscape behind them, with Misty buttons, sliders, text fields and tabs
- Misty settings redone: categories on the left (General, HUD, Zoom, Fullbright, Cosmetics) and a live animated preview of each one
- The Misty settings can also be opened from the classic Minecraft menu (top right), so the Misty menu can be turned back on
- HUD editor: dragging a module now follows the mouse
- New main menu: the launcher's night landscape scrolling behind (stars, moon, mountains, mist), the entries along a glowing path with custom blue icons, and you in 3D on the right with what you wear, your head following the mouse
- New wardrobe, laid out like Essential's: categories on the left (capes now; hats, wings, pets and emotes coming), the cosmetics in the middle, your outfit in 3D on the right. A cosmetic equipped in game is kept by the launcher
- New Misty loading screen instead of Mojang's red one: the same landscape, the Misty cloud and a blue progress bar
- New "Misty" versions at the top of the version picker, above Horizon: pick Misty 26.3 and play, Misty starts Fabric with Misty Client for you (even when the in-game setting is off). The choice is kept for the next start
- New: Misty now comes into the game, like PvP clients. Misty Client is a mod built into the launcher, installed on its own in **Fabric 26.3** profiles (nothing to download, switchable in Settings)
- Misty main menu instead of the Minecraft one: the cloud, your account, and the same entries as before
- HUD: FPS, ping, clicks per second, coordinates, keystrokes, armor and clock. Press Right Shift in game to arrange it: drag a module to move it, click to turn it on or off, scroll to resize
- The Misty cloud shows in front of the Misty players, in the player list (Tab) and in the chat
- The Misty cape and elytra of the Skin tab are now really equipped: you wear them in game, the other Misty players see them, and your choice is kept. Equip one while the game runs and it appears right away
- To recognize each other, the game proves your Microsoft account to the Misty site the way it does to a server (your Minecraft token never leaves the game) and tells it you are online and what you wear. "Misty network" in the in-game Misty settings turns this off

### Modpacks
- Modpacks can now be updated: a new button on a modpack profile lists its versions (the installed one is marked) and switches to the one you pick. The modpack's mods and configs are replaced, your worlds, options and the mods you added yourself are kept
- An imported modpack file (`.mrpack` or CurseForge `.zip`) is recognized on Modrinth or CurseForge when it is published there, so it can be updated too
- Files that CurseForge authors don't allow apps to download are now looked for on Modrinth (the exact same file first, then the project with the same name for your Minecraft version and loader). Only those that can't be found are left to download by hand

### Profiles
- The filters of a profile's content (All, Mods, Resource packs, Datapacks, Shaders) now show how many items each one holds, for example "Mods (12)"
### Home and versions
- New "Cancel" button while a version downloads, for when the wrong version was started by mistake
- Version picker: turning off every version type no longer empties the list (the last type stays on), and the "no version" message no longer covers Horizon

### Skin studio
- Opening a skin file picks the right arm model on its own: slim (Alex) or wide (Steve)

### Online players
- Misty counts the players online: while it is open, it sends an anonymous signal every minute (a random id drawn at each start, and whether a game is running). The total is public at `misty-launcher.online/api/online`

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