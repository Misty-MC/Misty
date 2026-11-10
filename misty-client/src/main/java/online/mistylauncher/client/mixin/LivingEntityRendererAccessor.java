package online.mistylauncher.client.mixin;

import net.minecraft.client.renderer.entity.LivingEntityRenderer;
import net.minecraft.client.renderer.entity.layers.RenderLayer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Invoker;

/** Adds a layer to a living entity renderer (the Misty hat on the players). */
@Mixin(LivingEntityRenderer.class)
public interface LivingEntityRendererAccessor {
	@Invoker("addLayer")
	@SuppressWarnings("rawtypes")
	boolean misty$addLayer(RenderLayer layer);
}
