//! Lanceur fixe : `<racine>\Misty.exe`, à côté des dossiers « Misty-<version> ».
//!
//! Chaque version a son propre dossier, supprimé après une mise à jour : un raccourci (et surtout
//! une épingle de la barre des tâches, dont Windows garde la cible en mémoire) qui vise
//! `Misty-0.3.7\Misty.exe` ne marche plus après la mise à jour. Les raccourcis visent donc ce
//! lanceur, dont le chemin ne change jamais : il démarre la version la plus récente puis se ferme.
//!
//! Le lanceur est le même exe que Misty (un lien physique vers l'exe de la version installée, ou
//! une copie). Misty sait qu'il joue ce rôle quand il ne tourne pas depuis un dossier « Misty-… ».
//!
//! Pour que l'épingle et la fenêtre de Misty ne fassent qu'un seul bouton dans la barre des tâches
//! (le lanceur et Misty sont deux chemins différents), Misty, ses raccourcis et sa fenêtre portent
//! le même identifiant d'application (AppUserModelID).

use super::{paths, update};
use std::path::{Path, PathBuf};

/// Identifiant d'application de Misty dans la barre des tâches (installer\Misty.iss : AppUserModelID).
pub const AUMID: &str = "MistyMC.Misty";
const PREFIXE: &str = "Misty-";

/// Dossier qui contient les dossiers « Misty-<version> », si Misty tourne depuis l'un d'eux.
pub fn racine() -> Option<PathBuf> {
    let dossier = paths::app_dir();
    if dossier.file_name()?.to_string_lossy().starts_with(PREFIXE) {
        dossier.parent().map(Path::to_path_buf)
    } else {
        None
    }
}

/// Chemin du lanceur fixe (installation seulement, pas un exe de développement).
pub fn chemin() -> Option<PathBuf> {
    racine().map(|r| r.join("Misty.exe"))
}

/// Ce que les raccourcis doivent viser : le lanceur s'il existe, sinon l'exe actuel.
pub fn cible() -> PathBuf {
    chemin().filter(|c| c.exists()).unwrap_or_else(paths::app_exe)
}

/// Icône des raccourcis : l'icône personnalisée (copiée à côté du lanceur, pour qu'elle survive aux
/// mises à jour), sinon celle de l'exe visé.
pub fn icone() -> PathBuf {
    let fixe = racine().map(|r| r.join("Icon.ico")).filter(|i| i.exists());
    fixe.or_else(|| Some(paths::custom_icon()).filter(|i| i.exists())).unwrap_or_else(cible)
}

/// La version la plus récente installée à côté de `dossier` : (version, exe).
fn plus_recente(dossier: &Path) -> Option<(String, PathBuf)> {
    std::fs::read_dir(dossier)
        .ok()?
        .flatten()
        .filter_map(|e| {
            let nom = e.file_name().to_string_lossy().to_string();
            let version = nom.strip_prefix(PREFIXE)?.to_string();
            let exe = e.path().join("Misty.exe");
            exe.is_file().then_some((version, exe))
        })
        .max_by(|a, b| update::comparer_versions(&a.0, &b.0))
}

/// Au tout début de `main` : si cet exe est le lanceur fixe, démarre la version la plus récente avec
/// les mêmes arguments et renvoie true (il faut alors quitter). Sinon renvoie false.
pub fn relayer() -> bool {
    if racine().is_some() {
        return false; // une vraie version
    }
    let Some((_, exe)) = plus_recente(&paths::app_dir()) else { return false };
    if exe == paths::app_exe() {
        return false;
    }
    let dossier = exe.parent().map(Path::to_path_buf).unwrap_or_default();
    std::process::Command::new(&exe).args(std::env::args_os().skip(1)).current_dir(dossier).spawn().is_ok()
}

/// Identifiant d'application du processus, avant la création de la fenêtre.
#[cfg(windows)]
pub fn identifier_processus() {
    use windows::core::HSTRING;
    // SAFETY : chaîne valide le temps de l'appel ; à faire avant d'afficher la moindre fenêtre.
    let _ = unsafe { windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(&HSTRING::from(AUMID)) };
}

#[cfg(not(windows))]
pub fn identifier_processus() {}

