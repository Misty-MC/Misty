package online.mistylauncher.client.mixin;

import com.llamalad7.mixinextras.injector.ModifyExpressionValue;
import net.minecraft.client.renderer.LightmapRenderStateExtractor;
import online.mistylauncher.client.MistyClient;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;

/**
 * Fullbright: the brightness the lightmap reads is pushed far above the game's maximum, so caves
 * and nights are lit. The option itself is not changed (nothing to restore).
 */
@Mixin(LightmapRenderStateExtractor.class)
abstract class LightmapMixin {
	/** The first Double.floatValue() of extract() is the brightness (gamma) option. */
	@ModifyExpressionValue(method = "extract", at = @At(value = "INVOKE", target = "Ljava/lang/Double;floatValue()F", ordinal = 0))
	private float misty$fullbright(final float gamma) {
		return MistyClient.config().fullbright ? 15.0F : gamma;
	}
}
