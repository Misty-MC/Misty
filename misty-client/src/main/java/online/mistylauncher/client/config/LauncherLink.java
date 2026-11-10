package online.mistylauncher.client.config;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.attribute.FileTime;
import online.mistylauncher.client.cosmetics.Cosmetic;

/**
 * What the launcher chose for this player: it writes {@code client.json} in its data folder and
 * gives that folder to the game with {@code -Dmisty.launcher.dir}. The file is read again when
 * it changes, so a cape equipped in the launcher shows up without restarting the game.
 */
public final class LauncherLink {
	private static final int CHECK_EVERY_TICKS = 40;

	private static volatile Cosmetic cosmetic = Cosmetic.NONE;
	private static FileTime lastModified;
	private static int ticks;

	private LauncherLink() {
	}

	/** Cosmetic equipped in the launcher (or in the wardrobe of the game). */
	public static Cosmetic cosmetic() {
		return cosmetic;
	}

	/**
	 * Equips a cosmetic from the game (the wardrobe): worn at once, and written to
	 * {@code client.json} with {@code "from": "game"} so the launcher keeps this choice instead of
	 * writing its own over it at the next start.
	 */
	public static void equip(final Cosmetic chosen) {
		cosmetic = chosen;
		String dir = System.getProperty("misty.launcher.dir");
		if (dir == null || dir.isBlank()) {
			return;
		}
		try {
			Path file = Path.of(dir, "client.json");
			JsonObject json = new JsonObject();
			json.addProperty("cosmetic", chosen.id());
			json.addProperty("from", "game");
			Path temporary = Path.of(dir, "client.json.tmp");
			Files.writeString(temporary, json.toString(), StandardCharsets.UTF_8);
			Files.move(temporary, file, java.nio.file.StandardCopyOption.REPLACE_EXISTING, java.nio.file.StandardCopyOption.ATOMIC_MOVE);
			lastModified = Files.getLastModifiedTime(file);
		} catch (IOException | RuntimeException e) {
			// not written: worn for this session only
		}
	}

	public static void tick() {
		if (++ticks >= CHECK_EVERY_TICKS) {
			ticks = 0;
			refresh();
		}
	}

	public static void refresh() {
		String dir = System.getProperty("misty.launcher.dir");
		if (dir == null || dir.isBlank()) {
			return;
		}
		try {
			Path file = Path.of(dir, "client.json");
			if (!Files.isRegularFile(file)) {
				cosmetic = Cosmetic.NONE;
				lastModified = null;
				return;
			}
			FileTime modified = Files.getLastModifiedTime(file);
			if (modified.equals(lastModified)) {
				return;
			}
			lastModified = modified;
			JsonObject json = JsonParser.parseString(Files.readString(file, StandardCharsets.UTF_8)).getAsJsonObject();
			cosmetic = json.has("cosmetic") ? Cosmetic.of(json.get("cosmetic").getAsString()) : Cosmetic.NONE;
		} catch (IOException | RuntimeException e) {
			// half-written or unreadable file: keep what we had, the next check reads it again
			lastModified = null;
		}
	}
}
