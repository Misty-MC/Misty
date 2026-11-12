package online.mistylauncher.client.hud;

import java.util.List;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.network.chat.Component;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.ui.Ui;

/** One piece of the HUD (FPS, keystrokes…): it draws itself from (0, 0), the manager places it. */
public abstract class HudModule {
	/** Background of every module: the launcher's night sky, see-through. */
	protected static final int BACKGROUND = 0xB00A0F1D;

	private final String id;
	private final MistyConfig.Module defaults;
	private MistyConfig.Module state;

	protected HudModule(final String id, final MistyConfig.Module defaults) {
		this.id = id;
		this.defaults = defaults;
		this.state = defaults.copy();
	}

	public final String id() {
		return this.id;
	}

	public final Component name() {
		return Component.translatable("misty.module." + this.id);
	}

	public final MistyConfig.Module state() {
		return this.state;
	}

	/** Uses the saved state of the settings, or puts the default one there. */
	final void bind(final MistyConfig config) {
		this.state = config.modules.computeIfAbsent(this.id, id -> this.defaults.copy());
	}

	final void reset(final MistyConfig config) {
		this.state = this.defaults.copy();
		config.modules.put(this.id, this.state);
	}

	/** One setting of the module, shown behind its gear in the settings screen. */
	public record Setting(String key, boolean byDefault) {
		public Component name(final HudModule module) {
			return Component.translatable("misty.module." + module.id() + "." + this.key);
		}

		public Component tip(final HudModule module) {
			return Component.translatable("misty.module." + module.id() + "." + this.key + ".tip");
		}
	}

	/** The module's own settings (none: no gear). */
	public List<Setting> settings() {
		return List.of();
	}

	/** Current value of one of the module's settings. */
	public final boolean setting(final String key) {
		for (Setting setting : this.settings()) {
			if (setting.key().equals(key)) {
				return this.state.option(key, setting.byDefault());
			}
		}
		return false;
	}

	/** False when the module has nothing to show right now (ping in a solo world…). */
	public boolean available(final Minecraft minecraft) {
		return true;
	}

	public abstract int width(Minecraft minecraft, boolean preview);

	public abstract int height(Minecraft minecraft, boolean preview);

	/**
	 * Draws the module with its top-left corner at (0, 0). {@code preview}: in the editor, where
	 * sample values stand in for what is missing (no world, no armor on…).
	 */
	public abstract void draw(GuiGraphicsExtractor graphics, Minecraft minecraft, boolean preview);

	protected static void background(final GuiGraphicsExtractor graphics, final int width, final int height) {
		Ui.panel(graphics, 0, 0, width, height, BACKGROUND);
	}
}
