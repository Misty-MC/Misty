package online.mistylauncher.client.hud;

import it.unimi.dsi.fastutil.longs.LongArrayFIFOQueue;
import net.minecraft.util.Util;

/** Mouse clicks of the last second, for the CPS module. */
public final class Clicks {
	private static final long WINDOW_MS = 1000;
	private static final LongArrayFIFOQueue LEFT = new LongArrayFIFOQueue();
	private static final LongArrayFIFOQueue RIGHT = new LongArrayFIFOQueue();

	private Clicks() {
	}

	/** A mouse button went down (0 left, 1 right). */
	public static void pressed(final int button) {
		if (button == 0) {
			LEFT.enqueue(Util.getMillis());
		} else if (button == 1) {
			RIGHT.enqueue(Util.getMillis());
		}
	}

	public static int left() {
		return count(LEFT);
	}

	public static int right() {
		return count(RIGHT);
	}

	private static int count(final LongArrayFIFOQueue clicks) {
		long oldest = Util.getMillis() - WINDOW_MS;
		while (!clicks.isEmpty() && clicks.firstLong() < oldest) {
			clicks.dequeueLong();
		}
		return clicks.size();
	}
}
