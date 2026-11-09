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

/// Ce qu'un build attend : la version de Minecraft qu'il prépare (le Java qu'il lui faut est celui que
/// Mojang fournit pour cette version, installé avec elle) et l'interface qu'il parle.
#[derive(Debug, Clone)]
pub struct Exigences {
    pub minecraft: String,
    /// SHA-1 du JSON de version que le build attend (Mojang republie parfois 26.3.json)
    pub manifeste_sha1: String,
    /// API de lancement pour les launchers tiers (`--launch-api`, `--output`, `--session-stdin`…) :
    /// les builds récents. Les plus anciens (v0.2.x) n'ont que `--prepare --root` et `--session-file`.
    pub api: bool,
}

/// Lit `research-lock.json` (la version épinglée, présente dans tous les builds) et demande au
/// préparateur s'il connaît l'API de lancement.
pub fn exigences(build: &Path) -> anyhow::Result<Exigences> {
    let verrou: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(build.join("research-lock.json"))?)?;
    let minecraft = verrou["minecraft"]["version"].as_str().unwrap_or("26.3").to_string();
    let manifeste_sha1 = verrou["minecraft"]["manifest_sha1"].as_str().unwrap_or("").to_string();
    let api = commande(&build.join(PREPARE))
        .arg("--launch-api")
        .current_dir(build)
        .output()
        .is_ok_and(|s| s.status.success() && serde_json::from_slice::<serde_json::Value>(&s.stdout).is_ok());
    Ok(Exigences { minecraft, manifeste_sha1, api })
}

/// Ressources préparées que lit horizon-client.
fn ressources(build: &Path, ex: &Exigences) -> PathBuf {
    if ex.api {
        build.join("prepared")
    } else {
        build.join(".local").join("horizon-client").join(&ex.minecraft)
    }
}

/// Marque posée par Misty quand un ancien build a été préparé jusqu'au bout.
fn marque(build: &Path) -> PathBuf {
    build.join(".misty-prepared")
}

/// Le build est-il prêt à jouer ?
pub fn est_prepare(build: &Path, ex: &Exigences) -> bool {
    if !ex.api {
        return marque(build).is_file() && ressources(build, ex).is_dir();
    }
    commande(&build.join(PREPARE))
        .args(["--check", "--output"])
        .arg(ressources(build, ex))
        .current_dir(build)
        .output()
        .is_ok_and(|s| s.status.success())
}

