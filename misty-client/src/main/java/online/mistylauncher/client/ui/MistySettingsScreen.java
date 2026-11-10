package online.mistylauncher.client.ui;

import java.util.List;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.Tooltip;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.CommonComponents;
import net.minecraft.network.chat.Component;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.hud.HudEditorScreen;
import online.mistylauncher.client.hud.HudManager;
import online.mistylauncher.client.hud.HudModule;
import org.jspecify.annotations.Nullable;

/** Settings of Misty Client: the general switches, and one switch per HUD module. */
public final class MistySettingsScreen extends MistyScreen {
	private static final int WIDTH = 264;
	private static final int HEIGHT = 192;
	private static final int PADDING = 10;
	private static final int ROW = 22;
	private static final int GAP = 4;

	/** 0 general, 1 HUD modules: kept when the window is resized. */
	private int tab;

	public MistySettingsScreen(final @Nullable Screen parent) {
		super(Component.translatable("misty.settings.title"), parent);
	}

	@Override
	protected float veil() {
		return 0.55F;
	}

	private int left() {
		return (this.width - WIDTH) / 2;
	}

	private int top() {
		return (this.height - HEIGHT) / 2;
	}

	@Override
	protected void init() {
		MistyConfig config = MistyClient.config();
		int x = this.left() + PADDING;
		int y = this.top() + 34;
		int inner = WIDTH - 2 * PADDING;

		int tabWidth = (inner - GAP) / 2;
		this.addRenderableWidget(new MistyButton(x, y, tabWidth, 16, Component.translatable("misty.settings.general"), () -> this.show(0)).selectedWhen(() -> this.tab == 0));
		this.addRenderableWidget(
			new MistyButton(x + tabWidth + GAP, y, tabWidth, 16, Component.translatable("misty.settings.modules"), () -> this.show(1)).selectedWhen(() -> this.tab == 1)
		);
		y += 16 + 8;

		if (this.tab == 0) {
			this.option(x, y, inner, "customMenu", () -> config.customMenu, value -> config.customMenu = value);
			this.option(x, y + ROW + GAP, inner, "hud", () -> config.hud, value -> config.hud = value);
			this.option(x, y + 2 * (ROW + GAP), inner, "network", () -> config.network, value -> config.network = value);
			this.option(x, y + 3 * (ROW + GAP), inner, "cosmetics", () -> config.cosmetics, value -> config.cosmetics = value);
		} else {
			List<HudModule> modules = HudManager.modules();
			int column = (inner - GAP) / 2;
			for (int i = 0; i < modules.size(); i++) {
				MistyConfig.Module state = modules.get(i).state();
				this.addRenderableWidget(
					new MistyToggle(
						x + i % 2 * (column + GAP), y + i / 2 * (ROW + GAP), column, ROW, modules.get(i).name(), () -> state.enabled, value -> state.enabled = value
					)
				);
			}
		}

		int footer = this.top() + HEIGHT - PADDING - 18;
		int half = (inner - GAP) / 2;
		this.addRenderableWidget(new MistyButton(x, footer, half, 18, Component.translatable("misty.settings.layout"), () -> {
			MistyClient.config().save();
			// back from the editor goes to where the settings were opened from, not into a loop
			this.minecraft.gui.setScreen(new HudEditorScreen(this.parent instanceof HudEditorScreen editor ? editor.parent() : this.parent));
		}));
		this.addRenderableWidget(new MistyButton(x + half + GAP, footer, half, 18, CommonComponents.GUI_DONE, this::onClose).primary());
	}

	private void option(final int x, final int y, final int width, final String key, final java.util.function.BooleanSupplier value, final java.util.function.Consumer<Boolean> change) {
		MistyToggle toggle = new MistyToggle(x, y, width, ROW, Component.translatable("misty.option." + key), value, change);
		toggle.setTooltip(Tooltip.create(Component.translatable("misty.option." + key + ".tip")));
		this.addRenderableWidget(toggle);
	}

	private void show(final int tab) {
		this.tab = tab;
		this.rebuildWidgets();
	}

	@Override
	public void extractRenderState(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
		int x = this.left();
		int y = this.top();
		Ui.card(graphics, x, y, WIDTH, HEIGHT, Ui.SURFACE, Ui.BORDER);
		Ui.logo(graphics, x + PADDING, y + 6, 22);
		graphics.text(this.font, Component.literal("MISTY"), x + PADDING + 28, y + 8, Ui.TEXT, false);
		graphics.text(this.font, Component.literal("CLIENT " + MistyClient.version()), x + PADDING + 28, y + 19, Ui.ACCENT, false);
		super.extractRenderState(graphics, mouseX, mouseY, a);
	}

	@Override
	public void removed() {
		MistyClient.config().save();
	}
}
