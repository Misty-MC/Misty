//! Musique en jeu : le morceau que joue Windows (Spotify, Deezer, YouTube dans le navigateur…),
//! lu par les contrôles multimédia du système (comme le panneau du volume), sans connexion à
//! aucun compte. Pendant qu'une partie tourne, le launcher écrit dans son dossier :
//!
//! - `music.json` : `{ "playing", "title", "artist", "app", "position", "duration", "at", "cover" }`
//!   (temps en millisecondes, `at` : heure Unix de la mesure, pour que le jeu fasse avancer la
//!   barre entre deux mesures ; `cover` : numéro de la pochette, change avec elle) ;
//! - `music-cover.png` : la pochette.
//!
//! Le mod Misty Client (module « Musique » du HUD) lit ces deux fichiers.

use super::{journal, paths};
use std::time::Duration;

/// Toutes les combien le morceau est relu.
const PERIODE: Duration = Duration::from_millis(1500);

/// Démarre la lecture du morceau en cours (un fil à part, qui ne fait rien sans partie lancée).
pub fn demarrer() {
    #[cfg(windows)]
    std::thread::Builder::new()
        .name("musique".into())
        .spawn(windows_musique::boucle)
        .ok();
}

/// Une partie tourne-t-elle ?
fn partie_en_cours() -> bool {
    journal::parties().iter().any(|p| p.en_cours)
}

/// Dossier des fichiers : celui de Misty (en test : `MISTY_MUSIQUE_DIR`).
fn dossier() -> std::path::PathBuf {
    if cfg!(test) {
        if let Some(d) = std::env::var_os("MISTY_MUSIQUE_DIR") {
            return d.into();
        }
    }
    paths::data_dir()
}

fn fichier() -> std::path::PathBuf {
    dossier().join("music.json")
}

fn fichier_pochette() -> std::path::PathBuf {
    dossier().join("music-cover.png")
}

/// Plus de musique (ou plus de partie) : le jeu n'affiche plus rien.
fn effacer() {
    let _ = std::fs::remove_file(fichier());
}

/// Écrit sans jamais laisser le jeu lire un fichier à moitié écrit.
fn ecrire(chemin: &std::path::Path, contenu: &[u8]) {
    let temporaire = chemin.with_extension("tmp");
    if std::fs::write(&temporaire, contenu).is_ok() {
        let _ = std::fs::rename(&temporaire, chemin);
    }
}

