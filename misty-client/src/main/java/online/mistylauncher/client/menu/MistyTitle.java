package online.mistylauncher.client.menu;

import java.util.function.Consumer;
import net.minecraft.ChatFormatting;
import net.minecraft.SharedConstants;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.AbstractButton;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.components.PlainTextButton;
import net.minecraft.client.gui.components.PlayerFaceExtractor;
import net.minecraft.client.gui.components.Tooltip;
import net.minecraft.client.gui.narration.NarrationElementOutput;
import net.minecraft.client.gui.screens.CreditsAndAttributionScreen;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.TitleScreen;
import net.minecraft.client.gui.screens.multiplayer.JoinMultiplayerScreen;
import net.minecraft.client.gui.screens.multiplayer.SafetyScreen;
import net.minecraft.client.gui.screens.options.AccessibilityOptionsScreen;
import net.minecraft.client.gui.screens.options.LanguageSelectScreen;
import net.minecraft.client.gui.screens.options.OptionsScreen;
import net.minecraft.client.gui.screens.worldselection.SelectWorldScreen;
import net.minecraft.client.input.InputWithModifiers;
import net.minecraft.network.chat.Component;
import net.minecraft.resources.Identifier;
import net.minecraft.util.Mth;
import net.minecraft.client.renderer.RenderPipelines;
import com.mojang.realmsclient.RealmsMainScreen;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.config.LauncherLink;
import online.mistylauncher.client.cosmetics.Cosmetic;
import online.mistylauncher.client.ui.MistyButton;
import online.mistylauncher.client.ui.MistySettingsScreen;
import online.mistylauncher.client.ui.PlayerPreview;
import online.mistylauncher.client.ui.Ui;
import online.mistylauncher.client.ui.WardrobeScreen;
import org.jspecify.annotations.Nullable;

/**
 * The Misty main menu, over the launcher's night landscape scrolling to the right:
 * <ul>
 * <li>the title in the top-left corner;</li>
 * <li>the entries along a glowing path of diamonds, each with an icon, its name and a line
 * saying where it leads;</li>
 * <li>you in the bottom-left corner (your head opens the wardrobe);</li>
 * <li>on the right, you in 3D with your cosmetics, and the wardrobe button.</li>
 * </ul>
 */
public final class MistyTitle {
	private static final Component COPYRIGHT = Component.translatable("title.credits");
	private static final int ENTRY_W = 176;
	private static final int ENTRIES = 5;
	/** Space kept for the title at the top and for you at the bottom. */
	private static final int TITLE = 66;
	private static final int BOTTOM = 56;
	private static final int ICON = 20;
	/** You in 3D on the right of the menu (hidden on narrow windows). */
	private static final int CHARACTER_W = 120;
	private static final int PANEL_MIN_SCREEN = 420;
	/** Right edge of the menu entries (with their slide). */
	private static final int MENU_RIGHT = 54 + 22 + ENTRY_W + 12;

	private MistyTitle() {
	}

	/** Distance between two entries: they close up on low windows (1080p with the automatic GUI scale is 270 high). */
	private static int step(final int height) {
		return Mth.clamp((height - TITLE - BOTTOM) / ENTRIES, 24, 34);
	}

	private static int entryHeight(final int height) {
		return step(height) - 6;
	}

	/** Y of the first entry: under the title, centred in what is left above you. */
	private static int entriesTop(final int height) {
		int free = height - TITLE - BOTTOM - ENTRIES * step(height);
		return TITLE + 8 + Math.max(0, free / 2);
	}

	/** X of an entry: the path bulges in the middle, like an arc. */
	private static int entryX(final int index) {
		return 54 + Math.round((float)Math.sin(Math.PI * index / (ENTRIES - 1)) * 22.0F);
	}

	private static boolean panel(final int width) {
		return width >= PANEL_MIN_SCREEN;
	}

	/** Height of the character, feet included (the name above, the wardrobe button under). */
	private static int characterHeight(final int height) {
		// the wardrobe button under the feet stays clear of the versions line
		return Math.min(200, height - 124);
	}

	/** Centre of the character: between the menu and the right edge, a little to the left. */
	private static int characterCenter(final int width) {
		int free = width - MENU_RIGHT;
		return MENU_RIGHT + Math.max(CHARACTER_W / 2, free * 9 / 20);
	}

	/** Y of the top of the character. */
	private static int characterTop(final int height) {
		return (height - characterHeight(height)) / 2 - 4;
	}

