//! Export d'un profil en modpack .mrpack (format Modrinth) : Misty, la Modrinth App, Prism… savent
//! l'importer.
//!
//! Tous les fichiers sont mis dans `overrides/` (mods compris) : le pack marche sans rien
//! télécharger. Misty Client y est ajouté si on le demande et que le profil le supporte (Fabric,
//! version où il existe), sous son nom avec sa version : le launcher qui l'importe compare cette
//! version à la sienne et ne garde que la plus récente (voir client::preparer).

use super::{client, instances::Instance};
use crate::minecraft::{loaders, McDirs};
use std::fs;
use std::io::Write;
use std::path::Path;

/// Ce qui part dans le pack : la configuration du jeu et du contenu, pas les mondes ni les journaux.
const DOSSIERS: &[&str] = &["mods", "config", "defaultconfigs", "kubejs", "scripts", "resourcepacks", "shaderpacks"];
const FICHIERS: &[&str] = &["options.txt"];

/// Version du loader à écrire dans le pack : celle du profil, sinon celle installée par Misty pour
/// cette version de Minecraft (la plus récente). None : le profil n'a encore jamais été lancé.
fn version_loader(instance: &Instance) -> Option<String> {
    if let Some(v) = &instance.loader_version {
        return Some(v.clone());
    }
    let mc = &instance.version_mc;
    let installees: Vec<String> = fs::read_dir(McDirs::misty().versions())
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    let candidats: Vec<String> = installees
        .iter()
        .filter_map(|nom| match instance.loader.as_str() {
            loaders::FABRIC => nom.strip_prefix("fabric-loader-")?.strip_suffix(&format!("-{mc}")).map(str::to_string),
            loaders::QUILT => nom.strip_prefix("quilt-loader-")?.strip_suffix(&format!("-{mc}")).map(str::to_string),
            loaders::FORGE => nom.strip_prefix(&format!("{mc}-forge-")).map(str::to_string),
            // NeoForge : "neoforge-21.1.77" ; sa version commence par celle de Minecraft sans le "1."
            loaders::NEOFORGE => {
                let v = nom.strip_prefix("neoforge-")?;
                let prefixe = mc.strip_prefix("1.").unwrap_or(mc);
                v.starts_with(prefixe).then(|| v.to_string())
            }
            _ => None,
        })
        .collect();
    candidats.into_iter().max_by(|a, b| super::update::comparer_versions(a, b))
}

fn cle_loader(loader: &str) -> Option<&'static str> {
    match loader {
        loaders::FABRIC => Some("fabric-loader"),
        loaders::QUILT => Some("quilt-loader"),
        loaders::FORGE => Some("forge"),
        loaders::NEOFORGE => Some("neoforge"),
        _ => None,
    }
}

/// Écrit le pack dans `destination`. `avec_misty` : y mettre Misty Client (si le profil le supporte).
pub fn exporter(instance: &Instance, destination: &Path, avec_misty: bool) -> anyhow::Result<()> {
    let mut dependances = serde_json::Map::new();
    dependances.insert("minecraft".into(), instance.version_mc.clone().into());
    if let Some(cle) = cle_loader(&instance.loader) {
        let version = version_loader(instance).ok_or_else(|| anyhow::anyhow!(super::loc::t("Export_JamaisLance")))?;
        dependances.insert(cle.into(), version.into());
    }
    let index = serde_json::json!({
        "formatVersion": 1,
        "game": "minecraft",
        "versionId": instance.modpack_version.clone().unwrap_or_else(|| "1.0.0".into()),
        "name": instance.nom,
        "summary": format!("{} · Minecraft {} · {}", instance.nom, instance.version_mc, instance.loader),
        "files": [],
        "dependencies": dependances,
    });

    // écrit à côté puis renommé : un export interrompu ne laisse pas un pack à moitié écrit
    let temporaire = destination.with_extension("mrpack.part");
    let resultat = (|| -> anyhow::Result<()> {
        let mut zip = zip::ZipWriter::new(fs::File::create(&temporaire)?);
        let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("modrinth.index.json", options)?;
        zip.write_all(serde_json::to_string_pretty(&index)?.as_bytes())?;

        let racine = instance.dossier();
        // les jars de Misty Client du profil ne partent pas tels quels : le bon est ajouté plus bas
        let misty_presents: Vec<_> = client::jars_presents(&racine.join("mods")).into_iter().map(|(p, _)| p).collect();
        for dossier in DOSSIERS {
            ajouter_dossier(&mut zip, &racine.join(dossier), &format!("overrides/{dossier}"), &misty_presents, options)?;
        }
        for fichier in FICHIERS {
            let chemin = racine.join(fichier);
            if chemin.is_file() {
                zip.start_file(format!("overrides/{fichier}"), options)?;
                zip.write_all(&fs::read(&chemin)?)?;
            }
        }
        if avec_misty && client::compatible(&instance.version_mc, &instance.loader) {
            if let Some((nom, jar)) = client::jar_a_exporter(&instance.version_mc) {
                // un jar est déjà compressé : stocké tel quel
                zip.start_file(format!("overrides/mods/{nom}"), options.compression_method(zip::CompressionMethod::Stored))?;
                zip.write_all(jar)?;
            }
        }
        zip.finish()?;
        Ok(())
    })();
    match resultat {
        Ok(()) => {
            let _ = fs::remove_file(destination);
            fs::rename(&temporaire, destination)?;
            Ok(())
        }
        Err(e) => {
            let _ = fs::remove_file(&temporaire);
            Err(e)
        }
    }
}

