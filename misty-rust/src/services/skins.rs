//! Skin et capes d'un compte Microsoft (API api.minecraftservices.com).

use super::{account, http};
use image::{imageops, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

const API: &str = "https://api.minecraftservices.com/minecraft/profile";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cape {
    pub id: String,
    pub nom: String,
    pub url: String,
    pub active: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Profil {
    pub skin_url: String,
    pub fin: bool,
    pub capes: Vec<Cape>,
}

#[derive(Deserialize)]
struct ProfilJson {
    #[serde(default)]
    skins: Vec<SkinJson>,
    #[serde(default)]
    capes: Vec<CapeJson>,
}

#[derive(Deserialize)]
struct SkinJson {
    #[serde(default)]
    state: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    variant: String,
}

#[derive(Deserialize)]
struct CapeJson {
    id: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    alias: String,
}

/// Jeton valide (renouvelé si besoin).
async fn jeton() -> anyhow::Result<String> {
    Ok(account::connecter().await?.jeton)
}

/// Mojang refuse les demandes (HTTP 429). `attente` : secondes demandées par Mojang (0 = non précisé).
#[derive(Debug)]
pub struct Limite {
    pub attente: u64,
}

impl std::fmt::Display for Limite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTTP 429 (trop de requêtes)")
    }
}

impl std::error::Error for Limite {}

/// Skin et capes actuels du compte.
pub async fn profil() -> anyhow::Result<Profil> {
    // Outil de développement : MISTY_FAKE_429=1 simule la limite de requêtes de Mojang (version debug seulement)
    if cfg!(debug_assertions) && std::env::var_os("MISTY_FAKE_429").is_some() {
        return Err(Limite { attente: 4 }.into());
    }
    let r = http::client().get(API).bearer_auth(jeton().await?).send().await?;
    if r.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let attente = r.headers().get("retry-after").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
        return Err(Limite { attente }.into());
    }
    let p: ProfilJson = r.error_for_status()?.json().await?;
    let skin = p.skins.iter().find(|s| s.state == "ACTIVE").or(p.skins.first());
    Ok(Profil {
        skin_url: skin.map(|s| s.url.clone()).unwrap_or_default(),
        fin: skin.map(|s| s.variant.eq_ignore_ascii_case("slim")).unwrap_or(false),
        capes: p
            .capes
            .into_iter()
            .map(|c| Cape { active: c.state == "ACTIVE", nom: majuscule(&c.alias), id: c.id, url: c.url })
            .collect(),
    })
}

fn majuscule(alias: &str) -> String {
    let mut c = alias.chars();
    match c.next() {
        Some(p) => p.to_uppercase().collect::<String>() + c.as_str(),
        None => "Cape".into(),
    }
}

/// Dossier où sont gardés les skins et capes déjà téléchargés.
/// Skins portés ou envoyés (« Mes skins » de la page Skin), les plus récents d'abord.
fn dossier_historique() -> std::path::PathBuf {
    super::paths::data_dir().join("skins")
}

/// Nombre de skins gardés dans l'historique.
const HISTORIQUE_MAX: usize = 12;

/// Garde un skin dans l'historique (ou le remonte en tête s'il y est déjà). Nom : md5 du PNG, et
/// « -fin » pour les bras fins.
pub fn ajouter_historique(png: &[u8], fin: bool) {
    if !skin_valide(png) {
        return;
    }
    let dossier = dossier_historique();
    let _ = std::fs::create_dir_all(&dossier);
    let base = format!("{:x}", md5::compute(png));
    // le même skin avec l'autre modèle de bras : on remplace
    let _ = std::fs::remove_file(dossier.join(format!("{base}.png")));
    let _ = std::fs::remove_file(dossier.join(format!("{base}-fin.png")));
    let _ = std::fs::write(dossier.join(format!("{base}{}.png", if fin { "-fin" } else { "" })), png);
    for (ancien, _) in historique().into_iter().skip(HISTORIQUE_MAX) {
        let _ = std::fs::remove_file(ancien);
    }
}

/// Les skins de l'historique (fichier, bras fins), du plus récent au plus ancien.
pub fn historique() -> Vec<(std::path::PathBuf, bool)> {
    let Ok(entrees) = std::fs::read_dir(dossier_historique()) else { return Vec::new() };
    let mut liste: Vec<(std::time::SystemTime, std::path::PathBuf)> = entrees
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "png"))
        .map(|p| (std::fs::metadata(&p).and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH), p))
        .collect();
    liste.sort_by(|a, b| b.0.cmp(&a.0));
    liste
        .into_iter()
        .map(|(_, p)| {
            let fin = p.file_stem().is_some_and(|n| n.to_string_lossy().ends_with("-fin"));
            (p, fin)
        })
        .collect()
}

/// Identifiant d'un skin de l'historique (son nom de fichier sans « -fin ») : md5 du PNG.
pub fn id_historique(png: &[u8]) -> String {
    format!("{:x}", md5::compute(png))
}

fn dossier_cache() -> std::path::PathBuf {
    super::paths::data_dir().join("cache").join("skins")
}

/// Télécharge une texture Mojang, ou la relit sur le disque si on l'a déjà eue.
/// L'adresse d'une texture contient son empreinte : le contenu d'une même adresse ne change jamais.
pub async fn telecharger(url: &str) -> anyhow::Result<Vec<u8>> {
    let fichier = dossier_cache().join(format!("{:x}.png", md5::compute(url)));
    if let Ok(octets) = std::fs::read(&fichier) {
        if !octets.is_empty() {
            return Ok(octets);
        }
    }
    if url.starts_with("local:") {
        anyhow::bail!("skin local introuvable"); // pas une vraie adresse : seulement sur le disque
    }
    let octets = http::client().get(url).send().await?.error_for_status()?.bytes().await?.to_vec();
    let _ = std::fs::create_dir_all(dossier_cache());
    let _ = std::fs::write(&fichier, &octets);
    Ok(octets)
}

/// Dernier profil connu d'un compte (relu quand Mojang ne répond pas).
pub fn profil_en_cache(uuid: &str) -> Option<Profil> {
    if uuid.is_empty() {
        return None;
    }
    serde_json::from_str(&std::fs::read_to_string(dossier_cache().join(format!("profil-{uuid}.json"))).ok()?).ok()
}

pub fn enregistrer_profil(uuid: &str, profil: &Profil) {
    if uuid.is_empty() {
        return;
    }
    let _ = std::fs::create_dir_all(dossier_cache());
    let _ = std::fs::write(dossier_cache().join(format!("profil-{uuid}.json")), serde_json::to_string(profil).unwrap_or_default());
}

/// Après l'envoi d'un skin : on le garde sur le disque, pour l'afficher même si Mojang ne répond pas ensuite.
pub fn memoriser_skin_envoye(uuid: &str, png: &[u8], fin: bool) {
    let adresse = format!("local:{uuid}");
    let _ = std::fs::create_dir_all(dossier_cache());
    let _ = std::fs::write(dossier_cache().join(format!("{:x}.png", md5::compute(&adresse))), png);
    let mut profil = profil_en_cache(uuid).unwrap_or_default();
    profil.skin_url = adresse;
    profil.fin = fin;
    enregistrer_profil(uuid, &profil);
}

/// Steve en entier : toujours le même, calculé une seule fois puis gardé en mémoire.
pub fn steve_tour() -> Option<&'static Vec<RgbaImage>> {
    static STEVE: std::sync::OnceLock<Option<Vec<RgbaImage>>> = std::sync::OnceLock::new();
    STEVE.get_or_init(|| rendre_tour(include_bytes!("../../assets/steve.png"), false, None)).as_ref()
}

