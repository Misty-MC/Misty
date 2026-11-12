package online.mistylauncher.client.ui;

import online.mistylauncher.client.compat.Versions;
import java.util.function.Supplier;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.narration.NarrationElementOutput;
import net.minecraft.client.renderer.entity.state.AvatarRenderState;
import net.minecraft.client.sounds.SoundManager;
import net.minecraft.network.chat.Component;
import net.minecraft.util.LightCoordsUtil;
import net.minecraft.util.Mth;
import net.minecraft.util.Util;
import net.minecraft.world.entity.Pose;
import net.minecraft.world.entity.player.PlayerSkin;
import online.mistylauncher.client.cosmetics.Cosmetic;
import online.mistylauncher.client.cosmetics.Cosmetics;
import online.mistylauncher.client.cosmetics.HatHolder;
import online.mistylauncher.client.cosmetics.Outfit;
import org.joml.Quaternionf;
import org.joml.Vector3f;

/**
 * The player in 3D, drawn by the game's own player renderer (like in the inventory): their
 * skin and what they wear on the back (the Misty cape, or their own Mojang cape), with the
 * right depth from every side. It turns slowly on its own; drag to turn it yourself.
 *
 * <p>No entity is needed: a player render state is filled by hand, so it works on the title
 * screen too. The game draws only one such 3D picture per area of a screen, so the body and the
 * cape must be one render, not two.
 */
public class PlayerPreview extends AbstractWidget {
	/** Degrees per second of the slow turn, while nobody drags. */
	private static final float AUTO_TURN = 18.0F;
	/** The player is 1.8 blocks tall: a little room above the head and under the feet. */
	private static final float HEIGHT_IN_BLOCKS = 2.05F;
	/** Room added above the head for a hat (it floats up to half a block over the head). */
	private static final float HAT_ROOM = 0.5F;

	private final Supplier<Outfit> outfit;
	private final AvatarRenderState state = new AvatarRenderState();
	private float rotationX = 0.0F;
	private float rotationY = -25.0F;
	private float headYaw;
	private float headPitch;
	private long lastFrame = Util.getMillis();
	/** Last time the player turned it: the slow turn starts again a moment after. */
	private long draggedAt;

	public PlayerPreview(final int x, final int y, final int width, final int height, final Supplier<Outfit> outfit) {
		super(x, y, width, height, Component.empty());
		this.outfit = outfit;
		AvatarRenderState s = this.state;
		s.entityType = Versions.playerType();
		//? if >=1.21.9 {
		s.lightCoords = LightCoordsUtil.FULL_BRIGHT;
		//?}
		s.boundingBoxWidth = 0.6F;
		s.boundingBoxHeight = 1.8F;
		s.eyeHeight = 1.62F;
		s.scale = 1.0F;
		s.ageScale = 1.0F;
		s.pose = Pose.STANDING;
		// every layer of the skin (hat, jacket, sleeves, trousers)
		s.showHat = true;
		s.showJacket = true;
		s.showLeftSleeve = true;
		s.showRightSleeve = true;
		s.showLeftPants = true;
		s.showRightPants = true;
		// the cape hangs a little away from the back, as when standing in the game
		s.capeLean = 6.0F;
	}

	/** The local player's skin. */
	public static PlayerSkin skin() {
		Minecraft minecraft = Minecraft.getInstance();
		//? if >=1.21.9 {
		return minecraft.getSkinManager().createLookup(minecraft.getGameProfile(), false).get();
		//?} else {
		/*return minecraft.getSkinManager().getInsecureSkin(minecraft.getGameProfile());
		*///?}
	}

	@Override
	protected void extractWidgetRenderState(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a) {
		long now = Util.getMillis();
		float seconds = Math.min(0.1F, (now - this.lastFrame) / 1000.0F);
		this.lastFrame = now;
		if (now - this.draggedAt > 2500L) {
			this.rotationY = Mth.wrapDegrees(this.rotationY + AUTO_TURN * seconds);
		}

		// what is worn: a Misty cape (its texture also holds the elytra), or the own Mojang cape;
		// the Misty elytra alone mean no cape on the back; and the hat
		Outfit worn = this.outfit.get();
		AvatarRenderState s = this.state;
		s.skin = worn.back() == Cosmetic.ELYTRA ? skin() : Cosmetics.wear(worn.back(), skin());
		s.showCape = worn.back() != Cosmetic.ELYTRA && Cosmetics.hasCape(s.skin);
		((HatHolder)s).misty$setHat(worn.hat());
		s.ageInTicks = (now % 1_000_000L) / 50.0F;
		// the whole body turns; the head looks at the mouse (as in the inventory), as long as the
		// body faces you: seen from behind, the head goes back to straight ahead
		s.bodyRot = 180.0F + this.rotationY;
		float headX = this.getX() + this.getWidth() / 2.0F;
		float headY = this.getY() + this.getHeight() * 0.18F;
		float lookX = (float)Math.atan((headX - mouseX) / 40.0F);
		float lookY = (float)Math.atan((headY - mouseY) / 40.0F);
		float facing = Mth.wrapDegrees(this.rotationY);
		float weight = Mth.clamp(1.0F - Math.abs(facing) / 100.0F, 0.0F, 1.0F);
		float targetYaw = Mth.clamp(lookX * 60.0F - facing, -70.0F, 70.0F) * weight;
		float targetPitch = Mth.clamp(-lookY * 20.0F, -30.0F, 30.0F) * weight;
		// eased, so the head turns smoothly instead of jumping
		this.headYaw = Mth.lerp(0.25F, this.headYaw, targetYaw);
		this.headPitch = Mth.lerp(0.25F, this.headPitch, targetPitch);
		s.yRot = this.headYaw;
		s.xRot = this.headPitch;

		// like the inventory: the model centred in its area, seen from a little above or below
		float room = worn.hat() == Cosmetic.NONE ? 0.0F : HAT_ROOM;
		float scale = this.getHeight() / (HEIGHT_IN_BLOCKS + room);
		// the extra room goes above the head: the feet stay where they were
		Vector3f translation = new Vector3f(0.0F, s.boundingBoxHeight / 2.0F + 0.0625F + room / 2.0F, 0.0F);
		Quaternionf camera = new Quaternionf().rotateX(this.rotationX * Mth.DEG_TO_RAD);
		Quaternionf pose = new Quaternionf().rotateZ((float)Math.PI).mul(camera);
		graphics.entity(s, scale, translation, pose, camera, this.getX(), this.getY(), this.getRight(), this.getBottom());
	}

	@Override
	//? if >=1.21.9 {
	protected void onDrag(final net.minecraft.client.input.MouseButtonEvent event, final double dragX, final double dragY) {
	//?} else {
	/*protected void onDrag(final double mouseX, final double mouseY, final double dragX, final double dragY) {
	*///?}
		this.rotationX = Mth.clamp(this.rotationX + (float)dragY * 1.5F, -35.0F, 35.0F);
		// the side under the mouse follows it: dragging right turns the front to the right
		this.rotationY -= (float)dragX * 2.5F;
		this.draggedAt = Util.getMillis();
	}

	@Override
	public void playDownSound(final SoundManager soundManager) {
	}

	@Override
	protected void updateWidgetNarration(final NarrationElementOutput output) {
	}
}
