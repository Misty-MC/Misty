# Misty Client

The in-game side of the Misty launcher: a Fabric mod for **Minecraft 26.1 to 26.3**, built into `Misty.exe` and installed by the launcher in the profiles that support it (Fabric, same Minecraft version).

- **Misty main menu**: replaces the Minecraft title screen (can be turned off)
- **HUD**: FPS, ping (coloured by latency), CPS, reach, coordinates, keystrokes, armor, clock. Right Shift opens the editor: drag to move, click to turn on or off, scroll to resize, right-click to reset
- **Misty badge**: the Misty cloud in front of the Misty players, in the player list and in the chat
- **Misty cosmetics**: the Misty cape and elytra equipped in the launcher, on you and on the other Misty players

Settings: the cloud button of the main menu, or "Misty settings" in the pause menu. They are kept in `config/misty-client.json` of the profile.

## Build

Requirements: JDK 25.

```bash
./gradlew build      # build/libs/misty-client-<version>+mc<minecraft>.jar
./gradlew runClient  # starts the game with the mod
```

The launcher embeds the jar found in `build/libs` when it is compiled (`misty-rust/build.rs`); `installer/build.ps1` builds the mod first.

## How it talks to the launcher and to the site

- The launcher starts the game with `-Dmisty.launcher.dir=<Misty data folder>` and writes `client.json` there (`{"cosmetic": "misty_cape"}`). The mod reads it again when it changes, so a cape equipped in the launcher shows up without restarting the game.
- **Misty network** (`misty-launcher.online/api/client`, can be turned off in the settings): the mod proves the Minecraft account the way a server does (`joinServer` at Mojang, checked by the site with `hasJoined`: the Minecraft token never leaves the game), then sends a sign of life every minute with the cosmetic worn, and asks which players of the current server are online with Misty. Offline accounts cannot be proven: they keep the menu, the HUD and their own cape, but are not shown to others.

## Layout

```
src/main/java/online/mistylauncher/client/
├── MistyClient.java   entry point
├── config/            settings, link with the launcher
├── net/               Misty network (who plays with Misty, what they wear)
├── cosmetics/         Misty cape and elytra
├── social/            Misty badge (player list, chat)
├── hud/               HUD modules, their placement and the editor
├── menu/              Misty main menu
├── ui/                colours, buttons, settings screen
└── mixin/             hooks into the game
src/main/resources/assets/misty/   textures, badge font, translations (en, fr, es, it, ru)
```

`textures/cape/misty.png` is the texture the launcher draws in its Skin tab (`skins::texture_misty`); a launcher test checks they stay identical.
