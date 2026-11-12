package online.mistylauncher.client.hud;

import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientPacketListener;
import net.minecraft.network.protocol.ping.ServerboundPingRequestPacket;
import net.minecraft.util.Util;

/**
 * Ping measured by the mod: a ping request every second, timed until the server's answer. The
 * latency of the player list is only refreshed by the server every few seconds, so it shows 0 or
 * an old value for a while after joining. Servers that never answer (too old) keep the player list.
 */
public final class Ping {
	private static final long INTERVAL_MS = 1000;
	private static final long TIMEOUT_MS = 5000;
	private static final int MAX_MISSES = 3;

	/** Time sent in the pending request, -1 when none is pending. */
	private static volatile long pending = -1;
	/** Last measured ping in ms, -1 when unknown. */
	private static volatile int measured = -1;
	private static long lastSent;
	private static int misses;
	private static ClientPacketListener connection;

	private Ping() {
	}

	public static void tick(final Minecraft minecraft) {
		ClientPacketListener current = minecraft.getConnection();
		if (current != connection) {
			// new server (or none): start again
			connection = current;
			pending = -1;
			measured = -1;
			misses = 0;
		}
		if (current == null || minecraft.isLocalServer() || misses >= MAX_MISSES) {
			return;
		}
		long now = Util.getMillis();
		long sent = pending;
		if (sent >= 0) {
			if (now - sent < TIMEOUT_MS) {
				return;
			}
			// no answer: the server may not know this request
			pending = -1;
			misses++;
			return;
		}
		if (now - lastSent >= INTERVAL_MS) {
			lastSent = now;
			pending = now;
			current.send(new ServerboundPingRequestPacket(now));
		}
	}

	/** The server answered a ping request (network thread). */
	public static void pong(final long time) {
		if (time == pending) {
			measured = (int) Math.max(0, Util.getMillis() - time);
			pending = -1;
			misses = 0;
		}
	}

	/** Measured ping in ms, or -1 when not known (then use the player list). */
	public static int measured() {
		return measured;
	}
}