#[cfg(windows)]
mod windows_musique {
    use super::*;
    pub(super) use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager as Gestionnaire, GlobalSystemMediaTransportControlsSessionPlaybackStatus as Statut,
    };
    use windows::Storage::Streams::{DataReader, InputStreamOptions};

    /// Le morceau lu à un instant.
    #[derive(PartialEq, Clone, Default)]
    pub(super) struct Morceau {
        titre: String,
        artiste: String,
        app: String,
    }

    pub fn boucle() {
        // WinRT sur ce fil (multithread)
        unsafe {
            let _ = windows::Win32::System::WinRT::RoInitialize(windows::Win32::System::WinRT::RO_INIT_MULTITHREADED);
        }
        let mut gestionnaire = None;
        let mut dernier = Morceau::default();
        let mut pochette = 0u64;
        let mut ecrit = false;
        loop {
            std::thread::sleep(PERIODE);
            if !partie_en_cours() {
                if ecrit {
                    effacer();
                    ecrit = false;
                    dernier = Morceau::default();
                }
                continue;
            }
            if gestionnaire.is_none() {
                gestionnaire = Gestionnaire::RequestAsync().and_then(|op| op.join()).ok();
            }
            let Some(g) = &gestionnaire else { continue };
            match lire(g, &mut dernier, &mut pochette) {
                Some(json) => {
                    ecrire(&fichier(), json.to_string().as_bytes());
                    ecrit = true;
                }
                None if ecrit => {
                    effacer();
                    ecrit = false;
                    dernier = Morceau::default();
                }
                None => {}
            }
        }
    }

    /// Le morceau en cours, en JSON pour le jeu (None : rien ne joue).
    pub(super) fn lire(g: &Gestionnaire, dernier: &mut Morceau, pochette: &mut u64) -> Option<serde_json::Value> {
        let session = g.GetCurrentSession().ok()?;
        let proprietes = session.TryGetMediaPropertiesAsync().ok()?.join().ok()?;
        let titre = proprietes.Title().ok()?.to_string();
        if titre.is_empty() {
            return None;
        }
        let artiste = proprietes.Artist().map(|a| a.to_string()).unwrap_or_default();
        let app = session.SourceAppUserModelId().map(|a| a.to_string()).unwrap_or_default();
        let lecture = session.GetPlaybackInfo().ok().and_then(|i| i.PlaybackStatus().ok());
        let joue = lecture == Some(Statut::Playing);
        let temps = session.GetTimelineProperties().ok();
        // TimeSpan : centaines de nanosecondes
        let position = temps.as_ref().and_then(|t| t.Position().ok()).map(|d| d.Duration / 10_000).unwrap_or(0);
        let duree = temps.as_ref().and_then(|t| t.EndTime().ok()).map(|d| d.Duration / 10_000).unwrap_or(0);
        // instant de la mesure de position (DateTime : centaines de ns depuis 1601)
        let mesure = temps
            .as_ref()
            .and_then(|t| t.LastUpdatedTime().ok())
            .map(|d| d.UniversalTime / 10_000 - 11_644_473_600_000)
            .filter(|&ms| ms > 0)
            .unwrap_or_else(maintenant);

        let morceau = Morceau { titre: titre.clone(), artiste: artiste.clone(), app: app.clone() };
        if morceau != *dernier {
            // nouveau morceau : sa pochette
            *pochette += 1;
            match proprietes.Thumbnail().ok().and_then(|t| lire_pochette(&t)) {
                Some(png) => ecrire(&fichier_pochette(), &png),
                None => {
                    let _ = std::fs::remove_file(fichier_pochette());
                }
            }
            *dernier = morceau;
        }
        Some(serde_json::json!({
            "playing": joue,
            "title": titre,
            "artist": artiste,
            "app": nom_app(&app),
            "position": position,
            "duration": duree,
            "at": mesure,
            "cover": if fichier_pochette().exists() { *pochette } else { 0 },
        }))
    }

    /// La pochette, réduite en PNG 128 × 128 (le jeu l'affiche petite).
    fn lire_pochette(reference: &windows::Storage::Streams::IRandomAccessStreamReference) -> Option<Vec<u8>> {
        let flux = reference.OpenReadAsync().ok()?.join().ok()?;
        let taille = flux.Size().ok()? as u32;
        if taille == 0 || taille > 8_000_000 {
            return None;
        }
        let lecteur = DataReader::CreateDataReader(&flux).ok()?;
        lecteur.SetInputStreamOptions(InputStreamOptions::None).ok()?;
        lecteur.LoadAsync(taille).ok()?.join().ok()?;
        let mut octets = vec![0u8; taille as usize];
        lecteur.ReadBytes(&mut octets).ok()?;
        let image = image::load_from_memory(&octets).ok()?;
        let carree = image.resize_to_fill(128, 128, image::imageops::FilterType::Triangle);
        let mut png = Vec::new();
        carree.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).ok()?;
        Some(png)
    }

    /// "Spotify.exe" / "SpotifyAB.SpotifyMusic_zpdnekdrzrea0!Spotify" → "Spotify".
    fn nom_app(id: &str) -> String {
        let bas = id.to_lowercase();
        for (cle, nom) in [("spotify", "Spotify"), ("deezer", "Deezer"), ("chrome", "Chrome"), ("firefox", "Firefox"), ("msedge", "Edge"), ("opera", "Opera"), ("brave", "Brave"), ("applemusic", "Apple Music"), ("tidal", "Tidal"), ("vlc", "VLC")] {
            if bas.contains(cle) {
                return nom.into();
            }
        }
        id.rsplit(['!', '\\']).next().unwrap_or(id).trim_end_matches(".exe").to_string()
    }

    fn maintenant() -> i64 {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    /// Lit le morceau en cours une fois (réseau : non ; Windows et un lecteur ouvert : oui).
    /// `cargo test musique -- --ignored --nocapture`
    /// Écrit music.json et la pochette dans `MISTY_MUSIQUE_DIR` (pour le jeu de test).
    #[cfg(windows)]
    #[test]
    #[ignore]
    fn musique_ecrite() {
        let g = super::windows_musique::Gestionnaire::RequestAsync().unwrap().join().unwrap();
        let json = super::windows_musique::lire(&g, &mut Default::default(), &mut 0).expect("rien ne joue");
        super::ecrire(&super::fichier(), json.to_string().as_bytes());
        println!("{json}");
    }

    #[cfg(windows)]
    #[test]
    #[ignore]
    fn musique_lue() {
        use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager as G;
        let g = G::RequestAsync().unwrap().join().unwrap();
        match g.GetCurrentSession() {
            Ok(s) => {
                let p = s.TryGetMediaPropertiesAsync().unwrap().join().unwrap();
                println!("{} — {} ({})", p.Title().unwrap(), p.Artist().unwrap(), s.SourceAppUserModelId().unwrap());
            }
            Err(_) => println!("rien ne joue"),
        }
    }
}
