//! Mise à jour automatique via les releases GitHub, avec le site en secours.
//!
//! Fonctionnement : le launcher est installé dans un dossier "Misty-{version}". La mise à jour
//! télécharge le zip de la dernière release à côté, l'extrait (=> "Misty-{nouvelle}"), note l'ancien
//! dossier dans data.txt (clé Suppr) puis lance le nouvel exe. Au démarrage suivant, le nouvel exe
//! supprime l'ancien dossier et met à jour le raccourci.
//!
//! Secours : le site garde une copie de la dernière version. Si l'API GitHub ne répond pas (quota,
//! panne), la version vient de `/api/latest` ; si le téléchargement depuis GitHub échoue, le zip
//! vient de `/download/portable`.

use super::{app_info, http, paths, settings, shortcut};
use std::fs;
use std::path::{Path, PathBuf};

/// Site de Misty : copie de la dernière version (voir Misty-Web, `miroir`).
const SITE: &str = "https://misty-launcher.online";

pub struct ReleaseInfo {
    pub version: String,
    /// Pré-release (bêta) de GitHub
    pub prerelease: bool,
    /// Adresses du zip, essayées dans l'ordre (GitHub puis le site)
    pub sources: Vec<String>,
    /// Taille du zip annoncée par GitHub ou le site (0 si inconnue)
    pub taille: u64,
}

/// Étape de l'installation, pour l'affichage.
pub enum Etape {
    /// octets reçus, taille totale (0 si inconnue), téléchargement depuis le site
    Telechargement { recu: u64, total: u64, site: bool },
    Extraction,
    Lancement,
}

fn url_site() -> String {
    format!("{SITE}/download/portable")
}

/// Renvoie la dernière release si elle est plus récente que la version actuelle, sinon None.
/// GitHub d'abord, le site si GitHub ne répond pas.
pub async fn verifier() -> anyhow::Result<Option<ReleaseInfo>> {
    match verifier_github().await {
        Ok(r) => Ok(r),
        Err(e_github) => match verifier_site().await {
            Ok(r) => Ok(r),
            // les deux sont injoignables : l'erreur de GitHub est la plus parlante
            Err(_) => Err(e_github),
        },
    }
}

async fn verifier_site() -> anyhow::Result<Option<ReleaseInfo>> {
    let doc: serde_json::Value = http::get_json(&format!("{SITE}/api/latest")).await?;
    let version = doc["version"].as_str().unwrap_or("").trim_start_matches('v').to_string();
    if version.is_empty() {
        anyhow::bail!("{SITE}/api/latest : version absente");
    }
    let taille = doc["portableSize"].as_u64().unwrap_or(0);
    Ok(est_plus_recente(&version, app_info::VERSION).then(|| ReleaseInfo { version, prerelease: false, sources: vec![url_site()], taille }))
}

async fn verifier_github() -> anyhow::Result<Option<ReleaseInfo>> {
    // GitHub refuse les requêtes sans User-Agent (le client partagé en a un)
    let doc: serde_json::Value = if super::settings::get_bool(super::settings::MAJ_PRERELEASE, false) {
        // canal pré-release : la plus haute version parmi les releases publiées, bêtas comprises
        let liste: serde_json::Value =
            http::get_json(&format!("https://api.github.com/repos/{}/releases?per_page=30", app_info::GITHUB_REPO)).await?;
        let mut candidates: Vec<serde_json::Value> = liste
            .as_array()
            .into_iter()
            .flatten()
            .filter(|r| !r["draft"].as_bool().unwrap_or(false) && r["tag_name"].as_str().is_some_and(|t| !t.is_empty()))
            .cloned()
            .collect();
        let tag = |r: &serde_json::Value| r["tag_name"].as_str().unwrap_or("").trim_start_matches('v').to_string();
        candidates.sort_by(|a, b| comparer_versions(&tag(a), &tag(b)));
        match candidates.pop() {
            Some(r) => r,
            None => return Ok(None),
        }
    } else {
        http::get_json(&format!("https://api.github.com/repos/{}/releases/latest", app_info::GITHUB_REPO)).await?
    };
    let prerelease = doc["prerelease"].as_bool().unwrap_or(false);

    let version = doc["tag_name"].as_str().unwrap_or("").trim_start_matches('v').to_string();
    // Le zip "Misty-x.y.zip" (la release peut aussi contenir l'installateur .exe)
    let zip = doc["assets"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|a| a["browser_download_url"].as_str().is_some_and(|u| u.to_lowercase().ends_with(".zip")));
    let url = zip.and_then(|a| a["browser_download_url"].as_str()).map(str::to_string);
    let taille = zip.and_then(|a| a["size"].as_u64()).unwrap_or(0);

    if version.is_empty() {
        anyhow::bail!("GitHub : release sans version");
    }
    if !est_plus_recente(&version, app_info::VERSION) {
        return Ok(None);
    }
    // sans zip sur GitHub (pas encore envoyé), le site peut quand même l'avoir
    // (le site ne garde que la dernière version stable : pas de secours pour une pré-release)
    let sources = if prerelease { url.into_iter().collect() } else { url.into_iter().chain([url_site()]).collect() };
    Ok(Some(ReleaseInfo { version, prerelease, sources, taille }))
}