/// Envoie un nouveau skin (PNG 64×64 ou 64×32).
pub async fn changer_skin(png: Vec<u8>, fin: bool) -> anyhow::Result<()> {
    let limite = "----MistyLimite7MA4YWxkTrZu0gW";
    let mut corps: Vec<u8> = Vec::new();
    corps.extend_from_slice(
        format!(
            "--{limite}\r\nContent-Disposition: form-data; name=\"variant\"\r\n\r\n{}\r\n--{limite}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"skin.png\"\r\nContent-Type: image/png\r\n\r\n",
            if fin { "slim" } else { "classic" }
        )
        .as_bytes(),
    );
    corps.extend_from_slice(&png);
    corps.extend_from_slice(format!("\r\n--{limite}--\r\n").as_bytes());
    let r = http::client()
        .post(format!("{API}/skins"))
        .bearer_auth(jeton().await?)
        .header("Content-Type", format!("multipart/form-data; boundary={limite}"))
        .body(corps)
        .send()
        .await?;
    verifier(r).await
}

/// Active une cape (`None` = ne plus en porter).
pub async fn activer_cape(id: Option<&str>) -> anyhow::Result<()> {
    if cfg!(debug_assertions) && std::env::var_os("MISTY_FAKE_429").is_some() {
        return Err(Limite { attente: 2 }.into()); // simulation (voir profil)
    }
    let jeton = jeton().await?;
    let url = format!("{API}/capes/active");
    let r = match id {
        Some(id) => http::client().put(url).bearer_auth(jeton).json(&serde_json::json!({ "capeId": id })).send().await?,
        None => http::client().delete(url).bearer_auth(jeton).send().await?,
    };
    verifier(r).await
}

async fn verifier(r: reqwest::Response) -> anyhow::Result<()> {
    if r.status().is_success() {
        return Ok(());
    }
    if r.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let attente = r.headers().get("retry-after").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
        return Err(Limite { attente }.into());
    }
    let statut = r.status();
    let texte = r.text().await.unwrap_or_default();
    let detail = serde_json::from_str::<serde_json::Value>(&texte)
        .ok()
        .and_then(|v| v["errorMessage"].as_str().or(v["error"].as_str()).map(str::to_string))
        .unwrap_or(texte);
    anyhow::bail!("{statut} {detail}")
}

/// Un skin est valide s'il fait 64×64 ou 64×32 pixels.
pub fn skin_valide(png: &[u8]) -> bool {
    image::load_from_memory(png).map(|i| i.width() == 64 && (i.height() == 64 || i.height() == 32)).unwrap_or(false)
}

/// Modèle d'un skin : vrai pour les bras fins (Alex, « slim »), faux pour les bras larges (Steve).
/// Un bras fin fait 3 pixels de large au lieu de 4 : sur la texture, les 2 dernières colonnes de la
/// zone de chaque bras restent vides (bras droit : x 54–55, y 20–31 ; bras gauche : x 46–47, y 52–63).
/// Les anciens skins 64 × 32 sont toujours larges.
pub fn est_fin(png: &[u8]) -> bool {
    let Ok(img) = image::load_from_memory(png) else { return false };
    let img = img.to_rgba8();
    if img.width() != 64 || img.height() != 64 {
        return false;
    }
    let vide = |x0: u32, y0: u32| (x0..x0 + 2).all(|x| (y0..y0 + 12).all(|y| img.get_pixel(x, y)[3] == 0));
    vide(54, 20) && vide(46, 52)
}

fn coller(dest: &mut RgbaImage, skin: &RgbaImage, src: (u32, u32, u32, u32), vers: (u32, u32), miroir: bool) {
    let (sx, sy, w, h) = src;
    for y in 0..h {
        for x in 0..w {
            let px = skin.get_pixel(if miroir { sx + w - 1 - x } else { sx + x }, sy + y);
            if px[3] == 0 {
                continue;
            }
            let (dx, dy) = (vers.0 + x, vers.1 + y);
            if px[3] == 255 {
                dest.put_pixel(dx, dy, *px);
            } else {
                let mut d = *dest.get_pixel(dx, dy);
                let a = px[3] as f32 / 255.0;
                for i in 0..3 {
                    d[i] = (px[i] as f32 * a + d[i] as f32 * (1.0 - a)) as u8;
                }
                d[3] = d[3].max(px[3]);
                dest.put_pixel(dx, dy, d);
            }
        }
    }
}

// ───────────────────────── 3D preview ─────────────────────────
// Tiny software ray caster: the player model is a handful of textured boxes,
// seen from the side with an orthographic camera that turns around the character.

/// Number of pre-rendered views for one full turn (keep in sync with ui/skin.slint).
pub const IMAGES_TOUR: usize = 36;
const ECHELLE: f32 = 9.0;
const LARGEUR: u32 = 224;
const HAUTEUR: u32 = 320;

/// Region of the texture used by one face of a box (x, y, width, height).
#[derive(Clone, Copy)]
struct Region(u32, u32, u32, u32);

struct Boite {
    min: [f32; 3],
    max: [f32; 3],
    /// +z, -z, -x, +x (front, back, character right, character left)
    faces: [Region; 4],
    /// the picture of each face is flipped horizontally
    miroir: bool,
    /// tilt around the X axis at the top of the box (radians): the bottom swings backwards
    incline: f32,
    /// second tilt, around the Z axis (radians): the bottom swings outward (elytra wings)
    roulis: f32,
    /// pivot of the tilts: (x, y, z)
    pivot: [f32; 3],
    /// translation after the tilts (crouching)
    decale: [f32; 3],
    /// top and bottom faces (only boxes that can show them when tilted, like the limbs)
    dessus: Option<[Region; 2]>,
}

/// Box of the player model, with the layout of the Minecraft skin (origin u,v and size w,h,d).
fn boite_skin(origine: (u32, u32), taille: (u32, u32, u32), min: [f32; 3], gonfle: f32, miroir: bool) -> Boite {
    let (u, v) = origine;
    let (w, h, d) = taille;
    let droite = Region(u, v + d, d, h);
    let devant = Region(u + d, v + d, w, h);
    let gauche = Region(u + d + w, v + d, d, h);
    let dos = Region(u + 2 * d + w, v + d, w, h);
    Boite {
        min: [min[0] - gonfle, min[1] - gonfle, min[2] - gonfle],
        max: [min[0] + w as f32 + gonfle, min[1] + h as f32 + gonfle, min[2] + d as f32 + gonfle],
        faces: if miroir { [devant, dos, gauche, droite] } else { [devant, dos, droite, gauche] },
        miroir,
        incline: 0.0,
        roulis: 0.0,
        pivot: [0.0, 0.0, 0.0],
        decale: [0.0, 0.0, 0.0],
        dessus: Some([Region(u + d, v, w, d), Region(u + d + w, v, w, d)]),
    }
}

/// Cape: a thin box behind the body, its outside faces backwards.
fn boite_cape(f: u32) -> Boite {
    Boite {
        min: [-5.0, 8.0, -3.25],
        max: [5.0, 24.0, -2.25],
        faces: [Region(12 * f, f, 10 * f, 16 * f), Region(f, f, 10 * f, 16 * f), Region(0, f, f, 16 * f), Region(11 * f, f, f, 16 * f)],
        miroir: false,
        incline: CAPE_INCLINAISON,
        roulis: 0.0,
        pivot: [0.0, 24.0, -2.25],
        decale: [0.0, 0.0, 0.0],
        dessus: None,
    }
}
/// Like on NameMC, the cape hangs diagonally instead of sticking to the body.
const CAPE_INCLINAISON: f32 = 0.25;

