package online.mistylauncher.client.hud.modules;

import java.util.ArrayList;
import java.util.List;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.client.renderer.RenderPipelines;
import net.minecraft.resources.Identifier;
import net.minecraft.world.entity.EquipmentSlot;
import net.minecraft.world.item.ItemStack;
import online.mistylauncher.client.config.MistyConfig;
import online.mistylauncher.client.hud.HudModule;
import online.mistylauncher.client.ui.Ui;

/** The armor worn and the item held, with what is left of their durability. */
public final class ArmorModule extends HudModule {
	private static final EquipmentSlot[] SLOTS = { EquipmentSlot.HEAD, EquipmentSlot.CHEST, EquipmentSlot.LEGS, EquipmentSlot.FEET, EquipmentSlot.MAINHAND };
	private static final int ROW = 17;
	private static final int WIDTH = 52;

	/** In the editor with nothing on: the empty armor slots of the inventory, and a full diamond set's durability. */
	private static final Identifier[] SAMPLE_SLOTS = {
		Identifier.withDefaultNamespace("container/slot/helmet"),
		Identifier.withDefaultNamespace("container/slot/chestplate"),
		Identifier.withDefaultNamespace("container/slot/leggings"),
		Identifier.withDefaultNamespace("container/slot/boots")
	};
	private static final String[] SAMPLE_DURABILITY = { "363", "528", "495", "429" };

	public ArmorModule() {
		super("armor", new MistyConfig.Module(false, 8, -4, -4));
	}

	private static List<ItemStack> items(final Minecraft minecraft) {
		List<ItemStack> items = new ArrayList<>();
		LocalPlayer player = minecraft.player;
		if (player != null) {
			for (EquipmentSlot slot : SLOTS) {
				ItemStack stack = player.getItemBySlot(slot);
				if (!stack.isEmpty() && (slot != EquipmentSlot.MAINHAND || stack.isDamageableItem())) {
					items.add(stack);
				}
			}
		}
		return items;
	}

	@Override
	public boolean available(final Minecraft minecraft) {
		return !items(minecraft).isEmpty();
	}

	@Override
	public int width(final Minecraft minecraft, final boolean preview) {
		return WIDTH;
	}

	@Override
	public int height(final Minecraft minecraft, final boolean preview) {
		int rows = items(minecraft).size();
		return (rows == 0 ? SAMPLE_SLOTS.length : rows) * ROW + 3;
	}

	@Override
	public void draw(final GuiGraphicsExtractor graphics, final Minecraft minecraft, final boolean preview) {
		Font font = minecraft.font;
		List<ItemStack> items = items(minecraft);
		background(graphics, WIDTH, this.height(minecraft, preview));
		int y = 2;
		if (items.isEmpty()) {
			for (int i = 0; i < SAMPLE_SLOTS.length; i++) {
				graphics.blitSprite(RenderPipelines.GUI_TEXTURED, SAMPLE_SLOTS[i], 3, y, 16, 16);
				graphics.text(font, SAMPLE_DURABILITY[i], 23, y + 4, Ui.TEXT, false);
				y += ROW;
			}
			return;
		}
		for (ItemStack stack : items) {
			graphics.item(stack, 3, y);
			if (stack.isDamageableItem()) {
				int left = stack.getMaxDamage() - stack.getDamageValue();
				float ratio = (float)left / stack.getMaxDamage();
				int color = ratio > 0.5F ? Ui.TEXT : ratio > 0.2F ? 0xFFFFD166 : 0xFFFF5A6E;
				graphics.text(font, Integer.toString(left), 23, y + 4, color, false);
			} else if (stack.getCount() > 1) {
				graphics.text(font, "x" + stack.getCount(), 23, y + 4, Ui.MUTED, false);
			}
			y += ROW;
		}
	}
}
