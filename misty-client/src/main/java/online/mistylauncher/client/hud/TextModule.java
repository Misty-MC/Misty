package online.mistylauncher.client.hud;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.MutableComponent;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.ui.Ui;

/** A module that is one short line: a value in white and its unit in the accent colour. */
public abstract class TextModule extends HudModule {
	private static final int HEIGHT = 14;
	private static final int PADDING = 5;

	private final int minWidth;

	protected TextModule(final String id, final MistyConfig.Module defaults, final int minWidth) {
		super(id, defaults);
		this.minWidth = minWidth;
	}

	protected abstract Component text(Minecraft minecraft, boolean preview);

	protected static MutableComponent value(final String value, final String unit) {
		return Component.literal(value).append(Component.literal(" " + unit).withColor(Ui.ACCENT & 0xFFFFFF));
	}

	@Override
	public int width(final Minecraft minecraft, final boolean preview) {
		return Math.max(this.minWidth, minecraft.font.width(this.text(minecraft, preview)) + 2 * PADDING);
	}

	@Override
	public int height(final Minecraft minecraft, final boolean preview) {
		return HEIGHT;
	}

	@Override
	public void draw(final GuiGraphicsExtractor graphics, final Minecraft minecraft, final boolean preview) {
		Component text = this.text(minecraft, preview);
		int width = this.width(minecraft, preview);
		background(graphics, width, HEIGHT);
		graphics.text(minecraft.font, text, (width - minecraft.font.width(text)) / 2, 3, Ui.TEXT, false);
	}
}
