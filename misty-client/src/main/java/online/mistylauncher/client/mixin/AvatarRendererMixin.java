package online.mistylauncher.client.mixin;

import net.minecraft.client.model.player.PlayerModel;
import net.minecraft.client.renderer.entity.EntityRendererProvider;
import net.minecraft.client.renderer.entity.RenderLayerParent;
import net.minecraft.client.renderer.entity.player.AvatarRenderer;
import net.minecraft.client.renderer.entity.state.AvatarRenderState;
import net.minecraft.world.entity.player.Player;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.cosmetics.CloudHatLayer;
import online.mistylauncher.client.cosmetics.Cosmetic;
import online.mistylauncher.client.cosmetics.HatHolder;
import online.mistylauncher.client.net.MistyPlayers;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** The Misty hat on the players: its layer, and which hat each player wears. */
@Mixin(AvatarRenderer.class)
abstract class AvatarRendererMixin {
	@Inject(method = "<init>", at = @At("TAIL"))
	@SuppressWarnings("unchecked")
	private void misty$hatLayer(final EntityRendererProvider.Context context, final boolean slim, final CallbackInfo ci) {
		RenderLayerParent<AvatarRenderState, PlayerModel> parent = (RenderLayerParent<AvatarRenderState, PlayerModel>)(Object)this;
		((LivingEntityRendererAccessor)this).misty$addLayer(new CloudHatLayer(parent));
	}

	// the method name is split in two: stonecutter.gradle.kts renames the screens' "extractRenderState", not this one
	//? if >=1.21.9 {
	@Inject(method = "extract" + "RenderState(Lnet/minecraft/world/entity/Avatar;Lnet/minecraft/client/renderer/entity/state/AvatarRenderState;F)V", at = @At("TAIL"))
	private void misty$hat(final net.minecraft.world.entity.Avatar entity, final AvatarRenderState state, final float partialTicks, final CallbackInfo ci) {
	//?} else {
	/*@Inject(method = "extract" + "RenderState(Lnet/minecraft/client/player/AbstractClientPlayer;Lnet/minecraft/client/renderer/entity/state/PlayerRenderState;F)V", at = @At("TAIL"))
	private void misty$hat(final net.minecraft.client.player.AbstractClientPlayer entity, final AvatarRenderState state, final float partialTicks, final CallbackInfo ci) {
	*///?}
		Cosmetic hat = entity instanceof Player player && MistyClient.config().cosmetics ? MistyPlayers.outfit(player.getUUID()).hat() : Cosmetic.NONE;
		((HatHolder)state).misty$setHat(hat);
	}
}
