package online.mistylauncher.client.cosmetics;

import java.util.UUID;
import net.minecraft.world.entity.player.PlayerSkin;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.net.MistyPlayers;
import org.jspecify.annotations.Nullable;

/**
 * Puts the Misty capes and elytra on the players who wear them (the hats are a layer: {@link CloudHatLayer}).
 *
 * <p>Since 1.21.9 a skin's textures are {@code ClientAsset.Texture}; before, plain texture ids.
 */
public final class Cosmetics {
	//? if >=1.21.9 {
	/** {@code assets/misty/textures/cape/misty.png}: cape and elytra on one 64 × 32 texture, like Mojang's. */
	private static final net.minecraft.core.ClientAsset.Texture MISTY = new net.minecraft.core.ClientAsset.ResourceTexture(MistyClient.id("cape/misty"));
	/** {@code assets/misty/textures/cape/early.png}: the early player cape and its elytra. */
	private static final net.minecraft.core.ClientAsset.Texture EARLY = new net.minecraft.core.ClientAsset.ResourceTexture(MistyClient.id("cape/early"));
	//?} else {
	/*private static final net.minecraft.resources.Identifier MISTY = MistyClient.id("textures/cape/misty.png");
	private static final net.minecraft.resources.Identifier EARLY = MistyClient.id("textures/cape/early.png");
	*///?}

	private Cosmetics() {
	}

	//? if >=1.21.9 {
	/** The Misty cape and elytra texture (shown in the menu and the wardrobe). */
	public static net.minecraft.core.ClientAsset.Texture texture() {
		return MISTY;
	}

	/** The texture of a cape or elytra cosmetic, null for the others. */
	public static net.minecraft.core.ClientAsset.@Nullable Texture texture(final Cosmetic cosmetic) {
	//?} else {
	/*public static net.minecraft.resources.Identifier texture() {
		return MISTY;
	}

	public static net.minecraft.resources.@Nullable Identifier texture(final Cosmetic cosmetic) {
	*///?}
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
		var texture = texture(back);
		if (texture == null) {
			return skin;
		}
		//? if >=1.21.9 {
		if (back == Cosmetic.ELYTRA) {
			return skin.elytra() == texture ? skin : new PlayerSkin(skin.body(), skin.cape(), texture, skin.model(), skin.secure());
		}
		return skin.cape() == texture ? skin : new PlayerSkin(skin.body(), texture, texture, skin.model(), skin.secure());
		//?} else {
		/*if (back == Cosmetic.ELYTRA) {
			return skin.elytraTexture() == texture ? skin : new PlayerSkin(skin.texture(), skin.textureUrl(), skin.capeTexture(), texture, skin.model(), skin.secure());
		}
		return skin.capeTexture() == texture ? skin : new PlayerSkin(skin.texture(), skin.textureUrl(), texture, texture, skin.model(), skin.secure());
		*///?}
	}

	/** Where the texture of the Misty cape (or of this cape) is, to draw it in a screen. */
	public static net.minecraft.resources.Identifier texturePath() {
		return texturePath(Cosmetic.CAPE);
	}

	public static net.minecraft.resources.Identifier texturePath(final Cosmetic cosmetic) {
		var texture = texture(cosmetic);
		if (texture == null) {
			texture = MISTY;
		}
		//? if >=1.21.9 {
		return texture.texturePath();
		//?} else {
		/*return texture;
		*///?}
	}

	/** Is there a cape in this skin? */
	public static boolean hasCape(final PlayerSkin skin) {
		//? if >=1.21.9 {
		return skin.cape() != null;
		//?} else {
		/*return skin.capeTexture() != null;
		*///?}
	}
}
