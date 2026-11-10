package online.mistylauncher.client.mixin;

import net.minecraft.client.Camera;
import online.mistylauncher.client.features.Zoom;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** Zoom: the field of view divided by the magnification. */
@Mixin(Camera.class)
abstract class CameraMixin {
	@Inject(method = "calculateFov", at = @At("RETURN"), cancellable = true)
	private void misty$zoom(final float partialTicks, final CallbackInfoReturnable<Float> cir) {
		float factor = Zoom.factor();
		// never hand the game a broken field of view: its culling loops forever on one
		if (factor > 1.0F && Float.isFinite(factor)) {
			cir.setReturnValue(cir.getReturnValueF() / factor);
		}
	}
}
