package online.mistylauncher.client;

import online.mistylauncher.client.compat.Versions;
import com.mojang.blaze3d.platform.InputConstants;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientLifecycleEvents;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keymapping.v1.KeyMappingHelper;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.minecraft.resources.Identifier;
import online.mistylauncher.client.config.LauncherLink;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.features.Zoom;
import net.minecraft.network.chat.Component;
import online.mistylauncher.client.hud.HudEditorScreen;
import online.mistylauncher.client.hud.HudManager;
import online.mistylauncher.client.net.MistyNetwork;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Misty Client: the in-game side of the Misty launcher (custom menu, HUD, Misty badge in the
 * player list and the chat, Misty cosmetics).
 */
public final class MistyClient implements ClientModInitializer {
	public static final String ID = "misty";
	public static final Logger LOG = LoggerFactory.getLogger("Misty");

	private static MistyConfig config = new MistyConfig();
	private static KeyMapping editorKey;
	private static KeyMapping fullbrightKey;

	public static MistyConfig config() {
		return config;
	}

	/** The fullbright switch key (shown in the settings). */
	public static KeyMapping fullbrightKey() {
		return fullbrightKey;
	}

	public static Identifier id(final String path) {
		return Identifier.fromNamespaceAndPath(ID, path);
	}

	/** Version of the mod, as written in fabric.mod.json ("1.0.0+mc26.3" gives "1.0.0"). */
	public static String version() {
		return FabricLoader.getInstance()
			.getModContainer("misty-client")
			.map(mod -> mod.getMetadata().getVersion().getFriendlyString().split("\\+")[0])
			.orElse("dev");
	}

	@Override
	public void onInitializeClient() {
		config = MistyConfig.load();
		HudManager.init(config);
		LauncherLink.refresh();

		// the keys of the controls screen, under "Misty Client" (key.category.misty.misty)
		//? if >=1.21.9 {
		KeyMapping.Category category = KeyMapping.Category.register(id("misty"));
		//?} else {
		/*String category = "key.category.misty.misty";
		*///?}
		editorKey = KeyMappingHelper.registerKeyMapping(new KeyMapping("key.misty.editor", InputConstants.KEY_RSHIFT, category));
		// zoom: held (C, like OptiFine); fullbright: a switch, no key until the player picks one
		Zoom.bind(KeyMappingHelper.registerKeyMapping(new KeyMapping("key.misty.zoom", InputConstants.KEY_C, category)));
		fullbrightKey = KeyMappingHelper.registerKeyMapping(new KeyMapping("key.misty.fullbright", InputConstants.UNKNOWN.getValue(), category));

		ClientTickEvents.END_CLIENT_TICK.register(MistyClient::tick);
		ClientLifecycleEvents.CLIENT_STARTED.register(MistyNetwork::start);
		// gradlew runClient: every hook is applied right away, and the ones that found nothing are
		// logged (mixin.debug.countInjections), to check a Minecraft version quickly
		if (Boolean.getBoolean("misty.audit")) {
			ClientLifecycleEvents.CLIENT_STARTED.register(minecraft -> org.spongepowered.asm.mixin.MixinEnvironment.getCurrentEnvironment().audit());
		}
		ClientLifecycleEvents.CLIENT_STOPPING.register(minecraft -> MistyNetwork.stop());
	}

	private static void tick(final Minecraft minecraft) {
		LauncherLink.tick();
		MistyNetwork.tick(minecraft);
		online.mistylauncher.client.hud.Ping.tick(minecraft);
		Zoom.tick();
		online.mistylauncher.client.features.Music.tick();
		while (fullbrightKey.consumeClick()) {
			config.fullbright = !config.fullbright;
			config.save();
			if (minecraft.player != null) {
				Versions.overlayMessage(Component.translatable(config.fullbright ? "misty.fullbright.on" : "misty.fullbright.off"));
			}
		}
		while (editorKey.consumeClick()) {
			if (Versions.screen() == null) {
				Versions.setScreen(new HudEditorScreen(null));
			}
		}
	}
}
