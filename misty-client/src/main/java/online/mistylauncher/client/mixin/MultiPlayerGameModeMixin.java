package online.mistylauncher.client.mixin;

import net.minecraft.client.multiplayer.MultiPlayerGameMode;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Player;
import online.mistylauncher.client.hud.Combo;
import online.mistylauncher.client.hud.Reach;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Each player the local player hits: its distance (Reach module), and the combo (Combo module). */
@Mixin(MultiPlayerGameMode.class)
abstract class MultiPlayerGameModeMixin {
	@Inject(method = "attack", at = @At("HEAD"))
	private void misty$attack(final Player player, final Entity target, final CallbackInfo ci) {
		Reach.attacked(player, target);
		Combo.attacked(player, target);
	}
}
