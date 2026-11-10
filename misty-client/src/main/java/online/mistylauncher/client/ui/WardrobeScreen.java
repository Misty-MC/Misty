package online.mistylauncher.client.ui;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.AbstractButton;
import net.minecraft.client.gui.narration.NarrationElementOutput;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.input.InputWithModifiers;
import net.minecraft.client.renderer.RenderPipelines;
import net.minecraft.network.chat.CommonComponents;
import net.minecraft.network.chat.Component;
import net.minecraft.resources.Identifier;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.config.LauncherLink;
import online.mistylauncher.client.cosmetics.Cosmetic;
import online.mistylauncher.client.cosmetics.Cosmetics;
import org.jspecify.annotations.Nullable;

/**
 * The wardrobe, laid out like Essential's: the categories on the left, the cosmetics of the
 * chosen one in the middle, your outfit in 3D on the right (drag to turn). A click equips at
 * once: the preview, the game and the other Misty players see it, and the launcher keeps it.
 */
public final class WardrobeScreen extends MistyScreen {
	private static final int HEADER = 22;
	private static final int CATEGORY_ROW = 22;
	private static final int GAP = 8;

	/** The categories: only the capes for now, the others are shown as coming. */
	private enum Category {
		CAPES("misty.wardrobe.capes", "cape", 0xFF9CC0FF, true),
		HATS("misty.wardrobe.hats", "hats", 0xFF8FE3C8, false),
		WINGS("misty.wardrobe.wings", "wings", 0xFFC9A8FF, false),
		PETS("misty.wardrobe.pets", "pets", 0xFFFFC46B, false),
		EMOTES("misty.wardrobe.emotes", "emotes", 0xFFFF9CB7, false);

		final String key;
		final Identifier icon;
		final int color;
		final boolean open;

		Category(final String key, final String icon, final int color, final boolean open) {
			this.key = key;
			this.icon = MistyClient.id("textures/gui/icons/" + icon + ".png");
			this.color = color;
			this.open = open;
		}
	}

	/** The capes you can wear (the Misty cape comes with its elytra). */
	private static final Cosmetic[] CAPES = {Cosmetic.NONE, Cosmetic.CAPE};

	private Category category = Category.CAPES;

	public WardrobeScreen(final @Nullable Screen parent) {
		super(Component.translatable("misty.wardrobe.title"), parent);
	}

	@Override
	protected float veil() {
		return 0.65F;
	}

	// ── the frame and its three columns ──

	private int frameW() {
		return Math.min(this.width - 20, 580);
	}

	private int frameH() {
		return Math.min(this.height - 20, 330);
	}

	private int frameX() {
		return (this.width - this.frameW()) / 2;
	}

	private int frameY() {
		return (this.height - this.frameH()) / 2;
	}

	private int leftW() {
		return Math.max(84, Math.min(110, this.frameW() / 5));
	}

	private int rightW() {
		return Math.max(110, Math.min(210, this.frameW() * 2 / 5));
	}

	private int middleX() {
		return this.frameX() + this.leftW();
	}

	private int middleW() {
		return this.frameW() - this.leftW() - this.rightW();
	}

	private int rightX() {
		return this.frameX() + this.frameW() - this.rightW();
	}

	@Override
	protected void init() {
		int fy = this.frameY();
		int fh = this.frameH();

		// categories
		Category[] all = Category.values();
		for (int i = 0; i < all.length; i++) {
			this.addRenderableWidget(new CategoryRow(this.frameX() + 1, fy + HEADER + 4 + i * CATEGORY_ROW, this.leftW() - 2, all[i]));
		}

		// the cosmetics of the chosen category
		if (this.category.open) {
			int room = this.middleW() - 2 * 10;
			int cardW = Math.min(76, (room - GAP) / 2);
			int cardH = Math.min(96, fh - HEADER - 50);
			int columns = Math.max(1, (room + GAP) / (cardW + GAP));
			for (int i = 0; i < CAPES.length; i++) {
				int x = this.middleX() + 10 + i % columns * (cardW + GAP);
				int y = fy + HEADER + 10 + i / columns * (cardH + GAP);
				this.addRenderableWidget(new Card(x, y, cardW, cardH, CAPES[i]));
			}
		}

		// your outfit, and done
		int previewH = fh - HEADER - 62;
		int previewW = Math.min(this.rightW() - 20, previewH * 3 / 5);
		this.addRenderableWidget(new PlayerPreview(this.rightX() + (this.rightW() - previewW) / 2, fy + HEADER + 12, previewW, previewH, LauncherLink::cosmetic));
		this.addRenderableWidget(new MistyButton(this.rightX() + 10, fy + fh - 26, this.rightW() - 20, 18, CommonComponents.GUI_DONE, this::onClose).primary());
	}

