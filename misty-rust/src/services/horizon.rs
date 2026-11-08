//! Horizon (https://playhorizon.dev) : Minecraft: Java Edition 26.3 réécrit en Rust, open source (MIT),
//! non officiel.
//!
//! Misty le lance lui-même, sans le launcher d'Horizon, en suivant son API de lancement pour les
//! launchers tiers (docs/api/launch.md) :
//! 1. les builds viennent des releases GitHub du projet (nightlies et versions stables), un dossier
//!    par build : on peut en garder plusieurs et choisir celui qu'on lance ;
//! 2. un build se prépare une fois (`horizon-prepare`) depuis Minecraft 26.3 installé par Misty, son
//!    Java et le server.jar de Mojang (téléchargé seulement pour un compte qui possède le jeu) ;
//! 3. `horizon-client` démarre avec la session Microsoft du joueur sur son entrée standard.

use super::{http, paths};
use futures::StreamExt;
use serde::Deserialize;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const SITE: &str = "https://playhorizon.dev";
pub const GITHUB: &str = "https://github.com/horizon-rust/horizon";
const RELEASES: &str = "https://api.github.com/repos/horizon-rust/horizon/releases?per_page=100";
const ARCHIVE: &str = "horizon-windows-x64.zip";
const CLIENT: &str = "horizon-client.exe";
const PREPARE: &str = "horizon-prepare.exe";

/// Un build publié sur GitHub.
#[derive(Clone, Debug)]
pub struct Build {
    pub tag: String,
    pub nightly: bool,
    /// « 2026-11-07 »
    pub date: String,
    pub url: String,
    pub taille: u64,
}

pub fn dossier() -> PathBuf {
    paths::data_dir().join("horizon")
}

fn dossier_builds() -> PathBuf {
    dossier().join("builds")
}

/// Dossier de jeu d'Horizon (mondes, options, packs) : le même pour tous les builds.
pub fn dossier_jeu() -> PathBuf {
    dossier().join("game")
}

/// Builds disponibles sur GitHub, du plus récent au plus ancien (sans le tag mobile « nightly »).
pub async fn builds_disponibles() -> anyhow::Result<Vec<Build>> {
    let releases: serde_json::Value = http::get_json(RELEASES).await?;
    let mut builds: Vec<Build> = releases
        .as_array()
        .into_iter()
        .flatten()
        .filter(|r| !r["draft"].as_bool().unwrap_or(false))
        .filter_map(|r| {
            let tag = r["tag_name"].as_str()?.to_string();
            let nightly = tag.starts_with("nightly-");
            if !nightly && !tag.starts_with('v') {
                return None; // launcher@…, « nightly » mobile…
            }
            let archive = r["assets"].as_array()?.iter().find(|a| a["name"].as_str() == Some(ARCHIVE))?;
            Some(Build {
                date: r["published_at"].as_str().unwrap_or("").get(..10).unwrap_or("").to_string(),
                url: archive["browser_download_url"].as_str()?.to_string(),
                taille: archive["size"].as_u64().unwrap_or(0),
                tag,
                nightly,
            })
        })
        .collect();
    builds.sort_by(|a, b| b.date.cmp(&a.date).then(b.tag.cmp(&a.tag)));
    Ok(builds)
}

/// Builds installés (dossier de chaque build : celui qui contient horizon-client.exe).
pub fn installes() -> Vec<String> {
    let Ok(entrees) = std::fs::read_dir(dossier_builds()) else { return Vec::new() };
    let mut tags: Vec<String> = entrees
        .flatten()
        .filter(|e| dossier_build(&e.file_name().to_string_lossy()).is_some())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    tags.sort_by(|a, b| b.cmp(a));
    tags
}

pub fn dossier_build(tag: &str) -> Option<PathBuf> {
    chercher(&dossier_builds().join(tag), CLIENT, 2).and_then(|p| p.parent().map(Path::to_path_buf))
}

