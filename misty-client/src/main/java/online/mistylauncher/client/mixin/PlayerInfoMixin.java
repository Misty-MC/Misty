package online.mistylauncher.client.mixin;

import com.mojang.authlib.GameProfile;
import net.minecraft.client.multiplayer.PlayerInfo;
import net.minecraft.world.entity.player.PlayerSkin;
import online.mistylauncher.client.cosmetics.Cosmetics;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** Misty cape and elytra: the skin every renderer asks for goes through here. */
@Mixin(PlayerInfo.class)
abstract class PlayerInfoMixin {
	@Shadow
	@Final
	private GameProfile profile;

	@Inject(method = "getSkin", at = @At("RETURN"), cancellable = true)
	private void misty$dress(final CallbackInfoReturnable<PlayerSkin> cir) {
		cir.setReturnValue(Cosmetics.dress(this.profile.id(), cir.getReturnValue()));
	}
}
