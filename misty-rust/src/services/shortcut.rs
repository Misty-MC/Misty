//! Création / suppression du raccourci "Misty.lnk" vers l'exe courant.

use super::paths;
use std::path::Path;

const NOM_RACCOURCI: &str = "Misty.lnk";

#[cfg(windows)]
pub fn creer(dossier: &Path) -> anyhow::Result<()> {
    if dossier.as_os_str().is_empty() {
        return Ok(());
    }
    supprimer(dossier);

    let mut lnk = mslnk::ShellLink::new(paths::app_exe()).map_err(|e| anyhow::anyhow!("{e:?}"))?;
    lnk.set_working_dir(Some(paths::app_dir().to_string_lossy().to_string()));
    if paths::custom_icon().exists() {
        lnk.set_icon_location(Some(paths::custom_icon().to_string_lossy().to_string()));
    }
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

/// Raccourcis vers Misty (menu Démarrer, bureau, épingles de la barre des tâches et du menu
/// Démarrer) : chaque version a son dossier « Misty-x.y.z », et l'ancien est supprimé après une mise
/// à jour. Un raccourci qui vise un dossier « Misty-… » disparu (ou `ancien_dossier`, avant sa
/// suppression) est redirigé vers l'exe actuel. Lancé à chaque démarrage, en arrière-plan.
/// (Lecture de la cible via WScript.Shell, comme dans la version C#.)
#[cfg(windows)]
pub fn rediriger(ancien_dossier: &str) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    // seulement pour une copie installée (dossier « Misty-x.y.z »), pas pour un exe de développement
    let dossier = paths::app_dir();
    if !dossier.file_name().is_some_and(|n| n.to_string_lossy().starts_with("Misty-")) {
        return;
    }
    let Some(racine) = dossier.parent() else { return };
    let echapper = |s: &str| s.replace('\'', "''");
    let icone = if paths::custom_icon().exists() { paths::custom_icon() } else { paths::app_exe() };
    let script = format!(
        "$sh = New-Object -ComObject WScript.Shell; $ql = Join-Path $env:APPDATA 'Microsoft\\Internet Explorer\\Quick Launch'; \
         foreach ($p in @([Environment]::GetFolderPath('Programs'), [Environment]::GetFolderPath('DesktopDirectory'), $ql, (Join-Path $ql 'User Pinned\\TaskBar'), (Join-Path $ql 'User Pinned\\StartMenu'))) {{ \
           if (Test-Path -LiteralPath $p) {{ Get-ChildItem -LiteralPath $p -Filter *.lnk | ForEach-Object {{ $l = $sh.CreateShortcut($_.FullName); $t = $l.TargetPath; \
             if ($t -and $t -ne '{exe}' -and $t -like '{racine}\\Misty-*' -and (-not (Test-Path -LiteralPath $t) -or ('{ancien}' -ne '' -and $t -like '{ancien}*'))) {{ \
               $l.TargetPath = '{exe}'; $l.WorkingDirectory = '{dir}'; $l.IconLocation = '{icone}'; $l.Save() }} }} }} }}",
        ancien = echapper(ancien_dossier),
        racine = echapper(&racine.to_string_lossy()),
        exe = echapper(&paths::app_exe().to_string_lossy()),
        dir = echapper(&dossier.to_string_lossy()),
        icone = echapper(&icone.to_string_lossy()),
    );
    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(CREATE_NO_WINDOW)
        .status();
}

#[cfg(not(windows))]
pub fn rediriger(_ancien_dossier: &str) {}
