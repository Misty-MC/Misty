package online.mistylauncher.client.mixin;

import com.mojang.authlib.GameProfile;
import java.time.Instant;
import net.minecraft.client.multiplayer.chat.ChatListener;
import net.minecraft.network.chat.ChatType;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.PlayerChatMessage;
import online.mistylauncher.client.social.Badge;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** Signed chat messages name their sender: no need to guess who is speaking. */
@Mixin(ChatListener.class)
abstract class ChatListenerMixin {
	@Inject(method = "showMessageToPlayer", at = @At("HEAD"))
	private void misty$sender(
		final ChatType.Bound boundChatType,
		final PlayerChatMessage message,
		final Component decoratedMessage,
		final GameProfile sender,
		final boolean onlyShowSecure,
		final Instant received,
		final CallbackInfoReturnable<Boolean> cir
	) {
		Badge.sender(message.sender());
	}

	@Inject(method = "showMessageToPlayer", at = @At("RETURN"))
	private void misty$senderDone(final CallbackInfoReturnable<Boolean> cir) {
		Badge.sender(null);
	}
}
