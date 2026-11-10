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

/// Misty Client (le mod Fabric, dossier misty-client/) : un jar par version de Minecraft, tous
/// intégrés à l'exe. Ils se compilent avant le launcher avec `gradlew build` dans misty-client/
/// (fait par installer/build.ps1), dans `versions/<minecraft>/build/libs/` (Stonecutter). Sans
/// jar, le launcher se compile quand même, sans le mod.
///
/// Un jar peut tourner sur plusieurs versions de Minecraft : celles de `mod.mc_releases` dans
/// `stonecutter.properties.toml` (sinon, seulement celle de son nom).
///
/// Génère `misty_client.rs` : `JARS: &[(&str, &[u8])]`, (version de Minecraft, jar), dans l'ordre
/// des versions.
fn misty_client() {
    let racine = std::path::Path::new("../misty-client");
    let mut dossiers = Vec::new();
    if let Ok(versions) = std::fs::read_dir(racine.join("versions")) {
        dossiers.extend(versions.flatten().map(|v| v.path().join("build/libs")));
    }
    println!("cargo:rerun-if-changed={}", racine.join("versions").display());

    // misty-client-<version>+mc<version de Minecraft>.jar : le plus récent pour chaque version
    let mut jars: std::collections::BTreeMap<String, (std::time::SystemTime, std::path::PathBuf)> = Default::default();
    for dossier in &dossiers {
        println!("cargo:rerun-if-changed={}", dossier.display());
        for f in std::fs::read_dir(dossier).into_iter().flatten().flatten() {
            let nom = f.file_name().to_string_lossy().to_string();
            if nom.ends_with("-sources.jar") {
                continue;
            }
            let Some(minecraft) = nom.strip_prefix("misty-client-").and_then(|n| n.strip_suffix(".jar")).and_then(|n| n.split_once("+mc")).map(|(_, mc)| mc.to_string()) else {
                continue;
            };
            let Ok(date) = f.metadata().and_then(|m| m.modified()) else { continue };
            if jars.get(&minecraft).is_none_or(|(d, _)| date > *d) {
                jars.insert(minecraft, (date, f.path()));
            }
        }
    }

    let proprietes = racine.join("stonecutter.properties.toml");
    println!("cargo:rerun-if-changed={}", proprietes.display());
    let versions_par_jar = mc_releases(&std::fs::read_to_string(&proprietes).unwrap_or_default());

    // chaque jar une fois dans l'exe, puis une entrée par version de Minecraft où il tourne
    let sortie = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("misty_client.rs");
    let mut code = String::new();
    let mut entrees = Vec::new();
    for (n, (minecraft, (_, chemin))) in jars.iter().enumerate() {
        println!("cargo:rerun-if-changed={}", chemin.display());
        let absolu = std::fs::canonicalize(chemin).expect("chemin du jar de Misty Client");
        code.push_str(&format!("const JAR_{n}: &[u8] = include_bytes!({:?});\n", absolu.display().to_string()));
        for version in versions_par_jar.get(minecraft).cloned().unwrap_or_else(|| vec![minecraft.clone()]) {
            entrees.push((version, n));
        }
    }
    entrees.sort_by(|a, b| ordre_versions(&a.0, &b.0));
    entrees.dedup_by(|a, b| a.0 == b.0);
    code.push_str("pub const JARS: &[(&str, &[u8])] = &[\n");
    for (version, n) in &entrees {
        code.push_str(&format!("    ({version:?}, JAR_{n}),\n"));
    }
    code.push_str("];\n");
    std::fs::write(&sortie, code).expect("écriture de misty_client.rs");
    if jars.is_empty() {
        println!("cargo:warning=Misty Client n'est pas compilé (misty-client) : le launcher est compilé sans le mod en jeu");
    }
}

/// `["26.1.2"]` puis `mod.mc_releases = ["26.1", "26.1.2"]` : version du jar → versions de Minecraft.
fn mc_releases(toml: &str) -> std::collections::HashMap<String, Vec<String>> {
    let mut liste = std::collections::HashMap::new();
    let mut section = None;
    for ligne in toml.lines().map(str::trim) {
        if let Some(nom) = ligne.strip_prefix("[\"").and_then(|l| l.strip_suffix("\"]")) {
            section = Some(nom.to_string());
        } else if let (Some(nom), Some(valeur)) = (&section, ligne.strip_prefix("mod.mc_releases")) {
            let versions = valeur.split('"').skip(1).step_by(2).map(str::to_string).collect();
            liste.insert(nom.clone(), versions);
        }
    }
    liste
}

/// "1.21.11" < "26.1" < "26.1.2" < "26.3" (nombre par nombre)
fn ordre_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let n = |v: &str| v.split(['.', '-']).map(|x| x.parse::<u64>().unwrap_or(0)).collect::<Vec<_>>();
    n(a).cmp(&n(b))
}