	public static void init(final TitleScreen screen, final int width, final int height, final @Nullable Component multiplayerDisabled, final Consumer<AbstractWidget> add) {
		Minecraft minecraft = Minecraft.getInstance();
		int top = entriesTop(height);
		int step = step(height);
		int entryH = entryHeight(height);

		add.accept(new Portal(entryX(0), top, entryH, "menu.singleplayer", "misty.title.singleplayer", "singleplayer",
			() -> minecraft.gui.setScreen(new SelectWorldScreen(screen))));
		Portal multiplayer = new Portal(entryX(1), top + step, entryH, "menu.multiplayer", "misty.title.multiplayer", "multiplayer", () -> {
			Screen next = minecraft.options.skipMultiplayerWarning ? new JoinMultiplayerScreen(screen) : new SafetyScreen(screen);
			minecraft.gui.setScreen(next);
		});
		Portal realms = new Portal(entryX(2), top + 2 * step, entryH, "menu.online", "misty.title.realms", "realms",
			() -> minecraft.gui.setScreen(new RealmsMainScreen(screen)));
		if (multiplayerDisabled != null) {
			multiplayer.active = false;
			realms.active = false;
			multiplayer.setTooltip(Tooltip.create(multiplayerDisabled));
			realms.setTooltip(Tooltip.create(multiplayerDisabled));
		}
		add.accept(multiplayer);
		add.accept(realms);
		add.accept(new Portal(entryX(3), top + 3 * step, entryH, "menu.options", "misty.title.options", "options",
			() -> minecraft.gui.setScreen(new OptionsScreen(screen, minecraft.options))));
		add.accept(new Portal(entryX(4), top + 4 * step, entryH, "menu.quit", "misty.title.quit", "quit", minecraft::stop));

		// you, bottom left: your head opens the wardrobe
		MistyButton you = new MistyButton(14, height - 50, 36, 36, Component.translatable("misty.wardrobe.title"),
			() -> minecraft.gui.setScreen(new WardrobeScreen(screen))) {
			@Override
			protected void extractContents(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
			}
		};
		you.setTooltip(Tooltip.create(Component.translatable("misty.wardrobe.title")));
		add.accept(you);

		// on the right: you in 3D, standing on the landscape, and the wardrobe under your feet
		if (panel(width)) {
			int center = characterCenter(width);
			int characterY = characterTop(height);
			int characterH = characterHeight(height);
			add.accept(new PlayerPreview(center - CHARACTER_W / 2, characterY, CHARACTER_W, characterH, LauncherLink::cosmetic));
			add.accept(new MistyButton(center - 50, characterY + characterH + 18, 100, 18, Component.translatable("misty.wardrobe.open"),
				() -> minecraft.gui.setScreen(new WardrobeScreen(screen))).primary());
		}

		// top right: language, accessibility, Misty settings
		int iconY = 10;
		int iconX = width - 10 - 3 * ICON - 2 * 4;
		add.accept(icon(iconX, iconY, "options.language", MistyClient.id("textures/gui/icons/language.png"),
			() -> minecraft.gui.setScreen(new LanguageSelectScreen(screen, minecraft.options, minecraft.getLanguageManager()))));
		add.accept(icon(iconX + ICON + 4, iconY, "options.accessibility", MistyClient.id("textures/gui/icons/accessibility.png"),
			() -> minecraft.gui.setScreen(new AccessibilityOptionsScreen(screen, minecraft.options))));
		MistyButton settings = new MistyButton(iconX + 2 * (ICON + 4), iconY, ICON, ICON, Component.translatable("misty.settings.title"),
			() -> minecraft.gui.setScreen(new MistySettingsScreen(screen))).cloud();
		settings.setTooltip(Tooltip.create(Component.translatable("misty.settings.title")));
		add.accept(settings);

		// required by Mojang on every title screen: the copyright line, which opens the credits
		int copyrightWidth = minecraft.font.width(COPYRIGHT);
		add.accept(new PlainTextButton(width - copyrightWidth - 4, height - 12, copyrightWidth, 10, COPYRIGHT,
			button -> minecraft.gui.setScreen(new CreditsAndAttributionScreen(screen)), minecraft.font));
	}

	private static MistyButton icon(final int x, final int y, final String key, final Identifier texture, final Runnable onPress) {
		MistyButton button = new MistyButton(x, y, ICON, ICON, Component.translatable(key), onPress).texture(texture);
		button.setTooltip(Tooltip.create(Component.translatable(key)));
		return button;
	}