/// Vrai si `distante` est plus récente que `locale` ("0.2.3.9.5.1" > "0.2.3.9.5",
/// "0.3.7" > "0.3.7-beta.2" > "0.3.7-beta.1" > "0.3.6").
pub fn est_plus_recente(distante: &str, locale: &str) -> bool {
    comparer_versions(distante, locale) == std::cmp::Ordering::Greater
}

/// Ordre des versions : les nombres d'abord ("0.3.7"), puis le suffixe de pré-release ("-beta.1",
/// "b2", "-rc1"…) : une version sans suffixe passe après ses pré-releases.
pub fn comparer_versions(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    // "0.3.7-beta.1" -> ([0, 3, 7], "beta.1")
    let decouper = |v: &str| -> (Vec<u64>, String) {
        let v = v.trim().trim_start_matches(['v', 'V']);
        let fin = v.find(|c: char| !(c.is_ascii_digit() || c == '.')).unwrap_or(v.len());
        let nombres = v[..fin].split('.').filter(|s| !s.is_empty()).map(|s| s.parse().unwrap_or(0)).collect();
        (nombres, v[fin..].trim_start_matches(['-', '.', '_', '+']).to_lowercase())
    };
    // "beta.10" -> ["beta", 10] : les nombres se comparent comme des nombres
    let morceaux = |s: &str| -> Vec<Result<u64, String>> {
        let mut liste = Vec::new();
        let mut courant = String::new();
        for c in s.chars() {
            let change = !courant.is_empty() && courant.chars().last().is_some_and(|d| d.is_ascii_digit()) != c.is_ascii_digit();
            if !c.is_ascii_alphanumeric() || change {
                if !courant.is_empty() {
                    liste.push(courant.parse::<u64>().map_err(|_| courant.clone()));
                    courant.clear();
                }
            }
            if c.is_ascii_alphanumeric() {
                courant.push(c);
            }
        }
        if !courant.is_empty() {
            liste.push(courant.parse::<u64>().map_err(|_| courant.clone()));
        }
        liste
    };
    let ((na, sa), (nb, sb)) = (decouper(a), decouper(b));
    for i in 0..na.len().max(nb.len()) {
        let (x, y) = (na.get(i).copied().unwrap_or(0), nb.get(i).copied().unwrap_or(0));
        if x != y {
            return x.cmp(&y);
        }
    }
    match (sa.is_empty(), sb.is_empty()) {
        (true, true) => return Ordering::Equal,
        (true, false) => return Ordering::Greater,
        (false, true) => return Ordering::Less,
        _ => {}
    }
    let (ma, mb) = (morceaux(&sa), morceaux(&sb));
    for i in 0..ma.len().max(mb.len()) {
        let ordre = match (ma.get(i), mb.get(i)) {
            (Some(Ok(x)), Some(Ok(y))) => x.cmp(y),
            (Some(Err(x)), Some(Err(y))) => x.cmp(y),
            // un nombre passe avant un mot ("1" < "beta")
            (Some(Ok(_)), Some(Err(_))) => Ordering::Less,
            (Some(Err(_)), Some(Ok(_))) => Ordering::Greater,
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => Ordering::Equal,
        };
        if ordre != Ordering::Equal {
            return ordre;
        }
    }
    Ordering::Equal
}

/// Dossier qui contient les dossiers "Misty-{version}".
fn racine() -> PathBuf {
    let dossier_actuel = format!("Misty-{}", app_info::VERSION);
    let app = paths::app_dir().to_string_lossy().to_string();
    match app.find(&dossier_actuel) {
        Some(i) => PathBuf::from(&app[..i]),
        None => PathBuf::from(app),
    }
}

