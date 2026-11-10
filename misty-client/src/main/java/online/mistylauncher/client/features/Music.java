package online.mistylauncher.client.features;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.mojang.blaze3d.platform.NativeImage;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.attribute.FileTime;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.texture.DynamicTexture;
import net.minecraft.resources.Identifier;
import online.mistylauncher.client.MistyClient;
import org.jspecify.annotations.Nullable;

/**
 * The song playing on the computer (Spotify, Deezer, a browser…). The launcher reads it from
 * Windows' media controls and writes {@code music.json} and {@code music-cover.png} in its folder
 * (misty-rust, services/musique.rs); this reads them again when they change.
 */
public final class Music {
	/** The song, as last read. Times in milliseconds; {@code at}: when the position was measured. */
	public record Song(String title, String artist, String app, boolean playing, long position, long duration, long at, long cover) {
		/** Where the song is now: the measured position, moved on by the time since (while playing). */
		public long positionNow() {
			long now = this.playing ? this.position + Math.max(0, System.currentTimeMillis() - this.at) : this.position;
			return this.duration > 0 ? Math.min(now, this.duration) : now;
		}
	}

	public static final Identifier COVER = MistyClient.id("music_cover");
	private static final int CHECK_EVERY_TICKS = 10;

	private static @Nullable Song song;
	private static @Nullable FileTime lastModified;
	private static long loadedCover = -1;
	private static boolean coverReady;
	private static int ticks;

	private Music() {
	}

	/** The song playing now, or null (nothing plays, or no launcher). */
	public static @Nullable Song song() {
		return song;
	}

	/** Whether the cover of the current song is loaded in {@link #COVER}. */
	public static boolean hasCover() {
		return coverReady && song != null && song.cover() == loadedCover;
	}

	public static void tick() {
		if (++ticks < CHECK_EVERY_TICKS) {
			return;
		}
		ticks = 0;
		String dir = System.getProperty("misty.launcher.dir");
		if (dir == null || dir.isBlank()) {
			return;
		}
		try {
			Path file = Path.of(dir, "music.json");
			if (!Files.isRegularFile(file)) {
				song = null;
				lastModified = null;
				return;
			}
			FileTime modified = Files.getLastModifiedTime(file);
			if (modified.equals(lastModified)) {
				return;
			}
			lastModified = modified;
			JsonObject json = JsonParser.parseString(Files.readString(file, StandardCharsets.UTF_8)).getAsJsonObject();
			song = new Song(
				text(json, "title"), text(json, "artist"), text(json, "app"),
				json.has("playing") && json.get("playing").getAsBoolean(),
				number(json, "position"), number(json, "duration"), number(json, "at"), number(json, "cover")
			);
			if (song.cover() != loadedCover) {
				loadCover(Path.of(dir, "music-cover.png"), song.cover());
			}
		} catch (IOException | RuntimeException e) {
			// half-written file: read again at the next check
			lastModified = null;
		}
	}

	/** Puts the cover in a texture of the game (replacing the previous song's). */
	private static void loadCover(final Path file, final long id) {
		loadedCover = id;
		coverReady = false;
		if (id == 0 || !Files.isRegularFile(file)) {
			return;
		}
		try {
			NativeImage image = NativeImage.read(Files.readAllBytes(file));
			Minecraft.getInstance().getTextureManager().register(COVER, new DynamicTexture(() -> "Misty music cover", image));
			coverReady = true;
		} catch (IOException | RuntimeException e) {
			// no cover: the module shows a note instead
		}
	}

	private static String text(final JsonObject json, final String key) {
		return json.has(key) && !json.get(key).isJsonNull() ? json.get(key).getAsString() : "";
	}

	private static long number(final JsonObject json, final String key) {
		return json.has(key) && json.get(key).isJsonPrimitive() ? json.get(key).getAsLong() : 0;
	}
}
