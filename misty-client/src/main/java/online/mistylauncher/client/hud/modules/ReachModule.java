package online.mistylauncher.client.hud.modules;

import java.util.List;
import java.util.Locale;
import net.minecraft.client.Minecraft;
import net.minecraft.network.chat.Component;
import net.minecraft.util.Mth;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.hud.Reach;
import online.mistylauncher.client.hud.TextModule;
import online.mistylauncher.client.ui.Ui;

/** Distance of the last player hit, in blocks. */
public final class ReachModule extends TextModule {
	private static final String COLOR = "color";
	/** Green up to here, then yellow, then red at the longest reach of survival (3 blocks). */
	private static final float NEAR = 2.0F;
	private static final float MIDDLE = 2.6F;
	private static final float FAR = 3.0F;
	private static final int GREEN = 0x55E07A;
	private static final int YELLOW = 0xE8D44D;
	private static final int RED = 0xEB5757;

	public ReachModule() {
		super("reach", new MistyConfig.Module(true, 3, 4, 64), 58);
	}

	@Override
	public List<Setting> settings() {
		return List.of(new Setting(COLOR, true));
	}

	@Override
	protected Component text(final Minecraft minecraft, final boolean preview) {
		double reach = Reach.last();
		if (reach < 0.0) {
			if (!preview) {
				return value("-", "m");
			}
			reach = 2.85;
		}
		int color = this.setting(COLOR) ? color((float)reach) : Ui.TEXT;
		return value(String.format(Locale.ROOT, "%.2f", reach), color, "m");
	}

	/** The farther the hit, the redder: green, yellow, then red, blended in between. */
	private static int color(final float reach) {
		if (reach <= NEAR) {
			return GREEN;
		}
		if (reach <= MIDDLE) {
			return blend(GREEN, YELLOW, (reach - NEAR) / (MIDDLE - NEAR));
		}
		return blend(YELLOW, RED, Mth.clamp((reach - MIDDLE) / (FAR - MIDDLE), 0.0F, 1.0F));
	}

	private static int blend(final int from, final int to, final float t) {
		int r = Math.round(Mth.lerp(t, from >> 16 & 0xFF, to >> 16 & 0xFF));
		int g = Math.round(Mth.lerp(t, from >> 8 & 0xFF, to >> 8 & 0xFF));
		int b = Math.round(Mth.lerp(t, from & 0xFF, to & 0xFF));
		return r << 16 | g << 8 | b;
	}
}
