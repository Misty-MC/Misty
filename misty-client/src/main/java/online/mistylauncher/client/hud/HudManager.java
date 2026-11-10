package online.mistylauncher.client.hud;

import java.util.List;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.hud.modules.ArmorModule;
import online.mistylauncher.client.hud.modules.ClockModule;
import online.mistylauncher.client.hud.modules.CoordinatesModule;
import online.mistylauncher.client.hud.modules.CpsModule;
import online.mistylauncher.client.hud.modules.FpsModule;
import online.mistylauncher.client.hud.modules.KeystrokesModule;
import online.mistylauncher.client.hud.modules.PingModule;

/** The HUD modules: where each one sits on the screen, and their drawing in game. */
public final class HudManager {
	public static final float MIN_SCALE = 0.5F;
	public static final float MAX_SCALE = 2.0F;

	private static final List<HudModule> MODULES = List.of(
		new FpsModule(), new PingModule(), new CpsModule(), new CoordinatesModule(), new KeystrokesModule(), new ArmorModule(), new ClockModule()
	);

	/** Rectangle a module covers on the screen, in GUI pixels. */
	public record Bounds(int x, int y, int width, int height) {
		public boolean contains(final double pointX, final double pointY) {
			return pointX >= this.x && pointX < this.x + this.width && pointY >= this.y && pointY < this.y + this.height;
		}
	}

	private HudManager() {
	}

	public static void init(final MistyConfig config) {
		MODULES.forEach(module -> module.bind(config));
	}

	public static List<HudModule> modules() {
		return MODULES;
	}

	public static void resetAll(final MistyConfig config) {
		MODULES.forEach(module -> module.reset(config));
	}

	/** Start of a module of {@code size} along an axis of {@code screen} pixels: 0 start, 1 centre, 2 end. */
	private static int base(final int side, final int screen, final int size) {
		return side == 0 ? 0 : side == 1 ? (screen - size) / 2 : screen - size;
	}

	public static Bounds bounds(final HudModule module, final Minecraft minecraft, final int screenWidth, final int screenHeight, final boolean preview) {
		MistyConfig.Module state = module.state();
		float scale = Math.clamp(state.scale, MIN_SCALE, MAX_SCALE);
		int width = Math.round(module.width(minecraft, preview) * scale);
		int height = Math.round(module.height(minecraft, preview) * scale);
		int anchor = Math.clamp(state.anchor, 0, 8);
		int x = base(anchor % 3, screenWidth, width) + state.x;
		int y = base(anchor / 3, screenHeight, height) + state.y;
		// always fully on screen, whatever the window size became
		return new Bounds(Math.clamp(x, 0, Math.max(0, screenWidth - width)), Math.clamp(y, 0, Math.max(0, screenHeight - height)), width, height);
	}

	/**
	 * Moves a module so its top-left corner is at (x, y). It is tied to the nearest third of the
	 * screen: a module dropped in a corner stays in that corner when the window is resized.
	 */
	public static void place(final HudModule module, final Bounds bounds, final int x, final int y, final int screenWidth, final int screenHeight) {
		MistyConfig.Module state = module.state();
		int column = third(x + bounds.width() / 2, screenWidth);
		int row = third(y + bounds.height() / 2, screenHeight);
		state.anchor = column + 3 * row;
		state.x = x - base(column, screenWidth, bounds.width());
		state.y = y - base(row, screenHeight, bounds.height());
	}

	private static int third(final int position, final int screen) {
		return position < screen / 3 ? 0 : position > screen * 2 / 3 ? 2 : 1;
	}

	public static void draw(final GuiGraphicsExtractor graphics, final Minecraft minecraft, final HudModule module, final Bounds bounds, final boolean preview) {
		float scale = Math.clamp(module.state().scale, MIN_SCALE, MAX_SCALE);
		graphics.pose().pushMatrix();
		graphics.pose().translate(bounds.x(), bounds.y());
		graphics.pose().scale(scale, scale);
		module.draw(graphics, minecraft, preview);
		graphics.pose().popMatrix();
	}

	/** In game, under the chat and the player list. */
	public static void render(final GuiGraphicsExtractor graphics, final Minecraft minecraft) {
		if (!MistyClient.config().hud || minecraft.gui.screen() instanceof HudEditorScreen || minecraft.getDebugOverlay().showDebugScreen()) {
			return;
		}
		int width = graphics.guiWidth();
		int height = graphics.guiHeight();
		for (HudModule module : MODULES) {
			if (module.state().enabled && module.available(minecraft)) {
				draw(graphics, minecraft, module, bounds(module, minecraft, width, height, false), false);
			}
		}
	}
}
