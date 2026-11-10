//! Misty Client : le mod Fabric de Misty (dossier `misty-client/` du dépôt), qui apporte en jeu le
//! menu principal Misty, le HUD, le nuage Misty devant les joueurs Misty (liste des joueurs et
//! chat) et les cosmétiques Misty.
//!
//! Son jar est intégré à l'exe à la compilation (voir build.rs) : rien à télécharger, et le mod a
//! toujours la version qui va avec le launcher. À chaque lancement, il est posé dans le dossier
//! `mods` des profils qu'il sait faire tourner (Fabric, sur sa version de Minecraft) et retiré des
//! autres. Le jeu reçoit le dossier de Misty (`-Dmisty.launcher.dir`) : le mod y lit `client.json`,
//! où le launcher écrit le cosmétique équipé.

use super::{journal, paths, settings};
use crate::minecraft::loaders;
use std::fs;
use std::path::Path;

/// Le jar du mod (vide si le mod n'était pas compilé quand le launcher l'a été).
const JAR: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/misty-client.jar"));
/// Version de Minecraft pour laquelle le mod est compilé ("" sans mod).
pub const VERSION_MC: &str = env!("MISTY_CLIENT_MC");
/// Nom du mod dans le dossier `mods` : toujours le même, pour remplacer l'ancien à chaque mise à jour.
const FICHIER: &str = "misty-client.jar";

/// Cosmétiques Misty, tels que le mod et le site les nomment.
pub const CAPE: &str = "misty_cape";
pub const ELYTRES: &str = "misty_elytra";

/// Le mod est-il dans cet exe ?
pub fn disponible() -> bool {
    !JAR.is_empty()
}

/// Misty Client en jeu (réglage, activé par défaut).
pub fn active() -> bool {
    settings::get_bool(settings::MISTY_CLIENT, true)
}

/// Le mod tourne-t-il sur ce profil ? (Fabric, et la version de Minecraft du mod)
pub fn compatible(version_mc: &str, mode: &str) -> bool {
    disponible() && mode == loaders::FABRIC && version_mc == VERSION_MC
}

/// Avant de lancer le jeu : pose le mod dans `mods` (ou le met à jour) si Misty Client est activé
/// et que le profil le supporte, sinon le retire (un mod d'une autre version de Minecraft
/// empêcherait le jeu de démarrer).
pub fn preparer(dossier_jeu: &Path, version_mc: &str, mode: &str) {
    let chemin = dossier_jeu.join("mods").join(FICHIER);
    if active() && compatible(version_mc, mode) {
        if fs::read(&chemin).is_ok_and(|actuel| actuel == JAR) {
            return;
        }
        let ecrit = fs::create_dir_all(dossier_jeu.join("mods")).and_then(|_| fs::write(&chemin, JAR));
        if let Err(e) = ecrit {
            journal::info(&format!("Misty Client non installé dans {} : {e}", chemin.display()));
        }
    } else if chemin.exists() {
        let _ = fs::remove_file(&chemin);
    }
}

/// Cosmétique Misty équipé ("" : aucun).
pub fn cosmetique() -> String {
    let c = settings::get(settings::COSMETIQUE);
    if c == CAPE || c == ELYTRES {
        c
    } else {
        String::new()
    }
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
    let contenu = serde_json::json!({ "cosmetic": cosmetique() });
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
        assert!(compatible(VERSION_MC, loaders::FABRIC));
        assert!(!compatible(VERSION_MC, loaders::VANILLA));
        assert!(!compatible(VERSION_MC, loaders::FORGE));
        assert!(!compatible("1.8.9", loaders::FABRIC));
    }

    /// La cape du mod est celle que le launcher dessine dans l'onglet Skin : mêmes pixels.
    #[test]
    fn cape_du_mod_identique() {
        let mod_ = image::load_from_memory(include_bytes!("../../../misty-client/src/main/resources/assets/misty/textures/cape/misty.png")).unwrap().to_rgba8();
        let launcher = image::load_from_memory(crate::services::skins::texture_misty()).unwrap().to_rgba8();
        assert!(mod_ == launcher, "misty-client : textures/cape/misty.png n'est plus la texture de skins::texture_misty()");
    }
}
