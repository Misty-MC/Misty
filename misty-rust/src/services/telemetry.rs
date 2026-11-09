//! Télémétrie anonyme, seulement si le joueur l'a acceptée (question au premier lancement,
//! interrupteur dans les options).
//!
//! Envoyé au site de Misty (`/api/telemetry`) : un identifiant d'installation tiré au hasard,
//! la version de Misty, Windows, la RAM, la langue, et les événements (démarrage, partie lancée,
//! partie finie ou plantée, mise à jour). Quand une partie plante, les dernières lignes de la
//! console partent avec, nettoyées (dossier personnel, pseudo, jetons). Jamais le compte,
//! le mot de passe ou l'adresse e-mail.
//!
//! Le bouton « Envoyer les logs » de la console envoie un rapport même sans télémétrie :
//! c'est le joueur qui le demande, et il reçoit un code à donner sur Discord.

use super::{app_info, http, journal, loc, settings, system_info};
use serde_json::{json, Value};
use std::sync::OnceLock;
use std::time::Duration;

const SITE: &str = "https://misty-launcher.online";

static RUNTIME: OnceLock<tokio::runtime::Handle> = OnceLock::new();

/// À appeler une fois au démarrage, avec le runtime des tâches de fond.
pub fn init(rt: tokio::runtime::Handle) {
    let _ = RUNTIME.set(rt);
}

/// Le joueur a-t-il déjà répondu à la question ?
pub fn demandee() -> bool {
    !settings::get(settings::TELEMETRIE).trim().is_empty()
}

pub fn active() -> bool {
    settings::get_bool(settings::TELEMETRIE, false)
}

pub fn activer(oui: bool) {
    settings::set_bool(settings::TELEMETRIE, oui);
    journal::info(&format!("Télémétrie {}", if oui { "activée" } else { "désactivée" }));
}

/// Identifiant d'installation (aléatoire, sans lien avec le compte).
fn id() -> String {
    let id = settings::get(settings::TELEMETRIE_ID);
    if !id.trim().is_empty() {
        return id;
    }
    let id = uuid::Uuid::new_v4().to_string();
    settings::set(settings::TELEMETRIE_ID, &id);
    id
}

fn contexte() -> Value {
    json!({
        "id": id(),
        "version": app_info::VERSION,
        "os": system_info::version_windows(),
        "ram_mo": system_info::ram_totale_mo(),
        "langue": loc::code(),
    })
}

/// Envoie un événement en arrière-plan (rien si la télémétrie est coupée ; une erreur réseau est ignorée).
pub fn evenement(nom: &str, donnees: Value) {
    if !active() {
        return;
    }
    let Some(rt) = RUNTIME.get() else { return };
    let mut corps = contexte();
    corps["event"] = json!(nom);
    corps["data"] = donnees;
    rt.spawn(async move {
        let _ = http::client()
            .post(format!("{SITE}/api/telemetry"))
            .timeout(Duration::from_secs(15))
            .json(&corps)
            .send()
            .await;
    });
}

/// Envoie les dernières lignes de la console. Renvoie le code du rapport.
pub async fn rapport(raison: &str, jeu: &str, lignes: Vec<String>, pseudo: &str) -> anyhow::Result<String> {
    let mut corps = contexte();
    corps["reason"] = json!(raison);
    corps["game"] = json!(jeu);
    corps["log"] = json!(nettoyer(&lignes.join("\n"), pseudo));
    let reponse: Value = http::client()
        .post(format!("{SITE}/api/telemetry/report"))
        .timeout(Duration::from_secs(30))
        .json(&corps)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let code = reponse["id"].as_str().unwrap_or_default().to_string();
    if code.is_empty() {
        anyhow::bail!("réponse du site sans code de rapport");
    }
    journal::info(&format!("Rapport envoyé : {code} ({raison})"));
    Ok(code)
}

/// Après une partie plantée : rapport automatique si la télémétrie est active.
pub fn plantage(partie: u64, pseudo: &str) {
    if !active() {
        return;
    }
    let Some(rt) = RUNTIME.get() else { return };
    let lignes = journal::dernieres(partie, 400);
    let jeu = journal::etat(partie).map(|e| e.titre).unwrap_or_default();
    let pseudo = pseudo.to_string();
    rt.spawn(async move {
        let _ = rapport("crash", &jeu, lignes, &pseudo).await;
    });
}

/// Retire ce qui pourrait identifier le joueur : dossier personnel, pseudo, jetons d'accès.
pub fn nettoyer(texte: &str, pseudo: &str) -> String {
    let mut t = texte.to_string();
    if let Some(maison) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        let maison = maison.to_string_lossy().to_string();
        if maison.len() > 3 {
            t = t.replace(&maison, "~");
            t = t.replace(&maison.replace('\\', "/"), "~");
        }
    }
    if let Ok(nom) = std::env::var("USERNAME") {
        if nom.len() > 2 {
            t = t.replace(&nom, "<user>");
        }
    }
    if pseudo.len() > 2 {
        t = t.replace(pseudo, "<player>");
    }
    // jetons : JWT (eyJ…) et valeurs après --accessToken / --session
    let mut sortie = String::with_capacity(t.len());
    let mut masquer = false;
    for (i, mot) in t.split(' ').enumerate() {
        if i > 0 {
            sortie.push(' ');
        }
        if masquer {
            sortie.push_str("<hidden>");
            masquer = false;
        } else if mot.starts_with("eyJ") && mot.len() > 40 {
            sortie.push_str("<token>");
        } else {
            sortie.push_str(mot);
            masquer = matches!(mot, "--accessToken" | "--session" | "--clientId" | "--xuid");
        }
    }
    sortie
}

#[cfg(test)]
mod tests {
    #[test]
    fn nettoyage() {
        let t = super::nettoyer("Setting user: Steve --accessToken abc123 next eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.sig ok", "Steve");
        assert_eq!(t, "Setting user: <player> --accessToken <hidden> next <token> ok");
    }
}
