package online.mistylauncher.client.ui;

import java.util.function.BooleanSupplier;
import java.util.function.Consumer;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.AbstractButton;
import net.minecraft.client.gui.narration.NarrationElementOutput;
import net.minecraft.client.input.InputWithModifiers;
import net.minecraft.network.chat.Component;

/** A setting on a line: its name, and a switch on the right. */
public class MistyToggle extends AbstractButton {
	private static final int SWITCH_WIDTH = 20;
	private static final int SWITCH_HEIGHT = 10;

	private final BooleanSupplier value;
	private final Consumer<Boolean> change;

	public MistyToggle(final int x, final int y, final int width, final int height, final Component message, final BooleanSupplier value, final Consumer<Boolean> change) {
		super(x, y, width, height, message);
		this.value = value;
		this.change = change;
	}

	@Override
	public void onPress(final InputWithModifiers input) {
		this.change.accept(!this.value.getAsBoolean());
	}

	@Override
	protected void extractContents(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
		int x = this.getX();
		int y = this.getY();
		boolean hot = this.isHoveredOrFocused();
		boolean on = this.value.getAsBoolean();
		Ui.card(graphics, x, y, this.getWidth(), this.getHeight(), hot ? Ui.GLASS_HOVER : Ui.GLASS, hot ? Ui.alpha(Ui.ACCENT, 0.6F) : Ui.BORDER);
		graphics.text(Minecraft.getInstance().font, this.getMessage(), x + 8, y + (this.getHeight() - 8) / 2, on ? Ui.TEXT : Ui.MUTED, false);

		int switchX = x + this.getWidth() - SWITCH_WIDTH - 7;
		int switchY = y + (this.getHeight() - SWITCH_HEIGHT) / 2;
		Ui.panel(graphics, switchX, switchY, SWITCH_WIDTH, SWITCH_HEIGHT, on ? Ui.FILL : Ui.TRACK);
		int knobX = on ? switchX + SWITCH_WIDTH - SWITCH_HEIGHT + 2 : switchX + 2;
		Ui.panel(graphics, knobX, switchY + 2, SWITCH_HEIGHT - 4, SWITCH_HEIGHT - 4, on ? 0xFFFFFFFF : Ui.MUTED);
	}

	@Override
	protected void updateWidgetNarration(final NarrationElementOutput output) {
		this.defaultButtonNarrationText(output);
	}
}
