//! Création / suppression du raccourci "Misty.lnk" (choisi dans les options) vers Misty : le
//! lanceur fixe une fois installé (voir lanceur.rs), sinon l'exe courant.

use std::path::Path;

const NOM_RACCOURCI: &str = "Misty.lnk";

#[cfg(windows)]
pub fn creer(dossier: &Path) -> anyhow::Result<()> {
    if dossier.as_os_str().is_empty() {
        return Ok(());
    }
    supprimer(dossier);

    let mut lnk = mslnk::ShellLink::new(super::lanceur::cible()).map_err(|e| anyhow::anyhow!("{e:?}"))?;
    lnk.set_working_dir(super::lanceur::cible().parent().map(|d| d.to_string_lossy().to_string()));
    lnk.set_icon_location(Some(super::lanceur::icone().to_string_lossy().to_string()));
    lnk.create_lnk(dossier.join(NOM_RACCOURCI)).map_err(|e| anyhow::anyhow!("{e:?}"))?;
    Ok(())
}

#[cfg(not(windows))]
pub fn creer(_dossier: &Path) -> anyhow::Result<()> {
    Ok(())
}

pub fn supprimer(dossier: &Path) {
    if dossier.as_os_str().is_empty() {
        return;
    }
    let chemin = dossier.join(NOM_RACCOURCI);
    if chemin.exists() {
        let _ = std::fs::remove_file(chemin);
    }
}
