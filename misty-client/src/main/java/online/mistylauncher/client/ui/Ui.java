package online.mistylauncher.client.ui;

import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.renderer.RenderPipelines;
import net.minecraft.network.chat.Component;
import net.minecraft.resources.Identifier;
import online.mistylauncher.client.MistyClient;

/** Colours and drawing helpers shared by the Misty screens: the launcher's "Night" theme. */
public final class Ui {
	public static final int INK = 0xFF0A0F1D;
	public static final int GLASS = 0xB8111A30;
	public static final int GLASS_HOVER = 0xF01C2A50;
	public static final int SURFACE = 0xF50E1527;
	public static final int BORDER = 0x1FFFFFFF;
	public static final int LINE = 0x14FFFFFF;
	public static final int ACCENT = 0xFF9CC0FF;
	public static final int FILL = 0xFF6F9CFF;
	public static final int FILL_HOVER = 0xFF8AAEFF;
	public static final int TEXT = 0xFFEEF3FF;
	public static final int MUTED = 0xFFA3B0CB;
	public static final int DIM = 0xFF6F7C99;
	public static final int SLATE = 0xFF3C5486;
	public static final int TRACK = 0x26FFFFFF;

	private static final Identifier LOGO = MistyClient.id("textures/gui/logo.png");
	private static final int LOGO_SIZE = 128;

	private Ui() {
	}

	/** Same colour with another opacity (0..1). */
	public static int alpha(final int color, final float alpha) {
		int a = Math.round((color >>> 24) * Math.clamp(alpha, 0.0F, 1.0F));
		return a << 24 | color & 0xFFFFFF;
	}

	/** Rectangle with its four corner pixels cut: the pixel-art rounding of the launcher. */
	public static void panel(final GuiGraphicsExtractor graphics, final int x, final int y, final int width, final int height, final int color) {
		if (width < 3 || height < 3) {
			graphics.fill(x, y, x + width, y + height, color);
			return;
		}
		graphics.fill(x + 1, y, x + width - 1, y + 1, color);
		graphics.fill(x, y + 1, x + width, y + height - 1, color);
		graphics.fill(x + 1, y + height - 1, x + width - 1, y + height, color);
	}

	/** One-pixel border following the cut corners of {@link #panel}. */
	public static void border(final GuiGraphicsExtractor graphics, final int x, final int y, final int width, final int height, final int color) {
		graphics.fill(x + 1, y, x + width - 1, y + 1, color);
		graphics.fill(x + 1, y + height - 1, x + width - 1, y + height, color);
		graphics.fill(x, y + 1, x + 1, y + height - 1, color);
		graphics.fill(x + width - 1, y + 1, x + width, y + height - 1, color);
	}

	/** Panel and its border. */
	public static void card(final GuiGraphicsExtractor graphics, final int x, final int y, final int width, final int height, final int color, final int borderColor) {
		panel(graphics, x, y, width, height, color);
		border(graphics, x, y, width, height, borderColor);
	}

	/** The Misty cloud. */
	public static void logo(final GuiGraphicsExtractor graphics, final int x, final int y, final int size) {
		graphics.blit(RenderPipelines.GUI_TEXTURED, LOGO, x, y, 0.0F, 0.0F, size, size, LOGO_SIZE, LOGO_SIZE, LOGO_SIZE, LOGO_SIZE);
	}

	/** Text drawn bigger or smaller than the font, centred on {@code centerX}. */
	public static void centeredScaled(
		final GuiGraphicsExtractor graphics, final Font font, final Component text, final int centerX, final int y, final float scale, final int color, final boolean shadow
	) {
		graphics.pose().pushMatrix();
		graphics.pose().translate(centerX, y);
		graphics.pose().scale(scale, scale);
		graphics.text(font, text, -font.width(text) / 2, 0, color, shadow);
		graphics.pose().popMatrix();
	}

	public static void centered(final GuiGraphicsExtractor graphics, final Font font, final Component text, final int centerX, final int y, final int color) {
		graphics.text(font, text, centerX - font.width(text) / 2, y, color, false);
	}
}