	private void choose(final Category chosen) {
		this.category = chosen;
		this.rebuildWidgets();
	}

	@Override
	public void extractBackground(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
		// the launcher's landscape behind the wardrobe (in the menu and in game), lightly veiled
		online.mistylauncher.client.menu.MistyBackground.draw(graphics, this.width, this.height);
		graphics.fill(0, 0, this.width, this.height, Ui.alpha(Ui.INK, 0.35F));
		int fx = this.frameX();
		int fy = this.frameY();
		int fw = this.frameW();
		int fh = this.frameH();

		// frame, headers, separators
		Ui.card(graphics, fx, fy, fw, fh, 0xF00B1120, 0x33FFFFFF);
		graphics.fill(fx + 1, fy + 1, fx + fw - 1, fy + HEADER, 0xFF111A30);
		graphics.fill(fx + 1, fy + HEADER, fx + fw - 1, fy + HEADER + 1, Ui.LINE);
		graphics.fill(this.middleX(), fy + 1, this.middleX() + 1, fy + fh - 1, Ui.LINE);
		graphics.fill(this.rightX(), fy + 1, this.rightX() + 1, fy + fh - 1, Ui.LINE);
		// the outfit stands on a pedestal of light, in a slightly lighter corner
		graphics.fill(this.rightX() + 1, fy + HEADER + 1, fx + fw - 1, fy + fh - 1, 0x40111A30);
		int center = this.rightX() + this.rightW() / 2;
		// under the feet of the preview (which ends 50 pixels above the bottom)
		int feet = fy + fh - 56;
		for (int i = 0; i < 6; i++) {
			int half = 50 - i * 8;
			graphics.fill(center - half, feet + i, center + half, feet + i + 1, Ui.alpha(Ui.FILL, 0.10F + i * 0.05F));
		}
	}

	@Override
	public void extractRenderState(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
		super.extractRenderState(graphics, mouseX, mouseY, a);
		Font font = this.font;
		int fy = this.frameY();
		int fh = this.frameH();
		int textY = fy + (HEADER - 8) / 2;

		graphics.text(font, this.title, this.frameX() + 10, textY, Ui.TEXT, false);

		// middle: the category and how many you have
		Component name = Component.translatable(this.category.key);
		graphics.text(font, name, this.middleX() + 10, textY, Ui.TEXT, false);
		if (this.category.open) {
			String count = " [" + CAPES.length + "/" + CAPES.length + "]";
			graphics.text(font, count, this.middleX() + 10 + font.width(name), textY, Ui.DIM, false);
			if (!MistyClient.config().cosmetics) {
				graphics.text(font, Component.translatable("misty.wardrobe.off"), this.middleX() + 10, fy + fh - 16, 0xFFFFB46B, false);
			}
		} else {
			// a category to come: its icon, big, and a word
			int cx = this.middleX() + this.middleW() / 2;
			int cy = fy + fh / 2 - 10;
			graphics.blit(RenderPipelines.GUI_TEXTURED, this.category.icon, cx - 16, cy - 24, 0.0F, 0.0F, 32, 32, 16, 16, 16, 16);
			Ui.centered(graphics, font, Component.translatable("misty.wardrobe.coming"), cx, cy + 16, Ui.TEXT);
			Ui.centered(graphics, font, Component.translatable("misty.wardrobe.coming.text"), cx, cy + 28, Ui.DIM);
		}

		// right: your outfit
		int center = this.rightX() + this.rightW() / 2;
		Ui.centered(graphics, font, Component.translatable("misty.wardrobe.outfit"), center, textY, Ui.ACCENT);
		Ui.centered(graphics, font, Component.literal(Minecraft.getInstance().getUser().getName()), center, fy + fh - 38, Ui.MUTED);
	}

	/** A category in the left column: its icon and its name, coloured; the coming ones dimmed. */
	private final class CategoryRow extends AbstractButton {
		private final Category category;

		CategoryRow(final int x, final int y, final int width, final Category category) {
			super(x, y, width, CATEGORY_ROW, Component.translatable(category.key));
			this.category = category;
		}

		@Override
		public void onPress(final InputWithModifiers input) {
			WardrobeScreen.this.choose(this.category);
		}

