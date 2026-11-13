//! Notes de version : les releases publiées sur GitHub (Misty-MC/Misty-Launcher), gardées en cache pour
//! s'afficher tout de suite et hors ligne. Sans cache ni réseau : les versions publiées sur GitHub
//! du CHANGELOG.md intégré à l'exe (à partir de 0.3.0).
//!
//! La page montre la liste des versions à gauche et les notes de la version choisie à droite :
//! intertitres (« ### Launcher ») et lignes, chacune avec son genre (nouveauté, correction).

use super::{lancer, modele, modele_textes, App};
use crate::services::{app_info, http, paths, system_info, update};
use crate::{ChangelogSection, ChangelogState};
use slint::ComponentHandle;
use std::rc::Rc;

const CHANGELOG: &str = include_str!("../../../CHANGELOG.md");
/// Première version publiée dans les releases GitHub de Misty-MC/Misty-Launcher.
const PREMIERE_RELEASE: &str = "0.3.0";

/// Genre d'une ligne (ChangelogSection.kinds) : intertitre, ligne simple, nouveauté, correction.
const INTERTITRE: i32 = -1;
const SIMPLE: i32 = 0;
const NOUVEAU: i32 = 1;
const CORRIGE: i32 = 2;

/// Images des versions dans la frise (les illustrations pixel art de la fenêtre des versions).
const ILLUSTRATIONS: [&str; 8] = ["26.x", "1.21", "1.20", "1.19", "1.18", "1.17", "1.16", "1.15"];

/// Partie de chaque ligne, d'après l'intertitre qui la précède : 0 launcher, 1 Misty Client (en jeu).
fn parties(lignes: &[String], genres: &[i32]) -> Vec<i32> {
    let mut partie = 0;
    lignes
        .iter()
        .zip(genres)
        .map(|(l, &g)| {
            if g == INTERTITRE {
                partie = if l.to_lowercase().contains("client") { 1 } else { 0 };
            }
            partie
        })
        .collect()
}

/// Une version publiée.
#[derive(Default)]
struct Section {
    titre: String,
    date: String,
    prerelease: bool,
    url: String,
    lignes: Vec<String>,
    genres: Vec<i32>,
}

fn fichier_cache() -> std::path::PathBuf {
    paths::data_dir().join("cache").join("releases.json")
}

pub fn installer(app: &Rc<App>) {
    app.ui.global::<ChangelogState>().on_open_url(|url| system_info::ouvrir(&url));
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
    use slint::Model;
    let etat = app.ui.global::<ChangelogState>();
    // la version choisie reste choisie quand la liste est rechargée depuis GitHub
    let choisie = usize::try_from(etat.get_selected()).ok().and_then(|i| etat.get_sections().row_data(i)).map(|s| s.title.to_string());
    let index = choisie.and_then(|t| sections.iter().position(|s| s.titre == t)).unwrap_or(0);
    let sections: Vec<ChangelogSection> = sections
        .into_iter()
        .enumerate()
        .map(|(i, s)| ChangelogSection {
            parts: modele(parties(&s.lignes, &s.genres)),
            summary: s.lignes.iter().zip(&s.genres).find(|(_, &g)| g != INTERTITRE).map(|(l, _)| l.clone()).unwrap_or_default().into(),
            art: super::picker::illustration(ILLUSTRATIONS[i % ILLUSTRATIONS.len()]),
            news: s.genres.iter().filter(|&&g| g == NOUVEAU).count() as i32,
            fixes: s.genres.iter().filter(|&&g| g == CORRIGE).count() as i32,
            title: s.titre.into(),
            date: s.date.into(),
            prerelease: s.prerelease,
            url: s.url.into(),
            lines: modele_textes(&s.lignes),
            kinds: modele(s.genres),
            recent: i == 0,
        })
        .collect();
    etat.set_sections(modele(sections));
    etat.set_selected(index as i32);
}

