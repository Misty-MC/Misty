package online.mistylauncher.client.features;

import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.minecraft.util.Util;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.config.MistyConfig;
import org.jspecify.annotations.Nullable;

/**
 * Zoom, like OptiFine's: hold the zoom key (C by default) to magnify, the mouse wheel changes how
 * much, the mouse turns slower while zoomed. It glides in and out unless turned off.
 */
public final class Zoom {
	public static final float MIN = 1.5F;
	public static final float MAX = 10.0F;

	private static @Nullable KeyMapping key;
	/** Current magnification (1 = no zoom), eased towards the wanted one. */
	private static float current = 1.0F;
	private static long lastFrame = Util.getMillis();
	/** The level changed with the wheel: saved when the key is released. */
	private static boolean changed;

	private Zoom() {
	}

	public static void bind(final KeyMapping zoomKey) {
		key = zoomKey;
	}

	/** The zoom key (shown in the settings). */
	public static @Nullable KeyMapping key() {
		return key;
	}

	/** Zooming now: the key held in game (not in a menu). */
	public static boolean held() {
		MistyConfig config = MistyClient.config();
		return config.zoom && key != null && key.isDown() && Minecraft.getInstance().gui.screen() == null;
	}

	/** Magnification for this frame (called with the field of view). */
	public static float factor() {
		MistyConfig config = MistyClient.config();
		long now = Util.getMillis();
		// the game's clock can change its origin after start-up: a negative or huge gap is ignored
		float seconds = Math.clamp((now - lastFrame) / 1000.0F, 0.0F, 0.1F);
		lastFrame = now;
		float wanted = held() ? Math.clamp(config.zoomLevel, MIN, MAX) : 1.0F;
		if (config.zoomSmooth) {
			// eased: about a fifth of a second to get there
			current += (wanted - current) * (1.0F - (float)Math.exp(-seconds * 14.0F));
			if (Math.abs(current - wanted) < 0.002F) {
				current = wanted;
			}
		} else {
			current = wanted;
		}
		if (!Float.isFinite(current) || current < 1.0F) {
			current = 1.0F;
		}
		return current;
	}

	/** Mouse sensitivity factor: slower when zoomed, so aiming stays precise. */
	public static double sensitivity() {
		return current > 1.01F ? 1.0 / Math.sqrt(current) : 1.0;
	}

	/** The mouse wheel while zooming: more or less zoom. True if the wheel was used for it. */
	public static boolean scroll(final double amount) {
		MistyConfig config = MistyClient.config();
		if (!held() || !config.zoomScroll || amount == 0.0) {
			return false;
		}
		float step = amount > 0 ? 1.15F : 1.0F / 1.15F;
		config.zoomLevel = Math.clamp(config.zoomLevel * step, MIN, MAX);
		changed = true;
		return true;
	}

	/** Each tick: the wheel's change is saved once the key is released. */
	public static void tick() {
		if (changed && !held()) {
			changed = false;
			MistyClient.config().save();
		}
	}
}
