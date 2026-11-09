//! Compteur de joueurs en ligne du site (`/api/online`) : tant que Misty est ouvert, un signe de
//! vie part toutes les 60 secondes avec un identifiant tiré au hasard à chaque démarrage (rien ne
//! reste d'une session à l'autre, rien ne relie au compte) et s'il y a une partie en cours.

use super::{http, journal};
use std::sync::OnceLock;
use std::time::Duration;

const URL: &str = "https://misty-launcher.online/api/presence";

/// Identifiant de cette session de Misty.
fn session() -> &'static str {
    static S: OnceLock<String> = OnceLock::new();
    S.get_or_init(|| uuid::Uuid::new_v4().simple().to_string())
}

async fn envoyer(corps: serde_json::Value) {
    let _ = http::client().post(URL).timeout(Duration::from_secs(10)).json(&corps).send().await;
}

/// Lance les signes de vie (sur le runtime des tâches de fond).
pub fn demarrer(rt: &tokio::runtime::Handle) {
    rt.spawn(async {
        loop {
            let en_jeu = journal::parties().iter().any(|p| p.en_cours);
            envoyer(serde_json::json!({ "session": session(), "playing": en_jeu })).await;
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });
}

/// À la fermeture de Misty : il ne compte plus tout de suite (attend au plus 2 secondes).
pub fn quitter(rt: &tokio::runtime::Handle) {
    let _ = rt.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), envoyer(serde_json::json!({ "session": session(), "bye": true }))).await
    });
}
