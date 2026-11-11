stonecutter active "26.3"

// Conditions in the sources: //? if >=26.2 { ... //?}
// See https://stonecutter.kikugie.dev/wiki/config/params
stonecutter parameters {
	// Names that only changed name. The sources use the newest ones; the versions older than
	// `depuis` (and not older than `jusque`) get `ancien`. Whole words only, applied to the sources
	// as written (one rename does not feed the next), and one way only: the sources must stay as
	// they are, so never make a version older than 26.1 the active one.
	fun renomme(nouveau: String, ancien: String, depuis: String = "26.1", jusque: String? = null) {
		val actif = current.parsed < depuis && (jusque == null || current.parsed >= jusque)
		replacements.regex(actif) {
			replace(java.util.regex.Pattern.compile("\\b$nouveau\\b"), ancien, java.util.regex.Pattern.compile("(?!)"), "")
		}
	}

	// The GUI is "extracted" instead of "rendered" since 26.1 (the most precise names first)
	renomme("PlayerFaceExtractor\\.extractRenderState", "PlayerFaceRenderer.draw")
	renomme("net\\.minecraft\\.util\\.LightCoordsUtil", "net.minecraft.client.renderer.LightTexture")
	renomme("extractRenderStateWithTooltipAndSubtitles", "renderWithTooltipAndSubtitles", jusque = "1.21.9")
	renomme("extractRenderStateWithTooltipAndSubtitles", "renderWithTooltip", "1.21.9")
	renomme("extractPanorama", "renderPanorama")
	renomme("extractScoreboardSidebar", "renderScoreboardSidebar")
	renomme("extractBlurredBackground", "renderBlurredBackground")
	renomme("GuiGraphicsExtractor", "GuiGraphics")
	renomme("extractRenderState", "render")
	renomme("extractBackground", "renderBackground")
	renomme("extractWidgetRenderState", "renderWidget")
	renomme("PlayerFaceExtractor", "PlayerFaceRenderer")
	renomme("LightCoordsUtil", "LightTexture")
	renomme("KeyMappingHelper", "KeyBindingHelper")
	renomme("keymapping\\.v1", "keybinding.v1")
	renomme("registerKeyMapping", "registerKeyBinding")
	renomme("graphics\\.text", "graphics.drawString")
	renomme("graphics\\.item", "graphics.renderItem")
	renomme("graphics\\.entity", "graphics.submitEntityRenderState")
	// buttons draw their whole look in renderWidget before 1.21.11
	renomme("extractContents", "renderContents", jusque = "1.21.11")
	renomme("extractContents", "renderWidget", "1.21.11")
	renomme("graphics\\.outline", "graphics.renderOutline", jusque = "1.21.11")
	renomme("graphics\\.outline", "graphics.submitOutline", "1.21.11", "1.21.9")
	renomme("graphics\\.outline", "graphics.renderOutline", "1.21.9")

	// Renamed in 1.21.11
	renomme("Identifier", "ResourceLocation", "1.21.11")
	renomme("net\\.minecraft\\.util\\.Util", "net.minecraft.Util", "1.21.11")
	renomme("net\\.minecraft\\.client\\.model\\.player\\.PlayerModel", "net.minecraft.client.model.PlayerModel", "1.21.11")
	renomme("net\\.minecraft\\.client\\.renderer\\.rendertype\\.RenderTypes", "net.minecraft.client.renderer.RenderType", "1.21.11")
	renomme("RenderTypes", "RenderType", "1.21.11")

	// Renamed in 1.21.9: players were not yet "avatars", skins were in the client package, and
	// buttons were pressed without the input that pressed them
	renomme("AvatarRenderState", "PlayerRenderState", "1.21.9")
	renomme("AvatarRenderer", "PlayerRenderer", "1.21.9")
	renomme("net\\.minecraft\\.world\\.entity\\.player\\.PlayerSkin", "net.minecraft.client.resources.PlayerSkin", "1.21.9")
	renomme("onPress\\(final InputWithModifiers input", "onPress(", "1.21.9")
	renomme("net\\.minecraft\\.client\\.input\\.InputWithModifiers", "java.lang.Object", "1.21.9")
}
