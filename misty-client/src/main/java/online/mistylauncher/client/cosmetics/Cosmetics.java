package online.mistylauncher.client.cosmetics;

import java.util.UUID;
import net.minecraft.core.ClientAsset;
import net.minecraft.world.entity.player.PlayerSkin;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.net.MistyPlayers;

/** Puts the Misty cape and elytra on the players who wear them. */
public final class Cosmetics {
	/** {@code assets/misty/textures/cape/misty.png}: cape and elytra on one 64 × 32 texture, like Mojang's. */
	private static final ClientAsset.Texture MISTY = new ClientAsset.ResourceTexture(MistyClient.id("cape/misty"));

	private Cosmetics() {
	}

	/** The skin of a player, with their Misty cosmetic on top. */
	public static PlayerSkin dress(final UUID player, final PlayerSkin skin) {
		if (!MistyClient.config().cosmetics) {
			return skin;
		}
		return switch (MistyPlayers.cosmetic(player)) {
			case CAPE -> skin.cape() == MISTY ? skin : new PlayerSkin(skin.body(), MISTY, MISTY, skin.model(), skin.secure());
			case ELYTRA -> skin.elytra() == MISTY ? skin : new PlayerSkin(skin.body(), skin.cape(), MISTY, skin.model(), skin.secure());
			case NONE -> skin;
		};
	}
}
