package online.mistylauncher.client.mixin;

import online.mistylauncher.client.compat.Versions;
import net.minecraft.client.gui.screens.PauseScreen;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;
import online.mistylauncher.client.ui.MistyButton;
import online.mistylauncher.client.ui.MistySettingsScreen;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** A way into the Misty settings from the pause menu, in the top-right corner. */
@Mixin(PauseScreen.class)
abstract class PauseScreenMixin extends Screen {
	@Shadow
	@Final
	private boolean showPauseMenu;

	private PauseScreenMixin(final Component title) {
		super(title);
	}

	@Inject(method = "init", at = @At("TAIL"))
	private void misty$button(final CallbackInfo ci) {
		if (this.showPauseMenu) {
			Component label = Component.translatable("misty.settings.title");
			int width = this.font.width(label) + 16;
			this.addRenderableWidget(new MistyButton(this.width - width - 8, 8, width, 18, label, () -> Versions.setScreen(new MistySettingsScreen(this))));
		}
	}
}
