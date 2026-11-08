//! « Dernières news » de l'accueil : la dernière release GitHub de Misty (titre, date, premières
//! lignes des notes). Hors ligne, on affiche la dernière entrée des notes de version intégrées.

use super::{lancer, App};
use crate::services::{app_info, http, loc, system_info};
use crate::HomeState;
use slint::ComponentHandle;
use std::rc::Rc;

struct News {
    titre: String,
    texte: String,
    date: String,
    /// Page de la release ; vide = ouvrir les notes de version du launcher
    url: String,
}

pub fn installer(app: &Rc<App>) {
    let a = app.clone();
    app.ui.global::<HomeState>().on_news_open(move || {
        let url = a.news_url.borrow().clone();
        if url.is_empty() {
            super::nav_clicked(&a, super::page::NOTES);
        } else {
            system_info::ouvrir(&url);
        }
    });
}

/// Charge la news en arrière-plan (sans bloquer le démarrage).
pub fn charger(app: &Rc<App>) {
    afficher(app, depuis_notes_integrees());
    let a = app.clone();
    lancer(async move {
        if let Ok(news) = a.bg(depuis_github()).await {
            afficher(&a, news);
        }
    });
}

fn afficher(app: &App, news: News) {
    let h = app.ui.global::<HomeState>();
    h.set_news_title(news.titre.into());
    h.set_news_text(news.texte.into());
    h.set_news_date(news.date.into());
    *app.news_url.borrow_mut() = news.url;
    h.set_news_visible(true);
}

async fn depuis_github() -> anyhow::Result<News> {
    let doc: serde_json::Value =
        http::get_json(&format!("https://api.github.com/repos/{}/releases/latest", app_info::GITHUB_REPO)).await?;
    let tag = doc["tag_name"].as_str().unwrap_or("").trim_start_matches('v').to_string();
    anyhow::ensure!(!tag.is_empty(), "release sans version");
    let nom = doc["name"].as_str().map(str::trim).filter(|n| !n.is_empty() && *n != tag);
    let titre = match nom {
        Some(n) => n.to_string(),
        None => format!("Misty {tag}"),
    };
    let date = doc["published_at"]
        .as_str()
        .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
        .map(|d| date_courte(d.with_timezone(&chrono::Local).date_naive()))
        .unwrap_or_default();
    Ok(News {
        titre,
        texte: premiere_ligne(doc["body"].as_str().unwrap_or("")),
        date,
        url: doc["html_url"].as_str().unwrap_or("").to_string(),
    })
}

/// Hors ligne : la première section de CHANGELOG.md (la version installée).
fn depuis_notes_integrees() -> News {
    let notes = include_str!("../../../CHANGELOG.md");
    let mut titre = String::new();
    let mut texte = String::new();
    for ligne in notes.lines() {
        if let Some(t) = ligne.strip_prefix("## ") {
            if !titre.is_empty() {
                break;
            }
            titre = format!("Misty {}", t.trim());
        } else if texte.is_empty() && !titre.is_empty() {
            texte = premiere_ligne(ligne);
        }
    }
    News { titre, texte, date: String::new(), url: String::new() }
}

/// Première ligne utile d'un texte markdown, sans la mise en forme.
fn premiere_ligne(markdown: &str) -> String {
    markdown
        .lines()
        .map(|l| l.trim().trim_start_matches(['-', '*', '#', '>', ' ']).replace("**", "").replace('`', ""))
        .find(|l| l.chars().filter(|c| c.is_alphanumeric()).count() > 3)
        .unwrap_or_default()
}

/// « 8 nov. 2026 » dans la langue du launcher.
fn date_courte(d: chrono::NaiveDate) -> String {
    use chrono::Datelike;
    let mois = loc::t(&format!("Mois_{}", d.month()));
    format!("{} {} {}", d.day(), mois, d.year())
}

#[cfg(test)]
mod tests {
    #[test]
    fn premiere_ligne_sans_markdown() {
        assert_eq!(super::premiere_ligne("## Nouveautés\n\n- **Skins** : aperçu 3D"), "Nouveautés");
        assert_eq!(super::premiere_ligne("\n- \n- Options en fenêtre"), "Options en fenêtre");
    }
}