/// Télécharge le zip depuis une source, en signalant l'avancement.
async fn telecharger(url: &str, version: &str, taille: u64, zip: &Path, avancement: &(dyn Fn(Etape) + Send + Sync)) -> anyhow::Result<()> {
    use futures::StreamExt;
    use tokio::io::AsyncWriteExt;
    let site = url.starts_with(SITE);
    let reponse = http::client()
        .get(url)
        // le délai du client partagé (5 min) est trop court pour une connexion lente
        .timeout(std::time::Duration::from_secs(3600))
        // un zip ne gagne rien à être recompressé, et la taille reste annoncée
        .header(reqwest::header::ACCEPT_ENCODING, "identity")
        .send()
        .await?
        .error_for_status()?;
    // le site sert toujours sa dernière version : elle doit être celle qu'on attend
    if site {
        let attendu = format!("Misty-{version}.zip");
        let nom = reponse
            .headers()
            .get(reqwest::header::CONTENT_DISPOSITION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !nom.contains(&attendu) {
            anyhow::bail!("le site propose un autre fichier que {attendu}");
        }
    }
    let total = reponse.content_length().filter(|&n| n > 0).unwrap_or(taille);
    let tmp = zip.with_extension("zip.part");
    let mut recu = 0u64;
    avancement(Etape::Telechargement { recu, total, site });
    let resultat = async {
        let mut sortie = tokio::fs::File::create(&tmp).await?;
        let mut flux = reponse.bytes_stream();
        while let Some(morceau) = flux.next().await {
            let morceau = morceau?;
            sortie.write_all(&morceau).await?;
            recu += morceau.len() as u64;
            avancement(Etape::Telechargement { recu, total, site });
        }
        sortie.flush().await?;
        if total != 0 && recu != total {
            anyhow::bail!("téléchargement incomplet ({recu} octets sur {total})");
        }
        anyhow::Ok(())
    }
    .await;
    if let Err(e) = resultat {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    let _ = fs::remove_file(zip);
    fs::rename(&tmp, zip)?;
    Ok(())
}

/// Télécharge, extrait et lance la nouvelle version. L'appelant doit ensuite quitter l'application.
pub async fn installer(release: &ReleaseInfo, avancement: impl Fn(Etape) + Send + Sync) -> anyhow::Result<()> {
    let racine = racine();
    let zip = racine.join(format!("Misty-{}.zip", release.version));
    let nouveau_dossier = racine.join(format!("Misty-{}", release.version));

    // chaque source à son tour : GitHub, puis la copie du site
    let mut erreur = None;
    for url in &release.sources {
        match telecharger(url, &release.version, release.taille, &zip, &avancement).await {
            Ok(()) => {
                erreur = None;
                break;
            }
            Err(e) => erreur = Some(e.context(url.clone())),
        }
    }
    if let Some(e) = erreur {
        return Err(e);
    }
    avancement(Etape::Extraction);
    {
        let zip = zip.clone();
        let racine = racine.clone();
        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            let mut archive = zip::ZipArchive::new(fs::File::open(&zip)?)?;
            archive.extract(&racine)?;
            Ok(())
        })
        .await??;
    }
    let _ = fs::remove_file(&zip);
    if !nouveau_dossier.join("Misty.exe").exists() {
        anyhow::bail!("Misty.exe absent de l'archive (Misty-{})", release.version);
    }

    settings::set(settings::SUPPR, &racine.join(format!("Misty-{}", app_info::VERSION)).to_string_lossy());

    if paths::custom_icon().exists() {
        let _ = fs::copy(paths::custom_icon(), nouveau_dossier.join("Icon.ico"));
    }

    avancement(Etape::Lancement);
    std::process::Command::new(nouveau_dossier.join("Misty.exe")).current_dir(&nouveau_dossier).spawn()?;
    Ok(())
}

/// Après une mise à jour : supprime l'ancien dossier et remet le raccourci sur le nouvel exe.
/// Renvoie true si une mise à jour vient d'être finalisée.
pub async fn finaliser() -> anyhow::Result<bool> {
    let ancien = settings::get(settings::SUPPR);
    if ancien.is_empty() {
        return Ok(false);
    }

    tokio::time::sleep(std::time::Duration::from_secs(1)).await; // laisse le temps à l'ancien exe de se fermer

    let raccourci = settings::get(settings::RACCOURCI);
    if !raccourci.is_empty() {
        let _ = shortcut::creer(Path::new(&raccourci));
    }

    // Installé avec le setup : ses raccourcis (menu Démarrer, bureau) et l'entrée
    // "Applications installées" pointent encore vers l'ancien dossier
    shortcut::rediriger(&ancien);
    maj_entree_desinstallation();

    if Path::new(&ancien).exists() {
        fs::remove_dir_all(&ancien)?;
    }
    settings::set(settings::SUPPR, "");
    Ok(true)
}

