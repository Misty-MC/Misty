package online.mistylauncher.client.mixin;

import online.mistylauncher.client.features.Zoom;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** Zoom: the field of view divided by the magnification. */
//? if >=26.1 {
@Mixin(net.minecraft.client.Camera.class)
//?} else {
/*@Mixin(net.minecraft.client.renderer.GameRenderer.class)
*///?}
abstract class CameraMixin {
	//? if >=26.1 {
	@Inject(method = "calculateFov", at = @At("RETURN"), cancellable = true)
	private void misty$zoom(final float partialTicks, final CallbackInfoReturnable<Float> cir) {
	//?} else {
	/*@Inject(method = "getFov", at = @At("RETURN"), cancellable = true)
	private void misty$zoom(final net.minecraft.client.Camera camera, final float partialTicks, final boolean useFovSetting, final CallbackInfoReturnable<Float> cir) {
	*///?}
		float factor = Zoom.factor();
		// never hand the game a broken field of view: its culling loops forever on one
		if (factor > 1.0F && Float.isFinite(factor)) {
			cir.setReturnValue(cir.getReturnValueF() / factor);
		}
	}
}
