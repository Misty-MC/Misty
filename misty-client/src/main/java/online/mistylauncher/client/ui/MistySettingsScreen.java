package online.mistylauncher.client.ui;

import online.mistylauncher.client.compat.Versions;
import java.util.List;
import java.util.function.BooleanSupplier;
import java.util.function.Consumer;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.AbstractButton;
import net.minecraft.client.gui.components.AbstractSliderButton;
import net.minecraft.client.gui.components.PlayerFaceExtractor;
import net.minecraft.client.gui.components.Tooltip;
import net.minecraft.client.gui.narration.NarrationElementOutput;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.input.InputWithModifiers;
import net.minecraft.client.renderer.RenderPipelines;
import net.minecraft.network.chat.CommonComponents;
import net.minecraft.network.chat.Component;
import net.minecraft.resources.Identifier;
import net.minecraft.util.Mth;
import net.minecraft.util.Util;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.cosmetics.Cosmetics;
import online.mistylauncher.client.features.Zoom;
import online.mistylauncher.client.hud.HudEditorScreen;
import online.mistylauncher.client.hud.HudManager;
import online.mistylauncher.client.hud.HudModule;
import online.mistylauncher.client.menu.MistyBackground;
import org.jspecify.annotations.Nullable;

/**
 * Settings of Misty Client: the categories on the left, and for the chosen one an animated
 * preview that loops (what the setting does, shown rather than told) and its settings.
 */
public final class MistySettingsScreen extends MistyScreen {
	private static final int HEADER = 22;
	private static final int CATEGORY_ROW = 22;
	private static final int ROW = 20;
	private static final int GAP = 4;
	private static final int PAD = 10;

	private enum Category {
		GENERAL("misty.settings.general", "options"),
		HUD("misty.settings.hud", "hud"),
		ZOOM("misty.settings.zoom", "zoom"),
		FULLBRIGHT("misty.settings.fullbright", "fullbright"),
		COSMETICS("misty.settings.cosmetics", "network");

		final String key;
		final Identifier icon;

		Category(final String key, final String icon) {
			this.key = key;
			this.icon = MistyClient.id("textures/gui/icons/" + icon + ".png");
		}
	}

	/** Kept when the window is resized or the screen comes back. */
	private static Category category = debugCategory();

	/** Development: -Dmisty.debug.category=ZOOM opens the settings on that category. */
	private static Category debugCategory() {
		try {
			return Category.valueOf(System.getProperty("misty.debug.category", "GENERAL"));
		} catch (IllegalArgumentException e) {
			return Category.GENERAL;
		}
	}

	/** Where the preview of the chosen category is drawn. */
	private int previewX;
	private int previewY;
	private int previewW;
	private int previewH;

	public MistySettingsScreen(final @Nullable Screen parent) {
		super(Component.translatable("misty.settings.title"), parent);
	}

	@Override
	protected float veil() {
		return 0.55F;
	}

	// ── the frame ──

	private int frameW() {
		return Math.min(this.width - 20, 600);
	}

	private int frameH() {
		return Math.min(this.height - 20, 340);
	}

	private int frameX() {
		return (this.width - this.frameW()) / 2;
	}

	private int frameY() {
		return (this.height - this.frameH()) / 2;
	}

	private int leftW() {
		return Math.max(96, Math.min(124, this.frameW() / 4));
	}

	private int rightX() {
		return this.frameX() + this.leftW();
	}

	private int rightW() {
		return this.frameW() - this.leftW();
	}

