//! Horizon (https://playhorizon.dev) : Minecraft: Java Edition 26.3 réécrit en Rust, open source (MIT),
//! non officiel. Misty ne l'embarque pas : il pilote le launcher d'Horizon par sa ligne de commande
//! (`horizon-launcher`), qui installe le jeu, prépare ses fichiers depuis Minecraft et gère le compte.
//!
//! - le launcher Horizon déjà installé est réutilisé ; sinon Misty télécharge sa dernière version
//!   (releases GitHub `launcher@…`) dans son propre dossier ;
//! - Horizon exige un compte Microsoft qui possède Minecraft: Java Edition, connecté dans son launcher.

use super::{http, paths};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const SITE: &str = "https://playhorizon.dev";
pub const GITHUB: &str = "https://github.com/horizon-rust/horizon";
const RELEASES: &str = "https://api.github.com/repos/horizon-rust/horizon/releases?per_page=40";
const ARCHIVE: &str = "horizon-launcher-windows-x64.zip";
const EXE: &str = "horizon-launcher.exe";

/// Où en est Horizon sur ce PC.
#[derive(Clone, Debug, Default)]
pub struct Etat {
    /// Le launcher Horizon (chemin) et sa version
    pub launcher: Option<PathBuf>,
    pub version_launcher: String,
    /// Version du jeu choisie dans le launcher Horizon (vide : pas encore installé)
    pub jeu: String,
    /// Compte Microsoft qui jouera (vide : aucun)
    pub compte: String,
    /// Le compte est prêt pour jouer en ligne
    pub compte_pret: bool,
}

impl Etat {
    pub fn pret(&self) -> bool {
        self.launcher.is_some() && !self.jeu.is_empty() && !self.compte.is_empty()
    }
}

fn dossier() -> PathBuf {
    paths::data_dir().join("horizon")
}

/// Le launcher Horizon : celui que Misty a installé, sinon une installation trouvée sur le PC.
pub fn trouver_launcher() -> Option<PathBuf> {
    let mut candidats: Vec<PathBuf> = Vec::new();
    if let Some(p) = chercher(&dossier(), 3) {
        candidats.push(p);
    }
    if let Some(local) = dirs::data_local_dir() {
        for d in ["Programs\\horizon-launcher", "Programs\\Horizon Launcher", "Programs\\Horizon", "horizon-launcher"] {
            candidats.push(local.join(d).join(EXE));
        }
    }
    if let Some(bureau) = dirs::desktop_dir() {
        candidats.push(bureau.join(EXE));
    }
    if let Some(dl) = dirs::download_dir() {
        candidats.push(dl.join(EXE));
    }
    // dans le PATH
    if let Ok(sortie) = Command::new("where").arg(EXE).output() {
        if let Some(l) = String::from_utf8_lossy(&sortie.stdout).lines().next() {
            candidats.push(PathBuf::from(l.trim()));
        }
    }
    candidats.into_iter().find(|p| p.is_file())
}

fn chercher(dossier: &Path, profondeur: u32) -> Option<PathBuf> {
    let entrees = std::fs::read_dir(dossier).ok()?;
    let mut sous = Vec::new();
    for e in entrees.flatten() {
        let p = e.path();
        if p.is_file() && p.file_name().is_some_and(|n| n.eq_ignore_ascii_case(EXE)) {
            return Some(p);
        }
        if p.is_dir() {
            sous.push(p);
        }
    }
    if profondeur == 0 {
        return None;
    }
    sous.into_iter().find_map(|d| chercher(&d, profondeur - 1))
}

/// Lance une commande du launcher Horizon et renvoie sa sortie (stdout), ou l'erreur affichée.
fn executer(launcher: &Path, args: &[&str]) -> anyhow::Result<String> {
    let mut commande = Command::new(launcher);
    commande.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        commande.creation_flags(0x0800_0000); // CREATE_NO_WINDOW : pas de console noire
    }
    let sortie = commande.output()?;
    let texte = String::from_utf8_lossy(&sortie.stdout).to_string();
    if sortie.status.success() {
        Ok(texte)
    } else {
        let erreur = String::from_utf8_lossy(&sortie.stderr).trim().to_string();
        anyhow::bail!("{}", if erreur.is_empty() { texte.trim().to_string() } else { erreur })
    }
}

