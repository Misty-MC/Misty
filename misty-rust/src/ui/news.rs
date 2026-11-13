//! « Dernières news » de l'accueil : les dernières releases GitHub de Misty (titre, date, premières
//! lignes des notes, première image), en carrousel. Hors ligne, on affiche la dernière entrée des
//! notes de version intégrées.
//!
//! Une release sans image reçoit une illustration pixel art (celles de la fenêtre des versions).

use super::{lancer, App};
use crate::services::{app_info, http, loc, system_info};
use crate::{HomeState, NewsItem};
use slint::{ComponentHandle, Image, Model, ModelRc, VecModel};
use std::rc::Rc;

/// Nombre de releases montrées dans le carrousel.
const NOMBRE: usize = 5;
/// Illustrations de remplacement, une par news (dans l'ordre).
const ILLUSTRATIONS: [&str; NOMBRE] = ["26.x", "1.21", "1.20", "1.19", "1.18"];

struct News {
    titre: String,
    texte: String,
    date: String,
    /// Page de la release ; vide = ouvrir les notes de version du launcher
    url: String,
    /// Première image des notes (vide = illustration)
    image: String,
}

pub fn installer(app: &Rc<App>) {
    let a = app.clone();
    app.ui.global::<HomeState>().on_news_open(move |i| {
        let url = a.news_urls.borrow().get(i.max(0) as usize).cloned().unwrap_or_default();
        if url.is_empty() {
            super::nav_clicked(&a, super::page::NOTES);
        } else {
            system_info::ouvrir(&url);
        }
    });
}

/// Charge les news en arrière-plan (sans bloquer le démarrage).
pub fn charger(app: &Rc<App>) {
    afficher(app, vec![depuis_notes_integrees()]);
    let a = app.clone();
    lancer(async move {
        let Ok(news) = a.bg(depuis_github()).await else { return };
        if news.is_empty() {
            return;
        }
        let images: Vec<String> = news.iter().map(|n| n.image.clone()).collect();
        afficher(&a, news);
        // les images arrivent ensuite, une par une
        for (i, url) in images.into_iter().enumerate() {
            if url.is_empty() {
                continue;
            }
            if let Some(img) = telecharger_image(&a, url).await {
                let modele = a.ui.global::<HomeState>().get_news_items();
                if let Some(mut item) = modele.row_data(i) {
                    item.image = img;
                    modele.set_row_data(i, item);
                }
            }
        }
    });
}

fn afficher(app: &App, news: Vec<News>) {
    let h = app.ui.global::<HomeState>();
    *app.news_urls.borrow_mut() = news.iter().map(|n| n.url.clone()).collect();
    let items: Vec<NewsItem> = news
        .into_iter()
        .enumerate()
        .map(|(i, n)| NewsItem {
            title: n.titre.into(),
            text: n.texte.into(),
            date: n.date.into(),
            image: super::picker::illustration(ILLUSTRATIONS[i % NOMBRE]),
        })
        .collect();
    h.set_news_index(0);
    h.set_news_items(ModelRc::new(VecModel::from(items)));
    h.set_news_visible(true);
}

/// Image d'une news, réduite pour la carte de l'accueil.
async fn telecharger_image(app: &App, url: String) -> Option<Image> {
    let tampon = app
        .bg(async move {
            let reponse = http::client().get(&url).send().await.ok()?.error_for_status().ok()?;
            let octets = reponse.bytes().await.ok()?;
            tokio::task::spawn_blocking(move || super::images::decoder(&octets, 720)).await.ok()?
        })
        .await;
    tampon.map(Image::from_rgba8)
}

async fn depuis_github() -> anyhow::Result<Vec<News>> {
    let docs: Vec<serde_json::Value> = http::get_json(&format!(
        "https://api.github.com/repos/{}/releases?per_page={NOMBRE}",
        app_info::GITHUB_REPO
    ))
    .await?;
    Ok(docs.iter().filter_map(depuis_release).take(NOMBRE).collect())
}

fn depuis_release(doc: &serde_json::Value) -> Option<News> {
    if doc["draft"].as_bool() == Some(true) {
        return None;
    }
    let tag = doc["tag_name"].as_str().unwrap_or("").trim_start_matches('v').to_string();
    if tag.is_empty() {
        return None;
    }
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
    let corps = doc["body"].as_str().unwrap_or("");
    Some(News {
        titre,
        texte: premiere_ligne(corps),
        date,
        url: doc["html_url"].as_str().unwrap_or("").to_string(),
        image: premiere_image(corps),
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
    News { titre, texte, date: String::new(), url: String::new(), image: String::new() }
}

/// Lien de la première image d'un texte markdown (`![…](lien)` ou `<img src="lien">`).
fn premiere_image(markdown: &str) -> String {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(r#"!\[[^\]]*\]\((https://[^)\s]+)[^)]*\)|<img[^>]*\ssrc="(https://[^"]+)""#).expect("regex image")
    });
    re.captures(markdown)
        .and_then(|c| c.get(1).or_else(|| c.get(2)))
        .map(|m| m.as_str().to_string())
        .unwrap_or_default()
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
pub(super) fn date_courte(d: chrono::NaiveDate) -> String {
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

    #[test]
    fn premiere_image_markdown_ou_html() {
        assert_eq!(super::premiere_image("Texte\n![capture](https://ex.com/a.png \"t\")"), "https://ex.com/a.png");
        assert_eq!(super::premiere_image(r#"<img width="600" src="https://ex.com/b.jpg" />"#), "https://ex.com/b.jpg");
        assert_eq!(super::premiere_image("pas d'image"), "");
    }
}