	@Override
	protected void init() {
		MistyConfig config = MistyClient.config();
		int fy = this.frameY();
		int fh = this.frameH();

		Category[] all = Category.values();
		for (int i = 0; i < all.length; i++) {
			this.addRenderableWidget(new CategoryRow(this.frameX() + 1, fy + HEADER + 4 + i * CATEGORY_ROW, this.leftW() - 2, all[i]));
		}

		int x = this.rightX() + PAD;
		int inner = this.rightW() - 2 * PAD;
		int footerY = fy + fh - PAD - 18;
		// HUD modules: 2 columns, or 3 when 2 would push the list under the footer
		int moduleCount = HudManager.modules().size();
		int hudColumns = footerY - 8 - (1 + (moduleCount + 1) / 2) * (ROW + GAP) - 8 - (fy + HEADER + 26) < 54 ? 3 : 2;
		int rows = switch (category) {
			case GENERAL -> 1;
			case HUD -> 1 + (moduleCount + hudColumns - 1) / hudColumns;
			case ZOOM -> 4;
			case FULLBRIGHT -> 1;
			case COSMETICS -> 3;
		};
		int optionsH = rows * (ROW + GAP);
		// the preview takes what the settings leave (between 54 and 130 high)
		this.previewX = x;
		this.previewY = fy + HEADER + 26;
		this.previewW = inner;
		this.previewH = Math.clamp(footerY - 8 - optionsH - 8 - this.previewY, 30, 130);
		int y = this.previewY + this.previewH + 8;

		switch (category) {
			case GENERAL -> this.option(x, y, inner, "customMenu", () -> config.customMenu, value -> config.customMenu = value);
			case HUD -> {
				this.option(x, y, inner, "hud", () -> config.hud, value -> config.hud = value);
				List<HudModule> modules = HudManager.modules();
				int column = (inner - (hudColumns - 1) * GAP) / hudColumns;
				for (int i = 0; i < modules.size(); i++) {
					MistyConfig.Module state = modules.get(i).state();
					this.addRenderableWidget(new MistyToggle(
						x + i % hudColumns * (column + GAP), y + (1 + i / hudColumns) * (ROW + GAP), column, ROW, modules.get(i).name(), () -> state.enabled, value -> state.enabled = value
					));
				}
			}
			case ZOOM -> {
				int half = (inner - GAP) / 2;
				this.option(x, y, half, "zoom", () -> config.zoom, value -> config.zoom = value);
				this.option(x + half + GAP, y, half, "zoomReset", () -> config.zoomReset, value -> config.zoomReset = value);
				this.option(x, y + ROW + GAP, half, "zoomSmooth", () -> config.zoomSmooth, value -> config.zoomSmooth = value);
				this.option(x + half + GAP, y + ROW + GAP, half, "zoomScroll", () -> config.zoomScroll, value -> config.zoomScroll = value);
				this.addRenderableWidget(new ZoomSlider(x, y + 2 * (ROW + GAP), inner, ROW));
			}
			case FULLBRIGHT -> this.option(x, y, inner, "fullbright", () -> config.fullbright, value -> config.fullbright = value);
			case COSMETICS -> {
				int half = (inner - GAP) / 2;
				this.option(x, y, half, "cosmetics", () -> config.cosmetics, value -> config.cosmetics = value);
				this.option(x + half + GAP, y, half, "network", () -> config.network, value -> config.network = value);
				this.option(x, y + ROW + GAP, half, "nametagBadge", () -> config.nametagBadge, value -> config.nametagBadge = value);
				this.option(x + half + GAP, y + ROW + GAP, half, "ownNametag", () -> config.ownNametag, value -> config.ownNametag = value);
				this.addRenderableWidget(new MistyButton(x, y + 2 * (ROW + GAP), inner, ROW, Component.translatable("misty.wardrobe.open"),
					() -> Versions.setScreen(new WardrobeScreen(this))));
			}
		}

		// footer: arrange the HUD (from the HUD category), done
		int half = (inner - GAP) / 2;
		if (category == Category.HUD) {
			this.addRenderableWidget(new MistyButton(x, footerY, half, 18, Component.translatable("misty.settings.layout"), () -> {
				MistyClient.config().save();
				// back from the editor goes to where the settings were opened from, not into a loop
				Versions.setScreen(new HudEditorScreen(this.parent instanceof HudEditorScreen editor ? editor.parent() : this.parent));
			}));
		}
		this.addRenderableWidget(new MistyButton(x + half + GAP, footerY, inner - half - GAP, 18, CommonComponents.GUI_DONE, this::onClose).primary());
	}

	private void option(final int x, final int y, final int width, final String key, final BooleanSupplier value, final Consumer<Boolean> change) {
		MistyToggle toggle = new MistyToggle(x, y, width, ROW, Component.translatable("misty.option." + key), value, change);
		toggle.setTooltip(Tooltip.create(Component.translatable("misty.option." + key + ".tip")));
		this.addRenderableWidget(toggle);
	}

	private void choose(final Category chosen) {
		category = chosen;
		this.rebuildWidgets();
	}