fn chercher(dossier: &Path, nom: &str, profondeur: u32) -> Option<PathBuf> {
    let entrees = std::fs::read_dir(dossier).ok()?;
    let mut sous = Vec::new();
    for e in entrees.flatten() {
        let p = e.path();
        if p.is_file() && p.file_name().is_some_and(|n| n.eq_ignore_ascii_case(nom)) {
            return Some(p);
        }
        if p.is_dir() {
            sous.push(p);
        }
    }
    if profondeur == 0 {
        return None;
    }
    sous.into_iter().find_map(|d| chercher(&d, nom, profondeur - 1))
}

/// Télécharge et extrait un build (`avancement` : de 0 à 1).
pub async fn installer_build(build: &Build, avancement: impl Fn(f32)) -> anyhow::Result<()> {
    let cible = dossier_builds().join(&build.tag);
    let archive = dossier().join(format!("{}.zip", build.tag));
    std::fs::create_dir_all(dossier())?;
    let reponse = http::client().get(&build.url).send().await?.error_for_status()?;
    let total = reponse.content_length().unwrap_or(build.taille).max(1);
    let mut fichier = std::fs::File::create(&archive)?;
    let mut recu = 0u64;
    let mut flux = reponse.bytes_stream();
    while let Some(morceau) = flux.next().await {
        let morceau = morceau?;
        fichier.write_all(&morceau)?;
        recu += morceau.len() as u64;
        avancement(recu as f32 / total as f32);
    }
    drop(fichier);
    let _ = std::fs::remove_dir_all(&cible);
    std::fs::create_dir_all(&cible)?;
    let resultat = (|| -> anyhow::Result<()> {
        let mut zip = zip::ZipArchive::new(std::fs::File::open(&archive)?)?;
        zip.extract(&cible)?;
        Ok(())
    })();
    let _ = std::fs::remove_file(&archive);
    resultat?;
    anyhow::ensure!(dossier_build(&build.tag).is_some(), "{CLIENT} introuvable dans l'archive");
    Ok(())
}

pub fn supprimer_build(tag: &str) -> std::io::Result<()> {
    std::fs::remove_dir_all(dossier_builds().join(tag))
}

fn commande(exe: &Path) -> Command {
    let mut c = Command::new(exe);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW : pas de console noire
    }
    c
}

/// Ce qu'un build sait préparer : la version de Minecraft dont il lit les fichiers (le Java qu'il lui
/// faut est celui que Mojang fournit pour cette version, installé avec elle).
#[derive(Debug, Clone)]
pub struct Exigences {
    pub minecraft: String,
}

/// `horizon-prepare --launch-api`.
pub fn exigences(build: &Path) -> anyhow::Result<Exigences> {
    let sortie = commande(&build.join(PREPARE)).arg("--launch-api").current_dir(build).output()?;
    anyhow::ensure!(sortie.status.success(), "{}", String::from_utf8_lossy(&sortie.stderr).trim());
    let v: serde_json::Value = serde_json::from_slice(&sortie.stdout)?;
    let minecraft = match &v["minecraft"] {
        serde_json::Value::String(s) => s.clone(),
        o => o["version"].as_str().or_else(|| o["id"].as_str()).unwrap_or("26.3").to_string(),
    };
    Ok(Exigences { minecraft })
}

fn dossier_prepare(build: &Path) -> PathBuf {
    build.join("prepared")
}

/// Le build est-il prêt à jouer ? (`horizon-prepare --check`)
pub fn est_prepare(build: &Path) -> bool {
    commande(&build.join(PREPARE))
        .args(["--check", "--output"])
        .arg(dossier_prepare(build))
        .current_dir(build)
        .output()
        .is_ok_and(|s| s.status.success())
}