	/** Under the widgets: the landscape, a veil on the left for the texts, the path, the panel. */
	public static void background(final GuiGraphicsExtractor graphics, final int width, final int height) {
		MistyBackground.draw(graphics, width, height);

		// darker on the left, where the menu is
		int veil = Math.min(330, width / 2);
		for (int i = 0; i < 12; i++) {
			int x0 = veil * i / 12;
			int x1 = veil * (i + 1) / 12;
			graphics.fill(x0, 0, x1, height, Ui.alpha(Ui.INK, 0.62F * (1.0F - i / 12.0F)));
		}

		// the path: from under the title, through a diamond in front of each entry
		int top = entriesTop(height);
		int step = step(height);
		int entryH = entryHeight(height);
		int previousX = 30;
		int previousY = top - 20;
		for (int i = 0; i < ENTRIES; i++) {
			int nodeX = entryX(i) - 18;
			int nodeY = top + i * step + entryH / 2;
			line(graphics, previousX, previousY, nodeX, nodeY);
			previousX = nodeX;
			previousY = nodeY;
		}
		for (int i = 0; i < ENTRIES; i++) {
			diamond(graphics, entryX(i) - 18, top + i * step + entryH / 2, 5, Ui.alpha(Ui.ACCENT, 0.9F), 0xFF0E1830);
		}

		if (panel(width)) {
			// a pedestal of light under the feet, and a soft glow behind
			int center = characterCenter(width);
			int feet = characterTop(height) + characterHeight(height) - 8;
			for (int i = 0; i < 6; i++) {
				int half = 46 - i * 7;
				graphics.fill(center - half, feet + i, center + half, feet + i + 1, Ui.alpha(Ui.FILL, 0.10F + i * 0.05F));
			}
		}
	}

	/** Over the widgets: title, the panel's texts, you, versions. */
	public static void foreground(final GuiGraphicsExtractor graphics, final Font font, final int width, final int height) {
		Minecraft minecraft = Minecraft.getInstance();

		// title, top left
		Ui.logo(graphics, 20, 16, 30);
		Component misty = Component.literal("MISTY");
		scaledLeft(graphics, font, misty, 57, 20, 3.0F, Ui.SLATE);
		scaledLeft(graphics, font, misty, 56, 19, 3.0F, 0xFFFFFFFF);
		graphics.text(font, Component.literal("C L I E N T"), 58, 46, Ui.ACCENT, false);
		Component tagline = Component.translatable("misty.title.portals");
		graphics.pose().pushMatrix();
		graphics.pose().translate(40.0F, entriesTop(height) - 12.0F);
		graphics.pose().scale(0.75F, 0.75F);
		graphics.text(font, tagline.copy().withStyle(ChatFormatting.BOLD), 0, 0, Ui.MUTED, false);
		graphics.pose().popMatrix();

		// you, bottom left: your head in a frame, your name, what you wear
		String name = minecraft.getUser().getName();
		int frameX = 14;
		int frameY = height - 50;
		Ui.card(graphics, frameX, frameY, 36, 36, 0xE00E1527, Ui.alpha(Ui.ACCENT, 0.7F));
		PlayerFaceExtractor.extractRenderState(graphics, PlayerPreview.skin(), frameX + 4, frameY + 4, 28);
		graphics.text(font, name, frameX + 44, frameY + 9, Ui.TEXT, false);
		graphics.text(font, Component.translatable("misty.title.wearing", cosmeticName(LauncherLink.cosmetic())), frameX + 44, frameY + 21, Ui.DIM, false);

		// your name above your head, like in the game, and what you wear under your feet
		if (panel(width)) {
			int center = characterCenter(width);
			int top = characterTop(height);
			int nameW = font.width(name) + 8;
			graphics.fill(center - nameW / 2, top - 12, center + nameW / 2, top - 1, 0x66000000);
			Ui.centered(graphics, font, Component.literal(name), center, top - 10, 0xFFFFFFFF);
			Ui.centered(graphics, font, cosmeticName(LauncherLink.cosmetic()), center, top + characterHeight(height) + 5, Ui.MUTED);
		}

		// versions, bottom right, above the copyright
		String versions = "Misty Client " + MistyClient.version() + " · Minecraft " + SharedConstants.getCurrentVersion().name();
		graphics.text(font, versions, width - 4 - font.width(versions), height - 23, Ui.DIM, false);
	}

	private static Component cosmeticName(final Cosmetic cosmetic) {
		return Component.translatable("misty.cosmetic." + (cosmetic == Cosmetic.NONE ? "none" : cosmetic.id()));
	}

	private static void scaledLeft(final GuiGraphicsExtractor graphics, final Font font, final Component text, final int x, final int y, final float scale, final int color) {
		graphics.pose().pushMatrix();
		graphics.pose().translate(x, y);
		graphics.pose().scale(scale, scale);
		graphics.text(font, text, 0, 0, color, false);
		graphics.pose().popMatrix();
	}

	/** A diamond (square turned 45°) of half-size {@code r}, with a border. */
	static void diamond(final GuiGraphicsExtractor graphics, final int cx, final int cy, final int r, final int border, final int inside) {
		for (int dy = -r; dy <= r; dy++) {
			int half = r - Math.abs(dy);
			graphics.fill(cx - half, cy + dy, cx + half + 1, cy + dy + 1, border);
			if (half > 0) {
				graphics.fill(cx - half + 1, cy + dy, cx + half, cy + dy + 1, inside);
			}
		}
	}

