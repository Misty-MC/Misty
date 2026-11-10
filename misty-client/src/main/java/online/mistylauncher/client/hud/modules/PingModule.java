package online.mistylauncher.client.hud.modules;

import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientPacketListener;
import net.minecraft.client.multiplayer.PlayerInfo;
import net.minecraft.network.chat.Component;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.hud.TextModule;

/** Latency to the server, as the server reports it in the player list. */
public final class PingModule extends TextModule {
	public PingModule() {
		super("ping", new MistyConfig.Module(true, 0, 4, 20), 50);
	}

	@Override
	public boolean available(final Minecraft minecraft) {
		return !minecraft.isLocalServer() && minecraft.getConnection() != null;
	}

	@Override
	protected Component text(final Minecraft minecraft, final boolean preview) {
		int ping = 0;
		ClientPacketListener connection = minecraft.getConnection();
		if (connection != null && minecraft.player != null) {
			PlayerInfo info = connection.getPlayerInfo(minecraft.player.getUUID());
			if (info != null) {
				ping = info.getLatency();
			}
		}
		return value(Integer.toString(ping), "ms");
	}
}
