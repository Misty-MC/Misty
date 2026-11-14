//! Amis : une liste gardée sur ce PC (`amis.json`), et qui d'entre eux joue en ce moment avec
//! Misty Client.
//!
//! - Un ami s'ajoute par son pseudo : Mojang donne son UUID et son nom exact.
//! - « En ligne » vient du réseau Misty (`misty-launcher.online/api/client/players`), le même que
//!   celui des badges en jeu : le site connaît les joueurs dont Misty Client a donné signe de vie
//!   depuis moins de 3 minutes. Il faut un jeton Misty : celui que Misty Client garde dans le
//!   dossier du launcher (`misty-network.json`) après avoir prouvé le compte auprès de Mojang. Le
//!   launcher ne fait pas lui-même cette preuve (pas de « join » en plus chez Mojang, donc pas de
//!   risque de 429) et ne touche jamais au jeton Minecraft.
//! - Il n'y a pas de demandes d'ami : la liste est la tienne, comme des favoris.

use super::{http, paths, skins};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

const API: &str = "https://misty-launcher.online/api/client";
/// Le site n'accepte pas plus de joueurs par demande (comme Misty Client).
const LOT: usize = 100;
/// Un skin d'ami est retéléchargé au plus une fois par jour.
const SKIN_FRAIS: std::time::Duration = std::time::Duration::from_secs(24 * 3600);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Ami {
    /// UUID avec tirets (comme `UUID.toString()` en Java, ce qu'attend le site)
    pub uuid: String,
    pub nom: String,
}

fn fichier() -> PathBuf {
    paths::data_dir().join("amis.json")
}

fn dossier_tetes() -> PathBuf {
    paths::data_dir().join("amis")
}

