package online.mistylauncher.client.ui;

import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;
import org.jspecify.annotations.Nullable;

/** Base of the Misty screens: the game (or the menu panorama) stays visible behind a night veil. */
public abstract class MistyScreen extends Screen {
	protected final @Nullable Screen parent;

	protected MistyScreen(final Component title, final @Nullable Screen parent) {
		super(title);
		this.parent = parent;
	}

	/** Screen shown again when this one closes (none: back to the game). */
	public final @Nullable Screen parent() {
		return this.parent;
	}

	/** Opacity of the veil over what is behind (0..1). */
	protected abstract float veil();

	/** Whether what is behind is blurred (as much as the game's "menu background blur" option says). */
	protected boolean blur() {
		return true;
	}

	@Override
	public void extractBackground(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
		if (this.minecraft.level == null) {
			this.extractPanorama(graphics, a);
		}
		if (this.blur()) {
			this.extractBlurredBackground(graphics);
		}
		graphics.fill(0, 0, this.width, this.height, Ui.alpha(Ui.INK, this.veil()));
		this.minecraft.gui.hud.extractDeferredSubtitles();
	}

	@Override
	public void onClose() {
		this.minecraft.gui.setScreen(this.parent);
	}

	@Override
	public boolean isPauseScreen() {
		return false;
	}
}
