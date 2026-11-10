package online.mistylauncher.client.mixin;

import net.minecraft.client.renderer.entity.state.AvatarRenderState;
import online.mistylauncher.client.cosmetics.Cosmetic;
import online.mistylauncher.client.cosmetics.HatHolder;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;

/** The Misty hat in the player render state. */
@Mixin(AvatarRenderState.class)
abstract class AvatarRenderStateMixin implements HatHolder {
	@Unique
	private Cosmetic misty$hat = Cosmetic.NONE;

	@Override
	public Cosmetic misty$hat() {
		return this.misty$hat;
	}

	@Override
	public void misty$setHat(final Cosmetic hat) {
		this.misty$hat = hat;
	}
}
