package online.mistylauncher.client.mixin;

import net.minecraft.client.gui.components.ChatComponent;
import net.minecraft.network.chat.Component;
import online.mistylauncher.client.social.Badge;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

/** Misty cloud in front of what the Misty players say in the chat. */
@Mixin(ChatComponent.class)
abstract class ChatComponentMixin {
	@ModifyVariable(method = { "addPlayerMessage", "addServerSystemMessage" }, at = @At("HEAD"), argsOnly = true)
	private Component misty$badge(final Component message) {
		return Badge.chat(message);
	}
}
