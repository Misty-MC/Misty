package online.mistylauncher.client.cosmetics;

/** A Misty cosmetic a player wears. The id is what the launcher and the Misty network exchange. */
public enum Cosmetic {
	NONE("", Slot.NONE),
	/** Misty cape (its elytra come with it, like the Mojang capes). */
	CAPE("misty_cape", Slot.BACK),
	/** Misty elytra only: no cape on the back. */
	ELYTRA("misty_elytra", Slot.BACK),
	/** Cape of the first Misty players (and its elytra): the Misty site says who may wear it. */
	EARLY_CAPE("early_cape", Slot.BACK),
	/** A little cloud floating over the head. */
	CLOUDLY("cloudly", Slot.HAT);

	/** Where a cosmetic is worn: one cosmetic per place. */
	public enum Slot {
		NONE, BACK, HAT
	}

	private final String id;
	private final Slot slot;

	Cosmetic(final String id, final Slot slot) {
		this.id = id;
		this.slot = slot;
	}

	public String id() {
		return this.id;
	}

	public Slot slot() {
		return this.slot;
	}

	public static Cosmetic of(final String id) {
		for (Cosmetic cosmetic : values()) {
			if (cosmetic.id.equals(id)) {
				return cosmetic;
			}
		}
		return NONE;
	}
}
