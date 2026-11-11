package online.mistylauncher.client.compat;

import java.util.List;
import java.util.Set;
import org.objectweb.asm.tree.ClassNode;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.spongepowered.asm.mixin.extensibility.IMixinConfigPlugin;
import org.spongepowered.asm.mixin.extensibility.IMixinInfo;
import org.spongepowered.asm.service.MixinService;

/**
 * Keeps the game running when a hook does not fit the Minecraft version: a jar can run on several
 * versions (mod.mc_releases), and a small change in one of them must turn a feature off, not crash
 * the game. A mixin whose target class is missing is skipped; an injection whose target method is
 * missing is skipped too ("defaultRequire": 0 in misty-client.mixins.json). Both are logged.
 */
public final class MistyMixinPlugin implements IMixinConfigPlugin {
	private static final Logger LOG = LoggerFactory.getLogger("misty-client");

	@Override
	public void onLoad(final String mixinPackage) {
	}

	@Override
	public String getRefMapperConfig() {
		return null;
	}

	@Override
	public boolean shouldApplyMixin(final String targetClassName, final String mixinClassName) {
		try {
			MixinService.getService().getBytecodeProvider().getClassNode(targetClassName);
			return true;
		} catch (final Throwable e) {
			LOG.warn("Misty Client: {} skipped, {} is not in this Minecraft version", mixinClassName, targetClassName);
			return false;
		}
	}

	@Override
	public void acceptTargets(final Set<String> myTargets, final Set<String> otherTargets) {
	}

	@Override
	public List<String> getMixins() {
		return null;
	}

	@Override
	public void preApply(final String targetClassName, final ClassNode targetClass, final String mixinClassName, final IMixinInfo mixinInfo) {
	}

	@Override
	public void postApply(final String targetClassName, final ClassNode targetClass, final String mixinClassName, final IMixinInfo mixinInfo) {
	}
}
