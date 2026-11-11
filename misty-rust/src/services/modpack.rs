//! Modpacks : installation dans un nouveau profil, import depuis un fichier (reconnu sur Modrinth
//! ou CurseForge quand c'est possible) et mise à jour d'un profil vers une autre version du pack.
//! Formats :
//! - Modrinth (.mrpack) : https://support.modrinth.com/en/articles/8802351-modrinth-modpack-format-mrpack
//! - CurseForge (zip + manifest.json)

use super::catalogue::{ProjetDistant, CURSEFORGE, MODRINTH};
use super::content::{self, ContenuMeta};
use super::instances::{self, Instance};
use super::{curseforge, download, loc, modrinth, Progress};
use crate::minecraft::loaders;
use futures::StreamExt;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Fichier qu'on n'a pas le droit de télécharger automatiquement (choix de l'auteur sur CurseForge).
#[derive(Clone)]
pub struct FichierBloque {
    pub nom: String,
    pub page: String,
    pub dossier: String,
}

pub struct Resultat {
    pub instance: Instance,
    pub bloques: Vec<FichierBloque>,
}

/// Version d'un modpack proposée à l'utilisateur.
#[derive(Clone)]
pub struct VersionPack {
    pub id: String,
    pub libelle: String,
}

fn etiquette_type(type_: &str) -> &'static str {
    match type_ {
        "beta" | "2" => " [beta]",
        "alpha" | "3" => " [alpha]",
        _ => "",
    }
}

/// Versions disponibles du modpack, de la plus récente à la plus ancienne.
pub async fn versions_disponibles(projet: &ProjetDistant) -> anyhow::Result<Vec<VersionPack>> {
    let mut liste = Vec::new();
    if projet.source == CURSEFORGE {
        for f in curseforge::fichiers(projet.id.parse()?, None).await? {
            if !f.is_available {
                continue;
            }
            let mc = f.game_versions.iter().find(|v| v.chars().next().is_some_and(|c| c.is_ascii_digit())).cloned().unwrap_or_default();
            let nom = if f.display_name.is_empty() { f.file_name.clone() } else { f.display_name.clone() };
            let libelle = if mc.is_empty() { nom } else { format!("{nom} · {mc}") };
            liste.push(VersionPack { id: f.id.to_string(), libelle: format!("{libelle}{}", etiquette_type(&f.release_type.to_string())) });
        }
    } else {
        for v in modrinth::versions(&projet.id, None, None).await? {
            if !v.files.iter().any(|f| f.filename.to_lowercase().ends_with(".mrpack")) {
                continue;
            }
            let nom = if v.version_number.is_empty() { v.name.clone() } else { v.version_number.clone() };
            let libelle = match v.game_versions.last() {
                Some(mc) => format!("{nom} · {mc}"),
                None => nom,
            };
            liste.push(VersionPack { id: v.id.clone(), libelle: format!("{libelle}{}", etiquette_type(&v.version_type)) });
        }
    }
    liste.truncate(60);
    Ok(liste)
}

/// `version_id` : version choisie (None = la dernière stable).
pub async fn installer(projet: &ProjetDistant, version_id: Option<String>, progression: Progress) -> anyhow::Result<Resultat> {
    let mut instance: Option<Instance> = None;
    let mut temporaire: Option<PathBuf> = None;

    let resultat = if projet.source == CURSEFORGE {
        installer_curseforge(projet, version_id.as_deref(), &progression, &mut instance, &mut temporaire).await
    } else {
        installer_modrinth(projet, version_id.as_deref(), &progression, &mut instance, &mut temporaire).await
    };

    if let Some(t) = temporaire {
        let _ = std::fs::remove_file(t);
    }
    // Si quelque chose échoue, le profil à moitié créé est supprimé
    if resultat.is_err() {
        if let Some(i) = &instance {
            let _ = instances::supprimer(i);
        }
    }
    resultat
}

/// Télécharge l'archive dans le dossier temporaire et l'ouvre.
async fn telecharger_archive(
    url: &str,
    sha1: Option<&str>,
    progression: &Progress,
    temporaire: &mut Option<PathBuf>,
) -> anyhow::Result<zip::ZipArchive<std::fs::File>> {
    progression(loc::t("Progres_Modpack"));
    let chemin = std::env::temp_dir().join(format!("misty-{}.zip", uuid::Uuid::new_v4().simple()));
    *temporaire = Some(chemin.clone());
    download::fichier(url, &chemin, sha1).await?;
    Ok(zip::ZipArchive::new(std::fs::File::open(&chemin)?)?)
}

/// D'où vient le modpack : le catalogue, ou un fichier importé (reconnu sur Modrinth / CurseForge
/// si c'est possible, pour pouvoir le mettre à jour ensuite).
struct Origine {
    titre: String,
    icone_url: Option<String>,
    /// (plateforme, id du projet) : None pour un fichier importé qu'on n'a pas reconnu
    projet: Option<(String, String)>,
    /// Version précise du pack (id Modrinth, ou id du fichier CurseForge)
    version_id: Option<String>,
    /// Importé depuis un fichier : le profil prend le nom écrit dans le pack
    depuis_fichier: bool,
}

impl Origine {
    fn catalogue(p: &ProjetDistant, version_id: Option<String>) -> Self {
        Origine {
            titre: p.titre.clone(),
            icone_url: p.icone_url.clone(),
            projet: Some((p.source.clone(), p.id.clone())),
            version_id,
            depuis_fichier: false,
        }
    }

