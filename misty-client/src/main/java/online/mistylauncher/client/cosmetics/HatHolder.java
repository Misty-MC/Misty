package online.mistylauncher.client.cosmetics;

/** Added to the game's player render state: the Misty hat the player wears ({@link CloudHatLayer}). */
public interface HatHolder {
	Cosmetic misty$hat();

	void misty$setHat(Cosmetic hat);
}