/// Le JSON de version exact que le build attend, à l'endroit où le préparateur le cherche avant celui
/// du launcher : si Mojang a republié la version, celui de Misty n'a plus le SHA-1 épinglé.
pub async fn manifeste_epingle(build: &Path, ex: &Exigences) -> anyhow::Result<PathBuf> {
    let chemin = build.join(".local").join("minecraft").join(&ex.minecraft).join("manifest.json");
    if chemin.is_file() || ex.manifeste_sha1.is_empty() {
        return Ok(chemin);
    }
    let url = format!("https://piston-meta.mojang.com/v1/packages/{}/{}.json", ex.manifeste_sha1, ex.minecraft);
    let octets = http::client().get(&url).send().await?.error_for_status()?.bytes().await?;
    std::fs::create_dir_all(chemin.parent().unwrap_or(build))?;
    std::fs::write(&chemin, &octets)?;
    Ok(chemin)
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

/// Les dernières lignes utiles d'une sortie d'erreur (pas l'aide complète du programme).
fn resume_erreur(texte: &str, defaut: &str) -> String {
    let lignes: Vec<&str> = texte.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    // « error: … » en premier s'il y en a un : c'est la cause
    if let Some(e) = lignes.iter().find(|l| l.contains("error:") || l.starts_with("Error")) {
        return e.to_string();
    }
    let fin = lignes.len().saturating_sub(4);
    let r = lignes[fin..].join("\n");
    if r.is_empty() { defaut.to_string() } else { r }
}

/// Lance le préparateur et suit sa sortie : JSON (`--progress json`) ou texte (anciens builds).
fn executer_preparateur(mut c: Command, avancement: &dyn Fn(f32, &str)) -> anyhow::Result<()> {
    let mut enfant = c.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    // la sortie d'erreur est lue à part : un tuyau plein bloquerait le préparateur
    let lecteur_erreur = enfant.stderr.take().map(|mut e| {
        std::thread::spawn(move || {
            let mut s = String::new();
            let _ = std::io::Read::read_to_string(&mut e, &mut s);
            s
        })
    });
    let mut derniere_erreur = None;
    let mut fraction = 0.0;
    let mut lignes_texte = 0u32;
    if let Some(sortie) = enfant.stdout.take() {
        for ligne in std::io::BufReader::new(sortie).lines().map_while(Result::ok) {
            match serde_json::from_str::<LigneProgression>(&ligne) {
                Ok(l) => {
                    if let Some(f) = l.fraction {
                        fraction = f;
                    }
                    if let Some(e) = l.error {
                        derniere_erreur = Some(e);
                    } else {
                        avancement(fraction, l.message.as_deref().unwrap_or(""));
                    }
                }
                Err(_) if !ligne.trim().is_empty() => {
                    // texte : avancement estimé, qui ralentit sans jamais atteindre la fin
                    lignes_texte += 1;
                    let f = lignes_texte as f32 / (lignes_texte as f32 + 25.0);
                    avancement(f.min(0.95), ligne.trim());
                }
                Err(_) => {}
            }
        }
    }
    let statut = enfant.wait()?;
    let erreur = lecteur_erreur.and_then(|t| t.join().ok()).unwrap_or_default();
    if statut.success() {
        return Ok(());
    }
    anyhow::bail!("{}", derniere_erreur.unwrap_or_else(|| resume_erreur(&erreur, "horizon-prepare a échoué")))
}

/// Prépare un build depuis Minecraft installé par Misty (dossier `minecraft` : versions/, assets/…).
pub fn preparer(build: &Path, ex: &Exigences, minecraft: &Path, server_jar: &Path, java_home: &Path, avancement: impl Fn(f32, &str)) -> anyhow::Result<()> {
    let mut c = commande(&build.join(PREPARE));
    c.current_dir(build).env("JAVA_HOME", java_home);
    if ex.api {
        c.args(["--prepare", "--complete", "--progress", "json", "--output"])
            .arg(ressources(build, ex))
            .arg("--minecraft-dir")
            .arg(minecraft)
            .arg("--server-jar")
            .arg(server_jar)
            .arg("--java-home")
            .arg(java_home);
        let manifeste = build.join(".local").join("minecraft").join(&ex.minecraft).join("manifest.json");
        if manifeste.is_file() {
            c.arg("--version-json").arg(manifeste);
        }
        return executer_preparateur(c, &avancement);
    }
    // ancien build : préparation, puis les sons (facultatifs : sans eux le jeu est muet)
    let _ = std::fs::remove_file(marque(build));
    c.arg("--prepare")
        .arg("--server-jar")
        .arg(server_jar)
        .arg("--minecraft-dir")
        .arg(minecraft)
        .arg("--root")
        .arg(build);
    executer_preparateur(c, &avancement)?;
    let mut sons = commande(&build.join(PREPARE));
    sons.current_dir(build).env("JAVA_HOME", java_home).arg("sounds").arg("--minecraft-dir").arg(minecraft).arg("--root").arg(build);
    let _ = executer_preparateur(sons, &avancement);
    std::fs::write(marque(build), &ex.minecraft)?;
    Ok(())
}

/// Session du joueur, telle qu'Horizon la lit.
pub struct Session {
    pub pseudo: String,
    pub uuid: String,
    pub jeton: String,
    pub xuid: String,
}

/// Lance horizon-client et attend la fin de la partie. Avec une session Microsoft, le joueur est
/// connecté (serveurs en ligne) ; sans, il joue hors ligne sous `pseudo`. Erreur : la dernière ligne
/// de sa sortie d'erreur (qui nomme le rapport de plantage).
pub fn jouer(build: &Path, ex: &Exigences, session: Option<&Session>, pseudo: &str) -> anyhow::Result<()> {
    let jeu = dossier_jeu();
    std::fs::create_dir_all(&jeu)?;
    let mut c = Command::new(build.join(CLIENT));
    c.arg("--resources").arg(ressources(build, ex)).current_dir(&jeu).env("HORIZON_HOME", &jeu).stderr(Stdio::piped());
    if ex.api {
        c.arg("--game-dir").arg(&jeu).arg("--home").arg(&jeu);
    }
    let mut json = None;
    let mut fichier_session = None;
    match session {
        Some(session) => {
            let texte = serde_json::json!({
                "name": session.pseudo,
                "uuid": session.uuid,
                "access_token": session.jeton,
                "user_type": "msa",
                "xuid": session.xuid,
            })
            .to_string();
            if ex.api {
                c.arg("--session-stdin").stdin(Stdio::piped());
                json = Some(texte);
            } else {
                // ancien build : la session passe par un fichier que le jeu supprime dès qu'il l'a lu
                let dossier = jeu.join("sessions");
                std::fs::create_dir_all(&dossier)?;
                let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
                let f = dossier.join(format!("session-{nanos:x}.json"));
                std::fs::write(&f, &texte)?;
                c.arg("--session-file").arg(&f);
                fichier_session = Some(f);
            }
        }
        None => {
            // hors ligne : le pseudo du compte choisi dans Misty (variable lue par les builds récents,
            // fichier gardé par le jeu, et --name pour les anciens builds)
            c.env("HORIZON_PLAYER_NAME", pseudo);
            let _ = std::fs::write(jeu.join("horizon-player.txt"), format!("name:{pseudo}\n"));
            if !ex.api {
                c.arg("--name").arg(pseudo);
            }
        }
    }
    c.stdout(Stdio::piped());
    let mut enfant = match c.spawn() {
        Ok(e) => e,
        Err(e) => {
            super::journal::ajouter(&format!("horizon-client : {e}"), super::journal::ERREUR);
            super::journal::fin(Some(-1));
            return Err(e.into());
        }
    };
    if let (Some(mut entree), Some(json)) = (enfant.stdin.take(), json) {
        entree.write_all(json.as_bytes())?;
    } // l'entrée est fermée ici, comme l'attend Horizon
    // la sortie va dans la console ; celle d'erreur est aussi gardée pour résumer un échec
    let copie = super::journal::Copie::default();
    let fils = super::journal::suivre(&mut enfant, Some(copie.clone()));
    let statut = enfant.wait()?;
    for f in fils {
        let _ = f.join();
    }
    super::journal::fin(statut.code());
    let erreur = copie.lock().map(|c| c.clone()).unwrap_or_default();
    if let Some(f) = fichier_session {
        let _ = std::fs::remove_file(f); // au cas où le jeu ne l'aurait pas lu
    }
    if statut.success() {
        Ok(())
    } else {
        anyhow::bail!("{}", resume_erreur(&erreur, "horizon-client s'est arrêté"))
    }
}
