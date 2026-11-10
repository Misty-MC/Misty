package online.mistylauncher.client.menu;

import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.util.Util;

/**
 * The Misty loading screen: the launcher's night landscape scrolling to the right (as on the
 * main menu), the Misty cloud and name, and a blue progress bar.
 *
 * <p>Everything is drawn with plain rectangles from {@link SceneData}: while the game loads its
 * resources, the textures of the mod (and even the font) cannot be used yet.
 */
public final class MistyLoading {
	private static final float SCENE_W = 1600.0F;
	private static final float SCENE_H = 440.0F;
	private static final int COLUMNS = 200;
	private static final int CELL = 8;
	private static final float[] RIDGE_SPEED = {4.0F, 7.0F, 11.0F, 17.0F};
	private static final int[] RIDGE_COLOR = {0x24345A, 0x1A2748, 0x121C36, 0x0D1528};

	/** Per ridge, per column: the filled rows as {from, to, from, to…}. */
	private static final int[][][] RIDGES = new int[SceneData.RIDGES.length][][];
	private static final int[][] STARS = points(SceneData.STARS);
	private static final int[][] TWINKLE = points(SceneData.TWINKLE);

	/** The Misty cloud with its two lines, as on the logo (20 × 13). */
	private static final String[] CLOUD = {
		".......#####........",
		".....#########......",
		"....###########.....",
		"....############....",
		"..###############...",
		".#################..",
		"####################",
		"####################",
		"####################",
		".##################.",
		"....................",
		"..########..######..",
		"....................",
	};

	/** "MISTY" in a 5 × 7 pixel font (the game's font is not loaded yet). */
	private static final String[] LETTERS = {
		"10001" + "11011" + "10101" + "10001" + "10001" + "10001" + "10001",
		"11111" + "00100" + "00100" + "00100" + "00100" + "00100" + "11111",
		"01111" + "10000" + "10000" + "01110" + "00001" + "00001" + "11110",
		"11111" + "00100" + "00100" + "00100" + "00100" + "00100" + "00100",
		"10001" + "10001" + "01010" + "00100" + "00100" + "00100" + "00100",
	};

	static {
		for (int r = 0; r < SceneData.RIDGES.length; r++) {
			String[] columns = SceneData.RIDGES[r].split(",", -1);
			RIDGES[r] = new int[columns.length][];
			for (int c = 0; c < columns.length; c++) {
				String column = columns[c];
				int[] rows = new int[column.length()];
				for (int i = 0; i < column.length(); i++) {
					rows[i] = column.charAt(i) - 'A';
				}
				RIDGES[r][c] = rows;
			}
		}
	}

	private MistyLoading() {
	}

	private static int[][] points(final String data) {
		if (data.isBlank()) {
			return new int[0][];
		}
		String[] parts = data.split(" ");
		int[][] result = new int[parts.length][];
		for (int i = 0; i < parts.length; i++) {
			int dot = parts[i].indexOf('.');
			result[i] = new int[] {Integer.parseInt(parts[i].substring(0, dot)), Integer.parseInt(parts[i].substring(dot + 1))};
		}
		return result;
	}