		@Override
		protected void extractContents(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
			Font font = Minecraft.getInstance().font;
			int x = this.getX();
			int y = this.getY();
			boolean chosen = WardrobeScreen.this.category == this.category;
			boolean hot = this.isHoveredOrFocused();
			if (chosen || hot) {
				graphics.fill(x, y, x + this.getWidth(), y + this.getHeight(), chosen ? 0xFF16223F : 0xFF111A30);
			}
			if (chosen) {
				graphics.fill(x, y, x + 2, y + this.getHeight(), this.category.color);
			}
			float alpha = this.category.open ? 1.0F : 0.55F;
			graphics.blit(RenderPipelines.GUI_TEXTURED, this.category.icon, x + 8, y + 3, 0.0F, 0.0F, 16, 16, 16, 16, 16, 16, Ui.alpha(0xFFFFFFFF, alpha));
			graphics.text(font, this.getMessage(), x + 29, y + 7, Ui.alpha(this.category.color, alpha), false);
		}

		@Override
		protected void updateWidgetNarration(final NarrationElementOutput output) {
			this.defaultButtonNarrationText(output);
		}
	}

	/** One cosmetic: its picture, its name, "equipped" when worn. */
	private static final class Card extends AbstractButton {
		private final Cosmetic cosmetic;

		Card(final int x, final int y, final int width, final int height, final Cosmetic cosmetic) {
			super(x, y, width, height, Component.translatable("misty.cosmetic." + (cosmetic == Cosmetic.NONE ? "none" : cosmetic.id())));
			this.cosmetic = cosmetic;
		}

		@Override
		public void onPress(final InputWithModifiers input) {
			LauncherLink.equip(this.cosmetic);
		}

		@Override
		protected void extractContents(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
			Font font = Minecraft.getInstance().font;
			int x = this.getX();
			int y = this.getY();
			int w = this.getWidth();
			int h = this.getHeight();
			boolean worn = LauncherLink.cosmetic() == this.cosmetic;
			boolean hot = this.isHoveredOrFocused();
			// picture area on top (lighter at the bottom, like a stand), name and state under it
			int pictureH = h - 30;
			Ui.card(graphics, x, y, w, h, worn ? 0xFF16223F : hot ? 0xFF131D36 : 0xFF0F172B, worn ? Ui.ACCENT : hot ? Ui.alpha(Ui.ACCENT, 0.6F) : Ui.BORDER);
			graphics.fillGradient(x + 1, y + 1, x + w - 1, y + pictureH, 0x00000000, 0x30FFFFFF);

			int cx = x + w / 2;
			Identifier misty = Cosmetics.texture().texturePath();
			int zoom = Math.clamp((pictureH - 8) / 16, 1, 4);
			switch (this.cosmetic) {
				// front of the cape: 10 × 16 at (1, 1) of the 64 × 32 texture
				case CAPE -> graphics.blit(RenderPipelines.GUI_TEXTURED, misty, cx - 5 * zoom, y + (pictureH - 16 * zoom) / 2, 1.0F, 1.0F, 10 * zoom, 16 * zoom, 10, 16, 64, 32);
				case ELYTRA -> {
				}
				case NONE -> {
					int s = Math.min(24, pictureH - 12);
					int top = y + (pictureH - s) / 2;
					for (int i = 0; i < s; i++) {
						graphics.fill(cx - s / 2 + i, top + i, cx - s / 2 + i + 2, top + i + 2, Ui.DIM);
						graphics.fill(cx + s / 2 - i - 2, top + i, cx + s / 2 - i, top + i + 2, Ui.DIM);
					}
				}
			}
			if (worn) {
				// a small check in the corner
				Ui.panel(graphics, x + w - 13, y + 3, 10, 10, Ui.FILL);
				graphics.text(font, "✔", x + w - 11, y + 4, Ui.INK, false);
			}

			float nameScale = Math.min(1.0F, (w - 6) / (float)font.width(this.getMessage()));
			Ui.centeredScaled(graphics, font, this.getMessage(), cx, y + h - 24, nameScale, worn ? 0xFFFFFFFF : Ui.TEXT, false);
			Component state = worn ? Component.translatable("misty.wardrobe.equipped") : Component.translatable("misty.wardrobe.equip");
			graphics.pose().pushMatrix();
			graphics.pose().translate(cx, y + h - 12);
			graphics.pose().scale(0.75F, 0.75F);
			graphics.text(font, state, -font.width(state) / 2, 0, worn ? Ui.ACCENT : Ui.DIM, false);
			graphics.pose().popMatrix();
		}

		@Override
		protected void updateWidgetNarration(final NarrationElementOutput output) {
			this.defaultButtonNarrationText(output);
		}
	}
}
