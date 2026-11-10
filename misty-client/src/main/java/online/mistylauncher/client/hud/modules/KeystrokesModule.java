package online.mistylauncher.client.hud.modules;

import java.util.Locale;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.minecraft.client.Options;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.hud.HudModule;
import online.mistylauncher.client.ui.Ui;

/** The movement keys, the two mouse buttons and the jump key, lit while they are held. */
public final class KeystrokesModule extends HudModule {
	private static final int KEY = 18;
	private static final int GAP = 2;
	private static final int WIDTH = 3 * KEY + 2 * GAP;
	private static final int MOUSE_HEIGHT = 16;
	private static final int JUMP_HEIGHT = 10;

	public KeystrokesModule() {
		super("keystrokes", new MistyConfig.Module(true, 3, 4, 0));
	}

	@Override
	public int width(final Minecraft minecraft, final boolean preview) {
		return WIDTH;
	}

	@Override
	public int height(final Minecraft minecraft, final boolean preview) {
		return 2 * (KEY + GAP) + MOUSE_HEIGHT + GAP + JUMP_HEIGHT;
	}

	@Override
	public void draw(final GuiGraphicsExtractor graphics, final Minecraft minecraft, final boolean preview) {
		Options options = minecraft.options;
		Font font = minecraft.font;
		key(graphics, font, KEY + GAP, 0, KEY, KEY, label(options.keyUp), options.keyUp.isDown());
		int y = KEY + GAP;
		key(graphics, font, 0, y, KEY, KEY, label(options.keyLeft), options.keyLeft.isDown());
		key(graphics, font, KEY + GAP, y, KEY, KEY, label(options.keyDown), options.keyDown.isDown());
		key(graphics, font, 2 * (KEY + GAP), y, KEY, KEY, label(options.keyRight), options.keyRight.isDown());
		y += KEY + GAP;
		int half = (WIDTH - GAP) / 2;
		key(graphics, font, 0, y, half, MOUSE_HEIGHT, "LMB", options.keyAttack.isDown());
		key(graphics, font, half + GAP, y, half, MOUSE_HEIGHT, "RMB", options.keyUse.isDown());
		y += MOUSE_HEIGHT + GAP;
		boolean jump = options.keyJump.isDown();
		Ui.panel(graphics, 0, y, WIDTH, JUMP_HEIGHT, jump ? Ui.FILL : BACKGROUND);
		graphics.fill(WIDTH / 2 - 10, y + JUMP_HEIGHT / 2 - 1, WIDTH / 2 + 10, y + JUMP_HEIGHT / 2 + 1, jump ? Ui.INK : Ui.TEXT);
	}

	private static void key(
		final GuiGraphicsExtractor graphics, final Font font, final int x, final int y, final int width, final int height, final String label, final boolean down
	) {
		Ui.panel(graphics, x, y, width, height, down ? Ui.FILL : BACKGROUND);
		graphics.text(font, label, x + (width - font.width(label) + 1) / 2, y + (height - 7) / 2, down ? Ui.INK : Ui.TEXT, false);
	}

	/** Name of the key bound to a movement: "W", or the start of a longer name ("Up Arrow" gives "UP"). */
	private static String label(final KeyMapping key) {
		String name = key.getTranslatedKeyMessage().getString().toUpperCase(Locale.ROOT);
		int space = name.indexOf(' ');
		if (space > 0) {
			name = name.substring(0, space);
		}
		return name.length() > 3 ? name.substring(0, 3) : name;
	}
}
