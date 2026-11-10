package online.mistylauncher.client.mixin;

import com.llamalad7.mixinextras.injector.ModifyExpressionValue;
import net.minecraft.client.MouseHandler;
import net.minecraft.client.input.MouseButtonInfo;
import online.mistylauncher.client.features.Zoom;
import online.mistylauncher.client.hud.Clicks;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Counts the mouse clicks for the CPS module; the zoom slows the mouse and takes the wheel. */
@Mixin(MouseHandler.class)
abstract class MouseHandlerMixin {
	@Inject(method = "onButton", at = @At("HEAD"))
	private void misty$click(final long handle, final MouseButtonInfo button, final int action, final CallbackInfo ci) {
		if (action == 1) {
			Clicks.pressed(button.button());
		}
	}

	/** Zoomed in: the mouse turns slower (the sensitivity option read by turnPlayer). */
	@ModifyExpressionValue(method = "turnPlayer", at = @At(value = "INVOKE", target = "Ljava/lang/Double;doubleValue()D", ordinal = 0))
	private double misty$zoomSensitivity(final double sensitivity) {
		return sensitivity * Zoom.sensitivity();
	}

	/** The wheel while zooming changes the zoom instead of the hotbar slot. */
	@Inject(method = "onScroll", at = @At("HEAD"), cancellable = true)
	private void misty$zoomScroll(final long handle, final double scrollX, final double scrollY, final CallbackInfo ci) {
		if (Zoom.scroll(scrollY)) {
			ci.cancel();
		}
	}
}