	/**
	 * Draws the whole loading screen, {@code alpha} being the game's own fade in or out, and
	 * {@code progress} the loading progress (0..1).
	 */
	public static void draw(final GuiGraphicsExtractor graphics, final int width, final int height, final float alpha, final float progress) {
		if (alpha <= 0.0F) {
			return;
		}
		float seconds = (Util.getMillis() % 3_600_000L) / 1000.0F;
		float scale = Math.max(width / SCENE_W, height * 0.74F / SCENE_H);
		float top = height - SCENE_H * scale;
		float left = (width - SCENE_W * scale) / 2.0F;

		graphics.fillGradient(0, 0, width, height, color(0x0A0F1D, alpha), color(0x17264D, alpha));

		// stars (2 scene pixels each): the twinkling ones fade in and out
		float starPixel = 2.0F * scale;
		float starShift = seconds * 1.5F * scale;
		float twinkle = 0.25F + 0.75F * (0.5F + 0.5F * (float)Math.sin(seconds / 8.0F * Math.PI * 2.0));
		stars(graphics, STARS, starPixel, top, starShift, width, color(0xDBE7FF, alpha * 0.55F));
		stars(graphics, TWINKLE, starPixel, top, starShift, width, color(0xFFFFFF, alpha * twinkle));

		// ridges, far to near, scrolling right (the far ones slower), each looping on its mirror
		float cell = CELL * scale;
		float tile = 2 * COLUMNS * cell;
		for (int r = 0; r < RIDGES.length; r++) {
			int fill = color(RIDGE_COLOR[r], alpha);
			float shift = (seconds * RIDGE_SPEED[r] * scale) % tile;
			for (float tileX = shift - tile; tileX < width; tileX += tile) {
				for (int c = 0; c < 2 * COLUMNS; c++) {
					float x = tileX + c * cell;
					if (x + cell < 0 || x > width) {
						continue;
					}
					int[] rows = RIDGES[r][c < COLUMNS ? c : 2 * COLUMNS - 1 - c];
					int x0 = Math.round(x);
					int x1 = Math.round(x + cell);
					for (int i = 0; i + 1 < rows.length; i += 2) {
						graphics.fill(x0, Math.round(top + rows[i] * cell), x1, Math.round(top + rows[i + 1] * cell), fill);
					}
				}
			}
		}
		int fade = Math.round(96 * scale);
		graphics.fillGradient(0, height - fade, width, height, color(0x070B16, 0.0F), color(0x070B16, alpha));

		// the Misty cloud and name, in the middle
		int pixel = Math.max(1, Math.round(Math.min(width, height) / 160.0F));
		int cloudPixel = pixel * 2;
		int cloudW = 20 * cloudPixel;
		int cloudH = CLOUD.length * cloudPixel;
		int nameW = (5 * 5 + 4 * 2) * pixel;
		int blockH = cloudH + 6 * pixel + 7 * pixel;
		int cx = width / 2;
		int y0 = (height - blockH) / 2 - 12 * pixel;
		int white = color(0xFFFFFF, alpha);
		int line = color(0xDCE6FF, alpha);
		for (int row = 0; row < CLOUD.length; row++) {
			for (int col = 0; col < 20; col++) {
				if (CLOUD[row].charAt(col) == '#') {
					int x = cx - cloudW / 2 + col * cloudPixel;
					int y = y0 + row * cloudPixel;
					graphics.fill(x, y, x + cloudPixel, y + cloudPixel, row > 10 ? line : white);
				}
			}
		}
		int letterY = y0 + cloudH + 6 * pixel;
		int letterX = cx - nameW / 2;
		int slate = color(0x3C5486, alpha);
		for (int l = 0; l < LETTERS.length; l++) {
			for (int i = 0; i < 35; i++) {
				if (LETTERS[l].charAt(i) == '1') {
					int x = letterX + l * 7 * pixel + (i % 5) * pixel;
					int y = letterY + (i / 5) * pixel;
					graphics.fill(x + pixel, y + pixel, x + 2 * pixel, y + 2 * pixel, slate);
					graphics.fill(x, y, x + pixel, y + pixel, white);
				}
			}
		}

		// progress bar
		int barW = Math.min(width - 40, 60 * pixel);
		int barH = Math.max(2, 2 * pixel);
		int barX = cx - barW / 2;
		int barY = letterY + 7 * pixel + 10 * pixel;
		graphics.fill(barX - 1, barY - 1, barX + barW + 1, barY + barH + 1, color(0xFFFFFF, alpha * 0.18F));
		graphics.fill(barX, barY, barX + barW, barY + barH, color(0x0E1527, alpha));
		int done = Math.round(barW * Math.clamp(progress, 0.0F, 1.0F));
		graphics.fill(barX, barY, barX + done, barY + barH, color(0x6F9CFF, alpha));
		graphics.fill(barX, barY, barX + done, barY + Math.max(1, barH / 2), color(0x9CC0FF, alpha));
	}

	private static void stars(final GuiGraphicsExtractor graphics, final int[][] points, final float pixel, final float top, final float shift, final int width, final int color) {
		// the star field is 800 pixels wide and loops on its mirror (1600)
		float tile = 1600 * pixel;
		float offset = shift % tile;
		for (float tileX = offset - tile; tileX < width; tileX += tile) {
			for (int[] p : points) {
				for (int mirror = 0; mirror < 2; mirror++) {
					int px = mirror == 0 ? p[0] : 1599 - p[0];
					float x = tileX + px * pixel;
					if (x < -pixel || x > width) {
						continue;
					}
					float y = top + p[1] * pixel;
					graphics.fill(Math.round(x), Math.round(y), Math.round(x + pixel), Math.round(y + pixel), color);
				}
			}
		}
	}

	private static int color(final int rgb, final float alpha) {
		return Math.round(255 * Math.clamp(alpha, 0.0F, 1.0F)) << 24 | rgb & 0xFFFFFF;
	}
}
