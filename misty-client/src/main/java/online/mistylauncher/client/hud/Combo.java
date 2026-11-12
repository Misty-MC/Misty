package online.mistylauncher.client.hud;

import net.minecraft.client.Minecraft;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.util.Util;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.player.Player;
import org.jspecify.annotations.Nullable;

/**
 * Combo of the Combo module: players hit in a row without being hit back.
 *
 * <p>Combat since 1.9 has an attack cooldown and invulnerability frames: clicking fast sends
 * attacks that do nothing. So an attack only counts when it lands, that is when the server shows
 * the target hurt (its hurt time starts again) shortly after the attack. Being hurt yourself ends
 * the combo, and so does a few seconds without landing a hit.
 */
public final class Combo {
	/** An attack that has not made the target hurt after this long missed (blocked, invulnerable…). */
	private static final long LAND_WINDOW_MS = 600;
	/** The combo ends after this long without a hit landing. */
	private static final long TIMEOUT_MS = 3000;

	private static int combo;
	private static long lastLanded;
	private static @Nullable LivingEntity pending;
	private static long pendingAt;
	private static int pendingHurt;
	private static int ownHurt;

	private Combo() {
	}

	/** The local player attacks {@code target} (called before the attack is sent). */
	public static void attacked(final Player player, final Entity target) {
		if (target instanceof Player living && target != player) {
			pending = living;
			pendingAt = Util.getMillis();
			pendingHurt = ((LivingEntity)living).hurtTime;
		}
	}

	/** Game thread, every tick. */
	public static void tick(final Minecraft minecraft) {
		LocalPlayer player = minecraft.player;
		if (player == null) {
			combo = 0;
			pending = null;
			ownHurt = 0;
			return;
		}
		long now = Util.getMillis();
		// hurt yourself (the hurt time starts again): the combo is over
		if (player.hurtTime > ownHurt) {
			combo = 0;
			pending = null;
		}
		ownHurt = player.hurtTime;

		LivingEntity target = pending;
		if (target != null) {
			if (target.hurtTime > pendingHurt) {
				combo++;
				lastLanded = now;
				pending = null;
			} else if (now - pendingAt > LAND_WINDOW_MS || target.isRemoved()) {
				pending = null;
			} else {
				pendingHurt = target.hurtTime;
			}
		}
		if (combo > 0 && now - lastLanded > TIMEOUT_MS) {
			combo = 0;
		}
	}

	public static int combo() {
		return combo;
	}
}
