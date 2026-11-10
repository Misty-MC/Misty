package online.mistylauncher.client.hud.modules;

import java.time.LocalTime;
import java.time.format.DateTimeFormatter;
import net.minecraft.client.Minecraft;
import net.minecraft.network.chat.Component;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.hud.TextModule;

/** Time of the computer: no need to leave the game to know how late it is. */
public final class ClockModule extends TextModule {
	private static final DateTimeFormatter FORMAT = DateTimeFormatter.ofPattern("HH:mm");

	public ClockModule() {
		super("clock", new MistyConfig.Module(false, 1, 0, 4), 40);
	}

	@Override
	protected Component text(final Minecraft minecraft, final boolean preview) {
		return Component.literal(LocalTime.now().format(FORMAT));
	}
}