/// Entry point of a ray into a box: distance, axis and whether the face is on the positive side.
fn entrer(o: [f32; 3], d: [f32; 3], b: &Boite) -> Option<(f32, usize, bool)> {
    let (mut tmin, mut tmax) = (f32::NEG_INFINITY, f32::INFINITY);
    let (mut axe, mut positif) = (1, false);
    for i in 0..3 {
        if d[i].abs() < 1e-6 {
            if o[i] < b.min[i] || o[i] > b.max[i] {
                return None;
            }
            continue;
        }
        let t1 = (b.min[i] - o[i]) / d[i];
        let t2 = (b.max[i] - o[i]) / d[i];
        let (proche, loin) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
        if proche > tmin {
            tmin = proche;
            axe = i;
            positif = d[i] < 0.0;
        }
        tmax = tmax.min(loin);
    }
    if tmin > tmax || tmax < 0.0 || (axe == 1 && b.dessus.is_none()) {
        return None;
    }
    Some((tmin, axe, positif))
}

fn echantillon(b: &Boite, texture: &RgbaImage, p: [f32; 3], axe: usize, positif: bool) -> Rgba<u8> {
    if axe == 1 {
        if let Some([haut, bas]) = b.dessus {
            let mut fu = (p[0] - b.min[0]) / (b.max[0] - b.min[0]);
            if b.miroir {
                fu = 1.0 - fu;
            }
            let profondeur = b.max[2] - b.min[2];
            let (Region(x, y, w, h), fv) = if positif { (haut, (p[2] - b.min[2]) / profondeur) } else { (bas, (b.max[2] - p[2]) / profondeur) };
            let tx = x + (fu.clamp(0.0, 0.9999) * w as f32) as u32;
            let ty = y + (fv.clamp(0.0, 0.9999) * h as f32) as u32;
            return *texture.get_pixel(tx.min(texture.width() - 1), ty.min(texture.height() - 1));
        }
    }
    let largeur = b.max[axe_largeur(axe)] - b.min[axe_largeur(axe)];
    let (face, mut fu) = match (axe, positif) {
        (2, true) => (0, (p[0] - b.min[0]) / largeur),
        (2, false) => (1, (b.max[0] - p[0]) / largeur),
        (0, false) => (2, (p[2] - b.min[2]) / largeur),
        _ => (3, (b.max[2] - p[2]) / largeur),
    };
    if b.miroir {
        fu = 1.0 - fu;
    }
    let fv = (b.max[1] - p[1]) / (b.max[1] - b.min[1]);
    let Region(x, y, w, h) = b.faces[face];
    let tx = x + ((fu.clamp(0.0, 0.9999)) * w as f32) as u32;
    let ty = y + ((fv.clamp(0.0, 0.9999)) * h as f32) as u32;
    *texture.get_pixel(tx.min(texture.width() - 1), ty.min(texture.height() - 1))
}

fn axe_largeur(axe: usize) -> usize {
    // faces facing z are as wide as the box is along x, faces facing x as the box is deep along z
    if axe == 2 {
        0
    } else {
        2
    }
}

/// The character turning on itself: `IMAGES_TOUR` views, the first one seen from the front.
/// The cape (texture bytes of the whole cape) is only visible from behind.
pub fn rendre_tour(png: &[u8], fin: bool, cape: Option<&[u8]>) -> Option<Vec<RgbaImage>> {
    let toutes: Vec<usize> = (0..IMAGES_TOUR).collect();
    rendre_vues(png, fin, cape, &toutes)
}

/// Seulement certaines vues du tour (ex. [0] : la face, pour afficher le personnage tout de suite).
pub fn rendre_vues(png: &[u8], fin: bool, cape: Option<&[u8]>, indices: &[usize]) -> Option<Vec<RgbaImage>> {
    rendre_vues_pose(png, fin, cape, indices, 0.0, false)
}

/// Le tour complet, avec des élytres à la place de la cape.
pub fn rendre_tour_elytres(png: &[u8], fin: bool, cape: Option<&[u8]>, elytres: bool) -> Option<Vec<RgbaImage>> {
    let toutes: Vec<usize> = (0..IMAGES_TOUR).collect();
    rendre_vues_pose(png, fin, cape, &toutes, 0.0, elytres)
}

/// Texture d'élytres quand le joueur n'a pas de cape : celle de Misty (nuit, plumes et croissant de lune).
fn elytres_par_defaut() -> RgbaImage {
    static ELYTRES: std::sync::OnceLock<RgbaImage> = std::sync::OnceLock::new();
    ELYTRES
        .get_or_init(|| image::load_from_memory(texture_misty()).map(|i| i.to_rgba8()).unwrap_or_else(|_| RgbaImage::from_pixel(64, 32, Rgba([60, 80, 140, 255]))))
        .clone()
}

/// Poses de la marche : nombre d'images du cycle, et nombre d'angles de vue (un tous les 30°).
pub const POSES_MARCHE: usize = 12;
pub const ANGLES_MARCHE: usize = 12;

/// Le personnage qui marche : POSES_MARCHE poses x ANGLES_MARCHE angles (pose d'abord).
pub fn rendre_marche(png: &[u8], fin: bool, cape: Option<&[u8]>, elytres: bool) -> Option<Vec<RgbaImage>> {
    let angles: Vec<usize> = (0..ANGLES_MARCHE).map(|a| a * IMAGES_TOUR / ANGLES_MARCHE).collect();
    let mut toutes = Vec::new();
    for pose in 0..POSES_MARCHE {
        let balancement = 0.5 * (pose as f32 * std::f32::consts::TAU / POSES_MARCHE as f32).sin();
        toutes.extend(rendre_vues_pose(png, fin, cape, &angles, balancement, elytres)?);
    }
    Some(toutes)
}

