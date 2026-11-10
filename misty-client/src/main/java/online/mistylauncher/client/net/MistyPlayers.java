package online.mistylauncher.client.net;

import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import net.minecraft.client.Minecraft;
import online.mistylauncher.client.config.LauncherLink;
import online.mistylauncher.client.cosmetics.Cosmetic;
import online.mistylauncher.client.cosmetics.Outfit;

/**
 * Who plays with Misty among the players around, and what they wear. Filled by
 * {@link MistyNetwork} from its own thread, read by the game thread.
 */
public final class MistyPlayers {
	/** An answer of the Misty network about one player ({@code misty} false: not a Misty player). */
	record Entry(boolean misty, Outfit outfit, long fetchedAt) {
	}

	private static final Map<UUID, Entry> KNOWN = new ConcurrentHashMap<>();

	private MistyPlayers() {
	}

	private static boolean isSelf(final UUID id) {
		return id.equals(Minecraft.getInstance().getUser().getProfileId());
	}

	/** True for yourself and for the players the Misty network reports as online with Misty. */
	public static boolean isMisty(final UUID id) {
		if (isSelf(id)) {
			return true;
		}
		Entry entry = KNOWN.get(id);
		return entry != null && entry.misty();
	}

	/** What a player wears (yourself: what the launcher or the wardrobe chose). */
	public static Outfit outfit(final UUID id) {
		if (isSelf(id)) {
			return LauncherLink.outfit();
		}
		Entry entry = KNOWN.get(id);
		return entry != null && entry.misty() ? entry.outfit() : Outfit.NONE;
	}

	/** What a player wears on the back. */
	public static Cosmetic cosmetic(final UUID id) {
		return outfit(id).back();
	}

	static Entry get(final UUID id) {
		return KNOWN.get(id);
	}

	static void put(final UUID id, final boolean misty, final Outfit outfit) {
		KNOWN.put(id, new Entry(misty, outfit, System.currentTimeMillis()));
	}

	static void clear() {
		KNOWN.clear();
	}
}
