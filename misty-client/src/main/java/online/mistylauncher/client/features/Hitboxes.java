package online.mistylauncher.client.features;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.components.debug.DebugScreenEntries;
import net.minecraft.client.gui.components.debug.DebugScreenEntryStatus;

/**
 * Entity hitboxes, as F3 + B shows them, but always on: the switch of the Misty settings sets the
 * game's own "entity hitboxes" debug entry, so F3 + B and the switch always agree.
 */
public final class Hitboxes {
	private Hitboxes() {
	}

	public static boolean shown() {
		return Minecraft.getInstance().debugEntries.getStatus(DebugScreenEntries.ENTITY_HITBOXES) == DebugScreenEntryStatus.ALWAYS_ON;
	}

	public static void show(final boolean shown) {
		Minecraft minecraft = Minecraft.getInstance();
		minecraft.debugEntries.setStatus(DebugScreenEntries.ENTITY_HITBOXES, shown ? DebugScreenEntryStatus.ALWAYS_ON : DebugScreenEntryStatus.NEVER);
	}
}
