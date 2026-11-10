package online.mistylauncher.client.ui;

import java.util.function.BooleanSupplier;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.AbstractButton;
import net.minecraft.client.gui.narration.NarrationElementOutput;
import net.minecraft.client.input.InputWithModifiers;
import net.minecraft.client.renderer.RenderPipelines;
import net.minecraft.network.chat.Component;
import net.minecraft.resources.Identifier;
import org.jspecify.annotations.Nullable;

/** Button of the Misty screens: frosted glass, accent bar on hover. */
public class MistyButton extends AbstractButton {
	private final Runnable onPress;
	private boolean primary;
	private boolean alignLeft;
	private @Nullable Identifier icon;
	private boolean cloud;
	private @Nullable BooleanSupplier selected;

	public MistyButton(final int x, final int y, final int width, final int height, final Component message, final Runnable onPress) {
		super(x, y, width, height, message);
		this.onPress = onPress;
	}

	/** Filled with the accent colour: the main action of a screen. */
	public MistyButton primary() {
		this.primary = true;
		return this;
	}

	/** Label on the left, after an accent bar: the entries of the main menu. */
	public MistyButton alignLeft() {
		this.alignLeft = true;
		return this;
	}

	/** A sprite of the GUI atlas instead of the label (the label stays for the narrator). */
	public MistyButton icon(final Identifier sprite) {
		this.icon = sprite;
		return this;
	}

	/** The Misty cloud instead of the label. */
	public MistyButton cloud() {
		this.cloud = true;
		return this;
	}

	/** Drawn as chosen while this holds: tabs. */
	public MistyButton selectedWhen(final BooleanSupplier selected) {
		this.selected = selected;
		return this;
	}

	@Override
	public void onPress(final InputWithModifiers input) {
		this.onPress.run();
	}

	@Override
	protected void extractContents(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
		Font font = Minecraft.getInstance().font;
		int x = this.getX();
		int y = this.getY();
		int width = this.getWidth();
		int height = this.getHeight();
		boolean hot = this.active && this.isHoveredOrFocused();
		boolean chosen = this.selected != null && this.selected.getAsBoolean();

		int background;
		int textColor;
		if (this.primary || chosen) {
			background = hot ? Ui.FILL_HOVER : Ui.FILL;
			textColor = Ui.INK;
		} else {
			background = hot ? Ui.GLASS_HOVER : Ui.GLASS;
			textColor = this.active ? (hot ? 0xFFFFFFFF : Ui.TEXT) : Ui.DIM;
		}
		Ui.card(graphics, x, y, width, height, Ui.alpha(background, this.alpha), Ui.alpha(hot ? Ui.ACCENT : Ui.BORDER, this.alpha * (hot ? 0.6F : 1.0F)));

		if (this.cloud) {
			int size = Math.min(width, height) - 6;
			Ui.logo(graphics, x + (width - size) / 2, y + (height - size) / 2, size);
			return;
		}
		if (this.icon != null) {
			int size = Math.min(width, height) - 5;
			graphics.blitSprite(RenderPipelines.GUI_TEXTURED, this.icon, x + (width - size) / 2, y + (height - size) / 2, size, size);
			return;
		}
		int textY = y + (height - 8) / 2;
		if (this.alignLeft) {
			if (hot) {
				graphics.fill(x + 1, y + 3, x + 3, y + height - 3, Ui.ACCENT);
			}
			graphics.text(font, this.getMessage(), x + (hot ? 12 : 9), textY, Ui.alpha(textColor, this.alpha), false);
		} else {
			Ui.centered(graphics, font, this.getMessage(), x + width / 2, textY, Ui.alpha(textColor, this.alpha));
		}
	}

	@Override
	protected void updateWidgetNarration(final NarrationElementOutput output) {
		this.defaultButtonNarrationText(output);
	}
}
