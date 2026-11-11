//! Misty Client : le mod Fabric de Misty (dossier `misty-client/` du dépôt), qui apporte en jeu le
//! menu principal Misty, le HUD, le zoom, la luminosité maximale, le nuage Misty devant les
//! joueurs Misty (liste des joueurs et chat) et les cosmétiques Misty.
//!
//! Un jar par version de Minecraft, tous intégrés à l'exe à la compilation (voir build.rs) : rien à
//! télécharger, et le mod a toujours la version qui va avec le launcher.
//!
//! Deux façons de jouer avec : la section « Misty Client » du choix de version (Fabric + le mod, sur
//! une des versions où il existe), et le réglage « Misty Client en jeu », qui le pose dans le
//! dossier `mods` des profils Fabric de ces versions. Il est retiré partout ailleurs. Le jeu reçoit
//! le dossier de Misty (`-Dmisty.launcher.dir`) : le mod y lit `client.json`, où le launcher
//! écrit le cosmétique équipé.

use super::instances::Instance;
use super::{journal, paths, settings};
use crate::minecraft::loaders;
use std::fs;
use std::path::Path;

// JARS : (version de Minecraft, jar du mod), généré par build.rs
include!(concat!(env!("OUT_DIR"), "/misty_client.rs"));

/// Nom du mod dans le dossier `mods` : toujours le même, pour remplacer l'ancien à chaque mise à jour.
const FICHIER: &str = "misty-client.jar";

/// Cosmétiques Misty, tels que le mod et le site les nomment. Dans le dos : une cape ou des élytres.
pub const CAPE: &str = "misty_cape";
pub const ELYTRES: &str = "misty_elytra";
/// Cape des premiers joueurs (le site dit qui y a droit : `early()`).
pub const CAPE_EARLY: &str = "early_cape";
/// Sur la tête : un petit nuage.
pub const CLOUDLY: &str = "cloudly";

const DOS: &[&str] = &[CAPE, ELYTRES, CAPE_EARLY];
const CHAPEAUX: &[&str] = &[CLOUDLY];

/// Le mod est-il dans cet exe (pour au moins une version) ?
pub fn disponible() -> bool {
    !JARS.is_empty()
}

/// Le jar du mod pour cette version de Minecraft.
fn jar(version_mc: &str) -> Option<&'static [u8]> {
    JARS.iter().find(|(mc, _)| *mc == version_mc).map(|(_, jar)| *jar)
}

/// Les versions de Minecraft où Misty Client existe, de la plus ancienne à la plus récente.
pub fn versions() -> Vec<&'static str> {
    JARS.iter().map(|(mc, _)| *mc).collect()
}

/// La version de Minecraft la plus récente où Misty Client existe ("" sans mod).
pub fn derniere() -> &'static str {
    JARS.last().map(|(mc, _)| *mc).unwrap_or("")
}

/// Misty Client existe-t-il pour cette version de Minecraft ?
pub fn existe(version_mc: &str) -> bool {
    jar(version_mc).is_some()
}

/// Les versions du mod en une ligne, pour l'affichage ("1.21.11, 26.1, 26.3").
pub fn versions_texte() -> String {
    versions().join(", ")
}

/// Case « Misty Client » cochée de base dans les profils Fabric (Options → Dépannage, activé de
/// base). Vaut aussi pour le jeu rapide en Fabric.
pub fn active() -> bool {
    settings::get_bool(settings::MISTY_CLIENT, true)
}

/// Misty Client voulu dans ce profil : sa case, ou le réglage par défaut s'il n'y a pas touché.
pub fn voulu(instance: &Instance) -> bool {
    instance.misty_client.unwrap_or_else(active)
}

/// Le mod tourne-t-il sur ce profil ? (Fabric, sur une version de Minecraft où il existe)
pub fn compatible(version_mc: &str, mode: &str) -> bool {
    mode == loaders::FABRIC && jar(version_mc).is_some()
}

/// Version d'un jar de Misty Client, lue dans son fabric.mod.json ("1.0.0+mc26.2"). None : ce
/// n'est pas Misty Client.
fn version_du_jar(contenu: impl std::io::Read + std::io::Seek) -> Option<String> {
    let mut zip = zip::ZipArchive::new(contenu).ok()?;
    let mut fichier = zip.by_name("fabric.mod.json").ok()?;
    let mut texte = String::new();
    std::io::Read::read_to_string(&mut fichier, &mut texte).ok()?;
    let json: serde_json::Value = serde_json::from_str(&texte).ok()?;
    (json["id"].as_str() == Some("misty-client")).then(|| json["version"].as_str().unwrap_or_default().to_string())
}

/// "1.0.0+mc26.2" → ("1.0.0", "26.2")
fn decouper(version: &str) -> (&str, &str) {
    version.split_once("+mc").unwrap_or((version, ""))
}

