//! Notes de version : les releases publiées sur GitHub (Misty-MC/Misty-Launcher), gardées en cache pour
//! s'afficher tout de suite et hors ligne. Sans cache ni réseau : les versions publiées sur GitHub
//! du CHANGELOG.md intégré à l'exe (à partir de 0.3.0).

use super::{lancer, modele, modele_textes, App};
use crate::services::{app_info, http, paths, update};
use crate::{ChangelogSection, ChangelogState};
use slint::ComponentHandle;
use std::rc::Rc;

const CHANGELOG: &str = include_str!("../../../CHANGELOG.md");
/// Première version publiée dans les releases GitHub de Misty-MC/Misty-Launcher.
const PREMIERE_RELEASE: &str = "0.3.0";

/// Une version : titre, lignes, et pour chaque ligne si c'est un intertitre (« ### … »).
type Section = (String, Vec<String>, Vec<bool>);

fn fichier_cache() -> std::path::PathBuf {
    paths::data_dir().join("cache").join("releases.json")
}

pub fn installer(app: &Rc<App>) {
    let depuis_cache = std::fs::read_to_string(fichier_cache()).ok().and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()).map(|v| depuis_releases(&v));
    afficher(app, depuis_cache.filter(|s| !s.is_empty()).unwrap_or_else(integre));
    let a = app.clone();
    lancer(async move {
        let url = format!("https://api.github.com/repos/{}/releases?per_page=100", app_info::GITHUB_REPO);
        if let Ok(v) = a.bg(async move { http::get_json::<serde_json::Value>(&url).await }).await {
            let sections = depuis_releases(&v);
            if !sections.is_empty() {
                let _ = std::fs::create_dir_all(fichier_cache().parent().unwrap_or(&paths::data_dir()));
                let _ = std::fs::write(fichier_cache(), v.to_string());
                afficher(&a, sections);
            }
        }
    });
}

fn afficher(app: &App, sections: Vec<Section>) {
    let sections: Vec<ChangelogSection> = sections
        .into_iter()
        .enumerate()
        .map(|(i, (titre, lignes, intertitres))| ChangelogSection {
            title: titre.into(),
            lines: modele_textes(&lignes),
            heads: modele(intertitres),
            recent: i == 0,
        })
        .collect();
    app.ui.global::<ChangelogState>().set_sections(modele(sections));
}

/// Les releases publiées (ni brouillons), les plus récentes d'abord.
fn depuis_releases(v: &serde_json::Value) -> Vec<Section> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter(|r| !r["draft"].as_bool().unwrap_or(false))
        .filter_map(|r| {
            let tag = r["tag_name"].as_str()?;
            let (lignes, intertitres) = lignes(r["body"].as_str().unwrap_or(""));
            Some((tag.trim_start_matches('v').to_uppercase(), lignes, intertitres))
        })
        .collect()
}

/// Markdown d'une version : « - texte » = une ligne, « ### titre » = un intertitre.
fn lignes(markdown: &str) -> (Vec<String>, Vec<bool>) {
    let (mut lignes, mut intertitres) = (Vec::new(), Vec::new());
    for l in markdown.lines().map(str::trim) {
        if let Some(t) = l.strip_prefix("- ").or_else(|| l.strip_prefix("* ")) {
            lignes.push(t.replace(['`', '*'], ""));
            intertitres.push(false);
        } else if l.starts_with('#') {
            let t = l.trim_start_matches('#').trim();
            if !t.is_empty() {
                lignes.push(t.to_uppercase());
                intertitres.push(true);
            }
        }
    }
    (lignes, intertitres)
}

/// CHANGELOG.md intégré : « ## version » = une section, gardée si elle a été publiée sur GitHub.
fn integre() -> Vec<Section> {
    let mut sections = Vec::new();
    let mut courante: Option<(String, String)> = None;
    let finir = |c: Option<(String, String)>, sections: &mut Vec<Section>| {
        if let Some((titre, texte)) = c {
            if !update::est_plus_recente(PREMIERE_RELEASE, &titre) {
                let (l, i) = lignes(&texte);
                sections.push((titre.to_uppercase(), l, i));
            }
        }
    };
    for ligne in CHANGELOG.lines() {
        if let Some(titre) = ligne.strip_prefix("## ") {
            finir(courante.take(), &mut sections);
            courante = Some((titre.trim().to_string(), String::new()));
        } else if let Some((_, texte)) = courante.as_mut() {
            texte.push_str(ligne);
            texte.push('\n');
        }
    }
    finir(courante, &mut sections);
    sections
}

#[cfg(test)]
mod tests {
    #[test]
    fn changelog_integre_depuis_la_premiere_release() {
        let s = super::integre();
        // titres en majuscules à l'affichage ("0.3.8-BETA.1")
        assert_eq!(s.first().map(|x| x.0.clone()), Some(crate::services::app_info::VERSION.to_uppercase()));
        assert_eq!(s.last().map(|x| x.0.as_str()), Some("0.3.0"));
        assert!(s[0].2.iter().any(|&h| h), "intertitres de 0.3.5");
    }
}