    /// Fichier importé : on cherche s'il est publié tel quel sur Modrinth (SHA-1) ou CurseForge
    /// (empreinte). Introuvable ou hors ligne : un simple profil, sans mise à jour possible.
    async fn reconnaitre(chemin: &Path, modrinth_format: bool) -> Self {
        let titre = chemin.file_stem().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let mut o = Origine { titre, icone_url: None, projet: None, version_id: None, depuis_fichier: true };
        let c = chemin.to_path_buf();
        if modrinth_format {
            let Ok(Ok(h)) = tokio::task::spawn_blocking(move || download::sha1_fichier(&c)).await else { return o };
            if let Ok(trouve) = modrinth::versions_par_hash(std::slice::from_ref(&h)).await {
                if let Some(v) = trouve.get(&h) {
                    o.projet = Some((MODRINTH.into(), v.project_id.clone()));
                    o.version_id = Some(v.id.clone());
                    if let Ok(p) = modrinth::projets(std::slice::from_ref(&v.project_id)).await {
                        o.icone_url = p.first().and_then(|p| p.icon_url.clone());
                    }
                }
            }
        } else if curseforge::disponible() {
            let Ok(Ok(e)) = tokio::task::spawn_blocking(move || curseforge::empreinte(&c)).await else { return o };
            if let Ok(trouves) = curseforge::empreintes(&[e]).await {
                if let Some(t) = trouves.first() {
                    o.projet = Some((CURSEFORGE.into(), t.id.to_string()));
                    o.version_id = Some(t.file.id.to_string());
                    if let Ok(m) = curseforge::mods(&[t.id]).await {
                        o.icone_url = m.first().and_then(|m| m.logo.as_ref()).and_then(|l| l.thumbnail_url.clone());
                    }
                }
            }
        }
        o
    }

    /// Un fichier importé prend le nom écrit dans le modpack, s'il en a un.
    fn titre_avec(&self, nom_du_pack: &str) -> String {
        if self.depuis_fichier && !nom_du_pack.trim().is_empty() {
            nom_du_pack.trim().to_string()
        } else {
            self.titre.clone()
        }
    }
}

/// Profil visé : un nouveau (installation, import) ou un existant (mise à jour du pack).
enum Cible<'a> {
    Nouveau(&'a Origine),
    /// Profil à mettre à jour, avec la version choisie et les fichiers de l'ancienne version
    Existant { instance: Instance, version_id: String, anciens: Vec<String> },
}

/// Crée le profil, ou met à jour celui du pack (version de Minecraft, loader, version du pack).
fn preparer_profil(cible: &Cible, titre: &str, version_mc: &str, loader: &str, version_loader: Option<String>, version_pack: String) -> anyhow::Result<Instance> {
    match cible {
        Cible::Nouveau(origine) => {
            let mut instance = instances::creer(titre, version_mc, loader, version_loader)?;
            instance.icone_url = origine.icone_url.clone();
            if let Some((source, id)) = &origine.projet {
                instance.modpack_source = Some(source.clone());
                instance.modpack_project_id = Some(id.clone());
                instance.modpack_version_id = origine.version_id.clone();
            }
            instance.modpack_version = Some(version_pack).filter(|v| !v.is_empty());
            instances::sauver(&instance)?;
            Ok(instance)
        }
        Cible::Existant { instance, version_id, .. } => {
            let mut instance = instance.clone();
            instance.version_mc = version_mc.to_string();
            instance.loader = loader.to_string();
            instance.loader_version = version_loader;
            instance.modpack_version = Some(version_pack).filter(|v| !v.is_empty());
            instance.modpack_version_id = Some(version_id.clone());
            instances::sauver(&instance)?;
            Ok(instance)
        }
    }
}

/// Importe un modpack depuis un fichier : .mrpack (Modrinth) ou .zip (CurseForge, avec manifest.json).
pub async fn importer_fichier(chemin: &Path, progression: Progress) -> anyhow::Result<Resultat> {
    let mut instance: Option<Instance> = None;
    let resultat = importer_fichier_interne(chemin, &progression, &mut instance).await;
    // Si quelque chose échoue, le profil à moitié créé est supprimé
    if resultat.is_err() {
        if let Some(i) = &instance {
            let _ = instances::supprimer(i);
        }
    }
    resultat
}

async fn importer_fichier_interne(chemin: &Path, progression: &Progress, sortie: &mut Option<Instance>) -> anyhow::Result<Resultat> {
    progression(loc::t("Progres_Modpack"));
    let mut zip = zip::ZipArchive::new(std::fs::File::open(chemin)?).map_err(|_| anyhow::anyhow!(loc::t("Erreur_ModpackIllisible")))?;
    let modrinth_format = zip.by_name("modrinth.index.json").is_ok();
    if !modrinth_format && zip.by_name("manifest.json").is_err() {
        anyhow::bail!(loc::t("Erreur_ModpackFormat"));
    }
    progression(loc::t("Progres_Reconnaissance"));
    let origine = Origine::reconnaitre(chemin, modrinth_format).await;
    if modrinth_format {
        appliquer_modrinth(&mut zip, Cible::Nouveau(&origine), None, progression, sortie).await
    } else {
        appliquer_curseforge(&mut zip, Cible::Nouveau(&origine), None, progression, sortie).await
    }
}

// ───────────────────────── Mise à jour ─────────────────────────