/// Le jar du mod pour cette version et son nom de fichier avec la version
/// ("misty-client-1.0.0+mc26.2.jar"), pour l'export d'un profil.
pub fn jar_a_exporter(version_mc: &str) -> Option<(String, &'static [u8])> {
    let jar = jar(version_mc)?;
    let version = version_du_jar(std::io::Cursor::new(jar)).unwrap_or_default();
    Some((format!("misty-client-{version}.jar"), jar))
}

/// Les jars de Misty Client d'un dossier `mods`, avec leur version : quel que soit leur nom (un
/// modpack exporté les nomme avec leur version, d'anciens launchers « misty-client.jar »).
pub fn jars_presents(mods: &Path) -> Vec<(std::path::PathBuf, String)> {
    let Ok(lecture) = fs::read_dir(mods) else { return Vec::new() };
    lecture
        .flatten()
        .map(|f| f.path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("jar")))
        .filter_map(|p| {
            let version = version_du_jar(fs::File::open(&p).ok()?)?;
            Some((p, version))
        })
        .collect()
}

/// Avant de lancer le jeu : pose le mod dans `mods` (ou le met à jour) si `voulu` et que le profil
/// le supporte, sinon le retire (un mod d'une autre version de Minecraft empêcherait le jeu de
/// démarrer). Les jars du mod déjà là sont comparés au nôtre : les anciens (ou ceux d'une autre
/// version de Minecraft) sont supprimés, pour qu'il n'en reste jamais qu'un ; un plus récent que
/// celui du launcher (modpack exporté par un launcher plus à jour) est gardé.
pub fn preparer(dossier_jeu: &Path, version_mc: &str, mode: &str, voulu: bool) {
    let mods = dossier_jeu.join("mods");
    let presents = jars_presents(&mods);
    let notre = jar(version_mc).filter(|_| voulu && compatible(version_mc, mode));
    let Some(notre) = notre else {
        for (chemin, _) in presents {
            let _ = fs::remove_file(&chemin);
        }
        return;
    };
    let version = version_du_jar(std::io::Cursor::new(notre)).unwrap_or_default();
    let (numero, mc) = decouper(&version);

    // le plus récent des jars déjà là, s'il est pour cette version de Minecraft et plus récent que le nôtre
    let garde = presents
        .iter()
        .filter(|(_, v)| decouper(v).1 == mc && super::update::comparer_versions(decouper(v).0, numero).is_gt())
        .max_by(|a, b| super::update::comparer_versions(decouper(&a.1).0, decouper(&b.1).0))
        .map(|(p, _)| p.clone());
    let chemin = garde.clone().unwrap_or_else(|| mods.join(FICHIER));
    for (autre, v) in &presents {
        if *autre != chemin {
            journal::info(&format!("Ancien Misty Client retiré : {} ({v})", autre.display()));
            let _ = fs::remove_file(autre);
        }
    }
    if garde.is_some() || fs::read(&chemin).is_ok_and(|actuel| actuel == notre) {
        return;
    }
    let ecrit = fs::create_dir_all(&mods).and_then(|_| fs::write(&chemin, notre));
    if let Err(e) = ecrit {
        journal::info(&format!("Misty Client non installé dans {} : {e}", chemin.display()));
    }
}

/// La garde-robe du jeu (Misty Client) écrit son choix dans `client.json` avec `"from": "game"` :
/// le launcher le reprend au lieu d'écrire le sien par-dessus.
pub fn reprendre_du_jeu() {
    let Ok(texte) = fs::read_to_string(paths::data_dir().join("client.json")) else { return };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&texte) else { return };
    if json["from"].as_str() != Some("game") {
        return;
    }
    let choix = json["cosmetic"].as_str().unwrap_or("");
    if choix.is_empty() || DOS.contains(&choix) {
        if choix != settings::get(settings::COSMETIQUE) {
            settings::set(settings::COSMETIQUE, choix);
            journal::info(&format!("Cosmétique choisi en jeu : {}", if choix.is_empty() { "aucun" } else { choix }));
        }
    }
    let chapeau = json["hat"].as_str().unwrap_or("");
    if (chapeau.is_empty() || CHAPEAUX.contains(&chapeau)) && chapeau != settings::get(settings::CHAPEAU) {
        settings::set(settings::CHAPEAU, chapeau);
        journal::info(&format!("Chapeau choisi en jeu : {}", if chapeau.is_empty() { "aucun" } else { chapeau }));
    }
}

/// Cosmétique Misty porté dans le dos ("" : aucun).
pub fn cosmetique() -> String {
    reprendre_du_jeu();
    let c = settings::get(settings::COSMETIQUE);
    if DOS.contains(&c.as_str()) {
        c
    } else {
        String::new()
    }
}

/// Chapeau Misty porté ("" : aucun).
pub fn chapeau() -> String {
    reprendre_du_jeu();
    let c = settings::get(settings::CHAPEAU);
    if CHAPEAUX.contains(&c.as_str()) {
        c
    } else {
        String::new()
    }
}

