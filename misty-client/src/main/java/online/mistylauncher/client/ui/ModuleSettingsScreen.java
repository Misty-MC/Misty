package online.mistylauncher.client.ui;

import java.util.List;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.Tooltip;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.CommonComponents;
import net.minecraft.network.chat.Component;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.hud.HudModule;
import org.jspecify.annotations.Nullable;

/** The settings of one HUD module (its gear in the settings screen): one switch per setting. */
public final class ModuleSettingsScreen extends MistyScreen {
	private static final int WIDTH = 240;
	private static final int ROW = 20;
	private static final int GAP = 4;
	private static final int PAD = 10;
	private static final int HEADER = 22;

	private final HudModule module;

	public ModuleSettingsScreen(final HudModule module, final @Nullable Screen parent) {
		super(Component.translatable("misty.module.settings", module.name()), parent);
		this.module = module;
	}

	@Override
	protected float veil() {
		return 0.55F;
	}

	private int frameW() {
		return Math.min(this.width - 20, WIDTH);
	}

	private int frameH() {
		return HEADER + PAD + this.module.settings().size() * (ROW + GAP) + PAD + 18 + PAD;
	}

	private int frameX() {
		return (this.width - this.frameW()) / 2;
	}

	private int frameY() {
		return (this.height - this.frameH()) / 2;
	}

	@Override
	protected void init() {
		int x = this.frameX() + PAD;
		int inner = this.frameW() - 2 * PAD;
		int y = this.frameY() + HEADER + PAD;
		List<HudModule.Setting> settings = this.module.settings();
		for (HudModule.Setting setting : settings) {
			String key = setting.key();
			MistyToggle toggle = new MistyToggle(x, y, inner, ROW, setting.name(this.module), () -> this.module.setting(key),
				value -> this.module.state().setOption(key, value));
			toggle.setTooltip(Tooltip.create(setting.tip(this.module)));
			this.addRenderableWidget(toggle);
			y += ROW + GAP;
		}
		this.addRenderableWidget(new MistyButton(x, y + PAD - GAP, inner, 18, CommonComponents.GUI_DONE, this::onClose).primary());
	}

	@Override
	public void extractBackground(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
		super.extractBackground(graphics, mouseX, mouseY, a);
		int fx = this.frameX();
		int fy = this.frameY();
		int fw = this.frameW();
		Ui.card(graphics, fx, fy, fw, this.frameH(), 0xF00B1120, 0x33FFFFFF);
		graphics.fill(fx + 1, fy + 1, fx + fw - 1, fy + HEADER, 0xFF111A30);
		graphics.fill(fx + 1, fy + HEADER, fx + fw - 1, fy + HEADER + 1, Ui.LINE);
	}

	@Override
	public void extractRenderState(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
		super.extractRenderState(graphics, mouseX, mouseY, a);
		int fx = this.frameX();
		int fy = this.frameY();
		Ui.logo(graphics, fx + 8, fy + 4, 14);
		graphics.text(this.font, this.title, fx + 26, fy + (HEADER - 8) / 2, Ui.TEXT, false);
	}

	@Override
	public void removed() {
		MistyClient.config().save();
	}
}
