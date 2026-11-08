//! Gestion du stockage : liste ce qui peut être supprimé sans perdre de données personnelles.
//!
//! Tout ce qui est listé ici se télécharge ou se régénère (versions, bibliothèques, ressources, Java,
//! caches, journaux). Les données personnelles (mondes, instances, captures…) ne sont jamais proposées.

use super::{paths, system_info};
use std::path::{Path, PathBuf};

/// Une ligne de la liste : un groupe (niveau 0) ou un élément d'un groupe (niveau 1).
#[derive(Clone, Debug)]
pub struct Ligne {
    /// Identifiant (chemin relatif au dossier de données, ou `@residus`)
    pub cle: String,
    pub nom: String,
    pub taille: u64,
    pub niveau: u8,
    /// Groupe dont on peut afficher le détail
    pub depliable: bool,
    /// Reste d'une installation incomplète
    pub reste: bool,
    /// Donnée personnelle : affichée mais jamais supprimable
    pub protege: bool,
}

/// Groupes supprimables : (dossier, détail par sous-dossier ?).
const GROUPES: &[(&str, bool)] = &[
    ("versions", true),
    ("libraries", false),
    ("assets", true),
    ("runtime", true),
    ("downloads", false),
    ("cache", false),
    ("webview", false),
];

/// Données personnelles : montrées pour information, jamais supprimables.
const PERSONNELLES: &[&str] = &["instances", "saves", "mymods", "resourcepacks", "resources", "screenshots", "templates", "local", "backups"];

const RESIDUS: &str = "@residus";

fn racine() -> PathBuf {
    paths::data_dir()
}

fn sous_dossiers(dossier: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entrees) = std::fs::read_dir(dossier) else { return Vec::new() };
    entrees
        .flatten()
        .filter(|e| e.metadata().map(|m| m.is_dir()).unwrap_or(false))
        .map(|e| (e.file_name().to_string_lossy().to_string(), e.path()))
        .collect()
}

/// Fichiers « résidus » à la racine : journaux de plantage Java, captures de debug, etc.
fn residus_racine() -> Vec<(String, u64)> {
    let Ok(entrees) = std::fs::read_dir(racine()) else { return Vec::new() };
    entrees
        .flatten()
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            let nom = e.file_name().to_string_lossy().to_string();
            let bas = nom.to_lowercase();
            let residu = meta.is_file() && (bas.starts_with("hs_err_pid") || bas.starts_with("debug.") || bas.ends_with(".tmp") || bas.ends_with(".log"));
            residu.then_some((nom, meta.len()))
        })
        .collect()
}

/// Une version est complète si son fichier .json existe (le .jar manque pour les loaders qui héritent d'une autre version).
fn version_incomplete(dossier: &Path, id: &str) -> bool {
    !dossier.join(format!("{id}.json")).exists()
}

/// Analyse le dossier de données (long : à appeler hors du fil de l'interface).
pub fn analyser() -> Vec<Ligne> {
    let r = racine();
    let mut lignes = Vec::new();

    for (nom, detail) in GROUPES {
        let dossier = r.join(nom);
        if !dossier.exists() {
            continue;
        }
        let mut enfants = Vec::new();
        if *detail {
            for (sous, chemin) in sous_dossiers(&dossier) {
                if sous.starts_with('.') {
                    continue; // dossier interne de Misty (non listé)
                }
                let reste = *nom == "versions" && version_incomplete(&chemin, &sous);
                enfants.push(Ligne {
                    cle: format!("{nom}/{sous}"),
                    nom: sous,
                    taille: system_info::taille_dossier(&chemin),
                    niveau: 1,
                    depliable: false,
                    reste,
                    protege: false,
                });
            }
            enfants.sort_by(|a, b| b.reste.cmp(&a.reste).then(b.taille.cmp(&a.taille)));
        }
        let taille = if enfants.is_empty() { system_info::taille_dossier(&dossier) } else { enfants.iter().map(|e| e.taille).sum() };
        lignes.push(Ligne {
            cle: nom.to_string(),
            nom: nom.to_string(),
            taille,
            niveau: 0,
            depliable: !enfants.is_empty(),
            reste: enfants.iter().any(|e| e.reste),
            protege: false,
        });
        lignes.extend(enfants);
    }

    // Résidus : journaux, rapports de plantage, fichiers de debug
    let mut enfants = Vec::new();
    for dossier in ["logs", "crash-reports"] {
        let chemin = r.join(dossier);
        if chemin.exists() {
            enfants.push(Ligne { cle: dossier.into(), nom: dossier.into(), taille: system_info::taille_dossier(&chemin), niveau: 1, depliable: false, reste: false, protege: false });
        }
    }
    for (nom, taille) in residus_racine() {
        enfants.push(Ligne { cle: nom.clone(), nom, taille, niveau: 1, depliable: false, reste: true, protege: false });
    }
    if !enfants.is_empty() {
        enfants.sort_by(|a, b| b.taille.cmp(&a.taille));
        lignes.push(Ligne {
            cle: RESIDUS.into(),
            nom: RESIDUS.into(),
            taille: enfants.iter().map(|e| e.taille).sum(),
            niveau: 0,
            depliable: true,
            reste: false,
            protege: false,
        });
        lignes.extend(enfants);
    }

    // Données personnelles : une seule ligne informative
    let perso: u64 = PERSONNELLES.iter().map(|d| system_info::taille_dossier(&r.join(d))).sum();
    lignes.push(Ligne { cle: "@perso".into(), nom: "@perso".into(), taille: perso, niveau: 0, depliable: false, reste: false, protege: true });
    lignes
}

/// Groupe d'une clé (« versions/1.20.1 » → « versions ») : sert à choisir le message d'avertissement.
pub fn groupe_de(cle: &str) -> String {
    let premier = cle.split('/').next().unwrap_or(cle);
    if cle == RESIDUS || premier == "logs" || premier == "crash-reports" || !GROUPES.iter().any(|(g, _)| *g == premier) {
        return "residus".into();
    }
    premier.to_string()
}

/// Supprime un élément listé par `analyser`. Refuse tout ce qui n'est pas dans la liste blanche.
pub fn supprimer(cle: &str) -> std::io::Result<()> {
    let r = racine();
    if cle == RESIDUS {
        for (nom, _) in residus_racine() {
            let _ = std::fs::remove_file(r.join(nom));
        }
        for d in ["logs", "crash-reports"] {
            let _ = std::fs::remove_dir_all(r.join(d));
        }
        return Ok(());
    }
    if cle.is_empty() || cle.contains("..") || cle.contains('\\') || cle.starts_with('/') || cle.split('/').count() > 2 {
        return Err(std::io::Error::other("clé invalide"));
    }
    let premier = cle.split('/').next().unwrap_or("");
    let autorise = GROUPES.iter().any(|(g, _)| *g == premier) || (matches!(premier, "logs" | "crash-reports") && !cle.contains('/')) || residus_racine().iter().any(|(n, _)| n == cle);
    if !autorise {
        return Err(std::io::Error::other("suppression non autorisée"));
    }
    let chemin = r.join(cle);
    if chemin.is_dir() {
        std::fs::remove_dir_all(&chemin)?;
        if cle.contains('/') {
            // dossier parent laissé vide : on le retire aussi pour ne pas laisser de reste
            if let Some(parent) = chemin.parent() {
                let _ = std::fs::remove_dir(parent);
            }
        }
    } else if chemin.exists() {
        std::fs::remove_file(&chemin)?;
    }
    Ok(())
}
