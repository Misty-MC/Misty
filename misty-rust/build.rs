// build.rs — intègre l'icône Windows, la clé CurseForge et Misty Client.
// (L'interface Slint se compile dans son propre crate : misty-ui/.)

fn main() {
    // Clé API CurseForge (privée, ignorée par git) : misty-rust/curseforge.key
    for chemin in ["curseforge.key"] {
        println!("cargo:rerun-if-changed={chemin}");
        if let Ok(cle) = std::fs::read_to_string(chemin) {
            let cle = cle.trim();
            if !cle.is_empty() {
                println!("cargo:rustc-env=MISTY_CF_KEY={cle}");
                break;
            }
        }
    }

    misty_client();

    // Notes de version affichées dans l'application
    println!("cargo:rerun-if-changed=../CHANGELOG.md");

    // Icône de l'exe (Windows)
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icone.ico");
        let _ = res.compile();
    }
}

/// Misty Client (le mod Fabric, dossier misty-client/) : son jar est intégré à l'exe.
/// Il se compile avant le launcher avec `gradlew build` dans misty-client/ (fait par
/// installer/build.ps1). S'il n'est pas compilé, le launcher se compile quand même, sans le mod.
fn misty_client() {
    let libs = std::path::Path::new("../misty-client/build/libs");
    println!("cargo:rerun-if-changed={}", libs.display());

    // misty-client-<version>+mc<version de Minecraft>.jar (le plus récent s'il y en a plusieurs)
    let jar = std::fs::read_dir(libs)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|f| {
            let nom = f.file_name().to_string_lossy().to_string();
            let minecraft = nom.strip_prefix("misty-client-")?.strip_suffix(".jar")?.split_once("+mc")?.1.to_string();
            let date = f.metadata().and_then(|m| m.modified()).ok()?;
            Some((date, f.path(), minecraft))
        })
        .max_by_key(|(date, ..)| *date);

    let sortie = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("misty-client.jar");
    match jar {
        Some((_, chemin, minecraft)) => {
            println!("cargo:rerun-if-changed={}", chemin.display());
            std::fs::copy(&chemin, &sortie).expect("copie du jar de Misty Client");
            println!("cargo:rustc-env=MISTY_CLIENT_MC={minecraft}");
        }
        None => {
            println!("cargo:warning=Misty Client n'est pas compilé (misty-client/build/libs) : le launcher est compilé sans le mod en jeu");
            std::fs::write(&sortie, []).expect("écriture du jar vide");
            println!("cargo:rustc-env=MISTY_CLIENT_MC=");
        }
    }
}
