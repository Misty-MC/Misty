package online.mistylauncher.client.hud.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.renderer.RenderPipelines;
import net.minecraft.network.chat.Component;
import net.minecraft.util.Util;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.features.Music;
import online.mistylauncher.client.hud.HudModule;
import online.mistylauncher.client.ui.Ui;

/**
 * The song playing on the computer: cover, title, artist, progress. Read by the launcher from
 * Windows (Spotify, Deezer, a browser…), nothing to sign in to.
 */
public final class MusicModule extends HudModule {
	private static final int WIDTH = 168;
	private static final int HEIGHT = 40;
	private static final int COVER = 32;
	/** A title too long for the module scrolls, this many pixels per second, with a pause at each end. */
	private static final float SCROLL_SPEED = 22.0F;
	private static final float SCROLL_PAUSE = 1.6F;

	/** Shown in the editor when nothing plays. */
	private static final Music.Song SAMPLE = new Music.Song("Night Sky", "Misty", "Spotify", true, 74_000, 196_000, 0, 0);

	public MusicModule() {
		super("music", new MistyConfig.Module(true, 5, -4, 0));
	}

	@Override
	public boolean available(final Minecraft minecraft) {
		return Music.song() != null;
	}

	@Override
	public int width(final Minecraft minecraft, final boolean preview) {
		return WIDTH;
	}

	@Override
	public int height(final Minecraft minecraft, final boolean preview) {
		return HEIGHT;
	}

	@Override
	public void draw(final GuiGraphicsExtractor graphics, final Minecraft minecraft, final boolean preview) {
		Music.Song song = Music.song();
		boolean sample = song == null;
		if (sample) {
			if (!preview) {
				return;
			}
			song = SAMPLE;
		}
		Font font = minecraft.font;
		background(graphics, WIDTH, HEIGHT);

		// cover (or a note on the night sky)
		int cx = 4;
		int cy = 4;
		if (!sample && Music.hasCover()) {
			graphics.blit(RenderPipelines.GUI_TEXTURED, Music.COVER, cx, cy, 0.0F, 0.0F, COVER, COVER, COVER, COVER, 128, 128);
		} else {
			graphics.fillGradient(cx, cy, cx + COVER, cy + COVER, 0xFF1D3F86, 0xFF0E1A38);
			Ui.centered(graphics, font, Component.literal("♪"), cx + COVER / 2, cy + 12, Ui.ACCENT);
		}

		// title (scrolling when too long), artist and app
		int tx = cx + COVER + 6;
		int room = WIDTH - tx - 5;
		this.scrolling(graphics, font, song.title(), tx, 5, room, Ui.TEXT);
		String under = song.artist().isEmpty() ? song.app() : song.artist();
		graphics.text(font, Component.literal(font.plainSubstrByWidth(under, room)), tx, 15, Ui.MUTED, false);

		// progress bar and time
		long position = sample ? song.position() : song.positionNow();
		int barY = 30;
		int barW = room;
		graphics.fill(tx, barY, tx + barW, barY + 2, Ui.TRACK);
		if (song.duration() > 0) {
			int done = (int)(barW * Math.min(1.0, (double)position / song.duration()));
			graphics.fill(tx, barY, tx + done, barY + 2, song.playing() ? Ui.FILL : Ui.DIM);
		}
		String time = clock(position) + (song.duration() > 0 ? " / " + clock(song.duration()) : "");
		String state = song.playing() ? "" : "❚❚ ";
		graphics.pose().pushMatrix();
		graphics.pose().translate(tx, barY + 3.5F);
		graphics.pose().scale(0.6F, 0.6F);
		graphics.text(font, Component.literal(state + time), 0, 0, Ui.DIM, false);
		graphics.pose().popMatrix();
	}

	/** Text that slides when it does not fit, pausing at both ends. */
	private void scrolling(final GuiGraphicsExtractor graphics, final Font font, final String text, final int x, final int y, final int room, final int color) {
		int width = font.width(text);
		if (width <= room) {
			graphics.text(font, Component.literal(text), x, y, color, false);
			return;
		}
		int overflow = width - room;
		float travel = overflow / SCROLL_SPEED;
		float cycle = 2 * (travel + SCROLL_PAUSE);
		float t = (Util.getMillis() % (long)(cycle * 1000)) / 1000.0F;
		float offset;
		if (t < SCROLL_PAUSE) {
			offset = 0;
		} else if (t < SCROLL_PAUSE + travel) {
			offset = (t - SCROLL_PAUSE) * SCROLL_SPEED;
		} else if (t < 2 * SCROLL_PAUSE + travel) {
			offset = overflow;
		} else {
			offset = overflow - (t - 2 * SCROLL_PAUSE - travel) * SCROLL_SPEED;
		}
		graphics.enableScissor(x, y - 1, x + room, y + 10);
		graphics.text(font, Component.literal(text), x - Math.round(offset), y, color, false);
		graphics.disableScissor();
	}

	private static String clock(final long millis) {
		long seconds = Math.max(0, millis / 1000);
		return seconds / 60 + ":" + String.format("%02d", seconds % 60);
	}
}
