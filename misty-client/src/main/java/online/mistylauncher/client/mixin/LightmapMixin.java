package online.mistylauncher.client.mixin;

import com.llamalad7.mixinextras.injector.ModifyExpressionValue;
import online.mistylauncher.client.MistyClient;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;

/**
 * Fullbright: the brightness the lightmap reads is pushed far above the game's maximum, so caves
 * and nights are lit. The option itself is not changed (nothing to restore).
 */
//? if >=26.1 {
@Mixin(net.minecraft.client.renderer.LightmapRenderStateExtractor.class)
//?} else {
/*@Mixin(net.minecraft.client.renderer.LightTexture.class)
*///?}
abstract class LightmapMixin {
	// The brightness (gamma) option: the first Double.floatValue() of extract() since 26.1, the
	// second of updateLightTexture() before (the first is the darkness effect)
	//? if >=26.1 {
	@ModifyExpressionValue(method = "extract", at = @At(value = "INVOKE", target = "Ljava/lang/Double;floatValue()F", ordinal = 0))
	//?} else {
	/*
	@ModifyExpressionValue(method = "updateLightTexture", at = @At(value = "INVOKE", target = "Ljava/lang/Double;floatValue()F", ordinal = 1))
	*///?}
	private float misty$fullbright(final float gamma) {
		return MistyClient.config().fullbright ? 15.0F : gamma;
	}
}
