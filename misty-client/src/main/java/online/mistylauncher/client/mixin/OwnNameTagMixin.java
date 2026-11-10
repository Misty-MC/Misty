package online.mistylauncher.client.mixin;

import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.entity.LivingEntityRenderer;
import net.minecraft.world.entity.LivingEntity;
import online.mistylauncher.client.MistyClient;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Your own name above your head (seen in third person, F5), when the option is on. The game
 * never shows the name of the player you look through; hidden with F1 and when invisible, like
 * the others.
 */
@Mixin(LivingEntityRenderer.class)
abstract class OwnNameTagMixin {
	@Inject(method = "shouldShowName(Lnet/minecraft/world/entity/LivingEntity;D)Z", at = @At("RETURN"), cancellable = true)
	private void misty$own(final LivingEntity entity, final double distance, final CallbackInfoReturnable<Boolean> cir) {
		Minecraft minecraft = Minecraft.getInstance();
		if (!cir.getReturnValueZ() && MistyClient.config().ownNametag && entity == minecraft.player
			&& !online.mistylauncher.client.compat.Versions.guiHidden() && !entity.isInvisible() && !entity.isVehicle()) {
			cir.setReturnValue(true);
		}
	}
}