/// Versions disponibles pour le modpack d'un profil (None : profil qui n'est pas un modpack connu).
pub async fn versions_du_profil(instance: &Instance) -> anyhow::Result<Vec<VersionPack>> {
    versions_disponibles(&projet_du_profil(instance)?).await
}

fn projet_du_profil(instance: &Instance) -> anyhow::Result<ProjetDistant> {
    let id = instance.modpack_project_id.clone().ok_or_else(|| anyhow::anyhow!(loc::t("Erreur_PasUnModpack")))?;
    Ok(ProjetDistant {
        source: instance.modpack_source.clone().unwrap_or_else(|| MODRINTH.into()),
        id,
        titre: instance.nom.clone(),
        icone_url: instance.icone_url.clone(),
        ..Default::default()
    })
}

/// Id de la version installée : celui enregistré, sinon retrouvé d'après son nom (profils créés
/// avant que Misty ne le garde).
pub async fn version_installee(instance: &Instance) -> Option<String> {
    if let Some(id) = &instance.modpack_version_id {
        return Some(id.clone());
    }
    let nom = instance.modpack_version.clone()?;
    let projet = projet_du_profil(instance).ok()?;
    if projet.source == CURSEFORGE {
        let fichiers = curseforge::fichiers(projet.id.parse().ok()?, None).await.ok()?;
        fichiers.iter().find(|f| f.display_name == nom || f.file_name == nom).map(|f| f.id.to_string())
    } else {
        let versions = modrinth::versions(&projet.id, None, None).await.ok()?;
        versions.iter().find(|v| v.version_number == nom || v.id == nom).map(|v| v.id.clone())
    }
}

/// Met le modpack d'un profil à la version `version_id` : les fichiers de l'ancienne version qui ne
/// sont plus dans la nouvelle sont retirés, les nouveaux téléchargés, la configuration du pack
/// remise. Les mondes, les options et ce que le joueur a ajouté lui-même restent.
pub async fn mettre_a_jour(instance: &Instance, version_id: String, progression: Progress) -> anyhow::Result<Resultat> {
    let projet = projet_du_profil(instance)?;
    // fichiers posés par la version actuelle
    progression(loc::t("Progres_AncienPack"));
    let anciens = match lire_fichiers_pack(instance) {
        Some(liste) => liste,
        None => match version_installee(instance).await {
            Some(id) => fichiers_d_une_version(&projet, &id, instance).await.unwrap_or_default(),
            None => Vec::new(),
        },
    };
    let cible = Cible::Existant { instance: instance.clone(), version_id: version_id.clone(), anciens };
    let mut temporaire = None;
    let mut sortie = None;
    let resultat = if projet.source == CURSEFORGE {
        let fichier = curseforge::fichiers_par_ids(&[version_id.parse()?]).await?.into_iter().next().ok_or_else(|| anyhow::anyhow!(loc::t("Erreur_ModpackSansVersion")))?;
        let Some(url) = fichier.download_url.clone().filter(|u| !u.is_empty()) else {
            anyhow::bail!(loc::t("Erreur_ModpackCFInterdit"));
        };
        let mut zip = telecharger_archive(&url, fichier.sha1(), &progression, &mut temporaire).await?;
        appliquer_curseforge(&mut zip, cible, Some(fichier.display_name.clone()), &progression, &mut sortie).await
    } else {
        let version = modrinth::version(&version_id).await?;
        let f = version.files.iter().find(|f| f.filename.to_lowercase().ends_with(".mrpack")).ok_or_else(|| anyhow::anyhow!(loc::t("Erreur_SansMrpack")))?;
        let mut zip = telecharger_archive(&f.url, f.hashes.get("sha1").map(String::as_str), &progression, &mut temporaire).await?;
        appliquer_modrinth(&mut zip, cible, Some(version.version_number.clone()), &progression, &mut sortie).await
    };
    if let Some(t) = temporaire {
        let _ = std::fs::remove_file(t);
    }
    resultat
}

/// Chemins des fichiers d'une version du pack (sans rien installer), pour savoir quoi retirer.
async fn fichiers_d_une_version(projet: &ProjetDistant, version_id: &str, instance: &Instance) -> anyhow::Result<Vec<String>> {
    let mut temporaire = None;
    let progression: Progress = std::sync::Arc::new(|_| {});
    let resultat = async {
        if projet.source == CURSEFORGE {
            let fichier = curseforge::fichiers_par_ids(&[version_id.parse()?]).await?.into_iter().next().ok_or_else(|| anyhow::anyhow!("?"))?;
            let url = fichier.download_url.clone().filter(|u| !u.is_empty()).ok_or_else(|| anyhow::anyhow!("?"))?;
            let mut zip = telecharger_archive(&url, fichier.sha1(), &progression, &mut temporaire).await?;
            let manifeste: curseforge::ManifestCF = lire_json(&mut zip, "manifest.json")?;
            let liste = fichiers_curseforge(&manifeste, instance).await?;
            Ok(liste.into_iter().map(|f| content::relatif(instance, &f.destination)).collect())
        } else {
            let version = modrinth::version(version_id).await?;
            let f = version.files.iter().find(|f| f.filename.to_lowercase().ends_with(".mrpack")).ok_or_else(|| anyhow::anyhow!("?"))?;
            let mut zip = telecharger_archive(&f.url, f.hashes.get("sha1").map(String::as_str), &progression, &mut temporaire).await?;
            let index: modrinth::IndexModpack = lire_json(&mut zip, "modrinth.index.json")?;
            Ok(index.files.iter().map(|f| f.path.clone()).collect())
        }
    }
    .await;
    if let Some(t) = temporaire {
        let _ = std::fs::remove_file(t);
    }
    resultat
}

