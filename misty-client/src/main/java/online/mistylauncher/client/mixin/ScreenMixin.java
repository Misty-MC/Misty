package online.mistylauncher.client.mixin;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.Screen;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.menu.MistyBackground;
import org.jspecify.annotations.Nullable;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * The menus outside the game (singleplayer, multiplayer, options and all the others) over the
 * launcher's landscape, sharp, instead of Minecraft's blurred panorama. Their buttons, sliders,
 * text fields and lists take the Misty look from the mod's textures (assets/minecraft/…).
 * With the Misty main menu turned off, the game's own background stays.
 */
@Mixin(Screen.class)
abstract class ScreenMixin {
	@Shadow
	protected @Nullable Minecraft minecraft;

	@Inject(method = "extractPanorama", at = @At("HEAD"), cancellable = true)
	private void misty$panorama(final GuiGraphicsExtractor graphics, final float a, final CallbackInfo ci) {
		if (MistyClient.config().customMenu) {
			MistyBackground.draw(graphics, graphics.guiWidth(), graphics.guiHeight());
			ci.cancel();
		}
	}

	@Inject(method = "extractBlurredBackground", at = @At("HEAD"), cancellable = true)
	private void misty$blur(final GuiGraphicsExtractor graphics, final CallbackInfo ci) {
		// out of the game, the landscape stays sharp; in game, the world is blurred as usual
		if (MistyClient.config().customMenu && this.minecraft != null && this.minecraft.level == null) {
			ci.cancel();
		}
	}
}
