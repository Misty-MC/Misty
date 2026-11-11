package online.mistylauncher.client.social;

import java.util.Collection;
import java.util.UUID;
import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientPacketListener;
import net.minecraft.client.multiplayer.PlayerInfo;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.Style;
import online.mistylauncher.client.compat.Versions;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.net.MistyPlayers;
import org.jspecify.annotations.Nullable;

/** The Misty cloud shown in front of the Misty players' names (player list and chat). */
public final class Badge {
	/** The cloud is a glyph of the font {@code misty:icons}: it flows with the text everywhere. */
	private static final Component ICON = Component.literal("")
		//? if >=1.21.9 {
		.withStyle(Style.EMPTY.withFont(new net.minecraft.network.chat.FontDescription.Resource(MistyClient.id("icons"))).withColor(0xFFFFFF));
		//?} else {
		/*.withStyle(Style.EMPTY.withFont(MistyClient.id("icons")).withColor(0xFFFFFF));
		*///?}
	/** A chat line is "name, then one of these, then the message" on about every server. */
	private static final String SEPARATORS = ":>»→›";
	/** The name must start within this many characters (ranks and prefixes come before it). */
	private static final int NAME_WITHIN = 48;

	/** Sender of the signed chat message being shown (set by the chat listener mixin). */
	private static @Nullable UUID sender;

	private Badge() {
	}

	public static Component prefix(final Component text) {
		return Component.empty().append(ICON).append(" ").append(text);
	}

	/** Name of the player list. */
	public static Component playerList(final PlayerInfo info, final Component name) {
		return MistyPlayers.isMisty(Versions.id(info.getProfile())) ? prefix(name) : name;
	}

	public static void sender(final @Nullable UUID id) {
		sender = id;
	}

	/** A chat line about to be added to the chat. */
	public static Component chat(final Component message) {
		if (sender != null) {
			return MistyPlayers.isMisty(sender) ? prefix(message) : message;
		}
		return speakerIsMisty(message.getString()) ? prefix(message) : message;
	}

	/**
	 * Most servers format the chat themselves and send it as a plain system message: the
	 * speaker is then the first player of the list whose name stands before a separator.
	 */
	private static boolean speakerIsMisty(final String line) {
		ClientPacketListener connection = Minecraft.getInstance().getConnection();
		if (connection == null) {
			return false;
		}
		Collection<PlayerInfo> players = connection.getOnlinePlayers();
		int first = Integer.MAX_VALUE;
		boolean misty = false;
		for (PlayerInfo info : players) {
			int at = nameBeforeSeparator(line, Versions.name(info.getProfile()));
			if (at >= 0 && at < first) {
				first = at;
				misty = MistyPlayers.isMisty(Versions.id(info.getProfile()));
			}
		}
		return misty;
	}

	/** Position of {@code name} as a whole word followed by a separator, or -1. */
	private static int nameBeforeSeparator(final String line, final String name) {
		if (name.isEmpty()) {
			return -1;
		}
		int at = line.indexOf(name);
		if (at < 0 || at > NAME_WITHIN) {
			return -1;
		}
		int end = at + name.length();
		if (at > 0 && isNameChar(line.charAt(at - 1)) || end < line.length() && isNameChar(line.charAt(end))) {
			return -1;
		}
		for (int i = end; i < line.length() && i < end + 6; i++) {
			if (SEPARATORS.indexOf(line.charAt(i)) >= 0) {
				return at;
			}
		}
		return -1;
	}

	private static boolean isNameChar(final char c) {
		return Character.isLetterOrDigit(c) || c == '_';
	}
}