/// Fichiers posés par le pack dans le profil (misty-modpack.json), pour une future mise à jour.
const FICHIER_PACK: &str = "misty-modpack.json";

fn lire_fichiers_pack(instance: &Instance) -> Option<Vec<String>> {
    let texte = std::fs::read_to_string(instance.dossier().join(FICHIER_PACK)).ok()?;
    serde_json::from_str(&texte).ok()
}

fn ecrire_fichiers_pack(instance: &Instance, chemins: &[String]) {
    let _ = std::fs::write(instance.dossier().join(FICHIER_PACK), serde_json::to_string_pretty(chemins).unwrap_or_default());
}

/// Mise à jour : retire les fichiers de l'ancienne version qui ne sont plus dans la nouvelle
/// (y compris ceux que le joueur avait désactivés).
fn retirer_anciens(instance: &Instance, anciens: &[String], nouveaux: &[String]) {
    let gardes: std::collections::HashSet<String> = nouveaux.iter().map(|c| c.replace('\\', "/").to_lowercase()).collect();
    for ancien in anciens {
        if gardes.contains(&ancien.replace('\\', "/").to_lowercase()) {
            continue;
        }
        let Ok(chemin) = chemin_sur(&instance.dossier(), ancien) else { continue };
        let _ = std::fs::remove_file(&chemin);
        let mut desactive = chemin.into_os_string();
        desactive.push(content::SUFFIXE_DESACTIVE);
        let _ = std::fs::remove_file(PathBuf::from(desactive));
    }
}

/// Où écrire un fichier du pack : à côté de sa version désactivée si le joueur l'avait désactivé.
/// None si le bon fichier est déjà là (mise à jour : rien à retélécharger).
fn destination_utile(destination: &Path, sha1: Option<&str>) -> Option<PathBuf> {
    let mut desactive = destination.as_os_str().to_owned();
    desactive.push(content::SUFFIXE_DESACTIVE);
    let desactive = PathBuf::from(desactive);
    let cible = if desactive.exists() && !destination.exists() { desactive } else { destination.to_path_buf() };
    if let Some(attendu) = sha1.filter(|s| !s.is_empty()) {
        if cible.exists() && download::sha1_fichier(&cible).is_ok_and(|h| h.eq_ignore_ascii_case(attendu)) {
            return None;
        }
    }
    Some(cible)
}

// ───────────────────────── Modrinth ─────────────────────────

async fn installer_modrinth(
    projet: &ProjetDistant,
    version_id: Option<&str>,
    progression: &Progress,
    sortie: &mut Option<Instance>,
    temporaire: &mut Option<PathBuf>,
) -> anyhow::Result<Resultat> {
    progression(loc::t("Progres_DerniereVersion"));
    let versions = modrinth::versions(&projet.id, None, None).await?;
    let version = versions
        .iter()
        .find(|v| version_id == Some(v.id.as_str()))
        .or_else(|| versions.iter().find(|v| version_id.is_none() && v.version_type == "release"))
        .or_else(|| versions.first())
        .ok_or_else(|| anyhow::anyhow!(loc::t("Erreur_ModpackSansVersion")))?;
    let fichier = version
        .files
        .iter()
        .find(|f| f.filename.to_lowercase().ends_with(".mrpack"))
        .ok_or_else(|| anyhow::anyhow!(loc::t("Erreur_SansMrpack")))?;

    let mut zip = telecharger_archive(&fichier.url, fichier.hashes.get("sha1").map(String::as_str), progression, temporaire).await?;
    let origine = Origine::catalogue(projet, Some(version.id.clone()));
    appliquer_modrinth(&mut zip, Cible::Nouveau(&origine), Some(version.version_number.clone()), progression, sortie).await
}

/// Applique l'archive .mrpack déjà ouverte au profil visé, puis télécharge ses fichiers.
/// `version_pack` : None = celle écrite dans l'archive.
async fn appliquer_modrinth(
    zip: &mut zip::ZipArchive<std::fs::File>,
    cible: Cible<'_>,
    version_pack: Option<String>,
    progression: &Progress,
    sortie: &mut Option<Instance>,
) -> anyhow::Result<Resultat> {
    let index: modrinth::IndexModpack = lire_json(zip, "modrinth.index.json")?;
    let version_mc = index
        .dependencies
        .get("minecraft")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!(loc::t("Erreur_ModpackSansMc")))?;
    let (loader, version_loader) = loader_modrinth(&index.dependencies);

    let version_pack = version_pack.unwrap_or_else(|| index.version_id.clone());
    let titre = match &cible {
        Cible::Nouveau(o) => o.titre_avec(&index.name),
        Cible::Existant { instance, .. } => instance.nom.clone(),
    };
    let instance = preparer_profil(&cible, &titre, &version_mc, loader, version_loader, version_pack)?;
    if matches!(cible, Cible::Nouveau(_)) {
        *sortie = Some(instance.clone());
    }

    // Fichiers à télécharger (sauf ceux réservés au serveur)
    let fichiers: Vec<&modrinth::FichierModpack> =
        index.files.iter().filter(|f| f.env.as_ref().and_then(|e| e.get("client")).map(String::as_str) != Some("unsupported")).collect();
    let chemins: Vec<String> = fichiers.iter().map(|f| f.path.clone()).collect();
    if let Cible::Existant { anciens, .. } = &cible {
        retirer_anciens(&instance, anciens, &chemins);
    }
    let mut taches = Vec::new();
    for f in fichiers {
        let destination = chemin_sur(&instance.dossier(), &f.path)?;
        let sha1 = f.hashes.get("sha1").cloned();
        if let Some(destination) = destination_utile(&destination, sha1.as_deref()) {
            taches.push((f.downloads.clone(), destination, sha1, f.path.clone()));
        }
    }
    en_parallele(taches, progression, |(urls, destination, sha1, chemin)| async move {
        let mut derniere = None;
        for url in &urls {
            match download::fichier(url, &destination, sha1.as_deref()).await {
                Ok(()) => return Ok(()),
                Err(e) => derniere = Some(e),
            }
        }
        let detail = derniere.map(|e| format!("\n{e}")).unwrap_or_default();
        anyhow::bail!("{}{detail}", loc::tf("Erreur_Telechargement", &[&chemin]))
    })
    .await?;

    progression(loc::t("Progres_Config"));
    extraire(zip, "overrides/", &instance.dossier())?;
    extraire(zip, "client-overrides/", &instance.dossier())?;
    ecrire_fichiers_pack(&instance, &chemins);

    Ok(Resultat { instance, bloques: Vec::new() })
}

