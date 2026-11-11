package online.mistylauncher.client.net;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.mojang.authlib.exceptions.AuthenticationException;
import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.security.SecureRandom;
import java.time.Duration;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;
import net.minecraft.client.Minecraft;
import net.minecraft.client.User;
import net.minecraft.client.multiplayer.ClientPacketListener;
import net.minecraft.client.multiplayer.PlayerInfo;
import online.mistylauncher.client.compat.Versions;
import online.mistylauncher.client.MistyClient;
import online.mistylauncher.client.config.LauncherLink;
import online.mistylauncher.client.cosmetics.Outfit;

/**
 * The Misty network ({@code misty-launcher.online/api/client}): tells the other Misty players
 * you are one of them and what you wear, and asks the same about the players around.
 *
 * <p>The account is proven the way a Minecraft server does it: the game "joins" a random server
 * id at Mojang's session service and the Misty site checks it there, so nobody can pose as someone
 * else and the Minecraft token never leaves the game. Offline accounts cannot be proven: they see
 * no badges and are not shown to others.
 *
 * <p>Everything runs on one background thread; the game thread only hands over the list of
 * players around ({@link #tick}).
 */
public final class MistyNetwork {
	private static final String API = System.getProperty("misty.api", "https://misty-launcher.online/api/client");
	private static final Duration TIMEOUT = Duration.ofSeconds(10);
	/** A sign of life at least this often (the site forgets a player after 3 minutes). */
	private static final long PRESENCE_EVERY_MS = 60_000;
	/** A player around is asked about again after this long. */
	private static final long LOOKUP_FRESH_MS = 60_000;
	private static final int LOOKUP_BATCH = 100;
	private static final long LOGIN_RETRY_MIN_MS = 30_000;
	private static final long LOGIN_RETRY_MAX_MS = 600_000;

	private static final HttpClient HTTP = HttpClient.newBuilder().connectTimeout(TIMEOUT).build();
	private static final SecureRandom RANDOM = new SecureRandom();

	private static ScheduledExecutorService worker;
	private static Minecraft minecraft;
	/** Players in the player list of the current server (written by the game thread). */
	private static volatile List<UUID> around = List.of();
	private static int ticks;

	// state of the background thread
	private static String token;
	private static long nextLogin;
	private static long loginDelay = LOGIN_RETRY_MIN_MS;
	private static long lastPresence;
	private static Outfit sentOutfit;
	/** The Misty site counts this account among the first Misty players (early player cape). */
	private static volatile boolean early;

	private MistyNetwork() {
	}

	public static void start(final Minecraft client) {
		minecraft = client;
		worker = Executors.newSingleThreadScheduledExecutor(runnable -> {
			Thread thread = new Thread(runnable, "Misty network");
			thread.setDaemon(true);
			return thread;
		});
		worker.scheduleWithFixedDelay(MistyNetwork::step, 2, 3, TimeUnit.SECONDS);
	}

	/** Game thread, every tick: once a second, notes who is in the player list. */
	public static void tick(final Minecraft client) {
		if (++ticks < 20) {
			return;
		}
		ticks = 0;
		ClientPacketListener connection = client.getConnection();
		if (connection == null || client.player == null) {
			if (!around.isEmpty()) {
				around = List.of();
				MistyPlayers.clear();
			}
			return;
		}
		UUID self = client.getUser().getProfileId();
		List<UUID> ids = new ArrayList<>();
		for (PlayerInfo info : connection.getOnlinePlayers()) {
			UUID id = Versions.id(info.getProfile());
			// version 4 only: real accounts (offline and NPC ids are other versions)
			if (!id.equals(self) && id.version() == 4) {
				ids.add(id);
			}
		}
		around = ids;
	}

	/** When the game closes: the other players stop seeing you right away. */
	public static void stop() {
		if (worker == null) {
			return;
		}
		worker.shutdownNow();
		if (token != null) {
			JsonObject body = new JsonObject();
			body.addProperty("bye", true);
			try {
				post("/presence", body, Duration.ofSeconds(2));
			} catch (IOException | InterruptedException | RuntimeException e) {
				// closing anyway: the site forgets the player by itself after a few minutes
			}
		}
	}

	private static void step() {
		try {
			if (!MistyClient.config().network) {
				if (token != null) {
					JsonObject body = new JsonObject();
					body.addProperty("bye", true);
					post("/presence", body, TIMEOUT);
					token = null;
				}
				MistyPlayers.clear();
				return;
			}
			long now = System.currentTimeMillis();
			if (token == null) {
				if (now < nextLogin) {
					return;
				}
				login();
				if (token == null) {
					return;
				}
			}
			Outfit outfit = LauncherLink.outfit();
			if (now - lastPresence >= PRESENCE_EVERY_MS || !outfit.equals(sentOutfit)) {
				presence(outfit);
			}
			lookup();
		} catch (InterruptedException e) {
			Thread.currentThread().interrupt();
		} catch (IOException | RuntimeException e) {
			// site unreachable: the next step tries again
			MistyClient.LOG.debug("Misty network", e);
		}
	}