/// Le server.jar de la version (downloads.server du JSON de version), gardé dans le dossier d'Horizon.
pub async fn server_jar(version_json: &serde_json::Value, version: &str) -> anyhow::Result<PathBuf> {
    let chemin = dossier().join(format!("server-{version}.jar"));
    if chemin.is_file() {
        return Ok(chemin);
    }
    let url = version_json["downloads"]["server"]["url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("pas de server.jar pour {version}"))?;
    let octets = http::client().get(url).send().await?.error_for_status()?.bytes().await?;
    std::fs::create_dir_all(dossier())?;
    std::fs::write(&chemin, &octets)?;
    Ok(chemin)
}

#[derive(Deserialize)]
struct LigneProgression {
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    fraction: Option<f32>,
    #[serde(default)]
    error: Option<String>,
}

/// Prépare un build depuis Minecraft installé par Misty (`--progress json` : avancement en direct).
pub fn preparer(build: &Path, minecraft: &Path, server_jar: &Path, java_home: &Path, avancement: impl Fn(f32, &str)) -> anyhow::Result<()> {
    let mut enfant = commande(&build.join(PREPARE))
        .args(["--prepare", "--complete", "--progress", "json", "--output"])
        .arg(dossier_prepare(build))
        .arg("--minecraft-dir")
        .arg(minecraft)
        .arg("--server-jar")
        .arg(server_jar)
        .arg("--java-home")
        .arg(java_home)
        .current_dir(build)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut derniere_erreur = None;
    let mut fraction = 0.0;
    if let Some(sortie) = enfant.stdout.take() {
        for ligne in std::io::BufReader::new(sortie).lines().map_while(Result::ok) {
            let Ok(l) = serde_json::from_str::<LigneProgression>(&ligne) else { continue };
            if let Some(f) = l.fraction {
                fraction = f;
            }
            if let Some(e) = l.error {
                derniere_erreur = Some(e);
            } else if let Some(m) = l.message {
                avancement(fraction, &m);
            } else {
                avancement(fraction, "");
            }
        }
    }
    let statut = enfant.wait()?;
    if statut.success() {
        return Ok(());
    }
    let mut erreur = String::new();
    if let Some(mut e) = enfant.stderr.take() {
        let _ = std::io::Read::read_to_string(&mut e, &mut erreur);
    }
    anyhow::bail!("{}", derniere_erreur.unwrap_or_else(|| erreur.lines().last().unwrap_or("horizon-prepare a échoué").to_string()))
}

/// Session du joueur, telle qu'Horizon la lit sur son entrée standard.
pub struct Session {
    pub pseudo: String,
    pub uuid: String,
    pub jeton: String,
    pub xuid: String,
}

/// Lance horizon-client et attend la fin de la partie. Erreur : la dernière ligne de sa sortie
/// d'erreur (qui nomme le rapport de plantage).
pub fn jouer(build: &Path, session: &Session) -> anyhow::Result<()> {
    let jeu = dossier_jeu();
    std::fs::create_dir_all(&jeu)?;
    let mut enfant = Command::new(build.join(CLIENT))
        .arg("--resources")
        .arg(dossier_prepare(build))
        .arg("--game-dir")
        .arg(&jeu)
        .arg("--home")
        .arg(&jeu)
        .arg("--session-stdin")
        .current_dir(build)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let json = serde_json::json!({
        "name": session.pseudo,
        "uuid": session.uuid,
        "access_token": session.jeton,
        "user_type": "msa",
        "xuid": session.xuid,
    });
    if let Some(mut entree) = enfant.stdin.take() {
        entree.write_all(json.to_string().as_bytes())?;
    } // l'entrée est fermée ici, comme l'attend Horizon
    let mut erreur = String::new();
    if let Some(mut e) = enfant.stderr.take() {
        let _ = std::io::Read::read_to_string(&mut e, &mut erreur);
    }
    let statut = enfant.wait()?;
    if statut.success() {
        Ok(())
    } else {
        anyhow::bail!("{}", erreur.lines().filter(|l| !l.trim().is_empty()).last().unwrap_or("horizon-client s'est arrêté").to_string())
    }
}
