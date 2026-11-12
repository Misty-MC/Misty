package online.mistylauncher.client.hud.modules;

import java.util.Locale;
import net.minecraft.client.Minecraft;
import net.minecraft.network.chat.Component;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.hud.Reach;
import online.mistylauncher.client.hud.TextModule;

/** Distance of the last player hit, in blocks. */
public final class ReachModule extends TextModule {
	public ReachModule() {
		super("reach", new MistyConfig.Module(true, 3, 4, 64), 58);
	}

	@Override
	protected Component text(final Minecraft minecraft, final boolean preview) {
		double reach = Reach.last();
		if (reach < 0.0) {
			return value(preview ? "3.00" : "-", "m");
		}
		return value(String.format(Locale.ROOT, "%.2f", reach), "m");
	}
}
