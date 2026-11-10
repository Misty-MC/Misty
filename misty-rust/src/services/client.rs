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

/// Misty Client en jeu dans les profils Fabric (réglage, activé de base).
pub fn active() -> bool {
    settings::get_bool(settings::MISTY_CLIENT, true)
}

/// Le mod tourne-t-il sur ce profil ? (Fabric, sur une version de Minecraft où il existe)
pub fn compatible(version_mc: &str, mode: &str) -> bool {
    mode == loaders::FABRIC && jar(version_mc).is_some()
}

/// Avant de lancer le jeu : pose le mod dans `mods` (ou le met à jour) si Misty Client est activé
/// (ou que la section Misty Client est lancée : `force`) et que le profil le supporte, sinon le
/// retire (un mod d'une autre version de Minecraft empêcherait le jeu de démarrer).
pub fn preparer(dossier_jeu: &Path, version_mc: &str, mode: &str, force: bool) {
    let chemin = dossier_jeu.join("mods").join(FICHIER);
    match jar(version_mc).filter(|_| (active() || force) && compatible(version_mc, mode)) {
        Some(jar) => {
            if fs::read(&chemin).is_ok_and(|actuel| actuel == jar) {
                return;
            }
            let ecrit = fs::create_dir_all(dossier_jeu.join("mods")).and_then(|_| fs::write(&chemin, jar));
            if let Err(e) = ecrit {
                journal::info(&format!("Misty Client non installé dans {} : {e}", chemin.display()));
            }
        }
        None => {
            if chemin.exists() {
                let _ = fs::remove_file(&chemin);
            }
        }
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

    /// La cape du mod est celle que le launcher dessine dans l'onglet Skin : mêmes pixels.
    #[test]
    fn cape_du_mod_identique() {
        let mod_ = image::load_from_memory(include_bytes!("../../../misty-client/src/main/resources/assets/misty/textures/cape/misty.png")).unwrap().to_rgba8();
        let launcher = image::load_from_memory(crate::services::skins::texture_misty()).unwrap().to_rgba8();
        assert!(mod_ == launcher, "misty-client : textures/cape/misty.png n'est plus la texture de skins::texture_misty()");
    }
}