/// Crée (ou met à jour) le lanceur fixe à partir de l'exe actuel. `forcer` : après une mise à jour,
/// le lanceur est remplacé même s'il existe déjà.
pub fn installer(forcer: bool) {
    let (Some(lanceur), Some(racine)) = (chemin(), racine()) else { return };
    let exe = paths::app_exe();
    if paths::custom_icon().exists() {
        let _ = std::fs::copy(paths::custom_icon(), racine.join("Icon.ico"));
    }
    if lanceur.exists() && !forcer {
        return;
    }
    // le nouveau à côté, puis il prend la place de l'ancien (qui peut être en train de se fermer)
    let temporaire = racine.join("Misty.exe.new");
    let _ = std::fs::remove_file(&temporaire);
    if std::fs::hard_link(&exe, &temporaire).is_err() && std::fs::copy(&exe, &temporaire).is_err() {
        return;
    }
    if std::fs::rename(&temporaire, &lanceur).is_err() {
        let _ = std::fs::remove_file(&temporaire);
    }
}

// ───────────────────────── Propriétés Windows (AppUserModelID) ─────────────────────────

#[cfg(windows)]
mod proprietes {
    use windows::core::{HSTRING, PCWSTR};
    use windows::Win32::Foundation::PROPERTYKEY;
    use windows::Win32::System::Com::StructuredStorage::{PropVariantClear, PROPVARIANT};
    use windows::Win32::System::Variant::VT_LPWSTR;
    use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
    use windows::Win32::UI::Shell::SHStrDupW;

    /// Écrit une chaîne dans un magasin de propriétés (sans l'enregistrer).
    pub fn ecrire(magasin: &IPropertyStore, cle: &PROPERTYKEY, valeur: &str) -> windows::core::Result<()> {
        // SAFETY : PROPVARIANT VT_LPWSTR avec une chaîne allouée par SHStrDupW (CoTaskMem), libérée
        // par PropVariantClear après SetValue (qui en fait sa propre copie).
        unsafe {
            let texte = SHStrDupW(PCWSTR(HSTRING::from(valeur).as_ptr()))?;
            let mut variante = PROPVARIANT::default();
            (*variante.Anonymous.Anonymous).vt = VT_LPWSTR;
            (*variante.Anonymous.Anonymous).Anonymous.pwszVal = texte;
            let resultat = magasin.SetValue(cle, &variante);
            let _ = PropVariantClear(&mut variante);
            resultat
        }
    }

    /// Lit une chaîne d'un magasin de propriétés.
    pub fn lire(magasin: &IPropertyStore, cle: &PROPERTYKEY) -> Option<String> {
        // SAFETY : la variante renvoyée est lue selon son type, puis libérée.
        unsafe {
            let mut variante = magasin.GetValue(cle).ok()?;
            let texte = ((*variante.Anonymous.Anonymous).vt == VT_LPWSTR).then(|| (*variante.Anonymous.Anonymous).Anonymous.pwszVal.to_string().ok()).flatten();
            let _ = PropVariantClear(&mut variante);
            texte
        }
    }
}

/// Fenêtre principale : même identifiant que les raccourcis, et ce que Windows doit relancer quand on
/// l'épingle (le lanceur fixe, pas l'exe de cette version).
#[cfg(windows)]
pub fn marquer_fenetre(hwnd: isize) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Storage::EnhancedStorage::{
        PKEY_AppUserModel_ID, PKEY_AppUserModel_RelaunchCommand, PKEY_AppUserModel_RelaunchDisplayNameResource, PKEY_AppUserModel_RelaunchIconResource,
    };
    use windows::Win32::UI::Shell::PropertiesSystem::{IPropertyStore, SHGetPropertyStoreForWindow};

    let Some(lanceur) = chemin().filter(|c| c.exists()) else { return };
    let lanceur = lanceur.to_string_lossy().to_string();
    // SAFETY : HWND de la fenêtre affichée ; le magasin est un objet COM libéré à la fin du bloc.
    unsafe {
        let Ok(magasin) = SHGetPropertyStoreForWindow::<IPropertyStore>(HWND(hwnd as _)) else { return };
        // l'identifiant d'abord : sans lui, Windows ignore les propriétés de relance
        let _ = proprietes::ecrire(&magasin, &PKEY_AppUserModel_ID, AUMID);
        let _ = proprietes::ecrire(&magasin, &PKEY_AppUserModel_RelaunchCommand, &format!("\"{lanceur}\""));
        let _ = proprietes::ecrire(&magasin, &PKEY_AppUserModel_RelaunchDisplayNameResource, "Misty");
        let _ = proprietes::ecrire(&magasin, &PKEY_AppUserModel_RelaunchIconResource, &format!("{lanceur},0"));
    }
}

