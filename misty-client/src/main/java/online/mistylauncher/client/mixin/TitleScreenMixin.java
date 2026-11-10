package online.mistylauncher.client.mixin;

import online.mistylauncher.client.compat.Versions;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.TitleScreen;
import net.minecraft.network.chat.Component;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.menu.MistyTitle;
import org.jspecify.annotations.Nullable;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Misty main menu. The title screen stays the game's own (everything that opens it keeps
 * working): only its widgets and its drawing are replaced, and only while the option is on.
 */
@Mixin(TitleScreen.class)
abstract class TitleScreenMixin extends Screen {
	/** Whether the widgets on screen are Misty's (the option can change while the menu is open). */
	@Unique
	private boolean misty$active;
	@Unique
	private static boolean misty$wardrobeShown;

	private TitleScreenMixin(final Component title) {
		super(title);
	}

	@Shadow
	private @Nullable Component getMultiplayerDisabledReason() {
		throw new AssertionError();
	}

	@Inject(method = "init", at = @At("HEAD"), cancellable = true)
	private void misty$init(final CallbackInfo ci) {
		this.misty$active = MistyClient.config().customMenu;
		if (this.misty$active) {
			MistyTitle.init((TitleScreen)(Object)this, this.width, this.height, this.getMultiplayerDisabledReason(), this::addRenderableWidget);
			// development: -Dmisty.debug.open=<screen> opens a screen once at start, to look at it
			// (wardrobe, settings, options, singleplayer, multiplayer)
			String open = System.getProperty("misty.debug.open", Boolean.getBoolean("misty.debug.wardrobe") ? "wardrobe" : "");
			if (!open.isEmpty() && !misty$wardrobeShown) {
				misty$wardrobeShown = true;
				TitleScreen self = (TitleScreen)(Object)this;
				net.minecraft.client.gui.screens.Screen next = switch (open) {
					case "wardrobe" -> new online.mistylauncher.client.ui.WardrobeScreen(self);
					case "settings" -> new online.mistylauncher.client.ui.MistySettingsScreen(self);
					case "editor" -> new online.mistylauncher.client.hud.HudEditorScreen(self);
					case "options" -> online.mistylauncher.client.compat.Versions.options(self);
					case "singleplayer" -> new net.minecraft.client.gui.screens.worldselection.SelectWorldScreen(self);
					case "multiplayer" -> new net.minecraft.client.gui.screens.multiplayer.JoinMultiplayerScreen(self);
					default -> null;
				};
				if (next != null) {
					Versions.setScreen(next);
				}
			}
			ci.cancel();
		}
	}

	/** Classic menu (custom menu off): a way back into the Misty settings, top right. */
	@Inject(method = "init", at = @At("TAIL"))
	private void misty$settingsButton(final CallbackInfo ci) {
		Component label = Component.translatable("misty.settings.title");
		int width = this.font.width(label) + 16;
		TitleScreen self = (TitleScreen)(Object)this;
		this.addRenderableWidget(new online.mistylauncher.client.ui.MistyButton(this.width - width - 8, 8, width, 18, label,
			() -> Versions.setScreen(new online.mistylauncher.client.ui.MistySettingsScreen(self))));
	}

	@Inject(method = "extractRenderState", at = @At("HEAD"), cancellable = true)
	private void misty$extract(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a, final CallbackInfo ci) {
		if (this.misty$active != MistyClient.config().customMenu) {
			this.rebuildWidgets();
		}
		if (this.misty$active) {
			// the launcher's landscape instead of the Minecraft panorama
			MistyTitle.background(graphics, this.width, this.height);
			super.extractRenderState(graphics, mouseX, mouseY, a);
			MistyTitle.foreground(graphics, this.font, this.width, this.height);
			ci.cancel();
		}
	}
}
