package online.mistylauncher.client.hud;

import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Player;

/** Distance of the last player the local player hit, for the Reach module. */
public final class Reach {
	/** In blocks, from the eyes to the nearest point of the target's hitbox; negative: no hit yet. */
	private static double last = -1.0;

	private Reach() {
	}

	/** The local player attacks {@code target} (called before the attack packet is sent). */
	public static void attacked(final Player player, final Entity target) {
		if (target instanceof Player && target != player) {
			last = Math.sqrt(target.getBoundingBox().distanceToSqr(player.getEyePosition()));
		}
	}

	public static double last() {
		return last;
	}
}