	@Override
	public void extractBackground(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
		super.extractBackground(graphics, mouseX, mouseY, a);
		int fx = this.frameX();
		int fy = this.frameY();
		int fw = this.frameW();
		int fh = this.frameH();
		Ui.card(graphics, fx, fy, fw, fh, 0xF00B1120, 0x33FFFFFF);
		graphics.fill(fx + 1, fy + 1, fx + fw - 1, fy + HEADER, 0xFF111A30);
		graphics.fill(fx + 1, fy + HEADER, fx + fw - 1, fy + HEADER + 1, Ui.LINE);
		graphics.fill(this.rightX(), fy + 1, this.rightX() + 1, fy + fh - 1, Ui.LINE);
	}

	@Override
	public void extractRenderState(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
		super.extractRenderState(graphics, mouseX, mouseY, a);
		Font font = this.font;
		int fx = this.frameX();
		int fy = this.frameY();
		int textY = fy + (HEADER - 8) / 2;

		// header: Misty, the version, the category and what it is about
		Ui.logo(graphics, fx + 8, fy + 4, 14);
		graphics.text(font, Component.literal("MISTY"), fx + 26, textY, Ui.TEXT, false);
		graphics.text(font, Component.literal(MistyClient.version()), fx + 26 + font.width("MISTY") + 4, textY, Ui.DIM, false);
		int x = this.rightX() + PAD;
		graphics.text(font, Component.translatable(category.key), x, textY, Ui.TEXT, false);
		graphics.text(font, Component.translatable(category.key + ".text"), x, fy + HEADER + 9, Ui.MUTED, false);

		// the preview, animated
		this.preview(graphics, font);
	}

	// ── previews: short loops that show what each category does ──

	private void preview(final GuiGraphicsExtractor graphics, final Font font) {
		int x = this.previewX;
		int y = this.previewY;
		int w = this.previewW;
		int h = this.previewH;
		float seconds = (Util.getMillis() % 3_600_000L) / 1000.0F;
		switch (category) {
			case GENERAL -> this.previewMenu(graphics, x, y, w, h, seconds);
			case HUD -> this.previewHud(graphics, font, x, y, w, h, seconds);
			case ZOOM -> this.previewZoom(graphics, font, x, y, w, h, seconds);
			case FULLBRIGHT -> this.previewFullbright(graphics, font, x, y, w, h, seconds);
			case COSMETICS -> this.previewCosmetics(graphics, font, x, y, w, h, seconds);
		}
		Ui.border(graphics, x - 1, y - 1, w + 2, h + 2, 0x4D9CC0FF);
	}

	/** The Misty menu: the landscape, the cloud and the entries sliding in one after the other. */
	private void previewMenu(final GuiGraphicsExtractor graphics, final int x, final int y, final int w, final int h, final float seconds) {
		MistyBackground.drawIn(graphics, x, y, w, h, 1.0F);
		graphics.fill(x, y, x + w / 2, y + h, 0x660A0F1D);
		Ui.logo(graphics, x + 8, y + 6, 12);
		graphics.fill(x + 24, y + 9, x + 52, y + 13, 0xFFFFFFFF);
		float loop = seconds % 4.0F;
		int entries = Math.min(5, (h - 26) / 11);
		for (int i = 0; i < entries; i++) {
			float t = Mth.clamp((loop - i * 0.25F) / 0.4F, 0.0F, 1.0F);
			float ease = 1.0F - (1.0F - t) * (1.0F - t);
			int bladeX = x + 12 + Math.round((1.0F - ease) * -40.0F) + Math.round((float)Math.sin(Math.PI * i / 4) * 6);
			int bladeY = y + 22 + i * 11;
			int alpha = Math.round(255 * ease);
			graphics.fill(bladeX, bladeY, bladeX + 60, bladeY + 8, alpha << 24 | 0x1A2A55);
			graphics.fill(bladeX, bladeY, bladeX + 2, bladeY + 8, alpha << 24 | 0x9CC0FF);
			graphics.fill(bladeX + 6, bladeY + 3, bladeX + 30, bladeY + 5, alpha << 24 | 0xEEF3FF);
		}
	}

