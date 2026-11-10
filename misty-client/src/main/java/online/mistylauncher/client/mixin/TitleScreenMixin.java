package online.mistylauncher.client.mixin;

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
			ci.cancel();
		}
	}

	@Inject(method = "extractRenderState", at = @At("HEAD"), cancellable = true)
	private void misty$extract(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a, final CallbackInfo ci) {
		if (this.misty$active != MistyClient.config().customMenu) {
			this.rebuildWidgets();
		}
		if (this.misty$active) {
			this.extractPanorama(graphics, a);
			MistyTitle.background(graphics, this.width, this.height);
			super.extractRenderState(graphics, mouseX, mouseY, a);
			MistyTitle.foreground(graphics, this.font, this.width, this.height);
			ci.cancel();
		}
	}
}
