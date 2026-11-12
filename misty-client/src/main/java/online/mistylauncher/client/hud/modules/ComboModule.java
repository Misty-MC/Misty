package online.mistylauncher.client.hud.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.network.chat.Component;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.hud.Combo;
import online.mistylauncher.client.hud.TextModule;

/** Players hit in a row without being hit back (see Combo). */
public final class ComboModule extends TextModule {
	public ComboModule() {
		super("combo", new MistyConfig.Module(true, 3, 4, 82), 58);
	}

	@Override
	protected Component text(final Minecraft minecraft, final boolean preview) {
		int combo = Combo.combo();
		return value(Integer.toString(preview && combo == 0 ? 3 : combo), "combo");
	}
}
