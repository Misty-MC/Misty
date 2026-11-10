package online.mistylauncher.client.config;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.attribute.FileTime;
import online.mistylauncher.client.cosmetics.Cosmetic;
import online.mistylauncher.client.cosmetics.Outfit;

/**
 * What the launcher chose for this player: it writes {@code client.json} in its data folder and
 * gives that folder to the game with {@code -Dmisty.launcher.dir}. The file is read again when
 * it changes, so a cape equipped in the launcher shows up without restarting the game.
 */
public final class LauncherLink {
	private static final int CHECK_EVERY_TICKS = 40;

	private static volatile Outfit outfit = Outfit.NONE;
	private static FileTime lastModified;
	private static int ticks;

	private LauncherLink() {
	}

	/** Outfit chosen in the launcher (or in the wardrobe of the game). */
	public static Outfit outfit() {
		return outfit;
	}

	/** What is worn on the back. */
	public static Cosmetic cosmetic() {
		return outfit.back();
	}

	/**
	 * Equips a cosmetic from the game (the wardrobe): worn at once, and written to
	 * {@code client.json} with {@code "from": "game"} so the launcher keeps this choice instead of
	 * writing its own over it at the next start.
	 */
	public static void equip(final Cosmetic.Slot slot, final Cosmetic chosen) {
		outfit = outfit.with(slot, chosen);
		String dir = System.getProperty("misty.launcher.dir");
		if (dir == null || dir.isBlank()) {
			return;
		}
		try {
			Path file = Path.of(dir, "client.json");
			JsonObject json = new JsonObject();
			json.addProperty("cosmetic", outfit.back().id());
			json.addProperty("hat", outfit.hat().id());
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
				outfit = Outfit.NONE;
				lastModified = null;
				return;
			}
			FileTime modified = Files.getLastModifiedTime(file);
			if (modified.equals(lastModified)) {
				return;
			}
			lastModified = modified;
			JsonObject json = JsonParser.parseString(Files.readString(file, StandardCharsets.UTF_8)).getAsJsonObject();
			outfit = Outfit.of(json.has("cosmetic") ? json.get("cosmetic").getAsString() : "", json.has("hat") ? json.get("hat").getAsString() : "");
		} catch (IOException | RuntimeException e) {
			// half-written or unreadable file: keep what we had, the next check reads it again
			lastModified = null;
		}
	}
}