/// Comme `rendre_vues`, avec les bras et les jambes balancés de `balancement` radians (marche).
pub fn rendre_vues_pose(png: &[u8], fin: bool, cape: Option<&[u8]>, indices: &[usize], balancement: f32, elytres: bool) -> Option<Vec<RgbaImage>> {
    let skin = image::load_from_memory(png).ok()?.to_rgba8();
    if skin.width() != 64 || skin.height() < 32 {
        return None;
    }
    let mut cape_img = cape.and_then(|c| image::load_from_memory(c).ok()).map(|i| i.to_rgba8());
    if elytres {
        // les élytres utilisent la texture de la cape (64x32), ou une texture grise par défaut
        cape_img = Some(match cape_img {
            Some(c) if c.width() != 64 || c.height() != 32 => imageops::resize(&c, 64, 32, imageops::FilterType::Nearest),
            Some(c) => c,
            None => elytres_par_defaut(),
        });
    }
    let moderne = skin.height() >= 64;
    let bras: u32 = if fin { 3 } else { 4 };
    let b = bras as f32;

    let mut boites: Vec<Boite> = vec![
        boite_skin((0, 0), (8, 8, 8), [-4.0, 24.0, -4.0], 0.0, false),
        boite_skin((16, 16), (8, 12, 4), [-4.0, 12.0, -2.0], 0.0, false),
        boite_skin((40, 16), (bras, 12, 4), [-4.0 - b, 12.0, -2.0], 0.0, false),
        boite_skin((0, 16), (4, 12, 4), [-4.0, 0.0, -2.0], 0.0, false),
    ];
    if moderne {
        boites.push(boite_skin((32, 48), (bras, 12, 4), [4.0, 12.0, -2.0], 0.0, false));
        boites.push(boite_skin((16, 48), (4, 12, 4), [0.0, 0.0, -2.0], 0.0, false));
    } else {
        boites.push(boite_skin((40, 16), (bras, 12, 4), [4.0, 12.0, -2.0], 0.0, true));
        boites.push(boite_skin((0, 16), (4, 12, 4), [0.0, 0.0, -2.0], 0.0, true));
    }
    // Clothes layer, a little bigger than the base layer
    boites.push(boite_skin((32, 0), (8, 8, 8), [-4.0, 24.0, -4.0], 0.5, false));
    if moderne {
        boites.push(boite_skin((16, 32), (8, 12, 4), [-4.0, 12.0, -2.0], 0.25, false));
        boites.push(boite_skin((40, 32), (bras, 12, 4), [-4.0 - b, 12.0, -2.0], 0.25, false));
        boites.push(boite_skin((48, 48), (bras, 12, 4), [4.0, 12.0, -2.0], 0.25, false));
        boites.push(boite_skin((0, 32), (4, 12, 4), [-4.0, 0.0, -2.0], 0.25, false));
        boites.push(boite_skin((0, 48), (4, 12, 4), [0.0, 0.0, -2.0], 0.25, false));
    }
    let nb_skin = boites.len();
    if balancement != 0.0 {
        // bras droit et jambe gauche d'un côté, bras gauche et jambe droite de l'autre
        let mut pencher = |i: usize, angle: f32, y: f32| {
            if let Some(b) = boites.get_mut(i) {
                b.incline = angle;
                b.pivot = [0.0, y, 0.0];
            }
        };
        for (i, angle, y) in [(2, balancement, 22.0), (3, -balancement, 12.0), (4, -balancement, 22.0), (5, balancement, 12.0)] {
            pencher(i, angle, y);
        }
        if moderne {
            for (i, angle, y) in [(8, balancement, 22.0), (9, -balancement, 22.0), (10, -balancement, 12.0), (11, balancement, 12.0)] {
                pencher(i, angle, y);
            }
        }
    }
    let facteur = cape_img.as_ref().map(|c| (c.width() / 64).max(1)).unwrap_or(1);
    if elytres {
        // like in the game: the wings hang from the shoulders, tilted backwards and spread outward
        for (cote, miroir) in [(1.0f32, false), (-1.0, true)] {
            let mut aile = boite_skin((22, 0), (10, 20, 2), [-5.0, 4.0, -4.0], 0.0, miroir);
            aile.incline = 0.26;
            aile.roulis = 0.38 * cote;
            aile.pivot = [5.0 * cote, 24.0, -2.0];
            aile.faces[0] = aile.faces[1];
            boites.push(aile);
        }
    } else if cape_img.is_some() {
        boites.push(boite_cape(facteur));
    }

    let rendre = |k: usize| -> RgbaImage {
        let a = k as f32 * std::f32::consts::TAU / IMAGES_TOUR as f32;
        let (s, c) = a.sin_cos();
        // the viewer is at (sin, 0, cos) and looks at the character; screen right is (cos, 0, -sin)
        let regard = [-s, 0.0, -c];
        let droite = [c, 0.0, -s];
        let mut vue = RgbaImage::from_pixel(LARGEUR, HAUTEUR, Rgba([0, 0, 0, 0]));
        for py in 0..HAUTEUR {
            let sy = (HAUTEUR as f32 / 2.0 - (py as f32 + 0.5)) / ECHELLE + 16.0;
            for px in 0..LARGEUR {
                let sx = (px as f32 + 0.5 - LARGEUR as f32 / 2.0) / ECHELLE;
                let o = [droite[0] * sx - regard[0] * 100.0, sy, droite[2] * sx - regard[2] * 100.0];
                let mut meilleur: Option<(f32, Rgba<u8>)> = None;
                for (i, boite) in boites.iter().enumerate() {
                    let (ol, dl) = if boite.incline == 0.0 && boite.roulis == 0.0 {
                        (o, regard)
                    } else {
                        // the ray in the frame of the tilted box (inverse rotations around the pivot)
                        let (sx, cx) = boite.incline.sin_cos();
                        let (sz, cz) = boite.roulis.sin_cos();
                        let inverse = |v: [f32; 3]| -> [f32; 3] {
                            let (x, y, z) = (v[0] * cz + v[1] * sz, -v[0] * sz + v[1] * cz, v[2]);
                            [x, y * cx + z * sx, -y * sx + z * cx]
                        };
                        let [px, py, pz] = boite.pivot;
                        let po = inverse([o[0] - px, o[1] - py, o[2] - pz]);
                        ([po[0] + px, po[1] + py, po[2] + pz], inverse(regard))
                    };
                    let Some((t, axe, positif)) = entrer(ol, dl, boite) else { continue };
                    if meilleur.map(|(m, _)| t >= m).unwrap_or(false) {
                        continue;
                    }
                    let p = [ol[0] + dl[0] * t, ol[1] + dl[1] * t, ol[2] + dl[2] * t];
                    let texture = if i >= nb_skin { cape_img.as_ref().unwrap() } else { &skin };
                    let couleur = echantillon(boite, texture, p, axe, positif);
                    if couleur[3] >= 128 {
                        meilleur = Some((t, Rgba([couleur[0], couleur[1], couleur[2], 255])));
                    }
                }
                if let Some((_, couleur)) = meilleur {
                    vue.put_pixel(px, py, couleur);
                }
            }
        }
        vue
    };

    // Les vues sont indépendantes : on les calcule en parallèle sur tous les cœurs
    let fils = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(1, indices.len().max(1));
    let mut resultats: Vec<(usize, RgbaImage)> = Vec::with_capacity(indices.len());
    std::thread::scope(|sc| {
        let rendre = &rendre;
        let poignees: Vec<_> = (0..fils)
            .map(|t| sc.spawn(move || indices.iter().skip(t).step_by(fils).map(|&k| (k, rendre(k))).collect::<Vec<_>>()))
            .collect();
        for p in poignees {
            if let Ok(v) = p.join() {
                resultats.extend(v);
            }
        }
    });
    resultats.sort_by_key(|(k, _)| indices.iter().position(|i| i == k));
    Some(resultats.into_iter().map(|(_, v)| v).collect())
}
/// Tête du skin avec le chapeau, agrandie à 64×64 sans lissage.
pub fn rendre_tete(png: &[u8]) -> Option<RgbaImage> {
    let skin = image::load_from_memory(png).ok()?.to_rgba8();
    if skin.width() != 64 || skin.height() < 32 {
        return None;
    }
    let mut r = RgbaImage::from_pixel(8, 8, Rgba([0, 0, 0, 0]));
    coller(&mut r, &skin, (8, 8, 8, 8), (0, 0), false);
    // anciens skins 64 × 32 : un calque du chapeau sans aucun pixel transparent est ignoré, comme
    // le fait Minecraft (sinon, souvent une tête toute noire)
    let chapeau_opaque = skin.height() == 32 && (0..32).all(|x| (0..16).all(|y| skin.get_pixel(32 + x, y)[3] == 255));
    if !chapeau_opaque {
        coller(&mut r, &skin, (40, 8, 8, 8), (0, 0), false);
    }
    Some(imageops::resize(&r, 64, 64, imageops::FilterType::Nearest))
}

/// Face extérieure d'une cape (10×16 sur une texture 64×32, ou un multiple).
pub fn rendre_cape(png: &[u8]) -> Option<RgbaImage> {
    let cape = image::load_from_memory(png).ok()?.to_rgba8();
    let f = (cape.width() / 64).max(1);
    if cape.width() < 22 * f || cape.height() < 17 * f {
        return None;
    }
    Some(imageops::crop_imm(&cape, f, f, 10 * f, 16 * f).to_image())
}

/// Face peinte des deux élytres côte à côte (la zone de 12×22 en (31, 0), la seconde en miroir).
pub fn rendre_elytres(png: &[u8]) -> Option<RgbaImage> {
    let tex = image::load_from_memory(png).ok()?.to_rgba8();
    let f = (tex.width() / 64).max(1);
    if tex.width() < 43 * f || tex.height() < 22 * f {
        return None;
    }
    let aile = imageops::crop_imm(&tex, 31 * f, 0, 12 * f, 22 * f).to_image();
    let mut r = RgbaImage::from_pixel(25 * f, 22 * f, Rgba([0, 0, 0, 0]));
    imageops::overlay(&mut r, &imageops::flip_horizontal(&aile), 0, 0);
    imageops::overlay(&mut r, &aile, (13 * f) as i64, 0);
    Some(r)
}

