package online.mistylauncher.client.mixin;

import net.minecraft.client.MouseHandler;
import net.minecraft.client.input.MouseButtonInfo;
import online.mistylauncher.client.hud.Clicks;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Counts the mouse clicks for the CPS module. */
@Mixin(MouseHandler.class)
abstract class MouseHandlerMixin {
	@Inject(method = "onButton", at = @At("HEAD"))
	private void misty$click(final long handle, final MouseButtonInfo button, final int action, final CallbackInfo ci) {
		if (action == 1) {
			Clicks.pressed(button.button());
		}
	}
}
