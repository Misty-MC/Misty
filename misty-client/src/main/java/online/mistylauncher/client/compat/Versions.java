package online.mistylauncher.client.compat;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.screens.Overlay;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.options.OptionsScreen;
import net.minecraft.network.chat.Component;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.entity.player.Player;
import org.jspecify.annotations.Nullable;

/**
 * What changes from one Minecraft version to the next, in one place. The rest of the mod calls
 * these instead of the game, so a new version is mostly a matter of this class.
 *
 * <p>The {@code //?} comments are Stonecutter conditions: each jar is built with the code of its
 * Minecraft version (see stonecutter.gradle.kts, which also renames what only changed name).
 */
public final class Versions {
	private Versions() {
	}

	/** Opens a screen ({@code null}: back to the game). */
	public static void setScreen(final @Nullable Screen screen) {
		//? if >=26.2 {
		Minecraft.getInstance().gui.setScreen(screen);
		//?} else {
		/*Minecraft.getInstance().setScreen(screen);
		*///?}
	}

	/** The screen open now ({@code null}: playing). */
	public static @Nullable Screen screen() {
		//? if >=26.2 {
		return Minecraft.getInstance().gui.screen();
		//?} else {
		/*return Minecraft.getInstance().screen;
		*///?}
	}

	/** Sets the overlay over the screens (the loading screen). */
	public static void setOverlay(final @Nullable Overlay overlay) {
		//? if >=26.2 {
		Minecraft.getInstance().gui.setOverlay(overlay);
		//?} else {
		/*Minecraft.getInstance().setOverlay(overlay);
		*///?}
	}

	/** The subtitles drawn above the screens. */
	public static void extractDeferredSubtitles() {
		//? if >=26.2 {
		Minecraft.getInstance().gui.hud.extractDeferredSubtitles();
		//?} else if >=26.1 {
		/*Minecraft.getInstance().gui.extractDeferredSubtitles();
		*///?} else if >=1.21.9 {
		/*Minecraft.getInstance().gui.renderDeferredSubtitles();
		*///?}
		// before 1.21.9 the subtitles are drawn with the rest of the interface
	}

	/** The interface is hidden (F1). */
	public static boolean guiHidden() {
		//? if >=26.2 {
		return Minecraft.getInstance().gui.hud.isHidden();
		//?} else {
		/*return Minecraft.getInstance().options.hideGui;
		*///?}
	}

	/** The player entity type. */
	public static EntityType<Player> playerType() {
		//? if >=26.2 {
		return net.minecraft.world.entity.EntityTypes.PLAYER;
		//?} else {
		/*return EntityType.PLAYER;
		*///?}
	}

	/** The game's Options screen, opened from a menu (not from a world). */
	public static Screen options(final Screen parent) {
		Minecraft minecraft = Minecraft.getInstance();
		//? if >=26.3 {
		return new OptionsScreen(parent, minecraft.options);
		//?} else if >=26.1 {
		/*return new OptionsScreen(parent, minecraft.options, false);
		*///?} else {
		/*return new OptionsScreen(parent, minecraft.options);
		*///?}
	}

	/** A player's id (a record since 1.21.9, a class with getters before). */
	public static java.util.UUID id(final com.mojang.authlib.GameProfile profile) {
		//? if >=1.21.9 {
		return profile.id();
		//?} else {
		/*return profile.getId();
		*///?}
	}

	/** A player's name. */
	public static String name(final com.mojang.authlib.GameProfile profile) {
		//? if >=1.21.9 {
		return profile.name();
		//?} else {
		/*return profile.getName();
		*///?}
	}

	/** A short message above the hotbar. */
	public static void overlayMessage(final Component message) {
		Minecraft minecraft = Minecraft.getInstance();
		if (minecraft.player == null) {
			return;
		}
		//? if >=26.1 {
		minecraft.player.sendOverlayMessage(message);
		//?} else {
		/*minecraft.player.displayClientMessage(message, true);
		*///?}
	}
}
