package online.mistylauncher.client.hud.modules;

import java.util.List;
import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientPacketListener;
import net.minecraft.client.multiplayer.PlayerInfo;
import net.minecraft.network.chat.Component;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.hud.Ping;
import online.mistylauncher.client.hud.TextModule;
import online.mistylauncher.client.ui.Ui;

/** Latency to the server, measured every second (see Ping). */
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
		// measured by the mod every second; the player list (refreshed by the server every few
		// seconds) only for the servers that do not answer
		int ping = Math.max(0, Ping.measured());
		ClientPacketListener connection = minecraft.getConnection();
		if (ping == 0 && connection != null && minecraft.player != null) {
			PlayerInfo info = connection.getPlayerInfo(minecraft.player.getUUID());
			if (info != null) {
				ping = info.getLatency();
			}
		}
		// the editor shows a typical value while not connected
		if (preview && ping == 0) {
			ping = 42;
		}
		return value(Integer.toString(ping), this.setting(COLOR) ? color(ping) : Ui.TEXT, "ms");
	}

	private static final String COLOR = "color";

	@Override
	public List<Setting> settings() {
		return List.of(new Setting(COLOR, true));
	}

	/** Green when the connection is good, then yellow, orange and red as the ping grows. */
	private static int color(final int ping) {
		if (ping <= 0) {
			return Ui.TEXT; // not known yet
		} else if (ping < 80) {
			return 0x55E07A;
		} else if (ping < 150) {
			return 0xE8D44D;
		} else if (ping < 250) {
			return 0xF2994A;
		}
		return 0xEB5757;
	}
}