/// Les releases publiées (ni brouillons), les plus récentes d'abord.
fn depuis_releases(v: &serde_json::Value) -> Vec<Section> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter(|r| !r["draft"].as_bool().unwrap_or(false))
        .filter_map(|r| {
            let tag = r["tag_name"].as_str()?;
            let (lignes, genres) = lignes(r["body"].as_str().unwrap_or(""));
            let date = r["published_at"]
                .as_str()
                .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
                .map(|d| super::news::date_courte(d.with_timezone(&chrono::Local).date_naive()))
                .unwrap_or_default();
            Some(Section {
                titre: tag.trim_start_matches('v').to_string(),
                date,
                prerelease: r["prerelease"].as_bool().unwrap_or(false),
                url: r["html_url"].as_str().unwrap_or("").to_string(),
                lignes,
                genres,
            })
        })
        .collect()
}

/// Markdown d'une version : « - texte » = une ligne, « ### titre » = un intertitre.
/// « New … » est une nouveauté, « Fixed: … » une correction (préfixe retiré : la page l'affiche
/// en étiquette).
fn lignes(markdown: &str) -> (Vec<String>, Vec<i32>) {
    let (mut lignes, mut genres) = (Vec::new(), Vec::new());
    for l in markdown.lines().map(str::trim) {
        if let Some(t) = l.strip_prefix("- ").or_else(|| l.strip_prefix("* ")) {
            let t = t.replace(['`', '*'], "");
            let (texte, genre) = if let Some(reste) = t.strip_prefix("Fixed:").or_else(|| t.strip_prefix("Fix:")) {
                (majuscule(reste.trim()), CORRIGE)
            } else if let Some(reste) = t.strip_prefix("New:") {
                (majuscule(reste.trim()), NOUVEAU)
            } else if t.starts_with("New ") {
                (t, NOUVEAU)
            } else {
                (t, SIMPLE)
            };
            lignes.push(texte);
            genres.push(genre);
        } else if l.starts_with('#') {
            let t = l.trim_start_matches('#').trim();
            if !t.is_empty() {
                lignes.push(t.to_string());
                genres.push(INTERTITRE);
            }
        }
    }
    (lignes, genres)
}

fn majuscule(texte: &str) -> String {
    let mut c = texte.chars();
    c.next().map(|p| p.to_uppercase().chain(c).collect()).unwrap_or_default()
}

/// CHANGELOG.md intégré : « ## version » = une section, gardée si elle a été publiée sur GitHub.
fn integre() -> Vec<Section> {
    let mut sections = Vec::new();
    let mut courante: Option<(String, String)> = None;
    let finir = |c: Option<(String, String)>, sections: &mut Vec<Section>| {
        if let Some((titre, texte)) = c {
            if !update::est_plus_recente(PREMIERE_RELEASE, &titre) {
                let (lignes, genres) = lignes(&texte);
                sections.push(Section { prerelease: titre.contains('-'), titre, lignes, genres, ..Default::default() });
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
    use super::*;

    #[test]
    fn changelog_integre_depuis_la_premiere_release() {
        let s = integre();
        // titres tels que publiés ("0.3.8-beta.1")
        assert_eq!(s.first().map(|x| x.titre.clone()), Some(app_info::VERSION.to_string()));
        assert_eq!(s.last().map(|x| x.titre.as_str()), Some("0.3.0"));
        assert!(s[0].genres.contains(&INTERTITRE), "intertitres de la version actuelle");
    }

    #[test]
    fn genres_des_lignes() {
        let (l, g) = lignes("### Launcher\n- New Reach module\n- Fixed: dragging was wrong\n- The ping is faster");
        assert_eq!(g, [INTERTITRE, NOUVEAU, CORRIGE, SIMPLE]);
        assert_eq!(l[0], "Launcher");
        assert_eq!(l[2], "Dragging was wrong");
    }
}
