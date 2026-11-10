package online.mistylauncher.client.cosmetics;

/** What a player wears: one cosmetic on the back (cape or elytra) and one on the head. */
public record Outfit(Cosmetic back, Cosmetic hat) {
	public static final Outfit NONE = new Outfit(Cosmetic.NONE, Cosmetic.NONE);

	public Outfit {
		back = back.slot() == Cosmetic.Slot.BACK ? back : Cosmetic.NONE;
		hat = hat.slot() == Cosmetic.Slot.HAT ? hat : Cosmetic.NONE;
	}

	/** From the ids the launcher and the Misty network use. */
	public static Outfit of(final String back, final String hat) {
		return new Outfit(Cosmetic.of(back), Cosmetic.of(hat));
	}

	/** This outfit with {@code cosmetic} in its place ({@code NONE} empties {@code slot}). */
	public Outfit with(final Cosmetic.Slot slot, final Cosmetic cosmetic) {
		return switch (slot) {
			case BACK -> new Outfit(cosmetic, this.hat);
			case HAT -> new Outfit(this.back, cosmetic);
			case NONE -> this;
		};
	}

	/** What is worn in {@code slot}. */
	public Cosmetic in(final Cosmetic.Slot slot) {
		return switch (slot) {
			case BACK -> this.back;
			case HAT -> this.hat;
			case NONE -> Cosmetic.NONE;
		};
	}
}