/// Met (ou enlève : "") le chapeau Misty.
pub fn equiper_chapeau(chapeau: &str) {
    if chapeau != settings::get(settings::CHAPEAU) {
        settings::set(settings::CHAPEAU, chapeau);
    }
    ecrire_lien();
}

/// Ce compte a-t-il droit à la cape early player ? (le site le sait : comptes vus avant une date)
pub async fn early(uuid: &str) -> bool {
    let url = format!("https://misty-launcher.online/api/client/early/{uuid}");
    super::http::get_json::<serde_json::Value>(&url).await.ok().and_then(|v| v["early"].as_bool()).unwrap_or(false)
}

/// Équipe un cosmétique Misty (ou aucun) : il est gardé d'une session à l'autre et le mod le
/// voit tout de suite, même en pleine partie.
pub fn equiper(cosmetique: &str) {
    if cosmetique != settings::get(settings::COSMETIQUE) {
        settings::set(settings::COSMETIQUE, cosmetique);
    }
    ecrire_lien();
}

/// Écrit `client.json` (lu par le mod) dans le dossier de Misty.
pub fn ecrire_lien() {
    let contenu = serde_json::json!({ "cosmetic": cosmetique(), "hat": chapeau() });
    let chemin = paths::data_dir().join("client.json");
    if fs::read_to_string(&chemin).is_ok_and(|actuel| actuel == contenu.to_string()) {
        return;
    }
    let _ = fs::write(chemin, contenu.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profils_compatibles() {
        if !disponible() {
            return;
        }
        for mc in versions() {
            assert!(compatible(mc, loaders::FABRIC));
            assert!(!compatible(mc, loaders::VANILLA));
            assert!(!compatible(mc, loaders::FORGE));
        }
        assert!(!compatible("0.0.1", loaders::FABRIC));
        // ordre des versions (build.rs) : de la plus ancienne à la plus récente
        let v = versions();
        assert!(v.windows(2).all(|p| crate::services::update::comparer_versions(p[0], p[1]).is_lt()));
    }

    /// Un faux jar de Misty Client (juste son fabric.mod.json).
    fn faux_jar(version: &str) -> Vec<u8> {
        use std::io::Write;
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        zip.start_file("fabric.mod.json", zip::write::SimpleFileOptions::default()).unwrap();
        write!(zip, r#"{{"id": "misty-client", "version": "{version}"}}"#).unwrap();
        zip.finish().unwrap().into_inner()
    }

    /// Les anciens jars du mod (autre nom, autre version de Minecraft) sont retirés, un seul reste.
    #[test]
    fn un_seul_jar_du_mod() {
        let Some(mc) = versions().last().copied() else { return };
        let dossier = std::env::temp_dir().join(format!("misty-test-mods-{}", std::process::id()));
        let mods = dossier.join("mods");
        let _ = fs::remove_dir_all(&dossier);
        fs::create_dir_all(&mods).unwrap();
        fs::write(mods.join("misty-client-0.0.1+mc{mc}.jar".replace("{mc}", mc)), faux_jar(&format!("0.0.1+mc{mc}"))).unwrap();
        fs::write(mods.join("ancien.jar"), faux_jar("9.9.9+mc0.0")).unwrap();
        fs::write(mods.join("autre-mod.jar"), b"pas un zip").unwrap();
        preparer(&dossier, mc, loaders::FABRIC, true);
        let restants = jars_presents(&mods);
        assert_eq!(restants.len(), 1, "{restants:?}");
        assert_eq!(restants[0].0, mods.join(FICHIER));
        assert!(mods.join("autre-mod.jar").exists());
        // décoché : plus aucun
        preparer(&dossier, mc, loaders::FABRIC, false);
        assert!(jars_presents(&mods).is_empty());
        // un plus récent que celui du launcher est gardé
        fs::write(mods.join("recent.jar"), faux_jar(&format!("99.0.0+mc{mc}"))).unwrap();
        preparer(&dossier, mc, loaders::FABRIC, true);
        let restants = jars_presents(&mods);
        assert_eq!(restants.len(), 1);
        assert_eq!(restants[0].0, mods.join("recent.jar"));
        let _ = fs::remove_dir_all(&dossier);
    }

    /// La cape du mod est celle que le launcher dessine dans l'onglet Skin : mêmes pixels.
    #[test]
    fn cape_du_mod_identique() {
        let mod_ = image::load_from_memory(include_bytes!("../../../misty-client/src/main/resources/assets/misty/textures/cape/misty.png")).unwrap().to_rgba8();
        let launcher = image::load_from_memory(crate::services::skins::texture_misty()).unwrap().to_rgba8();
        assert!(mod_ == launcher, "misty-client : textures/cape/misty.png n'est plus la texture de skins::texture_misty()");
    }
}
