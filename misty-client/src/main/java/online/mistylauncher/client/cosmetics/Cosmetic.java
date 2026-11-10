package online.mistylauncher.client.cosmetics;

/** A Misty cosmetic a player wears. The id is what the launcher and the Misty network exchange. */
public enum Cosmetic {
	NONE(""),
	/** Misty cape (its elytra come with it, like the Mojang capes). */
	CAPE("misty_cape"),
	/** Misty elytra only: no cape on the back. */
	ELYTRA("misty_elytra");

	private final String id;

	Cosmetic(final String id) {
		this.id = id;
	}

	public String id() {
		return this.id;
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