	/** Glowing line between two points: a soft halo and a bright core. */
	private static void line(final GuiGraphicsExtractor graphics, final int x0, final int y0, final int x1, final int y1) {
		int steps = Math.max(Math.abs(x1 - x0), Math.abs(y1 - y0));
		for (int s = 0; s <= steps; s++) {
			int x = Math.round(Mth.lerp((float)s / Math.max(1, steps), x0, x1));
			int y = Math.round(Mth.lerp((float)s / Math.max(1, steps), y0, y1));
			graphics.fill(x - 1, y - 1, x + 2, y + 2, Ui.alpha(Ui.ACCENT, 0.10F));
			graphics.fill(x, y, x + 1, y + 1, Ui.alpha(Ui.ACCENT, 0.65F));
		}
	}

	/**
	 * An entry of the menu: a glass blade pointing right, an icon in a diamond, its name and where
	 * it leads. It slides a little to the right under the mouse.
	 */
	private static final class Portal extends AbstractButton {
		private final Component subtitle;
		/** One of the Misty icons ({@code assets/misty/textures/gui/icons}), drawn in the theme's blues. */
		private final Identifier icon;
		private final Runnable onPress;
		private float slide;

		Portal(final int x, final int y, final int height, final String key, final String subtitleKey, final String icon, final Runnable onPress) {
			super(x, y, ENTRY_W, height, Component.translatable(key));
			this.subtitle = Component.translatable(subtitleKey);
			this.icon = MistyClient.id("textures/gui/icons/" + icon + ".png");
			this.onPress = onPress;
		}

		@Override
		public void onPress(final InputWithModifiers input) {
			this.onPress.run();
		}

		@Override
		protected void extractContents(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
			Font font = Minecraft.getInstance().font;
			boolean hot = this.active && this.isHoveredOrFocused();
			this.slide = Mth.lerp(0.25F, this.slide, hot ? 6.0F : 0.0F);
			int x = this.getX() + Math.round(this.slide);
			int y = this.getY();
			int h = this.getHeight();
			int w = ENTRY_W;
			float fade = this.active ? 1.0F : 0.45F;

			// the blade: straight on the left, a point on the right
			int body = hot ? 0xE81A2A55 : 0xC40B1226;
			int edge = hot ? Ui.ACCENT : 0x40FFFFFF;
			for (int r = 0; r < h; r++) {
				int inset = Math.abs(r * 2 - (h - 1)) * 5 / h;
				int right = x + w - 6 + (5 - inset);
				graphics.fill(x + 8, y + r, right, y + r + 1, Ui.alpha(body, fade * ((body >>> 24) / 255.0F)));
				graphics.fill(right, y + r, right + 1, y + r + 1, Ui.alpha(edge, fade));
			}
			graphics.fill(x + 8, y, x + w - 6, y + 1, Ui.alpha(edge, fade * 0.8F));
			graphics.fill(x + 8, y + h - 1, x + w - 6, y + h, Ui.alpha(edge, fade * 0.8F));
			if (hot) {
				// glow under the blade
				graphics.fill(x + 14, y + h, x + w - 12, y + h + 2, Ui.alpha(Ui.ACCENT, 0.25F));
			}

			// icon in a diamond, overlapping the blade's left end
			int cx = x + 12;
			int cy = y + h / 2;
			int radius = Math.min(13, h / 2 + 1);
			diamond(graphics, cx, cy, radius, Ui.alpha(hot ? Ui.ACCENT : 0xFF6F86B8, fade), Ui.alpha(0xFF101A33, fade));
			int size = radius >= 12 ? 16 : 12;
			graphics.blit(RenderPipelines.GUI_TEXTURED, this.icon, cx - size / 2, cy - size / 2, 0.0F, 0.0F, size, size, 16, 16, 16, 16, Ui.alpha(0xFFFFFFFF, fade));

			// name and where it leads
			int textColor = this.active ? (hot ? 0xFFFFFFFF : Ui.TEXT) : Ui.DIM;
			// short entries (low windows): the name alone, centred
			boolean withSubtitle = h >= 22;
			int nameY = withSubtitle ? y + (h - 17) / 2 : y + (h - 8) / 2;
			graphics.text(font, this.getMessage().copy().withStyle(ChatFormatting.BOLD), x + 30, nameY, textColor, false);
			if (withSubtitle) {
				graphics.pose().pushMatrix();
				graphics.pose().translate(x + 30, nameY + 11);
				graphics.pose().scale(0.75F, 0.75F);
				graphics.text(font, this.subtitle, 0, 0, this.active ? (hot ? Ui.ACCENT : Ui.MUTED) : Ui.DIM, false);
				graphics.pose().popMatrix();
			}
		}

		@Override
		protected void updateWidgetNarration(final NarrationElementOutput output) {
			this.defaultButtonNarrationText(output);
		}
	}
}