/// Texture 64 × 32 de la cape et des élytres Misty, dessinée comme sur la boutique du site :
/// cape en dégradé nuit avec un nuage, élytres en plumes avec un croissant de lune. PNG en mémoire.
/// Texture de la cape early player (cape et élytres), la même que celle du mod.
pub fn texture_early() -> &'static [u8] {
    include_bytes!("../../../misty-client/src/main/resources/assets/misty/textures/cape/early.png")
}

/// Aperçu du chapeau Cloudly : un petit nuage en pixels, 16 × 8 agrandi 4 fois.
pub fn apercu_cloudly() -> RgbaImage {
    // lignes du nuage, de haut en bas : début et longueur
    const LIGNES: [(u32, u32); 7] = [(6, 4), (4, 8), (3, 10), (1, 14), (0, 16), (0, 16), (1, 14)];
    let mut img = RgbaImage::new(64, 32);
    for (r, &(debut, long)) in LIGNES.iter().enumerate() {
        let couleur = if r < 2 { [255, 255, 255, 255] } else if r < 5 { [241, 246, 255, 255] } else { [214, 226, 255, 255] };
        for x in debut..debut + long {
            for dy in 0..4 {
                for dx in 0..4 {
                    img.put_pixel(x * 4 + dx, 2 + r as u32 * 4 + dy, image::Rgba(couleur));
                }
            }
        }
    }
    img
}

pub fn texture_misty() -> &'static [u8] {
    static PNG: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    PNG.get_or_init(|| {
        let mut img = RgbaImage::new(64, 32);
        let mut pose = |x: u32, y: u32, r: f32, g: f32, b: f32| {
            img.put_pixel(x, y, image::Rgba([r.clamp(0.0, 255.0) as u8, g.clamp(0.0, 255.0) as u8, b.clamp(0.0, 255.0) as u8, 255]));
        };
        // Cape : face extérieure (1,1) en dégradé avec un léger grain
        for y in 0..16u32 {
            for x in 1..11u32 {
                let t = y as f32 / 16.0;
                let d = ((x + y) % 2) as f32 * 6.0 - 3.0 + (((x - 1) * 7 + y * 3) % 5) as f32 - 2.0;
                pose(x, y + 1, (26.0 + 70.0 * t + d).round(), (40.0 + 105.0 * t + d).round(), (77.0 + 160.0 * t + d).round());
            }
        }
        // Nuage sur la cape
        let nuage = ["..###...", ".######.", "########", "########", ".######."];
        for (y, ligne) in nuage.iter().enumerate() {
            for (x, c) in ligne.chars().enumerate() {
                if c == '#' {
                    let (r, g, b) = if y == 4 { (185.0, 205.0, 245.0) } else { (234.0, 241.0, 255.0) };
                    pose(2 + x as u32, 5 + y as u32, r, g, b);
                }
            }
        }
        for (x0, y0, l) in [(2u32, 11u32, 6u32), (6, 13, 4)] {
            for x in x0..x0 + l {
                pose(x, y0, 207.0, 224.0, 255.0);
            }
        }
        // Face intérieure et tranches en bleu nuit uni
        for y in 1..17u32 {
            for x in 12..22u32 {
                pose(x, y, 18.0, 27.0, 51.0);
            }
            pose(0, y, 28.0, 42.0, 77.0);
            pose(11, y, 28.0, 42.0, 77.0);
        }
        for x in 1..21u32 {
            pose(x, 0, 28.0, 42.0, 77.0);
        }
        // Élytres : seule la face de dos est peinte (même masque que les capes de Mojang)
        const MASQUE: [(u32, u32); 22] = [(31, 33), (32, 33), (34, 42), (34, 42), (35, 42), (35, 42), (35, 42), (35, 42), (35, 42), (35, 42), (35, 42), (35, 42), (36, 42), (36, 42), (36, 42), (36, 42), (37, 41), (37, 41), (37, 41), (37, 41), (38, 40), (38, 39)];
        const LUNE: [&str; 6] = ["..##", ".#..", "#...", "#...", ".#..", "..##"];
        for (y, (x0, x1)) in MASQUE.iter().enumerate() {
            let y = y as u32;
            for x in *x0..=*x1 {
                let t = y as f32 / 21.0;
                let d = ((x * 7 + y * 3) % 5) as f32 - 2.0;
                let (mut r, mut g, mut b) = (24.0 + 62.0 * t + d, 36.0 + 98.0 * t + d, 70.0 + 150.0 * t + d * 1.5);
                let k = ((y as f32 - (x as f32 - 35.0) * 0.7) % 4.5 + 4.5) % 4.5;
                if y > 7 && y < 20 && k < 1.0 {
                    r += 70.0;
                    g += 72.0;
                    b += 30.0;
                }
                if x == *x0 {
                    r += 40.0;
                    g += 45.0;
                    b += 28.0;
                }
                let lune = y >= 4 && x >= 37 && LUNE.get((y - 4) as usize).and_then(|l| l.as_bytes().get((x - 37) as usize)) == Some(&b'#');
                if lune {
                    (r, g, b) = (234.0, 241.0, 255.0);
                }
                pose(x, y, r.min(245.0).floor(), g.min(248.0).floor(), b.min(255.0).floor());
            }
        }
        let mut png = Vec::new();
        let _ = image::DynamicImage::ImageRgba8(img).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png);
        png
    })
}

// ───────────────────────── Aperçu en temps réel ─────────────────────────
// Le studio de l'onglet Skin dessine le personnage à chaque image (et non plus des vues
// pré-calculées) : rotation et inclinaison de la caméra continues, animations, cape qui bouge.

/// Animations du studio (mêmes que la boutique du site).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Anim {
    Repos,
    Marche,
    Course,
    Vol,
    Accroupi,
}

impl Anim {
    pub fn depuis(i: i32) -> Anim {
        match i {
            1 => Anim::Marche,
            2 => Anim::Course,
            3 => Anim::Vol,
            4 => Anim::Accroupi,
            _ => Anim::Repos,
        }
    }
}

/// Taille de l'image du rendu en direct (carrée : en vol, le personnage est à l'horizontale).
pub const LIVE: u32 = 320;

/// Orientation d'une partie du corps : bascule avant/arrière (x) et écartement (z) autour d'un
/// pivot, puis déplacement.
#[derive(Clone, Copy, Default)]
struct Membre {
    incline: f32,
    roulis: f32,
    pivot: [f32; 3],
    decale: [f32; 3],
}

/// Pose complète à l'instant `t` (secondes).
struct Pose {
    tete: Membre,
    corps: Membre,
    bras_d: Membre,
    bras_g: Membre,
    jambe_d: Membre,
    jambe_g: Membre,
    /// inclinaison de la cape (elle flotte vers l'arrière)
    cape: f32,
    /// élytres : inclinaison et écartement
    ailes: (f32, f32),
    /// tout le personnage basculé (vol, course), autour de la taille
    global: f32,
}

