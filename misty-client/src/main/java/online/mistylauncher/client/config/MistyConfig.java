package online.mistylauncher.client.config;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.LinkedHashMap;
import java.util.Map;
import net.fabricmc.loader.api.FabricLoader;
import online.mistylauncher.client.MistyClient;

/** Settings of the mod, kept in {@code config/misty-client.json} of the profile. */
public final class MistyConfig {
	private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();

	/** Misty main menu instead of the Minecraft one. */
	public boolean customMenu = true;
	/** HUD modules shown in game. */
	public boolean hud = true;
	/**
	 * Misty network: other Misty players see your badge and cosmetics, and you see theirs.
	 * Off: nothing is sent and nothing is fetched.
	 */
	public boolean network = true;
	/** Misty capes and elytra are drawn (yours and the other players'). */
	public boolean cosmetics = true;
	/** State of each HUD module, by module id. */
	public Map<String, Module> modules = new LinkedHashMap<>();

	/** Where a HUD module sits: a corner, side or centre of the screen plus an offset in GUI pixels. */
	public static final class Module {
		public boolean enabled;
		/** 0..8: column (left, centre, right) + 3 × row (top, middle, bottom). */
		public int anchor;
		public int x;
		public int y;
		public float scale = 1.0F;

		public Module() {
		}

		public Module(final boolean enabled, final int anchor, final int x, final int y) {
			this.enabled = enabled;
			this.anchor = anchor;
			this.x = x;
			this.y = y;
		}

		public Module copy() {
			Module copy = new Module(this.enabled, this.anchor, this.x, this.y);
			copy.scale = this.scale;
			return copy;
		}
	}

	private static Path file() {
		return FabricLoader.getInstance().getConfigDir().resolve("misty-client.json");
	}

	public static MistyConfig load() {
		Path file = file();
		if (Files.isRegularFile(file)) {
			try {
				MistyConfig config = GSON.fromJson(Files.readString(file, StandardCharsets.UTF_8), MistyConfig.class);
				if (config != null) {
					if (config.modules == null) {
						config.modules = new LinkedHashMap<>();
					}
					config.modules.values().removeIf(module -> module == null);
					return config;
				}
			} catch (IOException | RuntimeException e) {
				MistyClient.LOG.warn("Unreadable settings, back to the defaults", e);
			}
		}
		return new MistyConfig();
	}

	public void save() {
		try {
			Path file = file();
			Files.createDirectories(file.getParent());
			Files.writeString(file, GSON.toJson(this), StandardCharsets.UTF_8);
		} catch (IOException e) {
			MistyClient.LOG.warn("Settings not saved", e);
		}
	}
}
