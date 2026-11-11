pluginManagement {
	repositories {
		mavenCentral()
		gradlePluginPortal()
		maven("https://maven.fabricmc.net/") { name = "Fabric" }
		maven("https://maven.kikugie.dev/releases") { name = "KikuGie Releases" }
	}
}

plugins {
	// One source, one jar per Minecraft version (https://stonecutter.kikugie.dev)
	id("dev.kikugie.stonecutter") version "0.10"
	// The right Loom for each version: obfuscated (up to 1.21.11) or not (26.1 and later)
	id("dev.kikugie.loom-back-compat") version "0.4.3"
}

stonecutter {
	create(rootProject) {
		// Minecraft versions Misty Client is built for (settings in stonecutter.properties.toml)
		versions("1.21.11", "26.1.2", "26.2", "26.3")
		// the version the sources are written for (the others are made from it)
		vcsVersion = "26.3"
	}
}

rootProject.name = "misty-client"
