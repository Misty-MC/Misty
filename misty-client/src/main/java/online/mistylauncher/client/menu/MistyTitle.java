package online.mistylauncher.client.menu;

import java.util.function.Consumer;
import net.minecraft.SharedConstants;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.components.PlainTextButton;
import net.minecraft.client.gui.components.PlayerFaceExtractor;
import net.minecraft.client.gui.components.Tooltip;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.TitleScreen;
import net.minecraft.client.gui.screens.CreditsAndAttributionScreen;
import net.minecraft.client.gui.screens.multiplayer.JoinMultiplayerScreen;
import net.minecraft.client.gui.screens.multiplayer.SafetyScreen;
import net.minecraft.client.gui.screens.options.AccessibilityOptionsScreen;
import net.minecraft.client.gui.screens.options.LanguageSelectScreen;
import net.minecraft.client.gui.screens.options.OptionsScreen;
import net.minecraft.client.gui.screens.worldselection.SelectWorldScreen;
import net.minecraft.network.chat.Component;
import net.minecraft.resources.Identifier;
import net.minecraft.world.entity.player.PlayerSkin;
import com.mojang.realmsclient.RealmsMainScreen;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.ui.MistyButton;
import online.mistylauncher.client.ui.MistySettingsScreen;
import online.mistylauncher.client.ui.Ui;
import org.jspecify.annotations.Nullable;

/**
 * The Misty main menu: a night-blue dock on the left with the cloud and the entries, the
 * panorama free on the right, the account in the top-right corner.
 */
public final class MistyTitle {
	private static final Component COPYRIGHT = Component.translatable("title.credits");
	private static final int DOCK = 148;
	private static final int BUTTON_WIDTH = 124;
	private static final int BUTTON_HEIGHT = 20;
	private static final int BUTTON_GAP = 4;
	private static final int ICON = 20;

	private MistyTitle() {
	}

	/** Y of the first menu entry: under the logo, centred in what is left on tall windows. */
	private static int entriesTop(final int height) {
		int entries = 5 * BUTTON_HEIGHT + 4 * BUTTON_GAP + 6;
		return Math.max(86, (height - entries) / 2 + 24);
	}

	public static void init(final TitleScreen screen, final int width, final int height, final @Nullable Component multiplayerDisabled, final Consumer<AbstractWidget> add) {
		Minecraft minecraft = Minecraft.getInstance();
		int x = (DOCK - BUTTON_WIDTH) / 2;
		int y = entriesTop(height);
		int step = BUTTON_HEIGHT + BUTTON_GAP;

		add.accept(entry(x, y, "menu.singleplayer", () -> minecraft.gui.setScreen(new SelectWorldScreen(screen))));

		MistyButton multiplayer = entry(x, y + step, "menu.multiplayer", () -> {
			Screen next = minecraft.options.skipMultiplayerWarning ? new JoinMultiplayerScreen(screen) : new SafetyScreen(screen);
			minecraft.gui.setScreen(next);
		});
		MistyButton realms = entry(x, y + 2 * step, "menu.online", () -> minecraft.gui.setScreen(new RealmsMainScreen(screen)));
		if (multiplayerDisabled != null) {
			multiplayer.active = false;
			realms.active = false;
			multiplayer.setTooltip(Tooltip.create(multiplayerDisabled));
			realms.setTooltip(Tooltip.create(multiplayerDisabled));
		}
		add.accept(multiplayer);
		add.accept(realms);

		add.accept(entry(x, y + 3 * step, "menu.options", () -> minecraft.gui.setScreen(new OptionsScreen(screen, minecraft.options))));
		// set apart from the rest, so it is not hit by mistake
		add.accept(entry(x, y + 4 * step + 6, "menu.quit", minecraft::stop));

		// top right, left of the account: language, accessibility, Misty settings
		int iconY = 8;
		int iconX = width - 8 - accountWidth(minecraft) - 3 * (ICON + 4);
		add.accept(icon(iconX, iconY, "options.language", Identifier.withDefaultNamespace("icon/language"),
			() -> minecraft.gui.setScreen(new LanguageSelectScreen(screen, minecraft.options, minecraft.getLanguageManager()))));
		add.accept(icon(iconX + ICON + 4, iconY, "options.accessibility", Identifier.withDefaultNamespace("icon/accessibility"),
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

	private static MistyButton entry(final int x, final int y, final String key, final Runnable onPress) {
		return new MistyButton(x, y, BUTTON_WIDTH, BUTTON_HEIGHT, Component.translatable(key), onPress).alignLeft();
	}

	private static MistyButton icon(final int x, final int y, final String key, final Identifier sprite, final Runnable onPress) {
		MistyButton button = new MistyButton(x, y, ICON, ICON, Component.translatable(key), onPress).icon(sprite);
		button.setTooltip(Tooltip.create(Component.translatable(key)));
		return button;
	}

	private static int accountWidth(final Minecraft minecraft) {
		return 6 + 12 + 5 + minecraft.font.width(minecraft.getUser().getName()) + 7;
	}

	/** Under the widgets: the dock and its veil over the panorama. */
	public static void background(final GuiGraphicsExtractor graphics, final int width, final int height) {
		graphics.fill(0, 0, DOCK, height, 0xD90A0F1D);
		graphics.fill(DOCK, 0, DOCK + 1, height, 0x59000000);
		// the panorama darkens towards the dock and towards the bottom, where the texts are
		graphics.fillGradient(DOCK + 1, height - 60, width, height, 0x000A0F1D, 0xA00A0F1D);
		graphics.fillGradient(DOCK + 1, 0, width, 44, 0x800A0F1D, 0x000A0F1D);
	}

	/** Over the widgets: logo, account, versions. */
	public static void foreground(final GuiGraphicsExtractor graphics, final Font font, final int width, final int height) {
		Minecraft minecraft = Minecraft.getInstance();
		int center = DOCK / 2;
		int top = entriesTop(height) - 80;

		Ui.logo(graphics, center - 20, top, 40);
		// pixel title with its slate shadow, as in the launcher
		Component misty = Component.literal("MISTY");
		Ui.centeredScaled(graphics, font, misty, center + 1, top + 41, 2.5F, Ui.SLATE, false);
		Ui.centeredScaled(graphics, font, misty, center, top + 40, 2.5F, 0xFFFFFFFF, false);
		Ui.centered(graphics, font, Component.literal("C L I E N T"), center, top + 63, Ui.ACCENT);

		// account: head, name
		String name = minecraft.getUser().getName();
		int accountWidth = accountWidth(minecraft);
		int accountX = width - 8 - accountWidth;
		Ui.card(graphics, accountX, 8, accountWidth, ICON, Ui.GLASS, Ui.BORDER);
		PlayerSkin skin = minecraft.getSkinManager().createLookup(minecraft.getGameProfile(), false).get();
		PlayerFaceExtractor.extractRenderState(graphics, skin, accountX + 6, 12, 12);
		graphics.text(font, name, accountX + 23, 14, Ui.TEXT, false);

		graphics.text(font, "Misty Client " + MistyClient.version(), 8, height - 22, Ui.MUTED, false);
		graphics.text(font, "Minecraft " + SharedConstants.getCurrentVersion().name(), 8, height - 12, Ui.DIM, false);
	}
}