fn pose(anim: Anim, t: f32, b: f32) -> Pose {
    use std::f32::consts::TAU;
    let m = |pivot: [f32; 3]| Membre { pivot, ..Default::default() };
    let mut p = Pose {
        tete: m([0.0, 24.0, 0.0]),
        corps: m([0.0, 24.0, 0.0]),
        bras_d: m([-4.0 - b / 2.0, 22.0, 0.0]),
        bras_g: m([4.0 + b / 2.0, 22.0, 0.0]),
        jambe_d: m([-2.0, 12.0, 0.0]),
        jambe_g: m([2.0, 12.0, 0.0]),
        cape: CAPE_INCLINAISON,
        ailes: (0.26, 0.38),
        global: 0.0,
    };
    // respiration : les bras s'écartent à peine, la cape ondule doucement
    let souffle = (t * TAU / 3.2).sin();
    p.bras_d.roulis = -0.03 - 0.025 * souffle;
    p.bras_g.roulis = 0.03 + 0.025 * souffle;
    p.cape += 0.05 * (t * TAU / 2.7).sin();
    match anim {
        Anim::Repos => {}
        Anim::Marche | Anim::Course => {
            let (amp, freq) = if anim == Anim::Course { (0.95, 1.55) } else { (0.55, 0.95) };
            let s = (t * TAU * freq).sin();
            p.bras_d.incline = amp * s;
            p.bras_g.incline = -amp * s;
            p.jambe_d.incline = -amp * s;
            p.jambe_g.incline = amp * s;
            // la cape se soulève avec la vitesse et bat au rythme des pas
            let vitesse = if anim == Anim::Course { 0.85 } else { 0.4 };
            p.cape = CAPE_INCLINAISON + vitesse + 0.08 * (t * TAU * freq * 2.0).sin();
        }
        Anim::Vol => {
            // à l'horizontale, bras le long du corps, jambes serrées, ailes grandes ouvertes
            p.global = 1.25;
            p.bras_d.incline = -0.25;
            p.bras_g.incline = -0.25;
            p.bras_d.roulis = -0.12;
            p.bras_g.roulis = 0.12;
            p.tete.incline = -0.9;
            p.cape = 1.2 + 0.1 * (t * TAU * 1.8).sin();
            p.ailes = (0.45, 0.7 + 0.08 * (t * TAU * 0.9).sin());
        }
        Anim::Accroupi => {
            // k : 0 debout, 1 accroupi ; on reste un peu dans chaque position (cycle de 2,4 s)
            let k = ((t * TAU / 2.4).sin() * 0.7 + 0.5).clamp(0.0, 1.0);
            let k = k * k * (3.0 - 2.0 * k);
            p.corps.incline = 0.5 * k;
            p.bras_d.incline = 0.4 * k;
            p.bras_g.incline = 0.4 * k;
            for haut in [&mut p.tete, &mut p.corps, &mut p.bras_d, &mut p.bras_g] {
                haut.decale = [0.0, -2.6 * k, 0.0];
            }
            p.jambe_d.decale = [0.0, 0.0, -3.6 * k];
            p.jambe_g.decale = [0.0, 0.0, -3.6 * k];
            p.cape += 0.5 * k;
            // ailes plus écartées accroupi, comme dans le jeu
            p.ailes = (0.26 + 0.04 * k, 0.38 + 0.52 * k);
        }
    }
    p
}

/// Une boîte prête à être touchée par les rayons : la caméra est ramenée dans son repère une fois
/// par image (et non à chaque pixel).
struct BoitePosee<'a> {
    boite: &'a Boite,
    texture: &'a RgbaImage,
    origine: [f32; 3],
    droite: [f32; 3],
    haut: [f32; 3],
    regard: [f32; 3],
}

fn tourner_inverse(v: [f32; 3], incline: f32, roulis: f32) -> [f32; 3] {
    let (sx, cx) = incline.sin_cos();
    let (sz, cz) = roulis.sin_cos();
    let (x, y, z) = (v[0] * cz + v[1] * sz, -v[0] * sz + v[1] * cz, v[2]);
    [x, y * cx + z * sx, -y * sx + z * cx]
}

/// Partie du corps visible en un pixel du rendu en direct (`rendre_live_parties`).
#[allow(dead_code)] // les bras et les jambes ne mènent à aucun emplacement (pour l'instant)
pub mod partie {
    pub const TETE: u8 = 0;
    pub const CORPS: u8 = 1;
    pub const BRAS_D: u8 = 2;
    pub const BRAS_G: u8 = 3;
    pub const JAMBE_D: u8 = 4;
    pub const JAMBE_G: u8 = 5;
    pub const CAPE: u8 = 6;
    pub const ELYTRES: u8 = 7;
    /// rien (le fond)
    pub const AUCUNE: u8 = 255;
}