	/** Whether this account may wear the early player cape (the Misty site says so at sign-in). */
	public static boolean early() {
		return early;
	}

	private static void loginFailed() {
		nextLogin = System.currentTimeMillis() + loginDelay;
		loginDelay = Math.min(loginDelay * 2, LOGIN_RETRY_MAX_MS);
	}

	private static void login() throws IOException, InterruptedException {
		User user = minecraft.getUser();
		byte[] random = new byte[20];
		RANDOM.nextBytes(random);
		String serverId = HexFormat.of().formatHex(random);
		try {
			//? if >=1.21.9 {
			minecraft.services().sessionService().joinServer(user.getProfileId(), user.getAccessToken(), serverId);
			//?} else {
			/*minecraft.getMinecraftSessionService().joinServer(user.getProfileId(), user.getAccessToken(), serverId);
			*///?}
		} catch (AuthenticationException e) {
			// offline account, or Mojang unreachable
			loginFailed();
			return;
		}
		JsonObject body = new JsonObject();
		body.addProperty("name", user.getName());
		body.addProperty("serverId", serverId);
		HttpResponse<String> response = post("/login", body, TIMEOUT);
		if (response.statusCode() != 200) {
			loginFailed();
			return;
		}
		JsonObject answer = JsonParser.parseString(response.body()).getAsJsonObject();
		token = answer.get("token").getAsString();
		early = answer.has("early") && answer.get("early").getAsBoolean();
		loginDelay = LOGIN_RETRY_MIN_MS;
		lastPresence = 0;
		sentOutfit = null;
	}

	private static void presence(final Outfit outfit) throws IOException, InterruptedException {
		JsonObject body = new JsonObject();
		boolean shown = MistyClient.config().cosmetics;
		body.addProperty("cosmetic", shown ? outfit.back().id() : "");
		body.addProperty("hat", shown ? outfit.hat().id() : "");
		HttpResponse<String> response = post("/presence", body, TIMEOUT);
		if (expired(response)) {
			return;
		}
		lastPresence = System.currentTimeMillis();
		sentOutfit = outfit;
	}

	private static void lookup() throws IOException, InterruptedException {
		long now = System.currentTimeMillis();
		List<UUID> ask = new ArrayList<>();
		for (UUID id : around) {
			MistyPlayers.Entry entry = MistyPlayers.get(id);
			if (entry == null || now - entry.fetchedAt() >= LOOKUP_FRESH_MS) {
				ask.add(id);
				if (ask.size() == LOOKUP_BATCH) {
					break;
				}
			}
		}
		if (ask.isEmpty()) {
			return;
		}
		JsonArray uuids = new JsonArray();
		ask.forEach(id -> uuids.add(id.toString()));
		JsonObject body = new JsonObject();
		body.add("uuids", uuids);
		HttpResponse<String> response = post("/players", body, TIMEOUT);
		if (expired(response) || response.statusCode() != 200) {
			return;
		}
		JsonObject players = JsonParser.parseString(response.body()).getAsJsonObject().getAsJsonObject("players");
		for (UUID id : ask) {
			MistyPlayers.put(id, false, Outfit.NONE);
		}
		for (Map.Entry<String, JsonElement> player : players.entrySet()) {
			JsonObject data = player.getValue().getAsJsonObject();
			String cosmetic = data.has("cosmetic") ? data.get("cosmetic").getAsString() : "";
			String hat = data.has("hat") ? data.get("hat").getAsString() : "";
			MistyPlayers.put(UUID.fromString(player.getKey()), true, Outfit.of(cosmetic, hat));
		}
	}

	/** The site no longer knows our token (it restarted, or we were away too long): sign in again. */
	private static boolean expired(final HttpResponse<String> response) {
		if (response.statusCode() == 401) {
			token = null;
			nextLogin = 0;
			return true;
		}
		return false;
	}

	private static HttpResponse<String> post(final String path, final JsonObject body, final Duration timeout) throws IOException, InterruptedException {
		HttpRequest.Builder request = HttpRequest.newBuilder(URI.create(API + path))
			.timeout(timeout)
			.header("Content-Type", "application/json")
			.header("User-Agent", "MistyClient/" + MistyClient.version())
			.POST(HttpRequest.BodyPublishers.ofString(body.toString(), StandardCharsets.UTF_8));
		if (token != null) {
			request.header("Authorization", "Bearer " + token);
		}
		return HTTP.send(request.build(), HttpResponse.BodyHandlers.ofString(StandardCharsets.UTF_8));
	}
}