fn loader_modrinth(dependances: &HashMap<String, String>) -> (&'static str, Option<String>) {
    if let Some(v) = dependances.get("fabric-loader") {
        return (loaders::FABRIC, Some(v.clone()));
    }
    if let Some(v) = dependances.get("quilt-loader") {
        return (loaders::QUILT, Some(v.clone()));
    }
    if let Some(v) = dependances.get("neoforge") {
        return (loaders::NEOFORGE, Some(v.clone()));
    }
    if let Some(v) = dependances.get("forge") {
        return (loaders::FORGE, Some(v.clone()));
    }
    (loaders::VANILLA, None)
}

// ───────────────────────── CurseForge ─────────────────────────

async fn installer_curseforge(
    projet: &ProjetDistant,
    version_id: Option<&str>,
    progression: &Progress,
    sortie: &mut Option<Instance>,
    temporaire: &mut Option<PathBuf>,
) -> anyhow::Result<Resultat> {
    progression(loc::t("Progres_DerniereVersion"));
    let fichiers_pack = curseforge::fichiers(projet.id.parse()?, None).await?;
    let fichier_pack = fichiers_pack
        .iter()
        .find(|f| version_id == Some(f.id.to_string().as_str()))
        .or_else(|| fichiers_pack.iter().find(|f| version_id.is_none() && f.release_type == 1 && f.is_available))
        .or_else(|| fichiers_pack.first())
        .ok_or_else(|| anyhow::anyhow!(loc::t("Erreur_ModpackSansVersion")))?;
    let Some(url) = fichier_pack.download_url.clone().filter(|u| !u.is_empty()) else {
        anyhow::bail!(loc::t("Erreur_ModpackCFInterdit"));
    };

    let mut zip = telecharger_archive(&url, fichier_pack.sha1(), progression, temporaire).await?;
    let origine = Origine::catalogue(projet, Some(fichier_pack.id.to_string()));
    appliquer_curseforge(&mut zip, Cible::Nouveau(&origine), Some(fichier_pack.display_name.clone()), progression, sortie).await
}

/// Un fichier d'un modpack CurseForge, résolu (adresse, destination, infos).
struct FichierPackCF {
    /// None : l'auteur interdit le téléchargement par une application
    url: Option<String>,
    destination: PathBuf,
    sha1: Option<String>,
    meta: ContenuMeta,
    bloque: FichierBloque,
}

/// Résout les fichiers requis du manifeste (URL, nom, type, icône).
async fn fichiers_curseforge(manifeste: &curseforge::ManifestCF, instance: &Instance) -> anyhow::Result<Vec<FichierPackCF>> {
    let requis: Vec<&curseforge::FichierManifestCF> = manifeste.files.iter().filter(|f| f.required).collect();
    let fichiers: HashMap<i64, curseforge::FichierCF> = curseforge::fichiers_par_ids(&requis.iter().map(|f| f.file_id).collect::<Vec<_>>())
        .await?
        .into_iter()
        .map(|f| (f.id, f))
        .collect();
    let mods: HashMap<i64, curseforge::ModCF> = curseforge::mods(&requis.iter().map(|f| f.project_id).collect::<Vec<_>>())
        .await?
        .into_iter()
        .map(|m| (m.id, m))
        .collect();

    let mut liste = Vec::new();
    for entree in requis {
        let Some(fichier) = fichiers.get(&entree.file_id) else { continue };
        let m = mods.get(&entree.project_id);
        let type_ = curseforge::type_depuis_classe(m.and_then(|m| m.class_id).unwrap_or(curseforge::CLASSE_MODS));
        let dossier = content::dossier(instance, type_, None)?;
        let nom_dossier = dossier.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let destination = chemin_sur(&instance.dossier(), &format!("{nom_dossier}/{}", download::nom_fichier_sur(&fichier.file_name)))?;
        let titre = m.map(|m| m.name.clone()).unwrap_or_else(|| fichier.display_name.clone());
        liste.push(FichierPackCF {
            url: fichier.download_url.clone().filter(|u| !u.is_empty()),
            sha1: fichier.sha1().map(str::to_string),
            meta: ContenuMeta {
                chemin: content::relatif(instance, &destination),
                source: Some(CURSEFORGE.into()),
                project_id: Some(entree.project_id.to_string()),
                version_id: Some(entree.file_id.to_string()),
                titre: titre.clone(),
                version_nom: Some(fichier.display_name.clone()),
                icone_url: m.and_then(|m| m.logo.as_ref()).and_then(|l| l.thumbnail_url.clone()),
                type_: type_.to_string(),
                client_side: Some(fichier.environnement().0),
                server_side: Some(fichier.environnement().1),
            },
            bloque: FichierBloque { nom: titre, page: curseforge::page_fichier(m, entree.project_id, entree.file_id), dossier: nom_dossier },
            destination,
        });
    }
    Ok(liste)
}

