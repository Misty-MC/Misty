package online.mistylauncher.client.mixin;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.LoadingOverlay;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.server.packs.resources.ReloadInstance;
import net.minecraft.util.Mth;
import net.minecraft.util.Util;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.menu.MistyLoading;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Misty loading screen, instead of Mojang's red one. The game's own logic is kept as it is
 * (the screen underneath during the fades, the smoothed progress, closing two seconds after the
 * fade out starts): only the drawing is Misty's. With the Misty main menu turned off, the game's
 * screen stays.
 */
@Mixin(LoadingOverlay.class)
abstract class LoadingOverlayMixin {
	@Shadow
	@Final
	private Minecraft minecraft;
	@Shadow
	@Final
	private ReloadInstance reload;
	@Shadow
	@Final
	private boolean fadeIn;
	@Shadow
	private float currentProgress;
	@Shadow
	private long fadeOutStart;
	@Shadow
	private long fadeInStart;

	@Inject(method = "extractRenderState", at = @At("HEAD"), cancellable = true)
	private void misty$extract(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a, final CallbackInfo ci) {
		if (!MistyClient.config().customMenu) {
			return;
		}
		long now = Util.getMillis();
		if (this.fadeIn && this.fadeInStart == -1L) {
			this.fadeInStart = now;
		}
		float fadeOut = this.fadeOutStart > -1L ? (now - this.fadeOutStart) / 1000.0F : -1.0F;
		float fadeInAnim = this.fadeInStart > -1L ? (now - this.fadeInStart) / 500.0F : -1.0F;
		Screen screen = this.minecraft.gui.screen();

		// same fades as the game: what is underneath shows through while fading
		float alpha;
		if (fadeOut >= 1.0F) {
			if (screen != null) {
				screen.extractRenderStateWithTooltipAndSubtitles(graphics, 0, 0, a);
			} else {
				this.minecraft.gui.hud.extractDeferredSubtitles();
			}
			alpha = 1.0F - Mth.clamp(fadeOut - 1.0F, 0.0F, 1.0F);
		} else if (this.fadeIn) {
			if (screen != null && fadeInAnim < 1.0F) {
				screen.extractRenderStateWithTooltipAndSubtitles(graphics, mouseX, mouseY, a);
			} else {
				this.minecraft.gui.hud.extractDeferredSubtitles();
			}
			alpha = Mth.clamp(fadeInAnim, 0.15F, 1.0F);
		} else {
			alpha = 1.0F;
		}

		graphics.nextStratum();
		// smoothed progress, as the game does it
		float actual = this.reload.getActualProgress();
		this.currentProgress = Mth.clamp(this.currentProgress * 0.95F + actual * 0.050000012F, 0.0F, 1.0F);
		MistyLoading.draw(graphics, graphics.guiWidth(), graphics.guiHeight(), alpha, this.currentProgress);

		if (fadeOut >= 2.0F) {
			this.minecraft.gui.setOverlay(null);
		}
		ci.cancel();
	}
}