#[cfg(not(windows))]
pub fn marquer_fenetre(_hwnd: isize) {}

/// Raccourcis vers Misty (menu Démarrer, bureau, épingles de la barre des tâches et du menu
/// Démarrer) : ceux qui visent un dossier « Misty-… » (ou le lanceur) sont redirigés vers le
/// lanceur fixe, avec l'identifiant de Misty. Lancé à chaque démarrage, en arrière-plan.
#[cfg(windows)]
pub fn rediriger_raccourcis() {
    use windows::core::{Interface, HSTRING, PCWSTR};
    use windows::Win32::Storage::EnhancedStorage::PKEY_AppUserModel_ID;
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, STGM_READWRITE};
    use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
    use windows::Win32::UI::Shell::{IShellLinkW, SHChangeNotify, ShellLink, SHCNE_UPDATEITEM, SHCNF_PATHW};

    let (Some(racine), Some(lanceur)) = (racine(), chemin().filter(|c| c.exists())) else { return };
    let racine_min = racine.to_string_lossy().to_lowercase();
    let lanceur_min = lanceur.to_string_lossy().to_lowercase();
    let icone = icone();

    let mut dossiers: Vec<PathBuf> = Vec::new();
    if let Some(d) = dirs::data_dir() {
        // %APPDATA%
        let ql = d.join(r"Microsoft\Internet Explorer\Quick Launch");
        dossiers.extend([d.join(r"Microsoft\Windows\Start Menu\Programs"), ql.join(r"User Pinned\TaskBar"), ql.join(r"User Pinned\StartMenu"), ql]);
    }
    if let Some(d) = dirs::desktop_dir() {
        dossiers.push(d);
    }

    // SAFETY : COM initialisé sur ce thread pour la durée de la boucle ; chaque objet COM est
    // libéré par son Drop ; les chaînes passées restent valides pendant les appels.
    unsafe {
        let com = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
        for dossier in dossiers {
            let Ok(lecture) = std::fs::read_dir(&dossier) else { continue };
            for fichier in lecture.flatten().map(|e| e.path()) {
                if !fichier.extension().is_some_and(|e| e.eq_ignore_ascii_case("lnk")) {
                    continue;
                }
                let Ok(lien) = CoCreateInstance::<_, IShellLinkW>(&ShellLink, None, CLSCTX_INPROC_SERVER) else { continue };
                let Ok(persistant) = lien.cast::<IPersistFile>() else { continue };
                let nom = HSTRING::from(fichier.as_os_str());
                if persistant.Load(PCWSTR(nom.as_ptr()), STGM_READWRITE).is_err() {
                    continue;
                }
                let mut tampon = [0u16; 1024];
                if lien.GetPath(&mut tampon, std::ptr::null_mut(), 0).is_err() {
                    continue;
                }
                let longueur = tampon.iter().position(|&c| c == 0).unwrap_or(tampon.len());
                let cible = String::from_utf16_lossy(&tampon[..longueur]).to_lowercase();
                let vise_misty = cible == lanceur_min || cible.strip_prefix(&racine_min).is_some_and(|r| r.trim_start_matches('\\').starts_with("misty-"));
                if !vise_misty {
                    continue;
                }
                let deja_bon = cible == lanceur_min
                    && lien.cast::<IPropertyStore>().ok().and_then(|m| proprietes::lire(&m, &PKEY_AppUserModel_ID)).is_some_and(|v| v == AUMID);
                if deja_bon {
                    continue;
                }
                let _ = lien.SetPath(PCWSTR(HSTRING::from(lanceur.as_os_str()).as_ptr()));
                let _ = lien.SetWorkingDirectory(PCWSTR(HSTRING::from(racine.as_os_str()).as_ptr()));
                let _ = lien.SetIconLocation(PCWSTR(HSTRING::from(icone.as_os_str()).as_ptr()), 0);
                if let Ok(magasin) = lien.cast::<IPropertyStore>() {
                    let _ = proprietes::ecrire(&magasin, &PKEY_AppUserModel_ID, AUMID);
                    let _ = magasin.Commit();
                }
                if persistant.Save(PCWSTR(nom.as_ptr()), true).is_ok() {
                    // Explorer relit le raccourci (icône et cible des épingles)
                    SHChangeNotify(SHCNE_UPDATEITEM, SHCNF_PATHW, Some(nom.as_ptr() as _), None);
                }
            }
        }
        if com {
            CoUninitialize();
        }
    }
}

#[cfg(not(windows))]
pub fn rediriger_raccourcis() {}
