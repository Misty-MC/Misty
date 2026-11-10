package online.mistylauncher.client.hud;

import java.util.List;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.input.MouseButtonEvent;
import net.minecraft.network.chat.CommonComponents;
import net.minecraft.network.chat.Component;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.ui.MistyButton;
import online.mistylauncher.client.ui.MistyScreen;
import online.mistylauncher.client.ui.MistySettingsScreen;
import online.mistylauncher.client.ui.Ui;
import org.jspecify.annotations.Nullable;

/**
 * HUD editor (Right Shift in game): every module is shown where it sits. Drag to move, click
 * to turn on or off, scroll to resize, right-click to put back.
 */
public final class HudEditorScreen extends MistyScreen {
	private static final int PANEL_WIDTH = 188;
	private static final int PANEL_HEIGHT = 70;
	/** A module sticks to the screen edges (with this margin) and to the centre lines. */
	private static final int MARGIN = 4;
	private static final int SNAP = 5;
	/** A press that moved less than this is a click, not a drag. */
	private static final int CLICK_SLOP = 2;

	private @Nullable HudModule dragged;
	private double grabX;
	private double grabY;
	private double pressX;
	private double pressY;
	private boolean moved;
	private boolean centeredX;
	private boolean centeredY;

	public HudEditorScreen(final @Nullable Screen parent) {
		super(Component.translatable("misty.editor.title"), parent);
	}

	@Override
	protected float veil() {
		return 0.35F;
	}

	@Override
	protected boolean blur() {
		return false;
	}

	@Override
	protected void init() {
		int x = (this.width - PANEL_WIDTH) / 2;
		int y = (this.height - PANEL_HEIGHT) / 2 + PANEL_HEIGHT - 26;
		int buttonWidth = (PANEL_WIDTH - 16 - 8) / 3;
		this.addRenderableWidget(new MistyButton(x + 8, y, buttonWidth, 18, Component.translatable("misty.editor.settings"), () -> {
			MistyClient.config().save();
			this.minecraft.gui.setScreen(new MistySettingsScreen(this));
		}));
		this.addRenderableWidget(new MistyButton(x + 12 + buttonWidth, y, buttonWidth, 18, Component.translatable("misty.editor.reset"), () -> {
			HudManager.resetAll(MistyClient.config());
			MistyClient.config().save();
		}));
		this.addRenderableWidget(new MistyButton(x + 16 + 2 * buttonWidth, y, buttonWidth, 18, CommonComponents.GUI_DONE, this::onClose).primary());
	}

	private HudManager.Bounds bounds(final HudModule module) {
		return HudManager.bounds(module, this.minecraft, this.width, this.height, true);
	}

	/** Topmost module under the pointer (the last drawn is on top). */
	private @Nullable HudModule moduleAt(final double x, final double y) {
		List<HudModule> modules = HudManager.modules();
		for (int i = modules.size() - 1; i >= 0; i--) {
			if (this.bounds(modules.get(i)).contains(x, y)) {
				return modules.get(i);
			}
		}
		return null;
	}

	@Override
	public void extractRenderState(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
		if (this.dragged != null && this.moved) {
			if (this.centeredX) {
				graphics.fill(this.width / 2, 0, this.width / 2 + 1, this.height, Ui.alpha(Ui.ACCENT, 0.5F));
			}
			if (this.centeredY) {
				graphics.fill(0, this.height / 2, this.width, this.height / 2 + 1, Ui.alpha(Ui.ACCENT, 0.5F));
			}
		}

		// the panel first: a module dragged over it stays visible
		int x = (this.width - PANEL_WIDTH) / 2;
		int y = (this.height - PANEL_HEIGHT) / 2;
		Ui.card(graphics, x, y, PANEL_WIDTH, PANEL_HEIGHT, Ui.SURFACE, Ui.BORDER);
		Ui.logo(graphics, x + 10, y + 7, 22);
		graphics.text(this.font, Component.literal("MISTY"), x + 38, y + 9, Ui.TEXT, false);
		graphics.text(this.font, Component.translatable("misty.editor.title"), x + 38, y + 20, Ui.ACCENT, false);
		super.extractRenderState(graphics, mouseX, mouseY, a);
		Ui.centered(graphics, this.font, Component.translatable("misty.editor.hint1"), this.width / 2, y + PANEL_HEIGHT + 8, Ui.TEXT);
		Ui.centered(graphics, this.font, Component.translatable("misty.editor.hint2"), this.width / 2, y + PANEL_HEIGHT + 20, Ui.MUTED);

		HudModule hovered = this.dragged != null ? this.dragged : this.moduleAt(mouseX, mouseY);
		for (HudModule module : HudManager.modules()) {
			HudManager.Bounds bounds = this.bounds(module);
			boolean enabled = module.state().enabled;
			HudManager.draw(graphics, this.minecraft, module, bounds, true);
			if (!enabled) {
				graphics.fill(bounds.x(), bounds.y(), bounds.x() + bounds.width(), bounds.y() + bounds.height(), Ui.alpha(Ui.INK, 0.72F));
			}
			int outline = module == hovered ? 0xFFFFFFFF : enabled ? Ui.ACCENT : Ui.DIM;
			graphics.outline(bounds.x() - 1, bounds.y() - 1, bounds.width() + 2, bounds.height() + 2, outline);
		}
		if (hovered != null) {
			this.label(graphics, hovered);
		}
	}