/// "Epic Terrain (Compatible)" → "epicterraincompatible" : pour comparer deux noms de projet.
fn nom_simple(nom: &str) -> String {
    nom.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// Fichiers que CurseForge interdit de télécharger : on les cherche sur Modrinth, d'abord le même
/// fichier exact (SHA-1), sinon le projet du même nom, pour cette version de Minecraft et ce loader.
/// Les fichiers trouvés prennent l'adresse Modrinth ; les autres restent à télécharger à la main.
async fn remplacer_par_modrinth(instance: &Instance, bloques: &mut [FichierPackCF], progression: &Progress) {
    if bloques.is_empty() {
        return;
    }
    progression(loc::tf("Progres_RechercheModrinth", &[&bloques.len()]));

    // 1. Le même fichier, publié aussi sur Modrinth
    let hashes: Vec<String> = bloques.iter().filter_map(|b| b.sha1.clone()).collect();
    let par_hash = modrinth::versions_par_hash(&hashes).await.unwrap_or_default();
    let ids: Vec<String> = par_hash.values().map(|v| v.project_id.clone()).collect();
    let projets: HashMap<String, modrinth::ProjetDetail> =
        modrinth::projets(&ids).await.unwrap_or_default().into_iter().map(|p| (p.id.clone(), p)).collect();
    for b in bloques.iter_mut() {
        let Some(sha1) = b.sha1.clone() else { continue };
        let Some(version) = par_hash.get(&sha1) else { continue };
        let Some(f) = version.files.iter().find(|f| f.hashes.get("sha1").is_some_and(|h| h.eq_ignore_ascii_case(&sha1))).or_else(|| version.fichier_principal()) else {
            continue;
        };
        let projet = projets.get(&version.project_id);
        trouve_sur_modrinth(instance, b, &f.url, &f.filename, f.hashes.get("sha1").cloned(), version, projet.map(|p| p.title.clone()), projet.and_then(|p| p.icon_url.clone()));
    }

    // 2. Le projet du même nom, dans une version compatible
    for b in bloques.iter_mut().filter(|b| b.url.is_none()) {
        let type_ = b.meta.type_.clone();
        let categories = content::categories_recherche(instance, &type_);
        let Ok(resultats) = modrinth::rechercher(&b.meta.titre, &type_, Some(&instance.version_mc), categories.as_deref(), "relevance", 0, 5).await else {
            continue;
        };
        let voulu = nom_simple(&b.meta.titre);
        let Some(projet) = resultats.hits.iter().find(|h| nom_simple(&h.title) == voulu) else { continue };
        let loaders = content::loaders_modrinth(instance, &type_);
        let Ok(versions) = modrinth::versions(&projet.project_id, loaders.as_deref(), Some(&instance.version_mc)).await else { continue };
        let Some(version) = versions.iter().find(|v| v.version_type == "release").or_else(|| versions.first()) else { continue };
        let Some(f) = version.fichier_principal() else { continue };
        trouve_sur_modrinth(instance, b, &f.url, &f.filename, f.hashes.get("sha1").cloned(), version, Some(projet.title.clone()), projet.icon_url.clone());
    }
}

#[allow(clippy::too_many_arguments)]
fn trouve_sur_modrinth(
    instance: &Instance,
    b: &mut FichierPackCF,
    url: &str,
    nom_fichier: &str,
    sha1: Option<String>,
    version: &modrinth::VersionModrinth,
    titre: Option<String>,
    icone: Option<String>,
) {
    let Some(dossier) = b.destination.parent() else { return };
    let destination = dossier.join(download::nom_fichier_sur(nom_fichier));
    b.url = Some(url.to_string());
    b.sha1 = sha1;
    b.meta = ContenuMeta {
        chemin: content::relatif(instance, &destination),
        source: Some(MODRINTH.into()),
        project_id: Some(version.project_id.clone()),
        version_id: Some(version.id.clone()),
        titre: titre.unwrap_or_else(|| b.meta.titre.clone()),
        version_nom: Some(version.version_number.clone()),
        icone_url: icone,
        type_: b.meta.type_.clone(),
        ..Default::default()
    };
    b.destination = destination;
}

/// Applique l'archive CurseForge déjà ouverte au profil visé, puis télécharge ses fichiers.
/// `version_repli` : nom de version à utiliser si le manifeste n'en a pas.
async fn appliquer_curseforge(
    zip: &mut zip::ZipArchive<std::fs::File>,
    cible: Cible<'_>,
    version_repli: Option<String>,
    progression: &Progress,
    sortie: &mut Option<Instance>,
) -> anyhow::Result<Resultat> {
    let manifeste: curseforge::ManifestCF = lire_json(zip, "manifest.json")?;
    let (loader, version_loader) = loader_curseforge(&manifeste.minecraft.mod_loaders)?;

    let version_pack = if manifeste.version.is_empty() { version_repli.unwrap_or_default() } else { manifeste.version.clone() };
    let titre = match &cible {
        Cible::Nouveau(o) => o.titre_avec(&manifeste.name),
        Cible::Existant { instance, .. } => instance.nom.clone(),
    };
    let instance = preparer_profil(&cible, &titre, &manifeste.minecraft.version, loader, version_loader, version_pack)?;
    if matches!(cible, Cible::Nouveau(_)) {
        *sortie = Some(instance.clone());
    }

    // Infos des fichiers (URL, nom) et des projets (type, nom, icône)
    progression(loc::t("Progres_ListeMods"));
    let mut fichiers = fichiers_curseforge(&manifeste, &instance).await?;
    // ceux que CurseForge interdit : cherchés sur Modrinth
    let mut bloques: Vec<FichierPackCF> = Vec::new();
    let mut i = 0;
    while i < fichiers.len() {
        if fichiers[i].url.is_none() {
            bloques.push(fichiers.remove(i));
        } else {
            i += 1;
        }
    }
    remplacer_par_modrinth(&instance, &mut bloques, progression).await;
    fichiers.extend(bloques);

    let chemins: Vec<String> = fichiers.iter().map(|f| content::relatif(&instance, &f.destination)).collect();
    if let Cible::Existant { anciens, .. } = &cible {
        retirer_anciens(&instance, anciens, &chemins);
    }

    let mut restants = Vec::new();
    let mut metas = Vec::new();
    let mut a_telecharger = Vec::new();
    for f in fichiers {
        let Some(url) = f.url else {
            restants.push(f.bloque);
            continue;
        };
        if let Some(destination) = destination_utile(&f.destination, f.sha1.as_deref()) {
            a_telecharger.push((url, destination, f.sha1));
        }
        metas.push(f.meta);
    }

    en_parallele(a_telecharger, progression, |(url, destination, sha1)| async move {
        download::fichier(&url, &destination, sha1.as_deref()).await
    })
    .await?;
    content::enregistrer_metas(&instance, metas)?;

    progression(loc::t("Progres_Config"));
    let overrides = manifeste.overrides.as_deref().map(|o| o.trim_matches('/')).filter(|o| !o.is_empty()).unwrap_or("overrides");
    extraire(zip, &format!("{overrides}/"), &instance.dossier())?;
    ecrire_fichiers_pack(&instance, &chemins);

    Ok(Resultat { instance, bloques: restants })
}

/// "forge-47.2.0" → (Forge, "47.2.0"). Le loader principal est privilégié.
fn loader_curseforge(liste: &[curseforge::LoaderManifestCF]) -> anyhow::Result<(&'static str, Option<String>)> {
    let Some(principal) = liste.iter().find(|l| l.primary).or_else(|| liste.first()) else {
        return Ok((loaders::VANILLA, None));
    };
    let (nom, version) = match principal.id.split_once('-') {
        Some((n, v)) => (n, Some(v.to_string())),
        None => (principal.id.as_str(), None),
    };
    Ok(match nom.to_lowercase().as_str() {
        "forge" => (loaders::FORGE, version),
        "neoforge" => (loaders::NEOFORGE, version),
        "fabric" => (loaders::FABRIC, version),
        "quilt" => (loaders::QUILT, version),
        _ => anyhow::bail!(loc::tf("Erreur_LoaderNonPris", &[&principal.id])),
    })
}

// ───────────────────────── Commun ─────────────────────────

fn lire_json<T: serde::de::DeserializeOwned>(zip: &mut zip::ZipArchive<std::fs::File>, nom: &str) -> anyhow::Result<T> {
    let mut entree = zip.by_name(nom).map_err(|_| anyhow::anyhow!(loc::tf("Erreur_Manquant", &[&nom])))?;
    let mut texte = String::new();
    entree.read_to_string(&mut texte)?;
    serde_json::from_str(texte.trim_start_matches('\u{feff}')).map_err(|e| anyhow::anyhow!("{nom} illisible : {e}"))
}

/// Exécute les téléchargements 6 par 6 en affichant l'avancement.
async fn en_parallele<T, F, Fut>(elements: Vec<T>, progression: &Progress, action: F) -> anyhow::Result<()>
where
    F: Fn(T) -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<()>>,
{
    let total = elements.len();
    let faits = Arc::new(AtomicUsize::new(0));
    let mut flux = futures::stream::iter(elements.into_iter().map(|e| {
        let faits = faits.clone();
        let progression = progression.clone();
        let tache = action(e);
        async move {
            tache.await?;
            let n = faits.fetch_add(1, Ordering::SeqCst) + 1;
            progression(loc::tf("Progres_Fichiers", &[&n, &total]));
            Ok::<(), anyhow::Error>(())
        }
    }))
    .buffer_unordered(6);
    while let Some(r) = flux.next().await {
        r?;
    }
    Ok(())
}

/// Extrait les fichiers de l'archive qui commencent par `prefixe` dans `racine`.
fn extraire(zip: &mut zip::ZipArchive<std::fs::File>, prefixe: &str, racine: &Path) -> anyhow::Result<()> {
    let prefixe_min = prefixe.to_lowercase();
    for i in 0..zip.len() {
        let mut entree = zip.by_index(i)?;
        let nom = entree.name().replace('\\', "/");
        if !nom.to_lowercase().starts_with(&prefixe_min) || nom.ends_with('/') {
            continue; // hors du préfixe, ou dossier
        }
        let destination = chemin_sur(racine, &nom[prefixe.len()..])?;
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut sortie = std::fs::File::create(&destination)?;
        std::io::copy(&mut entree, &mut sortie)?;
    }
    Ok(())
}

/// Refuse les chemins qui sortiraient du dossier du profil ("../", chemins absolus…).
pub fn chemin_sur(racine: &Path, relatif: &str) -> anyhow::Result<PathBuf> {
    let rel = Path::new(relatif);
    if rel.components().any(|c| !matches!(c, Component::Normal(_) | Component::CurDir)) {
        anyhow::bail!("Chemin invalide dans le modpack : {relatif}");
    }
    Ok(racine.join(rel))
}

/// Modpacks en vrai (réseau, ~50 Mo) sur un dossier de données à part :
/// `MISTY_DATA_DIR=<dossier temporaire> cargo test modpack_reseau -- --ignored --nocapture`.
#[cfg(test)]
mod tests_reseau {
    use super::*;

    fn dossier_de_test() -> bool {
        // jamais sur les vraies données du joueur
        std::env::var_os("MISTY_DATA_DIR").is_some() && super::super::paths::data_dir().starts_with(std::env::var_os("MISTY_DATA_DIR").unwrap())
    }

    #[tokio::test]
    #[ignore]
    async fn modpack_reseau_import_puis_mise_a_jour() {
        if !dossier_de_test() {
            panic!("MISTY_DATA_DIR requis");
        }
        let rien: Progress = std::sync::Arc::new(|t| println!("  {t}"));
        // Fabulously Optimized : une version ancienne, en fichier, importée comme le ferait le joueur
        let versions = modrinth::versions("1KVo5zza", None, None).await.unwrap();
        let avec_pack: Vec<&modrinth::VersionModrinth> =
            versions.iter().filter(|v| v.files.iter().any(|f| f.filename.ends_with(".mrpack"))).collect();
        let (ancienne, recente) = (avec_pack[6], avec_pack[0]);
        let f = ancienne.files.iter().find(|f| f.filename.ends_with(".mrpack")).unwrap();
        let fichier = std::env::temp_dir().join("misty-test-ancien.mrpack");
        download::fichier(&f.url, &fichier, None).await.unwrap();

        let r = importer_fichier(&fichier, rien.clone()).await.unwrap();
        let i = r.instance;
        assert_eq!(i.modpack_project_id.as_deref(), Some("1KVo5zza"), "pack reconnu sur Modrinth");
        assert_eq!(i.modpack_version_id.as_deref(), Some(ancienne.id.as_str()));
        let anciens = lire_fichiers_pack(&i).unwrap();
        println!("importé : {} ({} fichiers, MC {})", i.nom, anciens.len(), i.version_mc);

        let r = mettre_a_jour(&i, recente.id.clone(), rien).await.unwrap();
        let j = r.instance;
        let nouveaux = lire_fichiers_pack(&j).unwrap();
        assert_eq!(j.modpack_version_id.as_deref(), Some(recente.id.as_str()));
        assert_eq!(j.id, i.id, "même profil");
        for c in &nouveaux {
            assert!(chemin_sur(&j.dossier(), c).unwrap().exists(), "manque {c}");
        }
        let retires: Vec<&String> = anciens.iter().filter(|a| !nouveaux.iter().any(|n| n.eq_ignore_ascii_case(a))).collect();
        for c in &retires {
            assert!(!chemin_sur(&j.dossier(), c).unwrap().exists(), "pas retiré : {c}");
        }
        println!("mis à jour : MC {} → {}, {} fichiers, {} retirés", i.version_mc, j.version_mc, nouveaux.len(), retires.len());
        let _ = std::fs::remove_file(fichier);
    }

    #[tokio::test]
    #[ignore]
    async fn modpack_reseau_bloque_cherche_sur_modrinth() {
        if !dossier_de_test() {
            panic!("MISTY_DATA_DIR requis");
        }
        let i = instances::creer("test bloques", "1.21.1", loaders::FABRIC, None).unwrap();
        let mut liste = vec![FichierPackCF {
            url: None,
            destination: i.dossier().join("mods").join("sodium-cf.jar"),
            sha1: None,
            meta: ContenuMeta { titre: "Sodium".into(), type_: content::MOD.into(), ..Default::default() },
            bloque: FichierBloque { nom: "Sodium".into(), page: String::new(), dossier: "mods".into() },
        }];
        let rien: Progress = std::sync::Arc::new(|_| {});
        remplacer_par_modrinth(&i, &mut liste, &rien).await;
        let b = &liste[0];
        assert!(b.url.as_deref().is_some_and(|u| u.contains("modrinth")), "trouvé sur Modrinth");
        assert_eq!(b.meta.source.as_deref(), Some(MODRINTH));
        println!("Sodium → {} ({})", b.destination.display(), b.meta.version_nom.clone().unwrap_or_default());
        let _ = instances::supprimer(&i);
    }

    #[test]
    fn noms() {
        assert_eq!(nom_simple("Epic Terrain (Compatible)"), "epicterraincompatible");
        assert_eq!(nom_simple("Cloth Config API"), "clothconfigapi");
    }
}
