package online.mistylauncher.client.hud.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.network.chat.Component;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.hud.Clicks;
import online.mistylauncher.client.hud.TextModule;

/** Clicks per second: left | right. */
public final class CpsModule extends TextModule {
	public CpsModule() {
		super("cps", new MistyConfig.Module(true, 3, 4, 46), 58);
	}

	@Override
	protected Component text(final Minecraft minecraft, final boolean preview) {
		return value(Clicks.left() + " | " + Clicks.right(), "CPS");
	}
}