pub fn lire() -> Vec<Ami> {
    std::fs::read_to_string(fichier()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

fn ecrire(amis: &[Ami]) {
    let _ = std::fs::create_dir_all(paths::data_dir());
    if let Ok(texte) = serde_json::to_string_pretty(amis) {
        let _ = std::fs::write(fichier(), texte);
    }
}

/// Résultat d'un ajout.
pub enum Ajout {
    Ajoute(Ami),
    DejaLa,
    Introuvable,
}

/// Cherche le joueur chez Mojang et l'ajoute à la liste (triée par nom).
pub async fn ajouter(pseudo: &str) -> anyhow::Result<Ajout> {
    let pseudo = pseudo.trim();
    if !pseudo_valide(pseudo) {
        return Ok(Ajout::Introuvable);
    }
    let Some(ami) = chercher(pseudo).await? else { return Ok(Ajout::Introuvable) };
    let mut amis = lire();
    if amis.iter().any(|a| a.uuid == ami.uuid) {
        return Ok(Ajout::DejaLa);
    }
    amis.push(ami.clone());
    amis.sort_by_key(|a| a.nom.to_lowercase());
    ecrire(&amis);
    Ok(Ajout::Ajoute(ami))
}

pub fn retirer(uuid: &str) {
    let mut amis = lire();
    amis.retain(|a| a.uuid != uuid);
    ecrire(&amis);
    let _ = std::fs::remove_file(dossier_tetes().join(format!("{uuid}.png")));
}

/// Pseudo Minecraft possible : 1 à 16 lettres, chiffres ou « _ ».
pub fn pseudo_valide(pseudo: &str) -> bool {
    (1..=16).contains(&pseudo.len()) && pseudo.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// UUID et nom exact d'un pseudo, None s'il n'existe pas.
async fn chercher(pseudo: &str) -> anyhow::Result<Option<Ami>> {
    #[derive(Deserialize)]
    struct Reponse {
        id: String,
        name: String,
    }
    let url = format!("https://api.mojang.com/users/profiles/minecraft/{}", urlencoding::encode(pseudo));
    let r = http::client().get(url).send().await?;
    if r.status() == reqwest::StatusCode::NOT_FOUND || r.status() == reqwest::StatusCode::NO_CONTENT {
        return Ok(None);
    }
    let r: Reponse = r.error_for_status()?.json().await?;
    Ok(tirets(&r.id).map(|uuid| Ami { uuid, nom: r.name }))
}

/// « 069a79f444e94726a5befca90e38aaf5 » → « 069a79f4-44e9-4726-a5be-fca90e38aaf5 ».
pub fn tirets(id: &str) -> Option<String> {
    let id: String = id.chars().filter(|c| *c != '-').collect::<String>().to_lowercase();
    if id.len() != 32 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!("{}-{}-{}-{}-{}", &id[0..8], &id[8..12], &id[12..16], &id[16..20], &id[20..]))
}

// ───────────────────────── En ligne ─────────────────────────

/// Qui joue en ce moment.
pub enum Etat {
    /// UUID des amis en ligne
    Connu(HashSet<String>),
    /// Pas de jeton Misty (Misty Client n'a jamais tourné avec un compte Microsoft, ou le site l'a
    /// oublié) : on ne peut pas savoir.
    SansJeton,
}

/// Le jeton Misty gardé par Misty Client, s'il y en a un.
fn jeton() -> Option<String> {
    let texte = std::fs::read_to_string(paths::data_dir().join("misty-network.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&texte).ok()?;
    v.get("token")?.as_str().filter(|t| !t.is_empty()).map(str::to_string)
}

pub async fn en_ligne(amis: &[Ami]) -> anyhow::Result<Etat> {
    let Some(jeton) = jeton() else { return Ok(Etat::SansJeton) };
    let mut resultat = HashSet::new();
    for lot in amis.chunks(LOT) {
        let corps = serde_json::json!({ "uuids": lot.iter().map(|a| a.uuid.as_str()).collect::<Vec<_>>() });
        let r = http::client()
            .post(format!("{API}/players"))
            .bearer_auth(&jeton)
            .timeout(std::time::Duration::from_secs(10))
            .json(&corps)
            .send()
            .await?;
        // jeton refusé : Misty Client le remplacera à sa prochaine partie ; le fichier est à lui,
        // on ne l'efface pas
        if r.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Ok(Etat::SansJeton);
        }
        let v: serde_json::Value = r.error_for_status()?.json().await?;
        if let Some(joueurs) = v.get("players").and_then(|p| p.as_object()) {
            resultat.extend(joueurs.keys().map(|uuid| uuid.to_lowercase()));
        }
    }
    Ok(Etat::Connu(resultat))
}

// ───────────────────────── Têtes ─────────────────────────

/// Tête 64 × 64 d'un ami, depuis son skin (gardé un jour sur le disque).
pub async fn tete(uuid: &str) -> Option<image::RgbaImage> {
    let chemin = dossier_tetes().join(format!("{uuid}.png"));
    let frais = std::fs::metadata(&chemin).and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|age| age < SKIN_FRAIS);
    if !frais {
        if let Some(png) = telecharger_skin(uuid).await {
            let _ = std::fs::create_dir_all(dossier_tetes());
            let _ = std::fs::write(&chemin, &png);
        }
    }
    // même périmé (Mojang injoignable), l'ancien skin vaut mieux que rien
    let png = std::fs::read(&chemin).ok()?;
    skins::rendre_tete(&png)
}

async fn telecharger_skin(uuid: &str) -> Option<Vec<u8>> {
    let id = uuid.replace('-', "");
    let url = format!("https://sessionserver.mojang.com/session/minecraft/profile/{id}");
    let v: serde_json::Value = http::client().get(url).send().await.ok()?.error_for_status().ok()?.json().await.ok()?;
    let valeur = v.get("properties")?.as_array()?.iter().find(|p| p.get("name").and_then(|n| n.as_str()) == Some("textures"))?.get("value")?.as_str()?;
    let textures: serde_json::Value = serde_json::from_slice(&base64(valeur)?).ok()?;
    let url = textures.get("textures")?.get("SKIN")?.get("url")?.as_str()?;
    skins::telecharger(url).await.ok()
}

/// Décodage base64 standard (les textures de Mojang), sans dépendance de plus.
fn base64(texte: &str) -> Option<Vec<u8>> {
    let valeur = |c: u8| -> Option<u32> {
        Some(match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        } as u32)
    };
    let octets: Vec<u8> = texte.bytes().filter(|c| !c.is_ascii_whitespace() && *c != b'=').collect();
    let mut sortie = Vec::with_capacity(octets.len() * 3 / 4);
    for bloc in octets.chunks(4) {
        let mut n = 0u32;
        for (i, c) in bloc.iter().enumerate() {
            n |= valeur(*c)? << (18 - 6 * i);
        }
        sortie.push((n >> 16) as u8);
        if bloc.len() > 2 {
            sortie.push((n >> 8) as u8);
        }
        if bloc.len() > 3 {
            sortie.push(n as u8);
        }
    }
    Some(sortie)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_avec_tirets() {
        assert_eq!(tirets("069a79f444e94726a5befca90e38aaf5").as_deref(), Some("069a79f4-44e9-4726-a5be-fca90e38aaf5"));
        assert_eq!(tirets("069A79F4-44E9-4726-A5BE-FCA90E38AAF5").as_deref(), Some("069a79f4-44e9-4726-a5be-fca90e38aaf5"));
        assert!(tirets("pas-un-uuid").is_none());
    }

    #[test]
    fn decodage_base64() {
        assert_eq!(base64("TWlzdHk=").unwrap(), b"Misty");
        assert_eq!(base64("TWlzdHkh").unwrap(), b"Misty!");
    }

    #[test]
    fn pseudos() {
        assert!(pseudo_valide("Ender_Mythex"));
        assert!(!pseudo_valide(""));
        assert!(!pseudo_valide("un pseudo"));
        assert!(!pseudo_valide("abcdefghijklmnopq"));
    }
}