	/** Name and state of the module under the pointer, above it (below when it touches the top). */
	private void label(final GuiGraphicsExtractor graphics, final HudModule module) {
		HudManager.Bounds bounds = this.bounds(module);
		Component state = Component.translatable(module.state().enabled ? "misty.editor.on" : "misty.editor.off");
		Component text = Component.empty().append(module.name()).append(" · ").append(state);
		int width = this.font.width(text) + 8;
		int x = Math.clamp(bounds.x() + (bounds.width() - width) / 2, 2, Math.max(2, this.width - width - 2));
		int y = bounds.y() - 15 >= 2 ? bounds.y() - 15 : bounds.y() + bounds.height() + 3;
		Ui.panel(graphics, x, y, width, 12, Ui.SURFACE);
		graphics.text(this.font, text, x + 4, y + 2, Ui.TEXT, false);
	}

	@Override
	public boolean mouseClicked(final MouseButtonEvent event, final boolean doubleClick) {
		HudModule module = this.moduleAt(event.x(), event.y());
		if (module == null) {
			return super.mouseClicked(event, doubleClick);
		}
		if (event.button() == 1) {
			module.reset(MistyClient.config());
			return true;
		}
		if (event.button() == 0) {
			HudManager.Bounds bounds = this.bounds(module);
			this.dragged = module;
			this.grabX = event.x() - bounds.x();
			this.grabY = event.y() - bounds.y();
			this.pressX = event.x();
			this.pressY = event.y();
			this.moved = false;
			return true;
		}
		return false;
	}

	@Override
	public boolean mouseDragged(final MouseButtonEvent event, final double dx, final double dy) {
		if (this.dragged == null) {
			return super.mouseDragged(event, dx, dy);
		}
		if (!this.moved && Math.abs(event.x() - this.pressX) < CLICK_SLOP && Math.abs(event.y() - this.pressY) < CLICK_SLOP) {
			return true;
		}
		this.moved = true;
		HudManager.Bounds bounds = this.bounds(this.dragged);
		int centerX = (this.width - bounds.width()) / 2;
		int centerY = (this.height - bounds.height()) / 2;
		int x = snap((int)Math.round(event.x() - this.grabX), MARGIN, centerX, this.width - bounds.width() - MARGIN);
		int y = snap((int)Math.round(event.y() - this.grabY), MARGIN, centerY, this.height - bounds.height() - MARGIN);
		this.centeredX = x == centerX;
		this.centeredY = y == centerY;
		x = Math.clamp(x, 0, Math.max(0, this.width - bounds.width()));
		y = Math.clamp(y, 0, Math.max(0, this.height - bounds.height()));
		HudManager.place(this.dragged, bounds, x, y, this.width, this.height);
		return true;
	}

	private static int snap(final int value, final int... targets) {
		for (int target : targets) {
			if (Math.abs(value - target) <= SNAP) {
				return target;
			}
		}
		return value;
	}

	@Override
	public boolean mouseReleased(final MouseButtonEvent event) {
		if (this.dragged == null) {
			return super.mouseReleased(event);
		}
		if (!this.moved) {
			MistyConfig.Module state = this.dragged.state();
			state.enabled = !state.enabled;
		}
		this.dragged = null;
		MistyClient.config().save();
		return true;
	}

	@Override
	public boolean mouseScrolled(final double x, final double y, final double scrollX, final double scrollY) {
		HudModule module = this.moduleAt(x, y);
		if (module == null || scrollY == 0.0) {
			return super.mouseScrolled(x, y, scrollX, scrollY);
		}
		// resized around its top-left corner, by steps of 10 %
		HudManager.Bounds before = this.bounds(module);
		MistyConfig.Module state = module.state();
		float scale = Math.round((state.scale + (scrollY > 0 ? 0.1F : -0.1F)) * 10.0F) / 10.0F;
		state.scale = Math.clamp(scale, HudManager.MIN_SCALE, HudManager.MAX_SCALE);
		HudManager.place(module, this.bounds(module), before.x(), before.y(), this.width, this.height);
		return true;
	}

	@Override
	public void removed() {
		MistyClient.config().save();
	}
}