/// ID de l'installateur Inno Setup (installer\Misty.iss).
#[cfg(windows)]
const CLE_DESINSTALLATION: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{8F4C2A61-3B7E-4D19-9A55-6C1E0F2B7D43}_is1";

/// Met à jour la version et l'icône affichées dans "Applications installées".
#[cfg(windows)]
fn maj_entree_desinstallation() {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
    let Ok(cle) = winreg::RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(CLE_DESINSTALLATION, KEY_SET_VALUE) else { return };
    let _ = cle.set_value("DisplayVersion", &app_info::VERSION);
    let _ = cle.set_value("DisplayIcon", &paths::app_exe().to_string_lossy().to_string());
}

#[cfg(not(windows))]
fn maj_entree_desinstallation() {}

/// Migre l'ancien dossier de données C:\TEXT vers %LOCALAPPDATA%\Misty.
/// Renvoie true si une migration a eu lieu.
pub fn migrer_ancien_dossier() -> anyhow::Result<bool> {
    let ancien = Path::new(paths::LEGACY_DATA_DIR);
    if !ancien.exists() {
        return Ok(false);
    }
    if !paths::data_dir().exists() {
        copier_dossier(ancien, &paths::data_dir())?;
    }
    fs::remove_dir_all(ancien)?;
    Ok(true)
}

fn copier_dossier(source: &Path, destination: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(destination)?;
    for entree in fs::read_dir(source)?.flatten() {
        let cible = destination.join(entree.file_name());
        if entree.path().is_dir() {
            copier_dossier(&entree.path(), &cible)?;
        } else {
            fs::copy(entree.path(), cible)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordre_des_versions() {
        assert!(est_plus_recente("0.3.7", "0.3.6"));
        assert!(est_plus_recente("0.2.3.9.5.1", "0.2.3.9.5"));
        assert!(!est_plus_recente("0.3.6", "0.3.6"));
        assert!(est_plus_recente("0.3.7-beta.1", "0.3.6"));
        assert!(est_plus_recente("0.3.7", "0.3.7-beta.2"));
        assert!(est_plus_recente("0.3.7-beta.2", "0.3.7-beta.1"));
        assert!(est_plus_recente("0.3.7-beta.10", "0.3.7-beta.9"));
        assert!(est_plus_recente("0.3.7-rc1", "0.3.7-beta3"));
        assert!(!est_plus_recente("0.3.7-beta.1", "0.3.7"));
        assert!(est_plus_recente("v0.3.8", "0.3.7"));
    }

    /// Secours par le site (réseau) : `cargo test secours_site -- --ignored`.
    #[tokio::test]
    #[ignore]
    async fn secours_site() {
        let doc: serde_json::Value = http::get_json(&format!("{SITE}/api/latest")).await.unwrap();
        let version = doc["version"].as_str().unwrap().trim_start_matches('v').to_string();
        let zip = std::env::temp_dir().join(format!("misty-test-{version}.zip"));
        let max = std::sync::Mutex::new((0u64, 0u64, 0usize));
        let taille = doc["portableSize"].as_u64().unwrap();
        telecharger(&url_site(), &version, taille, &zip, &|e| {
            if let Etape::Telechargement { recu, total, site } = e {
                assert!(site);
                let mut m = max.lock().unwrap();
                *m = (recu, total, m.2 + 1);
            }
        })
        .await
        .unwrap();
        let (recu, total, appels) = *max.lock().unwrap();
        assert_eq!(recu, total);
        assert_eq!(fs::metadata(&zip).unwrap().len(), total);
        zip::ZipArchive::new(fs::File::open(&zip).unwrap()).unwrap();
        let _ = fs::remove_file(&zip);
        // une version qui n'est pas celle du site est refusée
        assert!(telecharger(&url_site(), "0.0.1", 0, &zip, &|_| {}).await.is_err());
        println!("{version} : {total} octets, {appels} étapes");
    }
}