/// Le personnage dans sa pose, vu sous l'angle `yaw` (autour de lui) et `pitch` (de haut ou de bas),
/// en radians. `t` : temps de l'animation, en secondes. Avec l'image, la partie du corps visible en
/// chaque pixel (une valeur de `partie` par pixel, ligne par ligne) : ce qui dit sur quoi on clique.
/// `surbrillance` : parties teintées de bleu (celles que la souris survole dans le studio).
#[allow(clippy::too_many_arguments)]
pub fn rendre_live_parties(skin: &RgbaImage, cape: Option<&RgbaImage>, fin: bool, elytres: bool, anim: Anim, t: f32, yaw: f32, pitch: f32, surbrillance: &[u8]) -> Option<(RgbaImage, Vec<u8>)> {
    if skin.width() != 64 || skin.height() < 32 {
        return None;
    }
    let cape_img: Option<RgbaImage> = match (elytres, cape) {
        (true, Some(c)) if c.width() != 64 || c.height() != 32 => Some(imageops::resize(c, 64, 32, imageops::FilterType::Nearest)),
        (true, Some(c)) => Some(c.clone()),
        (true, None) => Some(elytres_par_defaut()),
        (false, c) => c.cloned(),
    };
    let moderne = skin.height() >= 64;
    let bras: u32 = if fin { 3 } else { 4 };
    let b = bras as f32;
    let p = pose(anim, t, b);

    // (boîte, partie du corps) ; 0 tête, 1 corps, 2 bras d, 3 bras g, 4 jambe d, 5 jambe g
    let mut boites: Vec<(Boite, usize)> = vec![
        (boite_skin((0, 0), (8, 8, 8), [-4.0, 24.0, -4.0], 0.0, false), 0),
        (boite_skin((16, 16), (8, 12, 4), [-4.0, 12.0, -2.0], 0.0, false), 1),
        (boite_skin((40, 16), (bras, 12, 4), [-4.0 - b, 12.0, -2.0], 0.0, false), 2),
        (boite_skin((0, 16), (4, 12, 4), [-4.0, 0.0, -2.0], 0.0, false), 4),
    ];
    if moderne {
        boites.push((boite_skin((32, 48), (bras, 12, 4), [4.0, 12.0, -2.0], 0.0, false), 3));
        boites.push((boite_skin((16, 48), (4, 12, 4), [0.0, 0.0, -2.0], 0.0, false), 5));
        boites.push((boite_skin((32, 0), (8, 8, 8), [-4.0, 24.0, -4.0], 0.5, false), 0));
        boites.push((boite_skin((16, 32), (8, 12, 4), [-4.0, 12.0, -2.0], 0.25, false), 1));
        boites.push((boite_skin((40, 32), (bras, 12, 4), [-4.0 - b, 12.0, -2.0], 0.25, false), 2));
        boites.push((boite_skin((48, 48), (bras, 12, 4), [4.0, 12.0, -2.0], 0.25, false), 3));
        boites.push((boite_skin((0, 32), (4, 12, 4), [-4.0, 0.0, -2.0], 0.25, false), 4));
        boites.push((boite_skin((0, 48), (4, 12, 4), [0.0, 0.0, -2.0], 0.25, false), 5));
    } else {
        boites.push((boite_skin((40, 16), (bras, 12, 4), [4.0, 12.0, -2.0], 0.0, true), 3));
        boites.push((boite_skin((0, 16), (4, 12, 4), [0.0, 0.0, -2.0], 0.0, true), 5));
        boites.push((boite_skin((32, 0), (8, 8, 8), [-4.0, 24.0, -4.0], 0.5, false), 0));
    }
    let nb_skin = boites.len();
    let membres = [p.tete, p.corps, p.bras_d, p.bras_g, p.jambe_d, p.jambe_g];
    for (boite, partie) in boites.iter_mut() {
        let m = membres[*partie];
        boite.incline = m.incline;
        boite.roulis = m.roulis;
        boite.pivot = m.pivot;
        boite.decale = m.decale;
    }
    // cape ou élytres, accrochées au dos (elles suivent l'inclinaison du corps)
    let facteur = cape_img.as_ref().map(|c| (c.width() / 64).max(1)).unwrap_or(1);
    if elytres {
        for (cote, miroir) in [(1.0f32, false), (-1.0, true)] {
            let mut aile = boite_skin((22, 0), (10, 20, 2), [-5.0, 4.0, -4.0], 0.0, miroir);
            aile.incline = p.ailes.0 + p.corps.incline;
            aile.roulis = p.ailes.1 * cote;
            aile.pivot = [5.0 * cote, 24.0, -2.0];
            aile.decale = p.corps.decale;
            aile.faces[0] = aile.faces[1];
            boites.push((aile, 1));
        }
    } else if cape_img.is_some() {
        let mut c = boite_cape(facteur);
        c.incline = p.cape + p.corps.incline;
        c.decale = p.corps.decale;
        boites.push((c, 1));
    }

    // Caméra orthographique sur une sphère autour du personnage
    let (sy_, cy_) = yaw.sin_cos();
    let (sp, cp) = pitch.sin_cos();
    let vers_camera = [sy_ * cp, sp, cy_ * cp];
    let regard = [-vers_camera[0], -vers_camera[1], -vers_camera[2]];
    let droite = [cy_, 0.0, -sy_];
    let haut = [
        vers_camera[1] * droite[2] - vers_camera[2] * droite[1],
        vers_camera[2] * droite[0] - vers_camera[0] * droite[2],
        vers_camera[0] * droite[1] - vers_camera[1] * droite[0],
    ];
    let centre = [0.0, 16.0 - 4.0 * (p.global / 1.25).clamp(0.0, 1.0), 0.0];
    let origine = [centre[0] + vers_camera[0] * 100.0, centre[1] + vers_camera[1] * 100.0, centre[2] + vers_camera[2] * 100.0];

    // tout le modèle basculé (vol) autour de la taille, puis chaque boîte dans son propre repère
    let taille = [0.0f32, 12.0, 0.0];
    let vers_modele = |v: [f32; 3], point: bool| -> [f32; 3] {
        if p.global == 0.0 {
            return v;
        }
        if point {
            let r = tourner_inverse([v[0] - taille[0], v[1] - taille[1], v[2] - taille[2]], p.global, 0.0);
            [r[0] + taille[0], r[1] + taille[1], r[2] + taille[2]]
        } else {
            tourner_inverse(v, p.global, 0.0)
        }
    };
    let (o_m, d_m, h_m, r_m) = (vers_modele(origine, true), vers_modele(droite, false), vers_modele(haut, false), vers_modele(regard, false));
    // partie du corps de chaque boîte : celles du skin, puis la cape ou les élytres
    let parties: Vec<u8> = boites
        .iter()
        .enumerate()
        .map(|(i, (_, partie))| if i >= nb_skin { if elytres { partie::ELYTRES } else { partie::CAPE } } else { *partie as u8 })
        .collect();
    let posees: Vec<BoitePosee> = boites
        .iter()
        .enumerate()
        .map(|(i, (boite, _))| {
            let texture = if i >= nb_skin { cape_img.as_ref().unwrap() } else { skin };
            let [px, py, pz] = boite.pivot;
            let [dx, dy, dz] = boite.decale;
            let o = tourner_inverse([o_m[0] - dx - px, o_m[1] - dy - py, o_m[2] - dz - pz], boite.incline, boite.roulis);
            BoitePosee {
                boite,
                texture,
                origine: [o[0] + px, o[1] + py, o[2] + pz],
                droite: tourner_inverse(d_m, boite.incline, boite.roulis),
                haut: tourner_inverse(h_m, boite.incline, boite.roulis),
                regard: tourner_inverse(r_m, boite.incline, boite.roulis),
            }
        })
        .collect();

    // Rectangle de l'écran que couvre chaque boîte : ses 8 coins vus par la caméra (orthographique,
    // donc une simple projection). Un pixel ne teste que les boîtes dont le rectangle le contient,
    // et les lignes sans aucune boîte sont sautées.
    let demi = LIVE as f32 / 2.0;
    let rectangles: Vec<[u32; 4]> = posees
        .iter()
        .map(|bp| {
            let (mut x0, mut x1, mut y0, mut y1) = (f32::INFINITY, f32::NEG_INFINITY, f32::INFINITY, f32::NEG_INFINITY);
            for coin in 0..8 {
                let q = [
                    if coin & 1 == 0 { bp.boite.min[0] } else { bp.boite.max[0] },
                    if coin & 2 == 0 { bp.boite.min[1] } else { bp.boite.max[1] },
                    if coin & 4 == 0 { bp.boite.min[2] } else { bp.boite.max[2] },
                ];
                let v = [q[0] - bp.origine[0], q[1] - bp.origine[1], q[2] - bp.origine[2]];
                let sx = v[0] * bp.droite[0] + v[1] * bp.droite[1] + v[2] * bp.droite[2];
                let sy = v[0] * bp.haut[0] + v[1] * bp.haut[1] + v[2] * bp.haut[2];
                // pixel dont le centre est en (sx, sy) : voir le calcul de sx et sy plus bas
                let (px, py) = (sx * ECHELLE + demi - 0.5, demi - sy * ECHELLE - 0.5);
                x0 = x0.min(px);
                x1 = x1.max(px);
                y0 = y0.min(py);
                y1 = y1.max(py);
            }
            // un pixel de marge contre les arrondis
            let borne = |v: f32| v.clamp(0.0, LIVE as f32) as u32;
            [borne(x0.floor() - 1.0), borne(x1.ceil() + 2.0), borne(y0.floor() - 1.0), borne(y1.ceil() + 2.0)]
        })
        .collect();
    let lignes_utiles = rectangles.iter().fold((LIVE, 0), |(a, b), r| (a.min(r[2]), b.max(r[3])));

    // Rendu en bandes de lignes, directement dans l'image, sur tous les cœurs
    let mut image = RgbaImage::from_pixel(LIVE, LIVE, Rgba([0, 0, 0, 0]));
    let mut carte = vec![partie::AUCUNE; (LIVE * LIVE) as usize];
    let fils = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(1, 8) as u32;
    let (premiere, derniere) = lignes_utiles;
    if premiere < derniere {
        let bande = (derniere - premiere).div_ceil(fils);
        let largeur = LIVE as usize * 4;
        let utiles = &mut image.as_mut()[premiere as usize * largeur..derniere as usize * largeur];
        let carte_utile = &mut carte[(premiere * LIVE) as usize..(derniere * LIVE) as usize];
        std::thread::scope(|sc| {
            let (posees, rectangles, parties) = (&posees, &rectangles, &parties);
            let surbrillance = surbrillance;
            for (f, (morceau, morceau_carte)) in utiles.chunks_mut(bande as usize * largeur).zip(carte_utile.chunks_mut((bande * LIVE) as usize)).enumerate() {
                sc.spawn(move || {
                    let mut candidates: Vec<usize> = Vec::with_capacity(posees.len());
                    for (l, (ligne, ligne_carte)) in morceau.chunks_mut(largeur).zip(morceau_carte.chunks_mut(LIVE as usize)).enumerate() {
                        let py = premiere + f as u32 * bande + l as u32;
                        candidates.clear();
                        candidates.extend((0..posees.len()).filter(|&i| (rectangles[i][2]..rectangles[i][3]).contains(&py)));
                        if candidates.is_empty() {
                            continue;
                        }
                        let x0 = candidates.iter().map(|&i| rectangles[i][0]).min().unwrap_or(0);
                        let x1 = candidates.iter().map(|&i| rectangles[i][1]).max().unwrap_or(0);
                        let sy = (demi - (py as f32 + 0.5)) / ECHELLE;
                        for px in x0..x1 {
                            let sx = (px as f32 + 0.5 - demi) / ECHELLE;
                            let mut meilleur: Option<(f32, Rgba<u8>, u8)> = None;
                            for &i in candidates.iter() {
                                if !(rectangles[i][0]..rectangles[i][1]).contains(&px) {
                                    continue;
                                }
                                let bp = &posees[i];
                                let o = [
                                    bp.origine[0] + bp.droite[0] * sx + bp.haut[0] * sy,
                                    bp.origine[1] + bp.droite[1] * sx + bp.haut[1] * sy,
                                    bp.origine[2] + bp.droite[2] * sx + bp.haut[2] * sy,
                                ];
                                let Some((t, axe, positif)) = entrer(o, bp.regard, bp.boite) else { continue };
                                if meilleur.map(|(m, _, _)| t >= m).unwrap_or(false) {
                                    continue;
                                }
                                let point = [o[0] + bp.regard[0] * t, o[1] + bp.regard[1] * t, o[2] + bp.regard[2] * t];
                                let couleur = echantillon(bp.boite, bp.texture, point, axe, positif);
                                if couleur[3] >= 128 {
                                    meilleur = Some((t, Rgba([couleur[0], couleur[1], couleur[2], 255]), parties[i]));
                                }
                            }
                            if let Some((_, c, partie)) = meilleur {
                                let k = px as usize * 4;
                                let mut c = c;
                                // partie survolée : éclaircie et teintée du bleu de Misty
                                if surbrillance.contains(&partie) {
                                    let bleu = [111.0f32, 156.0, 255.0];
                                    for (canal, b) in c.0.iter_mut().take(3).zip(bleu) {
                                        *canal = (*canal as f32 * 0.55 + b * 0.45).round() as u8;
                                    }
                                }
                                ligne[k..k + 4].copy_from_slice(&c.0);
                                ligne_carte[px as usize] = partie;
                            }
                        }
                    }
                });
            }
        });
    }
    Some((image, carte))
}