#[derive(Deserialize)]
struct Installees {
    #[serde(default)]
    installed: Vec<String>,
    #[serde(default)]
    selected: Option<String>,
}

#[derive(Deserialize)]
struct Comptes {
    #[serde(default)]
    accounts: Vec<Compte>,
}

#[derive(Deserialize)]
struct Compte {
    name: String,
    #[serde(default)]
    selected: bool,
    #[serde(default)]
    authentication: Option<Authentification>,
}

#[derive(Deserialize)]
struct Authentification {
    #[serde(default)]
    state: String,
}

/// État d'Horizon (rapide : ne contacte pas le réseau).
pub fn etat() -> Etat {
    let Some(launcher) = trouver_launcher() else { return Etat::default() };
    let version_launcher = executer(&launcher, &["--version"])
        .map(|v| v.trim().trim_start_matches("horizon-launcher").trim().to_string())
        .unwrap_or_default();
    let jeu = executer(&launcher, &["versions", "--installed", "--json"])
        .ok()
        .and_then(|t| serde_json::from_str::<Installees>(&t).ok())
        .and_then(|i| i.selected.or_else(|| i.installed.into_iter().next()))
        .unwrap_or_default();
    let compte = executer(&launcher, &["account", "list", "--json"])
        .ok()
        .and_then(|t| serde_json::from_str::<Comptes>(&t).ok())
        .and_then(|c| c.accounts.into_iter().find(|a| a.selected));
    Etat {
        launcher: Some(launcher),
        version_launcher,
        jeu,
        compte_pret: compte.as_ref().and_then(|c| c.authentication.as_ref()).is_some_and(|a| a.state == "ready"),
        compte: compte.map(|c| c.name).unwrap_or_default(),
    }
}

/// Télécharge la dernière version du launcher Horizon dans le dossier de Misty.
pub async fn installer_launcher() -> anyhow::Result<PathBuf> {
    let releases: serde_json::Value = http::get_json(RELEASES).await?;
    let url = releases
        .as_array()
        .into_iter()
        .flatten()
        .filter(|r| r["tag_name"].as_str().is_some_and(|t| t.starts_with("launcher@")) && !r["draft"].as_bool().unwrap_or(false))
        .flat_map(|r| r["assets"].as_array().cloned().unwrap_or_default())
        .find(|a| a["name"].as_str() == Some(ARCHIVE))
        .and_then(|a| a["browser_download_url"].as_str().map(str::to_string))
        .ok_or_else(|| anyhow::anyhow!("aucune version du launcher Horizon pour Windows"))?;
    let octets = http::get_bytes(&url).await?;
    let cible = dossier();
    let _ = std::fs::remove_dir_all(&cible);
    std::fs::create_dir_all(&cible)?;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(octets))?;
    zip.extract(&cible)?;
    chercher(&cible, 3).ok_or_else(|| anyhow::anyhow!("{EXE} introuvable dans l'archive"))
}

/// Installe la dernière version du jeu Horizon (le launcher la télécharge).
pub fn installer_jeu(launcher: &Path) -> anyhow::Result<()> {
    executer(launcher, &["install", "latest"]).map(|_| ())
}

/// Connexion Microsoft dans Horizon : le launcher affiche un code à saisir dans le navigateur.
/// On l'ouvre dans sa propre fenêtre de console, pour que le joueur voie le code.
pub fn connecter(launcher: &Path) -> anyhow::Result<()> {
    Command::new("cmd")
        .args(["/c", "start", "Horizon - Microsoft", "/wait"])
        .arg(launcher)
        .args(["account", "sign-in"])
        .spawn()?;
    Ok(())
}

/// Lance Horizon. Si ses fichiers ne sont pas encore prêts, on les prépare (depuis une installation de
/// Minecraft trouvée sur le PC, sinon en les téléchargeant chez Mojang pour le compte connecté).
pub fn jouer(launcher: &Path) -> anyhow::Result<()> {
    if executer(launcher, &["play"]).is_ok() {
        return Ok(());
    }
    if executer(launcher, &["prepare"]).is_err() {
        executer(launcher, &["prepare", "--download"])?;
    }
    executer(launcher, &["play"]).map(|_| ())
}
