package online.mistylauncher.client.cosmetics;

import java.util.UUID;
import net.minecraft.core.ClientAsset;
import net.minecraft.world.entity.player.PlayerSkin;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.net.MistyPlayers;
import org.jspecify.annotations.Nullable;

/** Puts the Misty capes and elytra on the players who wear them (the hats are a layer: {@link CloudHatLayer}). */
public final class Cosmetics {
	/** {@code assets/misty/textures/cape/misty.png}: cape and elytra on one 64 × 32 texture, like Mojang's. */
	private static final ClientAsset.Texture MISTY = new ClientAsset.ResourceTexture(MistyClient.id("cape/misty"));
	/** {@code assets/misty/textures/cape/early.png}: the early player cape and its elytra. */
	private static final ClientAsset.Texture EARLY = new ClientAsset.ResourceTexture(MistyClient.id("cape/early"));

	private Cosmetics() {
	}

	/** The Misty cape and elytra texture (shown in the menu and the wardrobe). */
	public static ClientAsset.Texture texture() {
		return MISTY;
	}

	/** The texture of a cape or elytra cosmetic, null for the others. */
	public static ClientAsset.@Nullable Texture texture(final Cosmetic cosmetic) {
		return switch (cosmetic) {
			case CAPE, ELYTRA -> MISTY;
			case EARLY_CAPE -> EARLY;
			case NONE, CLOUDLY -> null;
		};
	}

	/** The skin of a player, with their Misty cape or elytra on top. */
	public static PlayerSkin dress(final UUID player, final PlayerSkin skin) {
		if (!MistyClient.config().cosmetics) {
			return skin;
		}
		return wear(MistyPlayers.cosmetic(player), skin);
	}

	/** A skin with this cosmetic on the back. */
	public static PlayerSkin wear(final Cosmetic back, final PlayerSkin skin) {
		ClientAsset.Texture texture = texture(back);
		if (texture == null) {
			return skin;
		}
		if (back == Cosmetic.ELYTRA) {
			return skin.elytra() == texture ? skin : new PlayerSkin(skin.body(), skin.cape(), texture, skin.model(), skin.secure());
		}
		return skin.cape() == texture ? skin : new PlayerSkin(skin.body(), texture, texture, skin.model(), skin.secure());
	}
}
