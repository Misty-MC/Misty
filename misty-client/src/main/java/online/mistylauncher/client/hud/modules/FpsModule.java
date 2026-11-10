package online.mistylauncher.client.hud.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.network.chat.Component;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.hud.TextModule;

public final class FpsModule extends TextModule {
	public FpsModule() {
		super("fps", new MistyConfig.Module(true, 0, 4, 4), 50);
	}

	@Override
	protected Component text(final Minecraft minecraft, final boolean preview) {
		return value(Integer.toString(minecraft.getFps()), "FPS");
	}
}
