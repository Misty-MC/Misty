package online.mistylauncher.client.mixin;

import net.minecraft.client.renderer.entity.EntityRenderer;
import net.minecraft.network.chat.Component;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Player;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.net.MistyPlayers;
import online.mistylauncher.client.social.Badge;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** The Misty cloud in front of the Misty players' name above their head. */
@Mixin(EntityRenderer.class)
abstract class NameTagMixin {
	@Inject(method = "getNameTag", at = @At("RETURN"), cancellable = true)
	private void misty$badge(final Entity entity, final CallbackInfoReturnable<Component> cir) {
		Component name = cir.getReturnValue();
		if (name != null && entity instanceof Player player && MistyClient.config().nametagBadge && MistyPlayers.isMisty(player.getUUID())) {
			cir.setReturnValue(Badge.prefix(name));
		}
	}
}
