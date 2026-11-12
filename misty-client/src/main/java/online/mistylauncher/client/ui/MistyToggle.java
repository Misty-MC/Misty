package online.mistylauncher.client.ui;

import java.util.function.BooleanSupplier;
import java.util.function.Consumer;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.AbstractButton;
import net.minecraft.client.gui.narration.NarrationElementOutput;
import net.minecraft.client.input.InputWithModifiers;
import net.minecraft.client.renderer.RenderPipelines;
import net.minecraft.network.chat.Component;
import net.minecraft.resources.Identifier;
import online.mistylauncher.client.MistyClient;
import org.jspecify.annotations.Nullable;

/** A setting on a line: its name, and a switch on the right (with a gear before it when it has settings). */
public class MistyToggle extends AbstractButton {
	private static final int SWITCH_WIDTH = 20;
	private static final int SWITCH_HEIGHT = 10;
	private static final int GEAR = 10;
	private static final Identifier GEAR_ICON = MistyClient.id("textures/gui/icons/options.png");

	private final BooleanSupplier value;
	private final Consumer<Boolean> change;
	/** Opens the settings of what this line turns on (null: no gear). */
	private @Nullable Runnable gear;
	/** The mouse is over the gear (noted while drawing, read when the line is clicked). */
	private boolean gearHot;

	public MistyToggle(final int x, final int y, final int width, final int height, final Component message, final BooleanSupplier value, final Consumer<Boolean> change) {
		super(x, y, width, height, message);
		this.value = value;
		this.change = change;
	}

	/** Adds a gear before the switch: a click on it opens the settings instead of switching. */
	public MistyToggle gear(final Runnable open) {
		this.gear = open;
		return this;
	}

	@Override
	public void onPress(final InputWithModifiers input) {
		if (this.gear != null && this.gearHot) {
			this.gear.run();
		} else {
			this.change.accept(!this.value.getAsBoolean());
		}
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

		if (this.gear != null) {
			int gearX = switchX - GEAR - 6;
			int gearY = y + (this.getHeight() - GEAR) / 2;
			this.gearHot = this.isHovered() && mouseX >= gearX - 2 && mouseX < gearX + GEAR + 2 && mouseY >= gearY - 2 && mouseY < gearY + GEAR + 2;
			if (this.gearHot) {
				Ui.panel(graphics, gearX - 2, gearY - 2, GEAR + 4, GEAR + 4, Ui.alpha(Ui.ACCENT, 0.35F));
			}
			graphics.blit(RenderPipelines.GUI_TEXTURED, GEAR_ICON, gearX, gearY, 0.0F, 0.0F, GEAR, GEAR, 16, 16, 16, 16);
		} else {
			this.gearHot = false;
		}
	}

	@Override
	protected void updateWidgetNarration(final NarrationElementOutput output) {
		this.defaultButtonNarrationText(output);
	}
}