	/** The HUD in game: FPS that move, the keys lighting up, the coordinates walking. */
	private void previewHud(final GuiGraphicsExtractor graphics, final Font font, final int x, final int y, final int w, final int h, final float seconds) {
		MistyBackground.drawIn(graphics, x, y, w, h, 1.4F);
		int fps = 140 + Math.round((float)Math.sin(seconds * 2.3F) * 6 + (float)Math.sin(seconds * 7.1F) * 2);
		box(graphics, font, x + 6, y + 6, "FPS " + fps);
		box(graphics, font, x + 6, y + 20, "XYZ " + Math.round(seconds * 4 % 200) + " 64 -32");
		String clock = String.format("%02d:%02d", (int)(seconds / 60) % 24, (int)seconds % 60);
		box(graphics, font, x + w - 6 - font.width(clock) - 6, y + 6, clock);
		// keystrokes: W, then A, S, D, in a loop
		int key = (int)(seconds * 2.5F) % 4;
		// low preview: the keys go left of the clock instead of under it
		boolean low = h < 56;
		int kx = low ? x + w - 6 - font.width(clock) - 6 - 8 - 42 : x + w - 46;
		int ky = low ? y + 4 : y + h - 32;
		keyCap(graphics, font, kx + 14, ky, "W", key == 0);
		keyCap(graphics, font, kx, ky + 14, "A", key == 1);
		keyCap(graphics, font, kx + 14, ky + 14, "S", key == 2);
		keyCap(graphics, font, kx + 28, ky + 14, "D", key == 3);
	}

	/** Zoom: the landscape magnified and back, the key lit while it zooms, the level shown. */
	private void previewZoom(final GuiGraphicsExtractor graphics, final Font font, final int x, final int y, final int w, final int h, final float seconds) {
		float level = Math.clamp(MistyClient.config().zoomLevel, Zoom.MIN, Zoom.MAX);
		// 4 s loop: rest, zoom in, hold, zoom out
		float loop = seconds % 4.0F;
		float t = loop < 0.6F ? 0.0F : loop < 1.2F ? (loop - 0.6F) / 0.6F : loop < 2.8F ? 1.0F : loop < 3.4F ? 1.0F - (loop - 2.8F) / 0.6F : 0.0F;
		float ease = t * t * (3.0F - 2.0F * t);
		float zoom = 1.0F + (level - 1.0F) * ease;
		MistyBackground.drawIn(graphics, x, y, w, h, zoom);
		// crosshair
		int cx = x + w / 2;
		int cy = y + h / 2;
		graphics.fill(cx - 4, cy, cx + 5, cy + 1, 0xCCFFFFFF);
		graphics.fill(cx, cy - 4, cx + 1, cy + 5, 0xCCFFFFFF);
		// the key, held while zooming, and the magnification
		boolean held = loop >= 0.6F && loop < 3.4F;
		KeyMapping key = Zoom.key();
		String name = key != null ? key.getTranslatedKeyMessage().getString() : "C";
		keyCap(graphics, font, x + 6, y + h - 18, name, held);
		String text = String.format("×%.1f", zoom);
		box(graphics, font, x + w - 6 - font.width(text) - 6, y + 6, text);
	}

	/** Fullbright: a dark night, and the same lit up, the line between them sweeping across. */
	private void previewFullbright(final GuiGraphicsExtractor graphics, final Font font, final int x, final int y, final int w, final int h, final float seconds) {
		MistyBackground.drawIn(graphics, x, y, w, h, 1.6F);
		float sweep = 0.5F + 0.42F * (float)Math.sin(seconds * 1.2F);
		int split = x + Math.round(w * sweep);
		// left: the night as the game shows it; right: lit by Misty
		graphics.fill(x, y, split, y + h, 0xC0000000);
		graphics.fill(split, y, x + w, y + h, 0x30FFF4D6);
		graphics.fill(split, y, split + 1, y + h, 0xFFFFFFFF);
		graphics.text(font, Component.translatable("misty.fullbright.before"), x + 6, y + 6, Ui.MUTED, false);
		Component after = Component.translatable("misty.fullbright.after");
		graphics.text(font, after, x + w - 6 - font.width(after), y + 6, 0xFFFFFFFF, false);
		graphics.blit(RenderPipelines.GUI_TEXTURED, Category.FULLBRIGHT.icon, split - 8, y + h / 2 - 8, 0.0F, 0.0F, 16, 16, 16, 16, 16, 16);
		KeyMapping key = MistyClient.fullbrightKey();
		if (key != null && !key.isUnbound()) {
			keyCap(graphics, font, x + 6, y + h - 18, key.getTranslatedKeyMessage().getString(), (int)seconds % 2 == 0);
		}
	}

