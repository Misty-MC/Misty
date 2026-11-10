package online.mistylauncher.client.hud;

import com.mojang.blaze3d.platform.InputConstants;
import it.unimi.dsi.fastutil.longs.LongArrayFIFOQueue;
import net.minecraft.util.Util;

/** Mouse clicks of the last second, for the CPS module. */
public final class Clicks {
	private static final long WINDOW_MS = 1000;
	private static final LongArrayFIFOQueue LEFT = new LongArrayFIFOQueue();
	private static final LongArrayFIFOQueue RIGHT = new LongArrayFIFOQueue();

	private Clicks() {
	}

	/**
	 * A mouse button went down. The game's numbers: left 0 and right 1 up to 26.2 (GLFW), left 1
	 * and right 3 since 26.3 (SDL), hence the constants.
	 */
	public static void pressed(final int button) {
		if (button == InputConstants.MOUSE_BUTTON_LEFT) {
			LEFT.enqueue(Util.getMillis());
		} else if (button == InputConstants.MOUSE_BUTTON_RIGHT) {
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