fn ajouter_dossier(
    zip: &mut zip::ZipWriter<fs::File>,
    dossier: &Path,
    dans_zip: &str,
    exclus: &[std::path::PathBuf],
    options: zip::write::SimpleFileOptions,
) -> anyhow::Result<()> {
    let Ok(lecture) = fs::read_dir(dossier) else { return Ok(()) };
    for entree in lecture.flatten() {
        let chemin = entree.path();
        if exclus.contains(&chemin) {
            continue;
        }
        let nom = entree.file_name().to_string_lossy().to_string();
        if chemin.is_dir() {
            ajouter_dossier(zip, &chemin, &format!("{dans_zip}/{nom}"), exclus, options)?;
        } else {
            zip.start_file(format!("{dans_zip}/{nom}"), options)?;
            zip.write_all(&fs::read(&chemin)?)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un profil Fabric exporté : index Modrinth, ses fichiers dans overrides/, et Misty Client une
    /// seule fois (celui du launcher, nommé avec sa version) si on le demande.
    #[test]
    fn export_mrpack() {
        let Some(mc) = client::versions().last().map(|v| v.to_string()) else { return };
        let mut instance: Instance = serde_json::from_str("{}").unwrap();
        instance.id = format!("misty-test-export-{}", std::process::id());
        instance.nom = "Test".into();
        instance.version_mc = mc.clone();
        instance.loader = loaders::FABRIC.into();
        instance.loader_version = Some("0.17.2".into());
        let racine = instance.dossier();
        fs::create_dir_all(racine.join("mods")).unwrap();
        fs::create_dir_all(racine.join("saves/monde")).unwrap();
        fs::write(racine.join("mods/sodium.jar"), b"sodium").unwrap();
        fs::write(racine.join("mods/misty-client.jar"), client::jar_a_exporter(&mc).unwrap().1).unwrap();
        fs::write(racine.join("saves/monde/level.dat"), b"monde").unwrap();
        fs::write(racine.join("options.txt"), b"fov:70").unwrap();

        let sortie = std::env::temp_dir().join(format!("{}.mrpack", instance.id));
        for avec in [true, false] {
            exporter(&instance, &sortie, avec).unwrap();
            let mut zip = zip::ZipArchive::new(fs::File::open(&sortie).unwrap()).unwrap();
            let noms: Vec<String> = zip.file_names().map(str::to_string).collect();
            assert!(noms.contains(&"overrides/mods/sodium.jar".to_string()));
            assert!(noms.contains(&"overrides/options.txt".to_string()));
            assert!(!noms.iter().any(|n| n.contains("saves")));
            let misty = noms.iter().filter(|n| n.contains("misty-client")).count();
            assert_eq!(misty, avec as usize, "{noms:?}");
            let index: serde_json::Value = serde_json::from_reader(zip.by_name("modrinth.index.json").unwrap()).unwrap();
            assert_eq!(index["dependencies"]["fabric-loader"], "0.17.2");
            assert_eq!(index["dependencies"]["minecraft"], mc.as_str());
        }
        let _ = fs::remove_file(&sortie);
        let _ = fs::remove_dir_all(&racine);
    }
}