	/** Cosmetics and network: you with the Misty cape, the cloud before your name, as the others see you. */
	private void previewCosmetics(final GuiGraphicsExtractor graphics, final Font font, final int x, final int y, final int w, final int h, final float seconds) {
		MistyBackground.drawIn(graphics, x, y, w, h, 1.0F);
		int cx = x + w / 2;
		int bob = Math.round((float)Math.sin(seconds * 2.0F) * 2.0F);
		// the cape, swaying
		int zoom = Math.max(1, Math.min(3, (h - 30) / 18));
		Identifier misty = Cosmetics.texturePath();
		graphics.blit(RenderPipelines.GUI_TEXTURED, misty, cx - 5 * zoom, y + 22 + bob, 1.0F, 1.0F, 10 * zoom, 16 * zoom, 10, 16, 64, 32);
		// the name tag: the Misty cloud before the name, like in the player list and the chat
		String name = Minecraft.getInstance().getUser().getName();
		int tagW = font.width(name) + 22;
		int tagX = cx - tagW / 2;
		int tagY = y + 6;
		graphics.fill(tagX, tagY, tagX + tagW, tagY + 12, 0x66000000);
		Ui.logo(graphics, tagX + 3, tagY + 2, 8);
		PlayerFaceExtractor.extractRenderState(graphics, PlayerPreview.skin(), tagX - 14, tagY + 1, 10);
		graphics.text(font, name, tagX + 14, tagY + 2, 0xFFFFFFFF, false);
	}

	private static void box(final GuiGraphicsExtractor graphics, final Font font, final int x, final int y, final String text) {
		graphics.fill(x, y, x + font.width(text) + 6, y + 12, 0x990A0F1D);
		graphics.text(font, text, x + 3, y + 2, 0xFFFFFFFF, false);
	}

	private static void keyCap(final GuiGraphicsExtractor graphics, final Font font, final int x, final int y, final String key, final boolean down) {
		int w = Math.max(12, font.width(key) + 6);
		Ui.card(graphics, x, y, w, 12, down ? 0xFF6F9CFF : 0xCC0A0F1D, down ? 0xFFEEF3FF : 0x66FFFFFF);
		graphics.text(font, key, x + (w - font.width(key)) / 2, y + 2, down ? Ui.INK : 0xFFFFFFFF, false);
	}

	@Override
	public void removed() {
		MistyClient.config().save();
	}

	/** A category in the left column: its icon and its name. */
	private final class CategoryRow extends AbstractButton {
		private final Category row;

		CategoryRow(final int x, final int y, final int width, final Category row) {
			super(x, y, width, CATEGORY_ROW, Component.translatable(row.key));
			this.row = row;
		}

		@Override
		public void onPress(final InputWithModifiers input) {
			MistySettingsScreen.this.choose(this.row);
		}

		@Override
		protected void extractContents(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
			Font font = Minecraft.getInstance().font;
			int x = this.getX();
			int y = this.getY();
			boolean chosen = category == this.row;
			if (chosen || this.isHoveredOrFocused()) {
				graphics.fill(x, y, x + this.getWidth(), y + this.getHeight(), chosen ? 0xFF16223F : 0xFF111A30);
			}
			if (chosen) {
				graphics.fill(x, y, x + 2, y + this.getHeight(), Ui.ACCENT);
			}
			graphics.blit(RenderPipelines.GUI_TEXTURED, this.row.icon, x + 8, y + 3, 0.0F, 0.0F, 16, 16, 16, 16, 16, 16);
			graphics.text(font, this.getMessage(), x + 29, y + 7, chosen ? 0xFFFFFFFF : Ui.TEXT, false);
		}

		@Override
		protected void updateWidgetNarration(final NarrationElementOutput output) {
			this.defaultButtonNarrationText(output);
		}
	}

	/** How much the zoom magnifies (×1.5 to ×10). */
	private static final class ZoomSlider extends AbstractSliderButton {
		ZoomSlider(final int x, final int y, final int width, final int height) {
			super(x, y, width, height, Component.empty(), (Math.clamp(MistyClient.config().zoomLevel, Zoom.MIN, Zoom.MAX) - Zoom.MIN) / (Zoom.MAX - Zoom.MIN));
			this.updateMessage();
		}

		private float level() {
			return Zoom.MIN + (float)this.value * (Zoom.MAX - Zoom.MIN);
		}

		@Override
		protected void updateMessage() {
			this.setMessage(Component.translatable("misty.option.zoomLevel", String.format("×%.1f", this.level())));
		}

		@Override
		protected void applyValue() {
			MistyClient.config().zoomLevel = Math.round(this.level() * 10.0F) / 10.0F;
		}
	}
}
