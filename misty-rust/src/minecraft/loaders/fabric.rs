//! Fabric et Quilt : leurs serveurs fournissent directement le JSON de version à installer.

use crate::minecraft::McDirs;
use crate::services::http;
use serde_json::Value;

#[derive(Clone, Copy)]
pub enum Sorte {
    Fabric,
    Quilt,
}

impl Sorte {
    fn api(self) -> &'static str {
        match self {
            Sorte::Fabric => "https://meta.fabricmc.net/v2",
            Sorte::Quilt => "https://meta.quiltmc.org/v3",
        }
    }
}

async fn loaders(sorte: Sorte, version: &str) -> anyhow::Result<Vec<Value>> {
    let url = format!("{}/versions/loader/{}", sorte.api(), urlencoding::encode(version));
    let v: Value = http::get_json(&url).await?;
    Ok(v.as_array().cloned().unwrap_or_default())
}

pub async fn disponible(sorte: Sorte, version: &str) -> bool {
    loaders(sorte, version).await.is_ok_and(|l| !l.is_empty())
}

/// "0.30.1" → [0, 30, 1] (pour comparer des versions de loader ; un suffixe "-beta.3" est ignoré).
fn numeros(version: &str) -> Vec<u64> {
    version.split('-').next().unwrap_or_default().split('.').map(|n| n.parse().unwrap_or(0)).collect()
}

/// Installe le JSON de version et renvoie son nom ("fabric-loader-0.16.5-1.21.1"…).
pub async fn installer(dirs: &McDirs, sorte: Sorte, version: &str, version_loader: Option<&str>) -> anyhow::Result<String> {
    let loader = match version_loader {
        Some(v) => v.to_string(),
        None => {
            let liste = loaders(sorte, version).await?;
            // Fabric trie du plus récent au plus ancien, Quilt dans le désordre : on cherche donc
            // la version la plus haute, en préférant une version stable (pas de "-beta" / "-rc")
            let nom = |l: &Value| l["loader"]["version"].as_str().unwrap_or_default().to_string();
            let stable = |l: &Value| l["loader"]["stable"].as_bool().unwrap_or_else(|| !nom(l).contains('-'));
            let choisi = liste
                .iter()
                .filter(|l| stable(l))
                .max_by_key(|l| numeros(&nom(l)))
                .or_else(|| liste.iter().max_by_key(|l| numeros(&nom(l))))
                .ok_or_else(|| anyhow::anyhow!("Aucun loader pour {version}"))?;
            nom(choisi)
        }
    };

    let url = format!("{}/versions/loader/{}/{}/profile/json", sorte.api(), urlencoding::encode(version), urlencoding::encode(&loader));
    let profil: Value = http::get_json(&url).await?;
    let id = profil["id"].as_str().ok_or_else(|| anyhow::anyhow!("Profil de loader invalide"))?.to_string();

    let chemin = dirs.version_json(&id);
    std::fs::create_dir_all(chemin.parent().unwrap())?;
    std::fs::write(&chemin, serde_json::to_string_pretty(&profil)?)?;
    Ok(id)
}