#[cfg(test)]
mod tests_modele {
    use super::*;

    #[test]
    fn steve_et_alex() {
        let steve = include_bytes!("../../assets/steve.png");
        assert!(!est_fin(steve));
        // la même texture, bras réduits à 3 pixels : les colonnes en trop sont effacées
        let mut alex = image::load_from_memory(steve).unwrap().to_rgba8();
        for (x0, y0) in [(54u32, 20u32), (46, 52)] {
            for x in x0..x0 + 2 {
                for y in y0..y0 + 12 {
                    alex.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                }
            }
        }
        let mut png = Vec::new();
        alex.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).unwrap();
        assert!(est_fin(&png));
    }
}

#[cfg(test)]
mod tests_live {
    use super::*;

    #[test]
    fn carte_des_parties() {
        let steve = image::load_from_memory(include_bytes!("../../assets/steve.png")).unwrap().to_rgba8();
        let cape = image::load_from_memory(texture_misty()).unwrap().to_rgba8();
        // de face : la tête est en haut, au milieu
        let (_, face) = rendre_live_parties(&steve, Some(&cape), false, false, Anim::Repos, 0.0, 0.0, 0.0, &[]).unwrap();
        let tete = (0..LIVE as usize).find_map(|y| (face[y * LIVE as usize + LIVE as usize / 2] != partie::AUCUNE).then_some(y)).unwrap();
        assert_eq!(face[tete * LIVE as usize + LIVE as usize / 2], partie::TETE);
        // de dos : la cape couvre le milieu du corps
        let (_, dos) = rendre_live_parties(&steve, Some(&cape), false, false, Anim::Repos, 0.0, std::f32::consts::PI, 0.0, &[]).unwrap();
        assert_eq!(dos[(LIVE as usize / 2) * LIVE as usize + LIVE as usize / 2], partie::CAPE);
        // avec des élytres
        let (_, ailes) = rendre_live_parties(&steve, Some(&cape), false, true, Anim::Repos, 0.0, std::f32::consts::PI, 0.0, &[]).unwrap();
        assert!(ailes.contains(&partie::ELYTRES));
    }

    #[test]
    fn toutes_les_animations_se_dessinent() {
        let steve = image::load_from_memory(include_bytes!("../../assets/steve.png")).unwrap().to_rgba8();
        for a in 0..5 {
            let img = rendre_live_parties(&steve, None, false, a == 3, Anim::depuis(a), 0.4, 0.5, 0.3, &[]).unwrap().0;
            assert_eq!(img.dimensions(), (LIVE, LIVE));
            assert!(img.pixels().any(|p| p[3] == 255), "animation {a} vide");
        }
    }

    /// Temps d'une image du studio : `cargo test --release temps_du_studio -- --ignored --nocapture`.
    /// `MISTY_STUDIO=<dossier>` y écrit aussi les images, pour comparer deux versions du rendu.
    #[test]
    #[ignore]
    fn temps_du_studio() {
        let steve = image::load_from_memory(include_bytes!("../../assets/steve.png")).unwrap().to_rgba8();
        let cape = image::load_from_memory(texture_misty()).unwrap().to_rgba8();
        let dossier = std::env::var("MISTY_STUDIO").ok();
        let debut = std::time::Instant::now();
        let mut n = 0;
        for a in 0..5 {
            for i in 0..24 {
                let yaw = i as f32 * 0.27;
                let img = rendre_live_parties(&steve, Some(&cape), i % 2 == 0, a == 3, Anim::depuis(a), i as f32 * 0.13, yaw, 0.2 + (i % 5) as f32 * 0.1, &[]).unwrap().0;
                n += 1;
                if let Some(d) = &dossier {
                    img.save(format!("{d}/{a}-{i}.png")).unwrap();
                }
            }
        }
        println!("studio : {:.2} ms par image", debut.elapsed().as_secs_f64() * 1000.0 / n as f64);
    }
}

/// Planche des poses, pour régler les animations : `MISTY_POSES=<dossier> cargo test planche_des_poses -- --ignored`.
#[cfg(test)]
mod tests_planche {
    use super::*;

    #[test]
    #[ignore]
    fn planche_des_poses() {
        let Ok(dossier) = std::env::var("MISTY_POSES") else { return };
        let steve = image::load_from_memory(include_bytes!("../../assets/steve.png")).unwrap().to_rgba8();
        let cape = image::load_from_memory(texture_misty()).unwrap().to_rgba8();
        let mut planche = RgbaImage::from_pixel(LIVE * 5, LIVE * 3, Rgba([30, 40, 70, 255]));
        for a in 0..5 {
            for (ligne, yaw) in [(0u32, 0.35f32), (1, std::f32::consts::FRAC_PI_2), (2, 2.6)] {
                let elytres = a == 3 || a == 4;
                let img = rendre_live_parties(&steve, Some(&cape), false, elytres, Anim::depuis(a), 0.3, yaw, 0.15, &[]).unwrap().0;
                imageops::overlay(&mut planche, &img, (a as u32 * LIVE) as i64, (ligne * LIVE) as i64);
            }
        }
        planche.save(std::path::Path::new(&dossier).join("poses.png")).unwrap();
    }
}

#[cfg(test)]
mod tests_misty {
    #[test]
    fn apercus_misty() {
        let cape = super::rendre_cape(super::texture_misty()).expect("cape");
        assert_eq!((cape.width(), cape.height()), (10, 16));
        let ailes = super::rendre_elytres(super::texture_misty()).expect("élytres");
        assert_eq!((ailes.width(), ailes.height()), (25, 22));
        // pas transparentes
        assert!(cape.pixels().filter(|p| p[3] > 0).count() > 100);
        assert!(ailes.pixels().filter(|p| p[3] > 0).count() > 200);
        if let Ok(d) = std::env::var("MISTY_APERCUS") {
            image::imageops::resize(&cape, 100, 160, image::imageops::FilterType::Nearest).save(format!("{d}/cape.png")).unwrap();
            image::imageops::resize(&ailes, 250, 220, image::imageops::FilterType::Nearest).save(format!("{d}/elytres.png")).unwrap();
        }
    }
}
