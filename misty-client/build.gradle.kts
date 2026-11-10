plugins {
	// The right Loom for this Minecraft version (obfuscated or not)
	id("dev.kikugie.loom-back-compat")
}

// misty-client-<mod version>+mc<Minecraft version>.jar: the launcher reads the version from the name
version = "${sc.properties["mod.version"]}+mc${sc.current.version}"
base.archivesName = "misty-client"

val requiredJava: JavaVersion = when {
	sc.current.parsed >= "26.1" -> JavaVersion.VERSION_25
	sc.current.parsed >= "1.20.5" -> JavaVersion.VERSION_21
	sc.current.parsed >= "1.18" -> JavaVersion.VERSION_17
	sc.current.parsed >= "1.17" -> JavaVersion.VERSION_16
	else -> JavaVersion.VERSION_1_8
}

dependencies {
	minecraft("com.mojang:minecraft:${sc.current.version}")
	// Mojang's names on the obfuscated versions, like the unobfuscated ones
	loomx.applyMojangMappings()
	modImplementation("net.fabricmc:fabric-loader:${sc.properties["deps.fabric_loader"]}")

	// Only the Fabric API modules the mod uses, shipped inside the jar: nothing else to install
	for (it in listOf("fabric-api-base", "fabric-resource-loader-v1", "fabric-key-mapping-api-v1", "fabric-lifecycle-events-v1")) {
		val module = fabricApi.module(it, sc.properties["deps.fabric_api"])
		modImplementation(module)
		include(module)
	}
}

loom {
	runConfigs.all {
		// one game folder for every version
		runDirectory = rootProject.file("run")
	}
}

java {
	targetCompatibility = requiredJava
	sourceCompatibility = requiredJava
}

tasks {
	processResources {
		val props = mapOf(
			"version" to version.toString(),
			"minecraft" to sc.properties["mod.mc_compat"],
			"loader" to sc.properties["deps.fabric_loader"],
			"java" to requiredJava.majorVersion,
		)
		val mixinProps = mapOf("java" to "JAVA_${requiredJava.majorVersion}")
		inputs.properties(props)
		filesMatching("fabric.mod.json") { expand(props) }
		filesMatching("*.mixins.json") { expand(mixinProps) }
	}

	val release = requiredJava.majorVersion.toInt()
	withType<JavaCompile>().configureEach {
		options.release = release
		options.encoding = "UTF-8"
	}

	val licence = rootProject.file("../LICENSE.txt")
	withType<Jar>().configureEach {
		from(licence) { rename { "LICENSE_misty-client.txt" } }
	}
}
