package online.mistylauncher.client.hud.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.player.LocalPlayer;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.hud.HudModule;
import online.mistylauncher.client.ui.Ui;

/** Block position, and the direction the player faces. */
public final class CoordinatesModule extends HudModule {
	private static final String[] AXES = { "X", "Y", "Z" };
	private static final String[] FACING = { "S", "SW", "W", "NW", "N", "NE", "E", "SE" };
	private static final int ROW = 10;
	private static final int PADDING = 5;

	public CoordinatesModule() {
		super("coords", new MistyConfig.Module(true, 0, 4, 36));
	}

	private static int[] position(final Minecraft minecraft) {
		LocalPlayer player = minecraft.player;
		return player == null ? new int[] { 128, 64, -256 } : new int[] { player.getBlockX(), player.getBlockY(), player.getBlockZ() };
	}

	private static String facing(final Minecraft minecraft) {
		LocalPlayer player = minecraft.player;
		// yaw 0 looks south and grows clockwise: eight slices of 45°
		return player == null ? "N" : FACING[Math.floorMod(Math.round(player.getYRot() / 45.0F), 8)];
	}

	@Override
	public int width(final Minecraft minecraft, final boolean preview) {
		Font font = minecraft.font;
		int widest = 0;
		for (int value : position(minecraft)) {
			widest = Math.max(widest, font.width(Integer.toString(value)));
		}
		return Math.max(72, PADDING + 10 + widest + 8 + font.width("NW") + PADDING);
	}

	@Override
	public int height(final Minecraft minecraft, final boolean preview) {
		return 3 * ROW + 2 * PADDING - 2;
	}

	@Override
	public void draw(final GuiGraphicsExtractor graphics, final Minecraft minecraft, final boolean preview) {
		Font font = minecraft.font;
		int width = this.width(minecraft, preview);
		background(graphics, width, this.height(minecraft, preview));
		int[] position = position(minecraft);
		for (int i = 0; i < 3; i++) {
			int y = PADDING + i * ROW;
			graphics.text(font, AXES[i], PADDING, y, Ui.ACCENT, false);
			graphics.text(font, Integer.toString(position[i]), PADDING + 10, y, Ui.TEXT, false);
		}
		String facing = facing(minecraft);
		graphics.text(font, facing, width - PADDING - font.width(facing), PADDING + ROW, Ui.MUTED, false);
	}
}
